//! Purchases and gifts. Payments show up several times in the activity log: a
//! `payment_succeeded` (or `external_payment_succeeded` for app-store payments), and
//! `transaction_completed` rows that also include settlement corrections (a charge, its
//! reversal, the charge again). Summing every row of one payment gives what it really cost, so
//! refunds net out to zero.

use std::collections::{HashMap, HashSet};

use serde::Serialize;

#[derive(Default)]
struct Payment {
    first_ms: i64,
    sku: Option<String>,
    currency: Option<String>,
    /// Largest single charge seen, in cents.
    gross: i64,
    /// Sum of `transaction_completed` rows, in cents.
    net: i64,
    seen_rows: bool,
    gift: Option<bool>,
    kind: Option<String>,
}

#[derive(Default)]
pub struct Money {
    payments: HashMap<String, Payment>,
    rows: HashSet<String>,
    titles: HashMap<String, String>,
    made: Vec<(i64, String, Option<String>)>,
    sent: HashMap<String, (i64, Option<String>, Option<String>)>,
    received: Vec<(i64, String)>,
    resolved: HashMap<String, String>,
    entitlements: Vec<(i64, String)>,
}

#[derive(Serialize, Debug)]
pub struct MoneyFacts {
    pub purchases: Vec<Purchase>,
    pub gifts_made: Vec<GiftMade>,
    pub gifts_received: Vec<GiftReceived>,
    /// Free things added to your account (games and apps you claimed).
    pub entitlements: Vec<GiftReceived>,
}

#[derive(Serialize, Debug)]
pub struct Purchase {
    pub ms: i64,
    pub sku: Option<String>,
    /// What it was, when any event names the item.
    pub title: Option<String>,
    pub currency: String,
    /// What it cost in the end, in cents. 0 when it was refunded.
    pub cents: i64,
    pub refunded: bool,
    pub gift: Option<bool>,
    /// `subscription` or `sku` (a one-off purchase), when known.
    pub kind: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct GiftMade {
    pub ms: i64,
    pub title: Option<String>,
    /// When and where you sent the link, if you did: a channel (a DM, or one in `guild`).
    pub sent_ms: Option<i64>,
    pub channel: Option<String>,
    pub guild: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct GiftReceived {
    pub ms: i64,
    pub title: Option<String>,
}

/// The fields of one money event this module cares about.
#[derive(Default)]
pub struct MoneyEvent<'a> {
    pub kind: &'a str,
    pub ms: i64,
    pub event_id: Option<&'a str>,
    pub payment_id: Option<String>,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    pub sku: Option<String>,
    pub title: Option<String>,
    pub gift: Option<bool>,
    pub payment_type: Option<String>,
    pub code: Option<String>,
    pub channel: Option<String>,
    pub guild: Option<String>,
    pub step: Option<String>,
}

impl Money {
    pub fn add(&mut self, e: MoneyEvent) {
        if let (Some(sku), Some(title)) = (&e.sku, &e.title) {
            self.titles.entry(sku.clone()).or_insert_with(|| title.clone());
        }
        match e.kind {
            "payment_succeeded" | "external_payment_succeeded" | "transaction_completed" => {
                let Some(id) = e.payment_id else { return };
                if e.kind == "transaction_completed" {
                    // The same row can be logged twice; count it once.
                    if let Some(ev) = e.event_id {
                        if !self.rows.insert(ev.to_owned()) {
                            return;
                        }
                    }
                }
                let p = self.payments.entry(id).or_default();
                if p.first_ms == 0 || e.ms < p.first_ms {
                    p.first_ms = e.ms;
                }
                p.sku = p.sku.take().or(e.sku);
                p.currency = p.currency.take().or(e.currency.map(|c| c.to_uppercase()));
                p.gift = p.gift.or(e.gift);
                p.kind = p.kind.take().or(e.payment_type);
                let amount = e.amount.unwrap_or(0);
                p.gross = p.gross.max(amount);
                if e.kind == "transaction_completed" {
                    p.net += amount;
                    p.seen_rows = true;
                }
            }
            "gift_code_created" => {
                if let Some(code) = e.code {
                    self.made.push((e.ms, code, e.sku));
                }
            }
            "gift_code_sent" => {
                if let Some(code) = e.code {
                    self.sent.entry(code).or_insert((e.ms, e.channel, e.guild));
                }
            }
            "gift_code_resolved" => {
                if let (Some(code), Some(title)) = (e.code, e.title) {
                    self.resolved.insert(code, title);
                }
            }
            "gift_accept_step" if e.step.as_deref() == Some("SUCCESS") => {
                if let Some(code) = e.code {
                    self.received.push((e.ms, code));
                }
            }
            "sku_entitlement_created" if e.amount.unwrap_or(0) == 0 => {
                if let Some(title) = e.title {
                    self.entitlements.push((e.ms, title));
                }
            }
            _ => {}
        }
    }

    pub fn finish(self) -> MoneyFacts {
        let title = |sku: &Option<String>| sku.as_ref().and_then(|s| self.titles.get(s)).cloned();
        let mut purchases: Vec<Purchase> = self
            .payments
            .values()
            .filter(|p| p.gross > 0)
            .map(|p| {
                let cents = if p.seen_rows { p.net.max(0) } else { p.gross };
                Purchase {
                    ms: p.first_ms,
                    sku: p.sku.clone(),
                    title: title(&p.sku),
                    currency: p.currency.clone().unwrap_or_else(|| "?".into()),
                    cents,
                    refunded: p.seen_rows && p.net <= 0,
                    gift: p.gift,
                    kind: p.kind.clone(),
                }
            })
            .collect();
        purchases.sort_by_key(|p| p.ms);

        let mut seen_codes = HashSet::new();
        let mut gifts_made: Vec<GiftMade> = self
            .made
            .iter()
            .filter(|(_, code, _)| seen_codes.insert(code.clone()))
            .map(|(ms, code, sku)| {
                let sent = self.sent.get(code);
                GiftMade {
                    ms: *ms,
                    title: title(sku),
                    sent_ms: sent.map(|s| s.0),
                    channel: sent.and_then(|s| s.1.clone()),
                    guild: sent.and_then(|s| s.2.clone()),
                }
            })
            .collect();
        gifts_made.sort_by_key(|g| g.ms);

        let mut seen_codes = HashSet::new();
        let mut gifts_received: Vec<GiftReceived> = self
            .received
            .iter()
            .filter(|(_, code)| seen_codes.insert(code.clone()))
            .map(|(ms, code)| GiftReceived { ms: *ms, title: self.resolved.get(code).cloned() })
            .collect();
        gifts_received.sort_by_key(|g| g.ms);

        let mut seen_titles = HashSet::new();
        let mut entitlements: Vec<GiftReceived> = self
            .entitlements
            .iter()
            .filter(|(_, t)| seen_titles.insert(t.clone()))
            .map(|(ms, t)| GiftReceived { ms: *ms, title: Some(t.clone()) })
            .collect();
        entitlements.sort_by_key(|g| g.ms);

        MoneyFacts { purchases, gifts_made, gifts_received, entitlements }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(ms: i64, ev: &str, amount: i64) -> MoneyEvent<'static> {
        MoneyEvent {
            kind: "transaction_completed",
            ms,
            event_id: Some(Box::leak(ev.to_owned().into_boxed_str())),
            payment_id: Some("p1".into()),
            amount: Some(amount),
            currency: Some("eur".into()),
            sku: Some("nitro".into()),
            ..Default::default()
        }
    }

    #[test]
    fn settlement_rows_net_out() {
        let mut m = Money::default();
        m.add(MoneyEvent { kind: "gift_code_resolved", code: Some("c".into()), sku: Some("nitro".into()), title: Some("Nitro".into()), ..Default::default() });
        m.add(row(10, "a", 826));
        m.add(row(20, "b", 826));
        m.add(row(20, "c", -826));
        m.add(row(20, "c", -826)); // logged twice
        let f = m.finish();
        assert_eq!(f.purchases.len(), 1);
        let p = &f.purchases[0];
        assert_eq!((p.cents, p.refunded, p.currency.as_str(), p.title.as_deref(), p.ms), (826, false, "EUR", Some("Nitro"), 10));

        let mut m = Money::default();
        m.add(row(10, "a", 399));
        m.add(row(11, "b", -399));
        assert!(m.finish().purchases[0].refunded);
    }
}

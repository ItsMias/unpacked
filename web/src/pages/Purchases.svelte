<script>
  import Avatar from "../components/Avatar.svelte";
  import AreaChart from "../components/AreaChart.svelte";
  import { price } from "../lib/names.js";
  import { channelName, fmt, purchaseStats, shortDate } from "../lib/stats.js";

  let { result, view, range } = $props();

  const facts = $derived(result.facts);
  const s = $derived(purchaseStats(facts, range, result.sources?.packageDate));
  const money = (currency) => (n) => price(Math.round(n * 100), currency).replace(/[.,]00$/, "");
  const servers = $derived(new Map(facts.servers.map((x) => [x.id, x.name])));
  const itemName = (p) => p.title ?? (p.kind === "subscription" ? "Subscription" : "Shop item");

  // Where a gift link went: a DM (to whom) or a server channel.
  function sentTo(g) {
    if (!g.sent_ms) return { text: "Not sent from Discord" };
    const c = facts.channels.find((x) => x.id === g.channel);
    if (c?.kind === "dm") {
      const p = view.people.get(c.person);
      return { text: p?.display_name ?? "a DM", person: p };
    }
    if (c?.kind === "group") return { text: c.name && c.name !== "None" ? c.name : "a group chat" };
    if (c) { const n = channelName(c, servers.get(c.guild)); return { text: `#${n.channel} in ${n.server}` }; }
    return { text: g.guild ? `a channel in ${servers.get(g.guild) ?? "a server"}` : "a chat that's gone now" };
  }
</script>

<div class="page">
  <section class="strip">
    <dl>
      {#each s.spent.length ? s.spent : [["EUR", 0]] as [currency, cents] (currency)}
        <div><dt>Spent in {currency}</dt><dd>{price(cents, currency)}</dd></div>
      {/each}
      <div><dt>Purchases</dt><dd>{fmt(s.purchases.length - s.refunded)}</dd></div>
      <div><dt>Gifts given</dt><dd>{fmt(s.made.length)}</dd></div>
      <div><dt>Gifts received</dt><dd>{fmt(s.received.length)}</dd></div>
      <div><dt>Server boosts</dt><dd>{fmt(s.boosts)}</dd></div>
    </dl>
  </section>

  <div class="grid12">
    {#each s.perMonth as c (c.currency)}
      <section class="card {s.perMonth.length > 1 ? 's6' : 's12'}">
        <h2>Spent per month in {c.currency}</h2>
        <AreaChart points={c.points} series={[{ key: "spent", label: "Spent", colour: "var(--acc)" }]} height={200} format={money(c.currency)} label="Spending per month in {c.currency}" />
      </section>
    {/each}

    <section class="card s12">
      <h2>Purchases</h2>
      {#if s.purchases.length}
        <table>
          <thead><tr><th>Date</th><th>Item</th><th></th><th class="r">Price</th></tr></thead>
          <tbody>
            {#each [...s.purchases].reverse() as p (p.ms + (p.sku ?? "") + p.cents)}
              <tr class:refunded={p.refunded}>
                <td>{shortDate(p.ms)}</td>
                <td>{itemName(p)}</td>
                <td>{#if p.gift}<span class="pill">Gift</span>{/if}{#if p.refunded}<span class="pill">Refunded</span>{/if}</td>
                <td class="r num">{price(p.refunded ? 0 : p.cents, p.currency)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {:else}
        <p class="muted">No purchases in this range.</p>
      {/if}
    </section>

    <section class="card s7">
      <h2>Gifts you made <small>(where you sent the link)</small></h2>
      <ol class="gifts">
        {#each [...s.made].reverse() as g (g.ms)}
          {@const to = sentTo(g)}
          <li>
            {#if to.person}<Avatar id={to.person.id} hash={to.person.avatar} name={to.person.display_name} size={32} />{:else}<span class="gift" aria-hidden="true">🎁</span>{/if}
            <div><strong>{g.title ?? "Gift"}</strong><small>{to.person ? `To ${to.text}` : to.text}</small></div>
            <span class="date">{shortDate(g.ms)}</span>
          </li>
        {:else}
          <li class="muted">No gifts in this range.</li>
        {/each}
      </ol>
    </section>

    <section class="card s5">
      <h2>Gifts you redeemed</h2>
      <ol class="gifts">
        {#each [...s.received].reverse() as g (g.ms)}
          <li>
            <span class="gift" aria-hidden="true">✨</span>
            <div><strong>{g.title ?? "Gift"}</strong></div>
            <span class="date">{shortDate(g.ms)}</span>
          </li>
        {:else}
          <li class="muted">No gifts in this range.</li>
        {/each}
      </ol>
      {#if s.entitlements.length}
        <h3>Free things you claimed</h3>
        <ol class="gifts">
          {#each s.entitlements as g (g.ms + g.title)}
            <li><span class="gift" aria-hidden="true">📦</span><div><strong>{g.title}</strong></div><span class="date">{shortDate(g.ms)}</span></li>
          {/each}
        </ol>
      {/if}
    </section>
  </div>
</div>

<style>
  table { width: 100%; border-collapse: collapse; font-size: 14px; }
  th { text-align: left; font-size: 12px; font-weight: 600; color: var(--t-muted); padding: 0 8px 8px; border-bottom: 1px solid var(--line); }
  td { padding: 9px 8px; border-bottom: 1px solid var(--sunk); color: var(--t-body); }
  tr:last-child td { border-bottom: 0; }
  .r { text-align: right; }
  td.r { color: var(--t-head); font-weight: 600; }
  tr.refunded td:not(:nth-child(3)) { color: var(--t-faint); text-decoration: line-through; }
  .pill { display: inline-block; font-size: 11px; font-weight: 700; padding: 2px 6px; border-radius: 4px; background: var(--raised); color: var(--t-body); margin-right: 4px; }

  h3 { margin: 20px 0 10px; font-size: 14px; font-weight: 600; color: var(--t-muted); }
  .gifts { list-style: none; margin: 0; padding: 0; display: grid; gap: 10px; }
  .gifts li { display: grid; grid-template-columns: 32px 1fr auto; align-items: center; gap: 12px; }
  .gifts .muted { display: block; }
  .gift { width: 32px; height: 32px; border-radius: 50%; background: var(--raised); display: grid; place-items: center; font-size: 16px; }
  .gifts div { min-width: 0; }
  .gifts strong { display: block; color: var(--t-head); font-weight: 600; }
  .gifts small { display: block; color: var(--t-muted); font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .date { color: var(--t-muted); font-size: 13px; white-space: nowrap; }
</style>

//! Emoji extraction from message text.

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Emoji {
    /// Server emoji written as `<:name:id>` / `<a:name:id>`.
    Custom { name: String, id: String, animated: bool },
    /// A Unicode emoji (possibly a multi-codepoint sequence).
    Unicode(String),
}

/// Calls `f` for every emoji in `text`, in order.
pub fn for_each(text: &str, mut f: impl FnMut(Emoji)) {
    let b = text.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'<' {
            if let Some((emoji, len)) = custom_at(&text[i..]) {
                f(emoji);
                i += len;
                continue;
            }
        }
        let c = text[i..].chars().next().unwrap();
        if is_pictographic(c) || is_regional(c) {
            let len = sequence_len(&text[i..]);
            f(Emoji::Unicode(text[i..i + len].trim_end_matches('\u{FE0F}').to_owned()));
            i += len;
            continue;
        }
        i += c.len_utf8();
    }
}

/// Parses `<a?:name:id>` at the start of `s`.
fn custom_at(s: &str) -> Option<(Emoji, usize)> {
    let rest = s.strip_prefix('<')?;
    let (animated, rest) = match rest.strip_prefix("a:") {
        Some(r) => (true, r),
        None => (false, rest.strip_prefix(':')?),
    };
    let name_end = rest.find(':')?;
    let name = &rest[..name_end];
    if !(2..=32).contains(&name.len()) || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') {
        return None;
    }
    let rest = &rest[name_end + 1..];
    let id_end = rest.find('>')?;
    let id = &rest[..id_end];
    if !(17..=20).contains(&id.len()) || !id.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let len = 1 + usize::from(animated) + 1 + name_end + 1 + id_end + 1;
    Some((Emoji::Custom { name: name.into(), id: id.into(), animated }, len))
}

/// Byte length of the emoji sequence starting at `s` (base + modifiers + ZWJ joins, or a flag pair).
fn sequence_len(s: &str) -> usize {
    let mut chars = s.char_indices().peekable();
    let (_, first) = chars.next().unwrap();
    let mut end = first.len_utf8();
    if is_regional(first) {
        if let Some(&(i, c)) = chars.peek() {
            if is_regional(c) {
                end = i + c.len_utf8();
            }
        }
        return end;
    }
    while let Some(&(i, c)) = chars.peek() {
        match c {
            '\u{FE0F}' | '\u{20E3}' | '\u{1F3FB}'..='\u{1F3FF}' | '\u{E0020}'..='\u{E007F}' => {
                end = i + c.len_utf8();
                chars.next();
            }
            '\u{200D}' => {
                chars.next();
                match chars.peek() {
                    Some(&(j, n)) if is_pictographic(n) => {
                        end = j + n.len_utf8();
                        chars.next();
                    }
                    _ => break,
                }
            }
            _ => break,
        }
    }
    end
}

fn is_regional(c: char) -> bool {
    ('\u{1F1E6}'..='\u{1F1FF}').contains(&c)
}

fn is_pictographic(c: char) -> bool {
    matches!(c as u32,
        0x1F300..=0x1F5FF | 0x1F600..=0x1F64F | 0x1F680..=0x1F6FF | 0x1F7E0..=0x1F7EB
        | 0x1F900..=0x1F9FF | 0x1FA70..=0x1FAFF | 0x1F004 | 0x1F0CF | 0x1F18E | 0x1F191..=0x1F19A
        | 0x2600..=0x27BF | 0x2B50 | 0x2B55 | 0x2B1B | 0x2B1C | 0x2B05..=0x2B07
        | 0x2934 | 0x2935 | 0x3030 | 0x303D | 0x3297 | 0x3299 | 0x231A | 0x231B | 0x23E9..=0x23FA)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all(s: &str) -> Vec<Emoji> {
        let mut v = vec![];
        for_each(s, |e| v.push(e));
        v
    }
    fn u(s: &str) -> Emoji {
        Emoji::Unicode(s.into())
    }

    #[test]
    fn custom_emojis() {
        assert_eq!(
            all("hi <:pepe_sad:123456789012345678> and <a:dance:12345678901234567890>"),
            vec![
                Emoji::Custom { name: "pepe_sad".into(), id: "123456789012345678".into(), animated: false },
                Emoji::Custom { name: "dance".into(), id: "12345678901234567890".into(), animated: true },
            ]
        );
        assert!(all("<@123456789012345678> <#123456789012345678> a<b").is_empty());
    }

    #[test]
    fn unicode_sequences() {
        assert_eq!(all("lol 😭😭"), vec![u("😭"), u("😭")]);
        assert_eq!(all("👍🏽"), vec![u("👍🏽")]);
        assert_eq!(all("👩‍💻 ok"), vec![u("👩‍💻")]);
        assert_eq!(all("❤️ and ❤"), vec![u("❤"), u("❤")]);
        assert_eq!(all("🇳🇱🇧🇪"), vec![u("🇳🇱"), u("🇧🇪")]);
        assert!(all("plain text, no emoji: ;) <3").is_empty());
    }
}

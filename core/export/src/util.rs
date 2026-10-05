//! Kleine, dependency-freie Helfer für die Exporte: Base64, HTML-Escaping,
//! UTC-Zeitformatierung.

/// Base64-Standardalphabet (RFC 4648) mit Padding, für `data:`-URIs.
/// Bewusst selbst implementiert statt einer weiteren Dependency
/// (Qualitätsregel: stdlib zuerst).
pub fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = u32::from(chunk.get(1).copied().unwrap_or(0));
        let b2 = u32::from(chunk.get(2).copied().unwrap_or(0));
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[(triple >> 18) as usize & 0x3F] as char);
        out.push(ALPHABET[(triple >> 12) as usize & 0x3F] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(triple >> 6) as usize & 0x3F] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[triple as usize & 0x3F] as char
        } else {
            '='
        });
    }
    out
}

/// Escaped Text für HTML-Fließtext und Attributwerte.
pub fn html_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

/// Formatiert Unix-Millisekunden als `YYYY-MM-DD HH:MM UTC`.
pub fn format_utc(timestamp_ms: u64) -> String {
    let total_seconds = timestamp_ms / 1000;
    let days = i64::try_from(total_seconds / 86_400).unwrap_or(0);
    let secs_of_day = total_seconds % 86_400;
    let (year, month, day) = steps_store::civil_from_days(days);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02} UTC")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_kodiert_rfc4648_testvektoren() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn html_escape_ersetzt_sonderzeichen() {
        assert_eq!(
            html_escape(r#"<a href="x">Kaffee & 'Kuchen'</a>"#),
            "&lt;a href=&quot;x&quot;&gt;Kaffee &amp; &#39;Kuchen&#39;&lt;/a&gt;"
        );
    }

    #[test]
    fn format_utc_bekannte_zeitpunkte() {
        assert_eq!(format_utc(0), "1970-01-01 00:00 UTC");
        // 2026-09-02 00:00:00 UTC
        assert_eq!(format_utc(1_788_307_200_000), "2026-09-02 00:00 UTC");
        // Schaltjahr: 2024-02-29 12:30 UTC
        assert_eq!(format_utc(1_709_209_800_000), "2024-02-29 12:30 UTC");
    }
}

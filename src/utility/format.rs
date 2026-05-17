/// Formats an integer with Persian thousands separators (Western digits).
pub fn format_count(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push('،');
        }
        out.push(ch);
    }
    let formatted: String = out.chars().rev().collect();
    if n < 0 {
        format!("-{formatted}")
    } else {
        formatted
    }
}

/// Human-readable byte size in Persian (e.g. «1.2 مگ» — number first, short unit).
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        return format!("{} بایت", format_count(bytes as i64));
    }
    if bytes < MB {
        let value = bytes as f64 / KB as f64;
        return format!("{} کیلو", format_decimal(value, 1));
    }
    if bytes < GB {
        let value = bytes as f64 / MB as f64;
        return format!("{} مگ", format_decimal(value, 2));
    }
    let value = bytes as f64 / GB as f64;
    format!("{} گیگ", format_decimal(value, 2))
}

fn format_decimal(value: f64, max_fraction_digits: usize) -> String {
    let rounded = format!("{value:.max_fraction_digits$}");
    let trimmed = rounded.trim_end_matches('0').trim_end_matches('.');
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_count_groups_thousands() {
        assert_eq!(format_count(565), "565");
        assert_eq!(format_count(1234567), "1،234،567");
    }

    #[test]
    fn format_bytes_scales() {
        assert_eq!(format_bytes(512), "512 بایت");
        assert!(format_bytes(1536).ends_with("کیلو"));
        assert!(format_bytes(5 * 1024 * 1024).ends_with("مگ"));
        assert!(format_bytes(2 * 1024 * 1024 * 1024).ends_with("گیگ"));
    }
}

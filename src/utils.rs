pub fn format_currency(amount: f64) -> String {
    format!("{:.8}", amount)
}

pub fn truncate_hash(hash: &str, length: usize) -> String {
    if hash.len() <= length {
        hash.to_string()
    } else {
        format!("{}...{}", &hash[..length / 2], &hash[hash.len() - length / 2..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_currency() {
        assert_eq!(format_currency(123.456789), "123.45678900");
        assert_eq!(format_currency(0.1), "0.10000000");
    }

    #[test]
    fn test_truncate_hash() {
        let hash = "abcdef1234567890";
        assert_eq!(truncate_hash(hash, 8), "abcd...7890");
    }
}

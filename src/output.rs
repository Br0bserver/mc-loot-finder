/// Format an integer with thousands separators.
pub fn grouped(value: i64) -> String {
    let negative = value < 0;
    let digits = value.unsigned_abs().to_string();
    let mut result = String::with_capacity(digits.len() + digits.len() / 3 + usize::from(negative));
    if negative {
        result.push('-');
    }
    for (index, character) in digits.chars().enumerate() {
        if index != 0 && (digits.len() - index).is_multiple_of(3) {
            result.push(',');
        }
        result.push(character);
    }
    result
}

/// Format a count with a singular or plural label.
pub fn quantity(count: i64, singular: &str) -> String {
    format!(
        "{} {singular}{}",
        grouped(count),
        if count == 1 { "" } else { "s" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_grouped_numbers() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(5_000), "5,000");
        assert_eq!(grouped(-1_234_567), "-1,234,567");
    }

    #[test]
    fn formats_quantities() {
        assert_eq!(quantity(1, "chest"), "1 chest");
        assert_eq!(quantity(3, "chest"), "3 chests");
    }
}

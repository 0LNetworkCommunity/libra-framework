pub fn parse_epoch_ending_number(folder_name: &str) -> Option<u64> {
    // Format: "epoch_ending_273-.0338"
    folder_name
        .strip_prefix("epoch_ending_")?
        .split(['-', '.'])  // split on either - or .
        .next()?
        .parse()
        .ok()
}

pub fn parse_state_epoch_info(folder_name: &str) -> Option<(u64, u64)> {
    // Format: "state_epoch_173_ver_58967541.df7d"
    if !folder_name.starts_with("state_epoch_") {
        return None;
    }

    // Split on underscores: ["state", "epoch", "173", "ver", "58967541.df7d"]
    let parts: Vec<&str> = folder_name.split('_').collect();
    if parts.len() != 5 || parts[3] != "ver" {
        return None;
    }

    let epoch = parts[2].parse().ok()?;
    // Split version on dot to remove hash suffix
    let version = parts[4].split('.').next()?.parse().ok()?;

    Some((epoch, version))
}

pub fn parse_transaction_number(folder_name: &str) -> Option<u64> {
    // Format: "transaction_111500001-.d5aa"
    folder_name
        .strip_prefix("transaction_")?
        .split(['-', '.'])  // split on either - or .
        .next()?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_name_parsing() {
        assert_eq!(parse_epoch_ending_number("epoch_ending_273-.0338"), Some(273));
        assert_eq!(
            parse_state_epoch_info("state_epoch_173_ver_58967541.df7d"),
            Some((173, 58967541))
        );
        assert_eq!(parse_transaction_number("transaction_111500001-.d5aa"), Some(111500001));
    }

    #[test]
    fn test_state_epoch_parsing() {
        // Test valid format
        assert_eq!(
            parse_state_epoch_info("state_epoch_173_ver_58967541.df7d"),
            Some((173, 58967541))
        );

        // Test invalid formats
        assert_eq!(parse_state_epoch_info("state_epoch_173"), None);
        assert_eq!(parse_state_epoch_info("state_epoch_173_ver"), None);
        assert_eq!(parse_state_epoch_info("transaction_58967541"), None);
        assert_eq!(parse_state_epoch_info("epoch_ending_173"), None);
    }
}

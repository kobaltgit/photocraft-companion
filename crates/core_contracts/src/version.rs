/// Parses a SemVer-like version string and compares whether `latest` is strictly newer than `current`.
/// Supports optional leading 'v' or 'V' (e.g. "v1.2.3" vs "1.2.0") and ignores prerelease tags.
pub fn is_version_newer(latest: &str, current: &str) -> bool {
    let parse_nums = |v: &str| -> Vec<u64> {
        let cleaned = v.trim().trim_start_matches(['v', 'V']);
        // Cut prerelease/build suffix if any (e.g. 1.0.0-beta.1)
        let main_part = cleaned.split('-').next().unwrap_or(cleaned);
        main_part
            .split('.')
            .filter_map(|p| p.parse::<u64>().ok())
            .collect()
    };

    let l_parts = parse_nums(latest);
    let c_parts = parse_nums(current);

    if l_parts.is_empty() || c_parts.is_empty() {
        return false;
    }

    let max_len = l_parts.len().max(c_parts.len());
    for i in 0..max_len {
        let l_val = *l_parts.get(i).unwrap_or(&0);
        let c_val = *c_parts.get(i).unwrap_or(&0);
        if l_val > c_val {
            return true;
        } else if l_val < c_val {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_version_newer() {
        assert!(is_version_newer("0.2.0", "0.1.0"));
        assert!(is_version_newer("1.0.0", "0.9.9"));
        assert!(is_version_newer("0.1.1", "0.1.0"));
        assert!(!is_version_newer("0.1.0", "0.1.0"));
        assert!(!is_version_newer("1.0.0", "1.0.0"));
        assert!(!is_version_newer("0.1.0", "0.2.0"));
        assert!(!is_version_newer("0.9.9", "1.0.0"));
        assert!(is_version_newer("v0.2.0", "0.1.0"));
        assert!(is_version_newer("0.2.0", "v0.1.0"));
        assert!(is_version_newer("V1.0.0", "v0.9.9"));
        assert!(!is_version_newer("v0.1.0", "v0.1.0"));
        assert!(is_version_newer("1.0.0-rc.1", "0.9.0"));
        assert!(is_version_newer("0.2.0-beta", "0.1.9"));
        assert!(is_version_newer("1.0.1", "1.0"));
        assert!(!is_version_newer("1.0", "1.0.1"));
        assert!(!is_version_newer("1.0", "1.0.0"));
    }
}

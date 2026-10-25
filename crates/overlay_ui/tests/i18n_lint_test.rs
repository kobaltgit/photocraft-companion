use std::fs;
use std::path::Path;

#[test]
fn test_no_unlocalized_cyrillic_in_ui_crates() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = Path::new(manifest_dir).join("src");

    // Files that should never contain any Cyrillic text at all
    let strictly_latin_files = [
        "icons.rs",
        "store.rs",
        "theme.rs",
        "toolstrip.rs",
        "updater.rs",
        "win32_window.rs",
    ];

    for file_name in strictly_latin_files {
        let file_path = src_dir.join(file_name);
        if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));
            for (line_num, line) in content.lines().enumerate() {
                // Ignore comments
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }
                for ch in line.chars() {
                    assert!(
                        !('\u{0400}'..='\u{04FF}').contains(&ch),
                        "Found unlocalized Cyrillic character '{}' in strictly Latin file {}:{}",
                        ch,
                        file_name,
                        line_num + 1
                    );
                }
            }
        }
    }
}

#[test]
fn test_manager_and_offline_modal_cyrillic_strictly_localized() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = Path::new(manifest_dir).join("src");

    let localized_files = ["manager.rs", "offline_modal.rs"];

    for file_name in localized_files {
        let file_path = src_dir.join(file_name);
        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));

        let mut in_test_module = false;
        let mut t_depth: i32 = 0;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.contains("#[cfg(test)]") || trimmed.contains("mod tests") {
                in_test_module = true;
            }
            if in_test_module {
                continue;
            }

            // Skip comments
            if trimmed.starts_with("//") {
                continue;
            }

            let opens_t = line.contains(".t(") || line.contains("t(");
            if opens_t || t_depth > 0 {
                let open_parens = line.matches('(').count() as i32;
                let close_parens = line.matches(')').count() as i32;
                t_depth += open_parens - close_parens;
                if t_depth < 0 {
                    t_depth = 0;
                }
            }

            let has_cyrillic = line.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));
            if has_cyrillic {
                let is_lang_selector = line.contains("Русский (RU)");
                let is_in_t_call = opens_t || t_depth > 0;

                assert!(
                    is_in_t_call || is_lang_selector,
                    "Found unlocalized Cyrillic text outside of .t(...) call in {}:{}: '{}'",
                    file_name,
                    line_num + 1,
                    trimmed
                );
            }
        }
    }
}

#[test]
fn test_all_t_translation_pairs_integrity() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_dir = Path::new(manifest_dir).join("src");

    let localized_files = ["manager.rs", "offline_modal.rs"];
    let mut total_pairs_checked = 0;

    for file_name in localized_files {
        let file_path = src_dir.join(file_name);
        let content_full = fs::read_to_string(&file_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));

        // Only scan production code before #[cfg(test)]
        let content = if let Some(test_idx) = content_full.find("#[cfg(test)]") {
            &content_full[..test_idx]
        } else {
            &content_full[..]
        };

        let mut idx = 0;

        while let Some(pos) = content[idx..].find(".t(") {
            let start = idx + pos + 3; // right after .t(
            if let Some((en, ru, next_idx)) = parse_two_string_args(content, start) {
                total_pairs_checked += 1;

                // 1. Neither variant can be empty
                assert!(
                    !en.trim().is_empty(),
                    "Empty English translation in {}: near index {}",
                    file_name,
                    start
                );
                assert!(
                    !ru.trim().is_empty(),
                    "Empty Russian translation in {}: near index {}",
                    file_name,
                    start
                );

                // 2. English variant must have Latin letters
                let has_latin = en.chars().any(|c| c.is_ascii_alphabetic());
                assert!(
                    has_latin,
                    "English translation has no Latin characters in {}: '{}'",
                    file_name, en
                );

                // 3. Russian variant must have Cyrillic letters (or be a technical port/IP string)
                let is_technical = ru.contains("127.0.0.1") || ru.contains("18492");
                let has_cyrillic = ru.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));
                assert!(
                    has_cyrillic || is_technical,
                    "Russian translation missing Cyrillic characters in {}: '{}'",
                    file_name,
                    ru
                );

                // 4. English and Russian variants must not be identical (prevents copy-paste mistakes)
                if en.len() > 4 && !is_technical {
                    assert_ne!(
                        en.trim(),
                        ru.trim(),
                        "English and Russian strings are identical in {}: '{}'",
                        file_name,
                        en
                    );
                }

                // 5. No placeholder tokens
                let placeholders = ["TODO", "FIXME", "???", "untranslated"];
                for ph in placeholders {
                    assert!(
                        !en.contains(ph) && !ru.contains(ph),
                        "Found placeholder '{}' in translation pair in {}: En: '{}', Ru: '{}'",
                        ph,
                        file_name,
                        en,
                        ru
                    );
                }

                idx = next_idx;
            } else {
                idx = start + 1;
            }
        }
    }

    assert!(
        total_pairs_checked >= 20,
        "Expected to validate at least 20 UI translation pairs, but only found {}",
        total_pairs_checked
    );
}

fn parse_two_string_args(content: &str, start: usize) -> Option<(String, String, usize)> {
    let remainder = &content[start..];
    // Find first quote '"'
    let q1 = remainder.find('"')?;
    let after_q1 = &remainder[q1 + 1..];
    let q2 = find_closing_quote(after_q1)?;
    let arg1 = &after_q1[..q2];

    // Find comma after first string
    let after_arg1 = &after_q1[q2 + 1..];
    let comma = after_arg1.find(',')?;

    // Find quote for second string
    let after_comma = &after_arg1[comma + 1..];
    let q3 = after_comma.find('"')?;
    let after_q3 = &after_comma[q3 + 1..];
    let q4 = find_closing_quote(after_q3)?;
    let arg2 = &after_q3[..q4];

    let final_pos = start + q1 + 1 + q2 + 1 + comma + 1 + q3 + 1 + q4 + 1;
    Some((arg1.to_string(), arg2.to_string(), final_pos))
}

fn find_closing_quote(s: &str) -> Option<usize> {
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return Some(i);
        }
    }
    None
}

#[test]
fn test_no_cjk_or_unsupported_glyphs_in_ui_dictionary() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mod_rs_path = Path::new(manifest_dir)
        .join("src")
        .join("i18n")
        .join("mod.rs");
    let content = fs::read_to_string(&mod_rs_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", mod_rs_path.display()));

    // CJK character ranges: Hiragana, Katakana, CJK Unified Ideographs, Hangul, etc.
    let is_cjk = |c: char| {
        matches!(c,
            '\u{3040}'..='\u{30ff}' | // Hiragana & Katakana
            '\u{3400}'..='\u{4dbf}' | // CJK Unified Ideographs Extension A
            '\u{4e00}'..='\u{9fff}' | // CJK Unified Ideographs
            '\u{f900}'..='\u{faff}' | // CJK Compatibility Ideographs
            '\u{ff66}'..='\u{ff9f}' | // Halfwidth Katakana
            '\u{ac00}'..='\u{d7af}'   // Hangul Syllables
        )
    };

    for (line_num, line) in content.lines().enumerate() {
        for ch in line.chars() {
            assert!(
                !is_cjk(ch),
                "Found unsupported CJK glyph '{}' (U+{:04X}) in {} at line {}: tofu box risk in egui!",
                ch,
                ch as u32,
                mod_rs_path.display(),
                line_num + 1
            );
        }
    }
}

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Auto,
    En,
    Ru,
    De,
    Fr,
    Es,
    It,
    Ja,
    Zh,
    Ko,
    Pt,
}

impl Language {
    pub fn from_str_code(code: &str) -> Self {
        match code.trim().to_ascii_lowercase().as_str() {
            "en" | "english" => Language::En,
            "ru" | "russian" => Language::Ru,
            "de" | "german" => Language::De,
            "fr" | "french" => Language::Fr,
            "es" | "spanish" => Language::Es,
            "it" | "italian" => Language::It,
            "ja" | "japanese" => Language::Ja,
            "zh" | "chinese" => Language::Zh,
            "ko" | "korean" => Language::Ko,
            "pt" | "portuguese" => Language::Pt,
            _ => Language::Auto,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::Auto => "auto",
            Language::En => "en",
            Language::Ru => "ru",
            Language::De => "de",
            Language::Fr => "fr",
            Language::Es => "es",
            Language::It => "it",
            Language::Ja => "ja",
            Language::Zh => "zh",
            Language::Ko => "ko",
            Language::Pt => "pt",
        }
    }

    /// Resolve effective language: if Auto, reads PhotoCraft preferences or defaults to En
    pub fn resolve(&self, photocraft_prefs_path: Option<&Path>) -> Language {
        if *self != Language::Auto {
            return *self;
        }

        // Try reading from given path
        if let Some(path) = photocraft_prefs_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(lang_str) = v
                        .get("interface")
                        .and_then(|i| i.get("language"))
                        .and_then(|l| l.as_str())
                    {
                        let detected = Language::from_str_code(lang_str);
                        if detected != Language::Auto {
                            return detected;
                        }
                    }
                }
            }
        }

        // Try default portable preferences path
        let default_portable = Path::new(
            r"D:\Portable Soft\ArtCraft\photocraft-0.6.0-windows-x64-portable\PhotoCraftData\preferences.json",
        );
        if default_portable.exists() {
            if let Ok(content) = std::fs::read_to_string(default_portable) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(lang_str) = v
                        .get("interface")
                        .and_then(|i| i.get("language"))
                        .and_then(|l| l.as_str())
                    {
                        let detected = Language::from_str_code(lang_str);
                        if detected != Language::Auto {
                            return detected;
                        }
                    }
                }
            }
        }

        Language::En
    }

    /// Suffix for cutout layers
    pub fn no_bg_suffix(&self) -> &'static str {
        match self {
            Language::En | Language::Auto => "No BG",
            Language::Ru => "Без фона",
            Language::De => "Ohne Hintergrund",
            Language::Fr => "Sans fond",
            Language::Es => "Sin fondo",
            Language::It => "Senza sfondo",
            Language::Ja => "背景なし",
            Language::Zh => "无背景",
            Language::Ko => "배경 없음",
            Language::Pt => "Sem fundo",
        }
    }

    /// Suffix for upscaled layers
    pub fn upscale_suffix(&self) -> &'static str {
        match self {
            Language::En | Language::Auto => "Upscaled 2x",
            Language::Ru => "Увеличение 2x",
            Language::De => "Hochskaliert 2x",
            Language::Fr => "Agrandissement 2x",
            Language::Es => "Aumentado 2x",
            Language::It => "Ingrandito 2x",
            Language::Ja => "2倍拡大",
            Language::Zh => "超分辨率 2x",
            Language::Ko => "2배 업스케일",
            Language::Pt => "Ampliado 2x",
        }
    }

    /// Suffix for inpaint / generative fill layers
    pub fn inpaint_suffix(&self) -> &'static str {
        match self {
            Language::En | Language::Auto => "Gen Fill",
            Language::Ru => "Генеративная заливка",
            Language::De => "Generatives Füllen",
            Language::Fr => "Remplissage génératif",
            Language::Es => "Relleno generativo",
            Language::It => "Riempimento generativo",
            Language::Ja => "生成塗りつぶし",
            Language::Zh => "生成式填充",
            Language::Ko => "생성형 채우기",
            Language::Pt => "Preenchimento generativo",
        }
    }

    /// Button label for remove bg action
    pub fn remove_bg_label(&self) -> &'static str {
        match self {
            Language::En | Language::Auto => "🪄 Remove BG",
            Language::Ru => "🪄 Удалить фон",
            Language::De => "🪄 Hintergrund entfernen",
            Language::Fr => "🪄 Supprimer l'arrière-plan",
            Language::Es => "🪄 Quitar fondo",
            Language::It => "🪄 Rimuovi sfondo",
            Language::Ja => "🪄 背景削除",
            Language::Zh => "🪄 抠图背景",
            Language::Ko => "🪄 배경 제거",
            Language::Pt => "🪄 Remover fundo",
        }
    }

    /// Format cutout layer name preserving source name and preventing duplicate suffixes
    pub fn format_cutout_name(&self, source_name: &str) -> String {
        let suffix = self.no_bg_suffix();
        let trimmed = source_name.trim();

        let all_suffixes = [
            "No BG",
            "Cutout",
            "Без фона",
            "Ohne Hintergrund",
            "Sans fond",
            "Sin fondo",
            "Senza sfondo",
            "背景なし",
            "无背景",
            "배경 없음",
            "Sem fundo",
        ];

        for s in all_suffixes {
            let pattern1 = format!("({s})");
            let pattern2 = format!("[{s}]");
            let pattern3 = format!("- {s}");
            if trimmed.ends_with(&pattern1)
                || trimmed.ends_with(&pattern2)
                || trimmed.ends_with(&pattern3)
            {
                return trimmed.to_string();
            }
        }

        format!("{trimmed} ({suffix})")
    }

    /// Format upscale layer name
    pub fn format_upscale_name(&self, source_name: &str) -> String {
        let suffix = self.upscale_suffix();
        let trimmed = source_name.trim();
        format!("{trimmed} ({suffix})")
    }

    /// Format generative fill layer name
    pub fn format_inpaint_name(&self, source_name: &str) -> String {
        let suffix = self.inpaint_suffix();
        let trimmed = source_name.trim();
        format!("{trimmed} ({suffix})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str_code() {
        assert_eq!(Language::from_str_code("en"), Language::En);
        assert_eq!(Language::from_str_code("english"), Language::En);
        assert_eq!(Language::from_str_code("EN "), Language::En);
        assert_eq!(Language::from_str_code("ru"), Language::Ru);
        assert_eq!(Language::from_str_code("russian"), Language::Ru);
        assert_eq!(Language::from_str_code("de"), Language::De);
        assert_eq!(Language::from_str_code("fr"), Language::Fr);
        assert_eq!(Language::from_str_code("es"), Language::Es);
        assert_eq!(Language::from_str_code("it"), Language::It);
        assert_eq!(Language::from_str_code("ja"), Language::Ja);
        assert_eq!(Language::from_str_code("zh"), Language::Zh);
        assert_eq!(Language::from_str_code("ko"), Language::Ko);
        assert_eq!(Language::from_str_code("pt"), Language::Pt);
        assert_eq!(Language::from_str_code("unknown_xyz"), Language::Auto);
        assert_eq!(Language::from_str_code(""), Language::Auto);
    }

    #[test]
    fn test_language_code() {
        assert_eq!(Language::Auto.code(), "auto");
        assert_eq!(Language::En.code(), "en");
        assert_eq!(Language::Ru.code(), "ru");
        assert_eq!(Language::De.code(), "de");
        assert_eq!(Language::Fr.code(), "fr");
        assert_eq!(Language::Es.code(), "es");
        assert_eq!(Language::It.code(), "it");
        assert_eq!(Language::Ja.code(), "ja");
        assert_eq!(Language::Zh.code(), "zh");
        assert_eq!(Language::Ko.code(), "ko");
        assert_eq!(Language::Pt.code(), "pt");
    }

    #[test]
    fn test_language_no_bg_suffix() {
        assert_eq!(Language::En.no_bg_suffix(), "No BG");
        assert_eq!(Language::Auto.no_bg_suffix(), "No BG");
        assert_eq!(Language::Ru.no_bg_suffix(), "Без фона");
        assert_eq!(Language::De.no_bg_suffix(), "Ohne Hintergrund");
        assert_eq!(Language::Fr.no_bg_suffix(), "Sans fond");
        assert_eq!(Language::Es.no_bg_suffix(), "Sin fondo");
        assert_eq!(Language::It.no_bg_suffix(), "Senza sfondo");
        assert_eq!(Language::Ja.no_bg_suffix(), "背景なし");
        assert_eq!(Language::Zh.no_bg_suffix(), "无背景");
        assert_eq!(Language::Ko.no_bg_suffix(), "배경 없음");
        assert_eq!(Language::Pt.no_bg_suffix(), "Sem fundo");
    }

    #[test]
    fn test_language_remove_bg_label() {
        assert!(Language::En.remove_bg_label().contains("Remove BG"));
        assert!(Language::Ru.remove_bg_label().contains("Удалить фон"));
        assert!(Language::De
            .remove_bg_label()
            .contains("Hintergrund entfernen"));
    }

    #[test]
    fn test_language_resolve() {
        // Explicit language always resolves to itself
        assert_eq!(Language::Ru.resolve(None), Language::Ru);
        assert_eq!(Language::De.resolve(None), Language::De);

        // Auto resolves from preferences JSON
        let temp_dir = std::env::temp_dir().join(format!(
            "i18n_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let prefs_file = temp_dir.join("preferences.json");
        std::fs::write(&prefs_file, r#"{"interface": {"language": "ru"}}"#).unwrap();

        assert_eq!(Language::Auto.resolve(Some(&prefs_file)), Language::Ru);

        // Invalid JSON resolves to En fallback
        let invalid_prefs = temp_dir.join("invalid.json");
        std::fs::write(&invalid_prefs, "not json").unwrap();
        assert_eq!(Language::Auto.resolve(Some(&invalid_prefs)), Language::En);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_format_cutout_name() {
        let ru = Language::Ru;
        let en = Language::En;

        // Normal layer
        assert_eq!(ru.format_cutout_name("Portrait"), "Portrait (Без фона)");
        assert_eq!(en.format_cutout_name("Portrait"), "Portrait (No BG)");

        // Deduplication when already has suffix in parentheses
        assert_eq!(
            ru.format_cutout_name("Portrait (Без фона)"),
            "Portrait (Без фона)"
        );
        assert_eq!(
            en.format_cutout_name("Portrait (No BG)"),
            "Portrait (No BG)"
        );
        assert_eq!(
            ru.format_cutout_name("Portrait (No BG)"),
            "Portrait (No BG)"
        );

        // Deduplication in brackets
        assert_eq!(
            en.format_cutout_name("Portrait [No BG]"),
            "Portrait [No BG]"
        );
        assert_eq!(
            ru.format_cutout_name("Portrait [Без фона]"),
            "Portrait [Без фона]"
        );

        // Deduplication with dash
        assert_eq!(
            en.format_cutout_name("Portrait - No BG"),
            "Portrait - No BG"
        );
        assert_eq!(
            ru.format_cutout_name("Portrait - Без фона"),
            "Portrait - Без фона"
        );

        // Whitespace handling
        assert_eq!(en.format_cutout_name("  Subject  "), "Subject (No BG)");
    }

    #[test]
    fn test_no_hardcoded_fallback_strings() {
        let non_english_languages = [
            Language::Ru,
            Language::De,
            Language::Fr,
            Language::Es,
            Language::It,
            Language::Ja,
            Language::Zh,
            Language::Ko,
            Language::Pt,
        ];

        let en_suffix = Language::En.no_bg_suffix();
        let en_label = Language::En.remove_bg_label();

        let mut seen_suffixes = std::collections::HashSet::new();
        seen_suffixes.insert(en_suffix);

        let mut seen_labels = std::collections::HashSet::new();
        seen_labels.insert(en_label);

        for lang in non_english_languages {
            let suffix = lang.no_bg_suffix();
            let label = lang.remove_bg_label();

            // Suffix must not be empty and must not be a lazy hardcoded fallback to English
            assert!(
                !suffix.trim().is_empty(),
                "Language {:?} has empty suffix",
                lang
            );
            assert_ne!(
                suffix, en_suffix,
                "Language {:?} has hardcoded English suffix",
                lang
            );
            assert!(
                seen_suffixes.insert(suffix),
                "Duplicate suffix '{}' for language {:?}",
                suffix,
                lang
            );

            // Label must not be empty and must not be a lazy hardcoded fallback to English
            assert!(
                !label.trim().is_empty(),
                "Language {:?} has empty label",
                lang
            );
            assert_ne!(
                label, en_label,
                "Language {:?} has hardcoded English label",
                lang
            );
            assert!(
                seen_labels.insert(label),
                "Duplicate label '{}' for language {:?}",
                label,
                lang
            );
        }
    }
}

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "en")]
    En,
    #[serde(rename = "ja")]
    Ja,
    #[serde(rename = "zh-hans", alias = "zh", alias = "zh-cn")]
    ZhHans,
    #[serde(rename = "zh-hant", alias = "zh-tw", alias = "zh-hk")]
    ZhHant,
    #[serde(rename = "es")]
    Es,
    #[serde(rename = "ru")]
    Ru,
    #[serde(rename = "uk")]
    Uk,
    #[serde(rename = "sr")]
    Sr,
    #[serde(rename = "cs")]
    Cs,
    #[serde(rename = "fr")]
    Fr,
    #[serde(rename = "id")]
    Id,
    #[serde(rename = "ko")]
    Ko,
    #[serde(rename = "pl")]
    Pl,
    #[serde(rename = "de")]
    De,
    #[serde(rename = "pt-br", alias = "pt", alias = "pt-pt")]
    PtBr,
    #[serde(rename = "el")]
    El,
    #[serde(rename = "nl")]
    Nl,
    #[serde(rename = "it")]
    It,
    #[serde(rename = "hu")]
    Hu,
    #[serde(rename = "tr")]
    Tr,
    #[serde(rename = "vi")]
    Vi,
    #[serde(rename = "hi")]
    Hi,
}

impl Language {
    /// Backward-compatibility alias for Zh (defaults to Simplified Chinese)
    #[allow(non_upper_case_globals)]
    pub const Zh: Language = Language::ZhHans;

    /// Backward-compatibility alias for Pt (defaults to Brazilian Portuguese)
    #[allow(non_upper_case_globals)]
    pub const Pt: Language = Language::PtBr;

    /// All 15 clean UI-supported languages (Latin and Cyrillic glyphs fully present in egui)
    pub const UI_SUPPORTED: [Language; 15] = [
        Language::En,
        Language::Ru,
        Language::Fr,
        Language::It,
        Language::De,
        Language::Es,
        Language::PtBr,
        Language::Pl,
        Language::Cs,
        Language::Nl,
        Language::Tr,
        Language::Hu,
        Language::Uk,
        Language::Sr,
        Language::Id,
    ];

    /// All 22 active host languages supported by PhotoCraft (excluding Auto)
    pub const ALL_SUPPORTED: [Language; 22] = [
        Language::En,
        Language::Ja,
        Language::ZhHans,
        Language::ZhHant,
        Language::Es,
        Language::Ru,
        Language::Uk,
        Language::Sr,
        Language::Cs,
        Language::Fr,
        Language::Id,
        Language::Ko,
        Language::Pl,
        Language::De,
        Language::PtBr,
        Language::El,
        Language::Nl,
        Language::It,
        Language::Hu,
        Language::Tr,
        Language::Vi,
        Language::Hi,
    ];

    pub fn is_ui_supported(&self) -> bool {
        matches!(
            self,
            Language::En
                | Language::Ru
                | Language::Fr
                | Language::It
                | Language::De
                | Language::Es
                | Language::PtBr
                | Language::Pl
                | Language::Cs
                | Language::Nl
                | Language::Tr
                | Language::Hu
                | Language::Uk
                | Language::Sr
                | Language::Id
        )
    }

    /// Returns a glyph-safe language for the UI runtime (falls back to En for CJK/Hindi/Greek/unsupported glyphs)
    pub fn safe_ui_language(&self) -> Language {
        if self.is_ui_supported() {
            *self
        } else {
            Language::En
        }
    }

    pub fn from_str_code(code: &str) -> Self {
        match code.trim().to_ascii_lowercase().as_str() {
            "en" | "english" => Language::En,
            "ja" | "japanese" => Language::Ja,
            "zh-hans" | "zh" | "zh-cn" | "chinese" => Language::ZhHans,
            "zh-hant" | "zh-tw" | "zh-hk" => Language::ZhHant,
            "es" | "spanish" => Language::Es,
            "ru" | "russian" => Language::Ru,
            "uk" | "ukrainian" => Language::Uk,
            "sr" | "serbian" => Language::Sr,
            "cs" | "czech" => Language::Cs,
            "fr" | "french" => Language::Fr,
            "id" | "indonesian" => Language::Id,
            "ko" | "korean" => Language::Ko,
            "pl" | "polish" => Language::Pl,
            "de" | "german" => Language::De,
            "pt-br" | "pt" | "pt-pt" | "portuguese" => Language::PtBr,
            "el" | "greek" => Language::El,
            "nl" | "dutch" => Language::Nl,
            "it" | "italian" => Language::It,
            "hu" | "hungarian" => Language::Hu,
            "tr" | "turkish" => Language::Tr,
            "vi" | "vietnamese" => Language::Vi,
            "hi" | "hindi" => Language::Hi,
            "auto" => Language::Auto,
            other => Self::from_bcp47_tag(other).unwrap_or(Language::Auto),
        }
    }

    /// Parse a BCP 47 or POSIX locale tag (e.g. "ru-KZ", "ru_RU", "ru_RU.UTF-8", "ru.UTF-8", "cs-CZ", "sr_RS@latin") into a supported Language.
    /// Strips encoding suffixes (.UTF-8), modifiers (@latin), and regional/script variants (e.g. "ru-KZ" -> "ru").
    pub fn from_bcp47_tag(tag: &str) -> Option<Language> {
        let clean = tag
            .trim()
            .split(['.', '@'])
            .next()
            .unwrap_or("")
            .replace('_', "-")
            .to_ascii_lowercase();
        if clean.is_empty() {
            return None;
        }

        if matches!(clean.as_str(), "c" | "posix") {
            return Some(Language::En);
        }

        // Direct matching for exact full tags
        let exact = match clean.as_str() {
            "zh-hans" | "zh-cn" | "zh-sg" => Some(Language::ZhHans),
            "zh-hant" | "zh-tw" | "zh-hk" | "zh-mo" => Some(Language::ZhHant),
            "pt-br" => Some(Language::PtBr),
            "pt" | "pt-pt" => Some(Language::PtBr),
            _ => None,
        };
        if exact.is_some() {
            return exact;
        }

        // Check primary language subtag (before '-' or '_')
        let primary = clean.split('-').next().unwrap_or(&clean);
        match primary {
            "en" => Some(Language::En),
            "ja" => Some(Language::Ja),
            "zh" => Some(Language::ZhHans),
            "es" => Some(Language::Es),
            "ru" => Some(Language::Ru),
            "uk" => Some(Language::Uk),
            "sr" => Some(Language::Sr),
            "cs" => Some(Language::Cs),
            "fr" => Some(Language::Fr),
            "id" => Some(Language::Id),
            "ko" => Some(Language::Ko),
            "pl" => Some(Language::Pl),
            "de" => Some(Language::De),
            "pt" => Some(Language::PtBr),
            "el" => Some(Language::El),
            "nl" => Some(Language::Nl),
            "it" => Some(Language::It),
            "hu" => Some(Language::Hu),
            "tr" => Some(Language::Tr),
            "vi" => Some(Language::Vi),
            "hi" => Some(Language::Hi),
            _ => None,
        }
    }

    /// Detect the primary system language using OS locale preferences.
    /// Iterates through `sys_locale::get_locales()` and falls back to `Language::En`.
    pub fn system_language() -> Language {
        let locales: Vec<String> = sys_locale::get_locales().collect();
        for loc in &locales {
            let lower = loc.to_lowercase();
            if lower.starts_with("ru") || lower.starts_with("uk") || lower.starts_with("be") {
                return Language::Ru;
            }
        }
        for locale in &locales {
            if let Some(lang) = Self::from_bcp47_tag(locale) {
                return lang;
            }
        }
        Language::En
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::Auto => "auto",
            Language::En => "en",
            Language::Ja => "ja",
            Language::ZhHans => "zh-hans",
            Language::ZhHant => "zh-hant",
            Language::Es => "es",
            Language::Ru => "ru",
            Language::Uk => "uk",
            Language::Sr => "sr",
            Language::Cs => "cs",
            Language::Fr => "fr",
            Language::Id => "id",
            Language::Ko => "ko",
            Language::Pl => "pl",
            Language::De => "de",
            Language::PtBr => "pt-br",
            Language::El => "el",
            Language::Nl => "nl",
            Language::It => "it",
            Language::Hu => "hu",
            Language::Tr => "tr",
            Language::Vi => "vi",
            Language::Hi => "hi",
        }
    }

    /// Native display name for UI language selectors
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Auto => "Auto",
            Language::En => "English",
            Language::Ja => "日本語",
            Language::ZhHans => "简体中文",
            Language::ZhHant => "繁體中文",
            Language::Es => "Español",
            Language::Ru => "Русский",
            Language::Uk => "Українська",
            Language::Sr => "Srpski",
            Language::Cs => "Čeština",
            Language::Fr => "Français",
            Language::Id => "Bahasa Indonesia",
            Language::Ko => "한국어",
            Language::Pl => "Polski",
            Language::De => "Deutsch",
            Language::PtBr => "Português (Brasil)",
            Language::El => "Ελληνικά",
            Language::Nl => "Nederlands",
            Language::It => "Italiano",
            Language::Hu => "Magyar",
            Language::Tr => "Türkçe",
            Language::Vi => "Tiếng Việt",
            Language::Hi => "हिन्दी",
        }
    }

    /// Full label for language selection menus including native name and ISO code
    pub fn selector_label(&self) -> String {
        match self {
            Language::Auto => "Auto (PhotoCraft)".to_string(),
            Language::En => "English (EN)".to_string(),
            Language::Ja => "日本語 (JA)".to_string(),
            Language::ZhHans => "简体中文 (ZH-HANS)".to_string(),
            Language::ZhHant => "繁體中文 (ZH-HANT)".to_string(),
            Language::Es => "Español (ES)".to_string(),
            Language::Ru => "Русский (RU)".to_string(),
            Language::Uk => "Українська (UK)".to_string(),
            Language::Sr => "Srpski (SR)".to_string(),
            Language::Cs => "Čeština (CS)".to_string(),
            Language::Fr => "Français (FR)".to_string(),
            Language::Id => "Bahasa Indonesia (ID)".to_string(),
            Language::Ko => "한국어 (KO)".to_string(),
            Language::Pl => "Polski (PL)".to_string(),
            Language::De => "Deutsch (DE)".to_string(),
            Language::PtBr => "Português (PT-BR)".to_string(),
            Language::El => "Ελληνικά (EL)".to_string(),
            Language::Nl => "Nederlands (NL)".to_string(),
            Language::It => "Italiano (IT)".to_string(),
            Language::Hu => "Magyar (HU)".to_string(),
            Language::Tr => "Türkçe (TR)".to_string(),
            Language::Vi => "Tiếng Việt (VI)".to_string(),
            Language::Hi => "हिन्दी (HI)".to_string(),
        }
    }

    /// Resolve effective language: if Auto, reads PhotoCraft preferences or falls back to system locale
    pub fn resolve(&self, photocraft_prefs_path: Option<&Path>) -> Language {
        if *self != Language::Auto {
            return *self;
        }

        // Helper to extract language from a preferences.json path
        let try_read_prefs = |path: &Path| -> Option<Language> {
            let actual_path = if path.is_dir() {
                path.join("preferences.json")
            } else if path
                .file_name()
                .map(|n| n == "photocraft.exe" || n == "photocraft")
                .unwrap_or(false)
            {
                path.parent()
                    .map(|p| p.join("PhotoCraftData").join("preferences.json"))?
            } else {
                path.to_path_buf()
            };

            if let Ok(content) = std::fs::read_to_string(&actual_path) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(lang_str) = v
                        .get("interface")
                        .and_then(|i| i.get("language"))
                        .and_then(|l| l.as_str())
                    {
                        let detected = Language::from_str_code(lang_str);
                        if detected != Language::Auto {
                            return Some(detected);
                        }
                    }
                }
            }
            None
        };

        // Try reading from given path
        if let Some(path) = photocraft_prefs_path {
            if let Some(lang) = try_read_prefs(path) {
                return lang;
            }
            if path.exists() {
                // Explicit path was provided and exists, but has no valid language (e.g. language is "auto")
                return Self::system_language();
            }
        }

        // Try candidate locations for portable and installed PhotoCraft
        let candidate_paths = [
            r"D:\Portable Soft\ArtCraft\photocraft-portable\PhotoCraftData\preferences.json",
            r"D:\Portable Soft\ArtCraft\photocraft-0.8.2-windows-x64-portable\PhotoCraftData\preferences.json",
            r"D:\Portable Soft\ArtCraft\photocraft-0.6.0-windows-x64-portable\PhotoCraftData\preferences.json",
        ];

        for candidate in candidate_paths {
            let p = Path::new(candidate);
            if p.exists() {
                if let Some(lang) = try_read_prefs(p) {
                    return lang;
                }
                // Preferences exist in portable installation, language is "auto"
                return Self::system_language();
            }
        }

        // Try APPDATA / LOCALAPPDATA
        if let Ok(appdata) = std::env::var("APPDATA") {
            let p = Path::new(&appdata)
                .join("PhotoCraft")
                .join("preferences.json");
            if p.exists() {
                if let Some(lang) = try_read_prefs(&p) {
                    return lang;
                }
                return Self::system_language();
            }
        }
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            let p = Path::new(&localappdata)
                .join("PhotoCraft")
                .join("preferences.json");
            if p.exists() {
                if let Some(lang) = try_read_prefs(&p) {
                    return lang;
                }
                return Self::system_language();
            }
        }

        Self::system_language()
    }
}

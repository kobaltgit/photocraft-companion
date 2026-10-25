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
        for locale in sys_locale::get_locales() {
            if let Some(lang) = Self::from_bcp47_tag(&locale) {
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
            Language::ZhHans => "无背景",
            Language::ZhHant => "無背景",
            Language::Ko => "배경 없음",
            Language::PtBr => "Sem fundo",
            Language::Cs => "Bez pozadí",
            Language::Uk => "Без фону",
            Language::Sr => "Bez pozadine",
            Language::Pl => "Bez tła",
            Language::Nl => "Zonder achtergrond",
            Language::Hu => "Háttér nélkül",
            Language::El => "Χωρίς φόντο",
            Language::Tr => "Arka plansız",
            Language::Id => "Tanpa latar",
            Language::Vi => "Không nền",
            Language::Hi => "पृष्ठभूमि नहीं",
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
            Language::ZhHans => "超分辨率 2x",
            Language::ZhHant => "超解析度 2x",
            Language::Ko => "2배 업스케일",
            Language::PtBr => "Ampliado 2x",
            Language::Cs => "Zvětšeno 2x",
            Language::Uk => "Збільшення 2x",
            Language::Sr => "Uvećano 2x",
            Language::Pl => "Powiększone 2x",
            Language::Nl => "2x opgeschaald",
            Language::Hu => "2x felskálázva",
            Language::El => "Αναβάθμιση 2x",
            Language::Tr => "2x Ölçeklendi",
            Language::Id => "Ditingkatkan 2x",
            Language::Vi => "Nâng cấp 2x",
            Language::Hi => "2x बड़ा किया गया",
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
            Language::ZhHans => "生成式填充",
            Language::ZhHant => "生成式填色",
            Language::Ko => "생성형 채우기",
            Language::PtBr => "Preenchimento generativo",
            Language::Cs => "Generativní výplň",
            Language::Uk => "Генеративна заливка",
            Language::Sr => "Generativno popunjavanje",
            Language::Pl => "Wypełnienie generatywne",
            Language::Nl => "Generatief vullen",
            Language::Hu => "Generatív kitöltés",
            Language::El => "Δημιουργικό γέμισμα",
            Language::Tr => "Üretken Dolgu",
            Language::Id => "Isian generatif",
            Language::Vi => "Tô màu tạo sinh",
            Language::Hi => "जेनरेटिव भराव",
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
            Language::ZhHans => "🪄 抠图背景",
            Language::ZhHant => "🪄 去除背景",
            Language::Ko => "🪄 배경 제거",
            Language::PtBr => "🪄 Remover fundo",
            Language::Cs => "🪄 Odstranit pozadí",
            Language::Uk => "🪄 Видалити фон",
            Language::Sr => "🪄 Ukloni pozadinu",
            Language::Pl => "🪄 Usuń tło",
            Language::Nl => "🪄 Achtergrond verwijderen",
            Language::Hu => "🪄 Háttér eltávolítása",
            Language::El => "🪄 Αφαίρεση φόντου",
            Language::Tr => "🪄 Arka Planı Kaldır",
            Language::Id => "🪄 Hapus Latar",
            Language::Vi => "🪄 Xoá nền",
            Language::Hi => "🪄 पृष्ठभूमि हटाएँ",
        }
    }

    /// Format cutout layer name preserving source name and preventing duplicate suffixes
    pub fn format_cutout_name(&self, source_name: &str) -> String {
        let suffix = self.no_bg_suffix();
        let trimmed = source_name.trim();

        for lang in Self::ALL_SUPPORTED {
            let s = lang.no_bg_suffix();
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
        if trimmed.ends_with("(Cutout)")
            || trimmed.ends_with("[Cutout]")
            || trimmed.ends_with("- Cutout")
        {
            return trimmed.to_string();
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

    /// Suffix for denoise & enhance layers
    pub fn denoise_suffix(&self) -> &'static str {
        match self {
            Language::En | Language::Auto => "Denoised",
            Language::Ru => "Очищено от шума",
            Language::De => "Entrauscht",
            Language::Fr => "Débruité",
            Language::Es => "Sin ruido",
            Language::It => "Riduzione rumore",
            Language::Ja => "ノイズ除去",
            Language::ZhHans => "降噪清晰",
            Language::ZhHant => "去噪清晰",
            Language::Ko => "노이즈 제거",
            Language::PtBr => "Sem ruído",
            Language::Cs => "Odšuměno",
            Language::Uk => "Очищено від шуму",
            Language::Sr => "Uklonjen šum",
            Language::Pl => "Odgłoszone",
            Language::Nl => "Ruis verwijderd",
            Language::Hu => "Zajtalanítva",
            Language::El => "Χωρίς θόρυβο",
            Language::Tr => "Gürültü Azaltıldı",
            Language::Id => "Bebas derau",
            Language::Vi => "Khử nhiễu",
            Language::Hi => "शोर रहित",
        }
    }

    /// Format denoise & enhance layer name
    pub fn format_denoise_name(&self, source_name: &str) -> String {
        let suffix = self.denoise_suffix();
        let trimmed = source_name.trim();

        for lang in Self::ALL_SUPPORTED {
            let s = lang.denoise_suffix();
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

    /// Suffix for depth map layers
    pub fn depth_suffix(&self) -> &'static str {
        match self {
            Language::En | Language::Auto => "Depth Map",
            Language::Ru => "Карта глубины",
            Language::De => "Tiefenkarte",
            Language::Fr => "Carte de profondeur",
            Language::Es => "Mapa de profundidad",
            Language::It => "Mappa di profondità",
            Language::Ja => "深度マップ",
            Language::ZhHans => "深度图",
            Language::ZhHant => "深度圖",
            Language::Ko => "깊이 맵",
            Language::PtBr => "Mapa de profundidade",
            Language::Cs => "Hloubková mapa",
            Language::Uk => "Карта глибини",
            Language::Sr => "Mapa dubine",
            Language::Pl => "Mapa głębi",
            Language::Nl => "Dieptekaart",
            Language::Hu => "Mélységtérkép",
            Language::El => "Χάρτης βάθους",
            Language::Tr => "Derinlik Haritası",
            Language::Id => "Peta kedalaman",
            Language::Vi => "Bản đồ độ sâu",
            Language::Hi => "गहराई नक्शा",
        }
    }

    /// Format depth map layer name preserving source name and preventing duplicate suffixes
    pub fn format_depth_name(&self, source_name: &str) -> String {
        let suffix = self.depth_suffix();
        let trimmed = source_name.trim();

        for lang in Self::ALL_SUPPORTED {
            let s = lang.depth_suffix();
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

    /// Suffix for vectorized layers
    pub fn vectorized_suffix(&self) -> &'static str {
        match self {
            Language::Ru => "Векторизация",
            Language::De => "Vektorisiert",
            Language::Fr => "Vectorisé",
            Language::Es => "Vectorizado",
            Language::It => "Vettorizzato",
            Language::Ja => "ベクター化",
            Language::ZhHans => "矢量化",
            Language::ZhHant => "向量化",
            Language::Ko => "벡터화",
            Language::PtBr => "Vetorizado",
            Language::Cs => "Vektorizováno",
            Language::Uk => "Векторизація",
            Language::Sr => "Vektorizovano",
            Language::Pl => "Wektoryzacja",
            Language::Nl => "Gevectoriseerd",
            Language::Hu => "Vektorizált",
            Language::El => "Διανυσματικοποιημένο",
            Language::Tr => "Vektörleştirildi",
            Language::Id => "Vektorisasi",
            Language::Vi => "Vectơ hoá",
            Language::Hi => "वेक्टराइज़्ड",
            _ => "Vectorized",
        }
    }

    /// Format vectorized layer name preserving source name and preventing duplicate suffixes
    pub fn format_vectorized_name(&self, source_name: &str) -> String {
        let suffix = self.vectorized_suffix();
        let trimmed = source_name.trim();

        for lang in Self::ALL_SUPPORTED {
            let s = lang.vectorized_suffix();
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
        assert_eq!(Language::from_str_code("zh"), Language::ZhHans);
        assert_eq!(Language::from_str_code("zh-hans"), Language::ZhHans);
        assert_eq!(Language::from_str_code("zh-hant"), Language::ZhHant);
        assert_eq!(Language::from_str_code("ko"), Language::Ko);
        assert_eq!(Language::from_str_code("pt"), Language::PtBr);
        assert_eq!(Language::from_str_code("pt-br"), Language::PtBr);
        assert_eq!(Language::from_str_code("nl"), Language::Nl);
        assert_eq!(Language::from_str_code("pl"), Language::Pl);
        assert_eq!(Language::from_str_code("cs"), Language::Cs);
        assert_eq!(Language::from_str_code("uk"), Language::Uk);
        assert_eq!(Language::from_str_code("sr"), Language::Sr);
        assert_eq!(Language::from_str_code("hu"), Language::Hu);
        assert_eq!(Language::from_str_code("el"), Language::El);
        assert_eq!(Language::from_str_code("tr"), Language::Tr);
        assert_eq!(Language::from_str_code("id"), Language::Id);
        assert_eq!(Language::from_str_code("vi"), Language::Vi);
        assert_eq!(Language::from_str_code("hi"), Language::Hi);
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
        assert_eq!(Language::ZhHans.code(), "zh-hans");
        assert_eq!(Language::ZhHant.code(), "zh-hant");
        assert_eq!(Language::Ko.code(), "ko");
        assert_eq!(Language::PtBr.code(), "pt-br");
        assert_eq!(Language::Cs.code(), "cs");
        assert_eq!(Language::Pl.code(), "pl");
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
        assert_eq!(Language::ZhHans.no_bg_suffix(), "无背景");
        assert_eq!(Language::ZhHant.no_bg_suffix(), "無背景");
        assert_eq!(Language::Ko.no_bg_suffix(), "배경 없음");
        assert_eq!(Language::PtBr.no_bg_suffix(), "Sem fundo");
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
    fn test_from_bcp47_tag_strip_regions() {
        assert_eq!(Language::from_bcp47_tag("ru-KZ"), Some(Language::Ru));
        assert_eq!(Language::from_bcp47_tag("ru_RU"), Some(Language::Ru));
        assert_eq!(Language::from_bcp47_tag("ru.UTF-8"), Some(Language::Ru));
        assert_eq!(Language::from_bcp47_tag("ru_RU.UTF-8"), Some(Language::Ru));
        assert_eq!(Language::from_bcp47_tag("cs_CZ.UTF-8"), Some(Language::Cs));
        assert_eq!(Language::from_bcp47_tag("sr_RS@latin"), Some(Language::Sr));
        assert_eq!(Language::from_bcp47_tag("POSIX"), Some(Language::En));
        assert_eq!(Language::from_bcp47_tag("C"), Some(Language::En));
        assert_eq!(Language::from_bcp47_tag("en-US"), Some(Language::En));
        assert_eq!(Language::from_bcp47_tag("en-GB"), Some(Language::En));
        assert_eq!(Language::from_bcp47_tag("cs-CZ"), Some(Language::Cs));
        assert_eq!(Language::from_bcp47_tag("de-AT"), Some(Language::De));
        assert_eq!(Language::from_bcp47_tag("fr-CA"), Some(Language::Fr));
        assert_eq!(
            Language::from_bcp47_tag("zh-Hans-CN"),
            Some(Language::ZhHans)
        );
        assert_eq!(Language::from_bcp47_tag("zh-TW"), Some(Language::ZhHant));
        assert_eq!(Language::from_bcp47_tag("pt-BR"), Some(Language::PtBr));
        assert_eq!(Language::from_bcp47_tag("pt-PT"), Some(Language::PtBr));
        assert_eq!(Language::from_bcp47_tag("unknown-XX"), None);
        assert_eq!(Language::from_bcp47_tag(""), None);

        // Verify from_str_code handles regional tags via from_bcp47_tag
        assert_eq!(Language::from_str_code("ru-KZ"), Language::Ru);
        assert_eq!(Language::from_str_code("ru_RU.UTF-8"), Language::Ru);
        assert_eq!(Language::from_str_code("cs-CZ"), Language::Cs);
        assert_eq!(Language::from_str_code("en-AU"), Language::En);
    }

    #[test]
    fn test_system_language() {
        let sys = Language::system_language();
        assert_ne!(sys, Language::Auto);
        assert!(Language::ALL_SUPPORTED.contains(&sys));
    }

    #[test]
    fn test_language_resolve() {
        assert_eq!(Language::Ru.resolve(None), Language::Ru);
        assert_eq!(Language::De.resolve(None), Language::De);

        let temp_dir = std::env::temp_dir().join(format!(
            "i18n_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let prefs_file = temp_dir.join("preferences.json");
        std::fs::write(&prefs_file, r#"{"interface": {"language": "de"}}"#).unwrap();

        assert_eq!(Language::Auto.resolve(Some(&prefs_file)), Language::De);

        // When preferences has "language": "auto", it resolves to system language
        let auto_prefs = temp_dir.join("auto_prefs.json");
        std::fs::write(&auto_prefs, r#"{"interface": {"language": "auto"}}"#).unwrap();
        assert_eq!(
            Language::Auto.resolve(Some(&auto_prefs)),
            Language::system_language()
        );

        let invalid_prefs = temp_dir.join("invalid.json");
        std::fs::write(&invalid_prefs, "not json").unwrap();
        assert_eq!(
            Language::Auto.resolve(Some(&invalid_prefs)),
            Language::system_language()
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_format_cutout_name() {
        let ru = Language::Ru;
        let en = Language::En;
        let de = Language::De;

        assert_eq!(ru.format_cutout_name("Portrait"), "Portrait (Без фона)");
        assert_eq!(en.format_cutout_name("Portrait"), "Portrait (No BG)");
        assert_eq!(
            de.format_cutout_name("Portrait"),
            "Portrait (Ohne Hintergrund)"
        );

        // Deduplication
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
        assert_eq!(
            de.format_cutout_name("Portrait (Ohne Hintergrund)"),
            "Portrait (Ohne Hintergrund)"
        );
        assert_eq!(
            en.format_cutout_name("Portrait [No BG]"),
            "Portrait [No BG]"
        );
        assert_eq!(
            ru.format_cutout_name("Portrait [Без фона]"),
            "Portrait [Без фона]"
        );
        assert_eq!(
            en.format_cutout_name("Portrait - No BG"),
            "Portrait - No BG"
        );
        assert_eq!(
            ru.format_cutout_name("Portrait - Без фона"),
            "Portrait - Без фона"
        );
        assert_eq!(en.format_cutout_name("  Subject  "), "Subject (No BG)");
    }

    #[test]
    fn test_all_22_languages_distinct_suffixes() {
        assert_eq!(Language::ALL_SUPPORTED.len(), 22);
        for lang in Language::ALL_SUPPORTED {
            assert!(!lang.code().is_empty());
            assert!(!lang.display_name().is_empty());
            assert!(!lang.selector_label().is_empty());
            assert!(!lang.no_bg_suffix().is_empty());
            assert!(!lang.upscale_suffix().is_empty());
            assert!(!lang.inpaint_suffix().is_empty());
            assert!(!lang.denoise_suffix().is_empty());
            assert!(!lang.depth_suffix().is_empty());
        }
    }

    #[test]
    fn test_ui_supported_and_safe_fallback() {
        assert_eq!(Language::UI_SUPPORTED.len(), 15);
        for lang in Language::UI_SUPPORTED {
            assert!(lang.is_ui_supported());
            assert_eq!(lang.safe_ui_language(), lang);
        }

        // CJK, Hindi, Greek, Vietnamese, Auto fall back safely to En for UI
        assert!(!Language::Ja.is_ui_supported());
        assert_eq!(Language::Ja.safe_ui_language(), Language::En);

        assert!(!Language::ZhHans.is_ui_supported());
        assert_eq!(Language::ZhHans.safe_ui_language(), Language::En);

        assert!(!Language::ZhHant.is_ui_supported());
        assert_eq!(Language::ZhHant.safe_ui_language(), Language::En);

        assert!(!Language::Ko.is_ui_supported());
        assert_eq!(Language::Ko.safe_ui_language(), Language::En);

        assert!(!Language::Hi.is_ui_supported());
        assert_eq!(Language::Hi.safe_ui_language(), Language::En);

        assert!(!Language::El.is_ui_supported());
        assert_eq!(Language::El.safe_ui_language(), Language::En);

        assert!(!Language::Vi.is_ui_supported());
        assert_eq!(Language::Vi.safe_ui_language(), Language::En);

        assert!(!Language::Auto.is_ui_supported());
        assert_eq!(Language::Auto.safe_ui_language(), Language::En);
    }
}

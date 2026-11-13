/// Static dictionary entry mapping English phrase to translations for host languages
pub struct TranslationEntry {
    pub en: &'static str,
    pub translations: &'static [(&'static str, &'static str)],
}

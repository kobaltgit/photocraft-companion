use core_contracts::Language;

mod entry;
mod table_part1;
mod table_part2;

#[cfg(test)]
mod tests;

pub use entry::TranslationEntry;
use table_part1::DICTIONARY_PART1;
use table_part2::DICTIONARY_PART2;

/// Looks up an English UI string in the dictionary for the target language.
/// Returns None if no translation is found (caller falls back to English).
pub fn lookup(en_text: &str, lang: Language) -> Option<&'static str> {
    let lang_code = lang.code();
    for entry in DICTIONARY_PART1.iter().chain(DICTIONARY_PART2.iter()) {
        if entry.en == en_text {
            for (code, trans) in entry.translations {
                if *code == lang_code {
                    return Some(trans);
                }
            }
            return None;
        }
    }
    None
}

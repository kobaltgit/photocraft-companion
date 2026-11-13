use super::*;
use core_contracts::Language;

#[test]
fn test_lookup_german() {
    assert_eq!(
        lookup("PhotoCraft Not Detected", Language::De),
        Some("PhotoCraft nicht erkannt")
    );
    assert_eq!(
        lookup("> Launch PhotoCraft", Language::De),
        Some("> PhotoCraft starten")
    );
    assert_eq!(lookup("About", Language::De), Some("Über"));
}

#[test]
fn test_lookup_french() {
    assert_eq!(
        lookup("PhotoCraft Not Detected", Language::Fr),
        Some("PhotoCraft non détecté")
    );
    assert_eq!(lookup("About", Language::Fr), Some("À propos"));
}

#[test]
fn test_lookup_ukrainian() {
    assert_eq!(
        lookup("PhotoCraft Not Detected", Language::Uk),
        Some("PhotoCraft не знайдено")
    );
    assert_eq!(lookup("About", Language::Uk), Some("Про програму"));
}

#[test]
fn test_lookup_missing_falls_back() {
    assert_eq!(lookup("NonExistentStringXYZ", Language::De), None);
}

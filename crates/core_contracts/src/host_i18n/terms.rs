/// Normalizes a term for case-insensitive, punctuation-tolerant comparison
pub fn normalize_term(s: &str) -> String {
    s.to_ascii_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '_' && *c != '-' && *c != '(' && *c != ')')
        .collect()
}

// Auto-generated blend modes dictionary from PhotoCraft TSVs
pub static BLEND_MODES: &[(&str, &[(&str, &str)])] = &[
    (
        "Pass Through",
        &[
            ("ja", "通過"),
            ("zh-hans", "穿透"),
            ("zh-hant", "穿過"),
            ("es", "Pasar a través"),
            ("ru", "Пропустить"),
            ("uk", "Наскрізне"),
            ("sr", "Propuštanje"),
            ("cs", "Průchozí"),
            ("fr", "Transfert"),
            ("ko", "통과"),
            ("pl", "Przepuszczanie"),
            ("de", "Durchreichen"),
            ("pt-br", "Passagem"),
            ("el", "Διέλευση"),
            ("nl", "Doorlaten"),
            ("it", "Passa attraverso"),
            ("hu", "Áteresztés"),
            ("tr", "Geçirgen"),
            ("vi", "Chuyển tiếp"),
        ],
    ),
    (
        "Normal",
        &[
            ("ja", "通常"),
            ("zh-hans", "正常"),
            ("zh-hant", "正常"),
            ("ru", "Обычная"),
            ("uk", "Звичайний"),
            ("sr", "Normalno"),
            ("cs", "Normální"),
            ("ko", "표준"),
            ("pl", "Normalny"),
            ("el", "Κανονική"),
            ("nl", "Normaal"),
            ("it", "Normale"),
            ("hu", "Normál"),
            ("vi", "Bình thường"),
        ],
    ),
    (
        "Dissolve",
        &[
            ("ja", "ディザ合成"),
            ("zh-hans", "溶解"),
            ("zh-hant", "溶解"),
            ("es", "Disolver"),
            ("ru", "Растворение"),
            ("uk", "Розчинення"),
            ("sr", "Rastapanje"),
            ("cs", "Rozpustit"),
            ("fr", "Fondu"),
            ("ko", "디졸브"),
            ("pl", "Rozpuszczanie"),
            ("de", "Sprenkeln"),
            ("pt-br", "Dissolver"),
            ("el", "Διάλυση"),
            ("nl", "Verspreiden"),
            ("it", "Dissolvi"),
            ("hu", "Szétszórás"),
            ("tr", "Eritme"),
            ("vi", "Hoà tan"),
        ],
    ),
    (
        "Darken",
        &[
            ("ja", "比較 (暗)"),
            ("zh-hans", "变暗"),
            ("zh-hant", "變暗"),
            ("es", "Oscurecer"),
            ("ru", "Замена тёмным"),
            ("uk", "Затемнення"),
            ("sr", "Zatamnjenje"),
            ("cs", "Ztmavit"),
            ("fr", "Obscurcir"),
            ("id", "Gelapkan"),
            ("ko", "어둡게 하기"),
            ("pl", "Przyciemnij"),
            ("de", "Abdunkeln"),
            ("pt-br", "Escurecer"),
            ("el", "Σκοτείνιασμα"),
            ("nl", "Donkerder"),
            ("it", "Scurisci"),
            ("hu", "Sötétítés"),
            ("tr", "Koyulaştır"),
            ("vi", "Làm tối"),
        ],
    ),
    (
        "Multiply",
        &[
            ("ja", "乗算"),
            ("zh-hans", "正片叠底"),
            ("zh-hant", "色彩增值"),
            ("es", "Multiplicar"),
            ("ru", "Умножение"),
            ("uk", "Множення"),
            ("sr", "Množenje"),
            ("cs", "Násobit"),
            ("fr", "Produit"),
            ("ko", "곱하기"),
            ("pl", "Mnożenie"),
            ("de", "Multiplizieren"),
            ("pt-br", "Multiplicação"),
            ("el", "Πολλαπλασιασμός"),
            ("nl", "Vermenigvuldigen"),
            ("it", "Moltiplica"),
            ("hu", "Szorzás"),
            ("tr", "Çoğalt"),
            ("vi", "Nhân"),
        ],
    ),
    (
        "Color Burn",
        &[
            ("ja", "焼き込みカラー"),
            ("zh-hans", "颜色加深"),
            ("zh-hant", "加深顏色"),
            ("es", "Subexposición de color"),
            ("ru", "Затемнение основы"),
            ("uk", "Затемнення основи"),
            ("sr", "Zatamnjivanje boje"),
            ("cs", "Ztmavit barvy"),
            ("fr", "Densité couleur +"),
            ("ko", "색상 번"),
            ("pl", "Wypalanie koloru"),
            ("de", "Farbig nachbelichten"),
            ("pt-br", "Superexposição de cores"),
            ("el", "Κάψιμο χρώματος"),
            ("nl", "Kleur doordrukken"),
            ("it", "Colore brucia"),
            ("hu", "Színégetés"),
            ("tr", "Renk Yakma"),
            ("vi", "Đốt màu"),
        ],
    ),
    (
        "Linear Burn",
        &[
            ("ja", "焼き込み (リニア)"),
            ("zh-hans", "线性加深"),
            ("zh-hant", "線性加深"),
            ("es", "Subexposición lineal"),
            ("ru", "Линейный затемнитель"),
            ("uk", "Лінійне затемнення"),
            ("sr", "Linearno zatamnjivanje"),
            ("cs", "Lineárně ztmavit"),
            ("fr", "Densité linéaire +"),
            ("ko", "선형 번"),
            ("pl", "Wypalanie liniowe"),
            ("de", "Linear nachbelichten"),
            ("pt-br", "Superexposição linear"),
            ("el", "Γραμμικό κάψιμο"),
            ("nl", "Lineair doordrukken"),
            ("it", "Brucia lineare"),
            ("hu", "Lineáris égetés"),
            ("tr", "Doğrusal Yakma"),
            ("vi", "Đốt tuyến tính"),
        ],
    ),
    (
        "Darker Color",
        &[
            ("ja", "カラー比較(暗)"),
            ("zh-hans", "深色"),
            ("zh-hant", "顏色變暗"),
            ("es", "Color más oscuro"),
            ("ru", "Замена тёмным"),
            ("uk", "Темніший колір"),
            ("sr", "Tamnija boja"),
            ("cs", "Tmavší barva"),
            ("fr", "Couleur plus foncée"),
            ("ko", "어두운 색상"),
            ("pl", "Ciemniejszy kolor"),
            ("de", "Dunklere Farbe"),
            ("pt-br", "Cor mais escura"),
            ("el", "Πιο σκούρο χρώμα"),
            ("nl", "Donkerdere kleur"),
            ("it", "Colore più scuro"),
            ("hu", "Sötétebb szín"),
            ("tr", "Daha Koyu Renk"),
            ("vi", "Màu tối hơn"),
        ],
    ),
    (
        "Lighten",
        &[
            ("ja", "比較 (明)"),
            ("zh-hans", "变亮"),
            ("zh-hant", "變亮"),
            ("es", "Aclarar"),
            ("ru", "Замена светлым"),
            ("uk", "Освітлення"),
            ("sr", "Posvetljenje"),
            ("cs", "Zesvětlit"),
            ("fr", "Éclaircir"),
            ("id", "Cerahkan"),
            ("ko", "밝게 하기"),
            ("pl", "Rozjaśnij"),
            ("de", "Aufhellen"),
            ("pt-br", "Clarear"),
            ("el", "Φώτισμα"),
            ("nl", "Lichter"),
            ("it", "Schiarisci"),
            ("hu", "Világosítás"),
            ("tr", "Açıklaştır"),
            ("vi", "Làm sáng"),
        ],
    ),
    (
        "Screen",
        &[
            ("ja", "スクリーン"),
            ("zh-hans", "滤色"),
            ("zh-hant", "濾色"),
            ("es", "Pantalla"),
            ("ru", "Экран"),
            ("uk", "Екран"),
            ("sr", "Ekran"),
            ("cs", "Závoj"),
            ("fr", "Écran"),
            ("id", "Layar"),
            ("ko", "스크린"),
            ("pl", "Ekran"),
            ("de", "Negativ multiplizieren"),
            ("pt-br", "Tela"),
            ("el", "Οθόνη"),
            ("nl", "Bleken"),
            ("it", "Scherma"),
            ("hu", "Képernyő"),
            ("tr", "Ekran"),
            ("vi", "Màn hình"),
        ],
    ),
    (
        "Color Dodge",
        &[
            ("ja", "覆い焼きカラー"),
            ("zh-hans", "颜色减淡"),
            ("zh-hant", "加亮顏色"),
            ("es", "Sobreexposición de color"),
            ("ru", "Осветление основы"),
            ("uk", "Освітлення основи"),
            ("sr", "Posvetljivanje boje"),
            ("cs", "Zesvětlit barvy"),
            ("fr", "Densité couleur −"),
            ("ko", "색상 닷지"),
            ("pl", "Rozjaśnianie koloru"),
            ("de", "Farbig abwedeln"),
            ("pt-br", "Subexposição de cores"),
            ("el", "Φωτισμός χρώματος"),
            ("nl", "Kleur tegenhouden"),
            ("it", "Colore scherma"),
            ("hu", "Színderítés"),
            ("tr", "Renk Soldurma"),
            ("vi", "Làm sáng màu"),
        ],
    ),
    (
        "Linear Dodge (Add)",
        &[
            ("ja", "覆い焼き (リニア)- 加算"),
            ("zh-hans", "线性减淡（添加）"),
            ("zh-hant", "線性加亮 (增加)"),
            ("es", "Sobreexposición lineal (añadir)"),
            ("ru", "Линейный осветлитель (добавить)"),
            ("uk", "Лінійне освітлення (додавання)"),
            ("sr", "Linearno posvetljivanje (sabiranje)"),
            ("cs", "Lineárně zesvětlit (přidat)"),
            ("fr", "Densité linéaire − (Addition)"),
            ("ko", "선형 닷지(추가)"),
            ("pl", "Rozjaśnianie liniowe (dodaj)"),
            ("de", "Linear abwedeln (Hinzufügen)"),
            ("pt-br", "Subexposição linear (adicionar)"),
            ("el", "Γραμμική υπερέκθεση (προσθήκη)"),
            ("nl", "Lineair tegenhouden (toevoegen)"),
            ("it", "Scherma lineare (aggiungi)"),
            ("hu", "Lineáris derítés (összeadás)"),
            ("tr", "Doğrusal Soldurma (Ekle)"),
            ("vi", "Trộn tuyến tính sáng (Cộng)"),
        ],
    ),
    (
        "Lighter Color",
        &[
            ("ja", "カラー比較 (明)"),
            ("zh-hans", "浅色"),
            ("zh-hant", "顏色變亮"),
            ("es", "Color más claro"),
            ("ru", "Замена светлым"),
            ("uk", "Світліший колір"),
            ("sr", "Svetlija boja"),
            ("cs", "Světlejší barva"),
            ("fr", "Couleur plus claire"),
            ("ko", "밝은 색상"),
            ("pl", "Jaśniejszy kolor"),
            ("de", "Hellere Farbe"),
            ("pt-br", "Cor mais clara"),
            ("el", "Πιο ανοιχτό χρώμα"),
            ("nl", "Lichtere kleur"),
            ("it", "Colore più chiaro"),
            ("hu", "Világosabb szín"),
            ("tr", "Daha Açık Renk"),
            ("vi", "Màu sáng hơn"),
        ],
    ),
    (
        "Overlay",
        &[
            ("ja", "オーバーレイ"),
            ("zh-hans", "叠加"),
            ("zh-hant", "覆蓋"),
            ("es", "Superponer"),
            ("ru", "Перекрытие"),
            ("uk", "Перекриття"),
            ("sr", "Preklapanje"),
            ("cs", "Překrýt"),
            ("fr", "Incrustation"),
            ("ko", "오버레이"),
            ("pl", "Nakładka"),
            ("de", "Überlagern"),
            ("pt-br", "Sobrepor"),
            ("el", "Επικάλυψη"),
            ("nl", "Bedekken"),
            ("it", "Sovrapponi"),
            ("hu", "Átfedés"),
            ("tr", "Kaplama"),
            ("vi", "Phủ"),
        ],
    ),
    (
        "Soft Light",
        &[
            ("ja", "ソフトライト"),
            ("zh-hans", "柔光"),
            ("zh-hant", "柔光"),
            ("es", "Luz suave"),
            ("ru", "Мягкий свет"),
            ("uk", "М'яке світло"),
            ("sr", "Meko svetlo"),
            ("cs", "Měkké světlo"),
            ("fr", "Lumière douce"),
            ("ko", "소프트 라이트"),
            ("pl", "Łagodne światło"),
            ("de", "Weiches Licht"),
            ("pt-br", "Luz suave"),
            ("el", "Απαλό ανοιχτό"),
            ("nl", "Zacht licht"),
            ("it", "Luce soffusa"),
            ("hu", "Lágy fény"),
            ("tr", "Yumuşak Işık"),
            ("vi", "Ánh sáng mềm"),
        ],
    ),
    (
        "Hard Light",
        &[
            ("ja", "ハードライト"),
            ("zh-hans", "强光"),
            ("zh-hant", "實光"),
            ("es", "Luz fuerte"),
            ("ru", "Жёсткий свет"),
            ("uk", "Жорстке світло"),
            ("sr", "Oštro svetlo"),
            ("cs", "Tvrdé světlo"),
            ("fr", "Lumière crue"),
            ("ko", "하드 라이트"),
            ("pl", "Ostre światło"),
            ("de", "Hartes Licht"),
            ("pt-br", "Luz direta"),
            ("el", "Έντονο φως"),
            ("nl", "Hard licht"),
            ("it", "Luce intensa"),
            ("hu", "Kemény fény"),
            ("tr", "Sert Işık"),
            ("vi", "Ánh sáng cứng"),
        ],
    ),
    (
        "Vivid Light",
        &[
            ("ja", "ビビッドライト"),
            ("zh-hans", "亮光"),
            ("zh-hant", "強烈光源"),
            ("es", "Luz intensa"),
            ("ru", "Яркий свет"),
            ("uk", "Яскраве світло"),
            ("sr", "Živo svetlo"),
            ("cs", "Jasné světlo"),
            ("fr", "Lumière vive"),
            ("ko", "선명한 라이트"),
            ("pl", "Jaskrawe światło"),
            ("de", "Deutliches Licht"),
            ("pt-br", "Luz brilhante"),
            ("el", "Ζωντανό φως"),
            ("nl", "Fel licht"),
            ("it", "Luce vivida"),
            ("hu", "Élénk fény"),
            ("tr", "Canlı Işık"),
            ("vi", "Ánh sáng rực rỡ"),
        ],
    ),
    (
        "Linear Light",
        &[
            ("ja", "リニアライト"),
            ("zh-hans", "线性光"),
            ("zh-hant", "線性光源"),
            ("es", "Luz lineal"),
            ("ru", "Линейный свет"),
            ("uk", "Лінійне світло"),
            ("sr", "Linearno svetlo"),
            ("cs", "Lineární světlo"),
            ("fr", "Lumière linéaire"),
            ("ko", "선형 라이트"),
            ("pl", "Światło liniowe"),
            ("de", "Lineares Licht"),
            ("pt-br", "Luz linear"),
            ("el", "Γραμμικό φως"),
            ("nl", "Lineair licht"),
            ("it", "Luce lineare"),
            ("hu", "Lineáris fény"),
            ("tr", "Doğrusal Işık"),
            ("vi", "Ánh sáng tuyến tính"),
        ],
    ),
    (
        "Pin Light",
        &[
            ("ja", "ピンライト"),
            ("zh-hans", "点光"),
            ("zh-hant", "小光源"),
            ("es", "Luz focal"),
            ("ru", "Точечный свет"),
            ("uk", "Точкове світло"),
            ("sr", "Tačkasto svetlo"),
            ("cs", "Bodové světlo"),
            ("fr", "Lumière ponctuelle"),
            ("ko", "핀 라이트"),
            ("pl", "Światło punktowe"),
            ("de", "Lichtpunkt"),
            ("pt-br", "Luz do pino"),
            ("el", "Φως σημείου"),
            ("nl", "Puntlicht"),
            ("it", "Luce puntiforme"),
            ("hu", "Pontfény"),
            ("tr", "Nokta Işığı"),
            ("vi", "Ánh sáng ghim"),
        ],
    ),
    (
        "Hard Mix",
        &[
            ("ja", "ハードミックス"),
            ("zh-hans", "实色混合"),
            ("zh-hant", "實色疊印混合"),
            ("es", "Mezcla definida"),
            ("ru", "Жёсткое смешивание"),
            ("uk", "Жорстке змішування"),
            ("sr", "Oštro mešanje"),
            ("cs", "Tvrdé míchání"),
            ("fr", "Mélange maximal"),
            ("ko", "하드 혼합"),
            ("pl", "Twarde mieszanie"),
            ("de", "Hart mischen"),
            ("pt-br", "Mistura sólida"),
            ("el", "Έντονη ανάμειξη"),
            ("nl", "Harde mix"),
            ("it", "Miscela dura"),
            ("hu", "Kemény keverés"),
            ("tr", "Sert Karışım"),
            ("vi", "Trộn cứng"),
        ],
    ),
    (
        "Difference",
        &[
            ("ja", "差の絶対値"),
            ("zh-hans", "差值"),
            ("zh-hant", "差異化"),
            ("es", "Diferencia"),
            ("ru", "Разница"),
            ("uk", "Різниця"),
            ("sr", "Razlika"),
            ("cs", "Rozdíl"),
            ("fr", "Différence"),
            ("id", "Selisih"),
            ("ko", "차이"),
            ("pl", "Różnica"),
            ("de", "Differenz"),
            ("pt-br", "Diferença"),
            ("el", "Διαφορά"),
            ("nl", "Verschil"),
            ("it", "Differenza"),
            ("hu", "Különbség"),
            ("tr", "Fark"),
            ("vi", "Hiệu"),
        ],
    ),
    (
        "Exclusion",
        &[
            ("ja", "除外"),
            ("zh-hans", "排除"),
            ("zh-hant", "排除"),
            ("es", "Exclusión"),
            ("ru", "Исключение"),
            ("uk", "Виключення"),
            ("sr", "Isključenje"),
            ("cs", "Vyloučit"),
            ("ko", "제외"),
            ("pl", "Wykluczenie"),
            ("de", "Ausschluss"),
            ("pt-br", "Exclusão"),
            ("el", "Αποκλεισμός"),
            ("nl", "Uitsluiting"),
            ("it", "Esclusione"),
            ("hu", "Kizárás"),
            ("tr", "Dışlama"),
            ("vi", "Loại trừ"),
        ],
    ),
    (
        "Subtract",
        &[
            ("ja", "減算"),
            ("zh-hans", "减去"),
            ("zh-hant", "減去"),
            ("es", "Restar"),
            ("ru", "Вычесть"),
            ("uk", "Відняти"),
            ("sr", "Oduzimanje"),
            ("cs", "Odečíst"),
            ("fr", "Soustraction"),
            ("id", "Kurangi"),
            ("ko", "빼기"),
            ("pl", "Odejmij"),
            ("de", "Subtrahieren"),
            ("pt-br", "Subtrair"),
            ("el", "Αφαίρεση"),
            ("nl", "Aftrekken"),
            ("it", "Sottrai"),
            ("hu", "Kivonás"),
            ("tr", "Çıkar"),
            ("vi", "Trừ"),
        ],
    ),
    (
        "Divide",
        &[
            ("ja", "除算"),
            ("zh-hans", "划分"),
            ("zh-hant", "分割"),
            ("es", "Dividir"),
            ("ru", "Разделить"),
            ("uk", "Ділення"),
            ("sr", "Deljenje"),
            ("cs", "Dělit"),
            ("fr", "Division"),
            ("ko", "나누기"),
            ("pl", "Dzielenie"),
            ("de", "Dividieren"),
            ("pt-br", "Divisão"),
            ("el", "Διαίρεση"),
            ("nl", "Delen"),
            ("it", "Dividi"),
            ("hu", "Osztás"),
            ("tr", "Böl"),
            ("vi", "Chia"),
        ],
    ),
    (
        "Hue",
        &[
            ("ja", "色相"),
            ("zh-hans", "色相"),
            ("zh-hant", "色相"),
            ("es", "Tono"),
            ("ru", "Тон"),
            ("uk", "Тон"),
            ("sr", "Nijansa"),
            ("cs", "Odstín"),
            ("fr", "Teinte"),
            ("ko", "색조"),
            ("pl", "Odcień"),
            ("de", "Farbton"),
            ("pt-br", "Matiz"),
            ("el", "Απόχρωση"),
            ("nl", "Kleurtoon"),
            ("it", "Tonalità"),
            ("hu", "Színárnyalat"),
            ("tr", "Ton"),
            ("vi", "Sắc màu"),
            ("hi", "ह्यू"),
        ],
    ),
    (
        "Saturation",
        &[
            ("ja", "彩度"),
            ("zh-hans", "饱和度"),
            ("zh-hant", "飽和度"),
            ("es", "Saturación"),
            ("ru", "Насыщенность"),
            ("uk", "Насиченість"),
            ("sr", "Zasićenost"),
            ("cs", "Sytost"),
            ("id", "Saturasi"),
            ("ko", "채도"),
            ("pl", "Nasycenie"),
            ("de", "Sättigung"),
            ("pt-br", "Saturação"),
            ("el", "Κορεσμός"),
            ("nl", "Verzadiging"),
            ("it", "Saturazione"),
            ("hu", "Telítettség"),
            ("tr", "Doygunluk"),
            ("vi", "Độ bão hòa"),
            ("hi", "सैचुरेशन"),
        ],
    ),
    (
        "Color",
        &[
            ("ja", "カラー"),
            ("zh-hans", "颜色"),
            ("zh-hant", "顏色"),
            ("ru", "Цвет"),
            ("uk", "Колір"),
            ("sr", "Boja"),
            ("cs", "Barvy"),
            ("fr", "Couleur"),
            ("id", "Warna"),
            ("ko", "색상"),
            ("pl", "Kolor"),
            ("de", "Farbe"),
            ("pt-br", "Cor"),
            ("el", "Χρώμα"),
            ("nl", "Kleur"),
            ("it", "Colore"),
            ("hu", "Szín"),
            ("tr", "Renk"),
            ("vi", "Màu"),
            ("hi", "रंग"),
        ],
    ),
    (
        "Luminosity",
        &[
            ("ja", "輝度"),
            ("zh-hans", "明度"),
            ("zh-hant", "明度"),
            ("es", "Luminosidad"),
            ("ru", "Яркость"),
            ("uk", "Світність"),
            ("sr", "Svetlina tona"),
            ("cs", "Světlost"),
            ("fr", "Luminosité"),
            ("ko", "광도"),
            ("pl", "Luminancja"),
            ("de", "Luminanz"),
            ("pt-br", "Luminosidade"),
            ("el", "Φωτεινότητα"),
            ("nl", "Lichtsterkte"),
            ("it", "Luminosità"),
            ("hu", "Fényesség"),
            ("tr", "Parlaklık"),
            ("vi", "Độ sáng"),
        ],
    ),
];

/// Checks if a parameter key is known to hold translatable host terminology (such as blend modes)
pub fn is_translatable_param(key: &str) -> bool {
    matches!(
        key,
        "blend" | "mode" | "highlightBlend" | "shadowBlend" | "gradientBlendMode" | "blend_mode"
    )
}

/// Translates a canonical English term to the target host locale.
/// Returns None if the canonical term is not recognized in the dictionary.
pub fn to_host_locale(canonical: &str, lang_code: &str) -> Option<&'static str> {
    let norm = normalize_term(canonical);
    for (mode_canonical, translations) in BLEND_MODES {
        if normalize_term(mode_canonical) == norm {
            if lang_code == "en" || lang_code == "auto" {
                return Some(mode_canonical);
            }
            for (code, trans) in *translations {
                if *code == lang_code {
                    return Some(trans);
                }
            }
            // If this language has no translation in catalog, fallback to canonical
            return Some(mode_canonical);
        }
    }
    None
}

/// Reverse lookup: translates a localized host term back to canonical English.
/// If lang_code is provided, checks that language first; otherwise checks all languages.
pub fn to_canonical(host_term: &str, lang_code: Option<&str>) -> Option<&'static str> {
    let norm = normalize_term(host_term);

    // First check if it is already canonical
    for (mode_canonical, _) in BLEND_MODES {
        if normalize_term(mode_canonical) == norm {
            return Some(mode_canonical);
        }
    }

    // Check specific language if provided
    if let Some(target_lang) = lang_code {
        for (mode_canonical, translations) in BLEND_MODES {
            for (code, trans) in *translations {
                if *code == target_lang && normalize_term(trans) == norm {
                    return Some(mode_canonical);
                }
            }
        }
    }

    // Check across all languages
    for (mode_canonical, translations) in BLEND_MODES {
        for (_, trans) in *translations {
            if normalize_term(trans) == norm {
                return Some(mode_canonical);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_host_locale_german() {
        assert_eq!(to_host_locale("Soft Light", "de"), Some("Weiches Licht"));
        assert_eq!(to_host_locale("Linear Light", "de"), Some("Lineares Licht"));
        assert_eq!(to_host_locale("Luminosity", "de"), Some("Luminanz"));
        assert_eq!(to_host_locale("Multiply", "de"), Some("Multiplizieren"));
        assert_eq!(
            to_host_locale("Screen", "de"),
            Some("Negativ multiplizieren")
        );
    }

    #[test]
    fn test_to_host_locale_russian() {
        assert_eq!(to_host_locale("Soft Light", "ru"), Some("Мягкий свет"));
        assert_eq!(to_host_locale("Multiply", "ru"), Some("Умножение"));
        assert_eq!(to_host_locale("Screen", "ru"), Some("Экран"));
        assert_eq!(to_host_locale("Luminosity", "ru"), Some("Яркость"));
    }

    #[test]
    fn test_to_host_locale_fallback() {
        assert_eq!(to_host_locale("Soft Light", "en"), Some("Soft Light"));
        assert_eq!(to_host_locale("Soft Light", "auto"), Some("Soft Light"));
        assert_eq!(to_host_locale("UnknownModeXYZ", "de"), None);
    }

    #[test]
    fn test_to_canonical() {
        assert_eq!(
            to_canonical("Weiches Licht", Some("de")),
            Some("Soft Light")
        );
        assert_eq!(to_canonical("Мягкий свет", Some("ru")), Some("Soft Light"));
        assert_eq!(to_canonical("Weiches Licht", None), Some("Soft Light"));
        assert_eq!(to_canonical("Soft Light", None), Some("Soft Light"));
    }

    #[test]
    fn test_is_translatable_param() {
        assert!(is_translatable_param("blend"));
        assert!(is_translatable_param("mode"));
        assert!(is_translatable_param("highlightBlend"));
        assert!(!is_translatable_param("opacity"));
        assert!(!is_translatable_param("name"));
    }
}

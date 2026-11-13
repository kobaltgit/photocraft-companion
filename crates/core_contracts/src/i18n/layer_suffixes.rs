use super::language::Language;

impl Language {
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

    /// Suffix for bokeh blur layers
    pub fn bokeh_suffix(&self) -> &'static str {
        match self {
            Language::Ru | Language::Uk => "Боке",
            Language::ZhHans | Language::ZhHant => "散景",
            Language::Ja => "ボケ",
            Language::Ko => "보케",
            Language::Es => "Desenfoque",
            Language::PtBr => "Desfoque",
            Language::Tr => "Bulanıklık",
            Language::Vi => "Làm mờ",
            Language::El => "Μποκέ",
            Language::Hi => "बोकेह",
            Language::Cs => "Bokeh",
            Language::De => "Bokeh",
            Language::Fr => "Bokeh",
            Language::It => "Bokeh",
            Language::Pl => "Bokeh",
            Language::Nl => "Bokeh",
            Language::Hu => "Bokeh",
            Language::Sr => "Boke",
            Language::Id => "Bokeh",
            Language::En | Language::Auto => "Bokeh",
        }
    }

    /// Format bokeh blur layer name preserving source name and preventing duplicate suffixes
    pub fn format_bokeh_name(&self, source_name: &str) -> String {
        let suffix = self.bokeh_suffix();
        let trimmed = source_name.trim();

        for lang in Self::ALL_SUPPORTED {
            let s = lang.bokeh_suffix();
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

    /// Localized name for the atmospheric fog layer
    pub fn fog_layer_name(&self) -> &'static str {
        match self {
            Language::En | Language::Auto => "Atmospheric Fog",
            Language::Ru => "Атмосферный туман",
            Language::Uk => "Атмосферний туман",
            Language::ZhHans => "大气雾",
            Language::ZhHant => "大氣霧",
            Language::Ja => "大気フォグ",
            Language::Ko => "대기 안개",
            Language::Es => "Niebla atmosférica",
            Language::Fr => "Brouillard atmosphérique",
            Language::De => "Atmosphärischer Nebel",
            Language::It => "Nebbia atmosferica",
            Language::PtBr => "Névoa atmosférica",
            Language::Pl => "Mgła atmosferyczna",
            Language::Cs => "Atmosférická mlha",
            Language::Nl => "Atmosferische mist",
            Language::Hu => "Atmoszférikus köd",
            Language::El => "Ατμοσφαιρική ομίχλη",
            Language::Tr => "Atmosferik Sis",
            Language::Id => "Kabut Atmosfer",
            Language::Vi => "Sương mù khí quyển",
            Language::Sr => "Atmosferska magla",
            Language::Hi => "वायुमंडलीय कोहरा",
        }
    }
}

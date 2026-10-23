# Cinematic Teal & Orange (Hollywood Grade)

[![Plugin: Teal & Orange](https://img.shields.io/badge/Plugin-Teal_%26_Orange-blue.svg)](plugin.json)
[![Type: No-Code Macro](https://img.shields.io/badge/Type-No--Code_Macro-orange.svg)](plugin.json)

**English:**
Professional Hollywood cinematic color grading macro suite for PhotoCraft portraits, street photography, and filmmaking stills.
In a single click, it builds a fully non-destructive 3-layer color grading stack directly in your active document:
1. **Teal & Orange (3D LUT):** Applies PhotoCraft's native high-precision 3D Look-Up Table with an optimized 85% opacity. It splits the color spectrum by pushing cool cyan/teal hues into shadows and backgrounds while preserving golden amber warmth in midtones and skin highlights.
2. **Film Contrast (S-Curve):** A custom non-destructive Curves adjustment layer calibrated with a smooth cinematic S-curve (`[64, 54]` shadows and `[192, 202]` highlights) set to `Luminosity` blend mode. This delivers rich, punchy celluloid contrast without inducing unnatural color saturation spikes or tinting.
3. **Skin Vibrance & Protection:** A Vibrance adjustment layer (`+15` Vibrance, `-5` Saturation) engineered to enrich muted teal/cyan shadows while preventing human skin tones from becoming over-saturated or plasticky.

Every layer in the generated stack is 100% non-destructive and equipped with an editable raster mask, granting retouchers full manual control over local intensity, brushing, and opacity blending.

### Practical Use Cases:
- **Cinematic Portrait Retouching:** Instantly separate human subjects from backgrounds by establishing complementary color tension (warm skin vs. deep teal shadows) without manually painting color masks or losing delicate skin tones.
- **Urban Night & Street Photography:** Transform everyday nighttime cityscapes, neon signs, wet asphalt reflections, and street scenes into moody, atmospheric neo-noir movie frames reminiscent of Blade Runner and Michael Bay cinematography.
- **Travel & Video Production Stills:** Give drone captures, golden hour horizons, and vacation snapshots an authoritative blockbuster film aesthetic with deep dynamic range and velvety celluloid shadows.

### Neural Weights:
- None required. Operates purely through high-speed deterministic native PhotoCraft 3D LUT and layer adjustment engines (0 MB download, ~0.05s execution).

---

**Русский:**
Профессиональный макро-плагин кинематографической цветокоррекции Teal & Orange (Голливудский грейдинг) для портретной, уличной и художественной фотографии в PhotoCraft.
В один клик за доли секунды разворачивает в активном документе полноценный недеструктивный трехслойный стек цветокоррекции:
1. **Teal & Orange (3D LUT):** Применяет встроенную высокоточную 3D-таблицу преобразования цвета (Look-Up Table) с выверенной базовой непрозрачностью 85%. Расщепляет цветовые диапазоны: уводит тени и фон в прохладные бирюзовые тона (Teal), акцентируя теплые янтарные и золотистые оттенки в светах и полутонах (Orange).
2. **Film Contrast (S-Curve):** Корректирующий слой кривых (`Curves`) с калиброванной пленочной S-образной формой (`[64, 54]` в тенях и `[192, 202]` в светах) в режиме наложения `Luminosity` (Яркость). Создает плотный киношный контраст и объем без паразитного перенасыщения цветов и искажения тональности.
3. **Skin Vibrance & Protection:** Корректирующий слой сочности (`Vibrance +15`, `Saturation -5`), защищающий естественные оттенки кожи от кислотности и одновременно углубляющий фоновые сине-зеленые полутона.

Все создаваемые слои абсолютно недеструктивны, снабжены собственными масками и позволяют фотографу в любой момент отрегулировать общую силу эффекта, изменить прозрачность или стереть тонирование с отдельных участков мягкой кистью.

### Практическое применение:
- **Кинематографическая портретная ретушь:** Мгновенное отделение модели от фона за счет классического контраста дополнительных цветов цветового круга (теплая кожа против холодного окружения) без необходимости сложной ручной маскировки.
- **Ночная городская и уличная съемка:** Превращение кадров ночного города, неоновых вывесок, мокрого асфальта и витрин в атмосферные кинокадры в стиле неонуара, *Бегущего по лезвию* или блокбастеров Майкла Бэя.
- **Трэвел-фотография и видеостоп-кадры:** Придание снимкам с дронов, закатным пейзажам и репортажным кадрам законченного вида дорогого кинофильма с глубокими тенями и чистыми светами.

### Нейросетевые веса:
- Не требуются. Работает на базе высокоскоростных встроенных 32-битных 3D LUT и корректирующих слоев ядра PhotoCraft (0 МБ загрузки, ~0.05 с на применение).

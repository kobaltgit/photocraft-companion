# Timelapse Studio (Студия таймлапсов)

![Plugin Type](https://img.shields.io/badge/Type-Interactive%20UI-blue.svg)
![Version](https://img.shields.io/badge/Version-1.1.0-green.svg)
![Status](https://img.shields.io/badge/Platform-Zero--Freeze%20Speedpaint-orange.svg)

---

**English:**
**Timelapse Studio** is a zero-latency drawing process recorder for PhotoCraft Companion. It automatically captures lightweight canvas previews as you paint without freezing the brush, zapping the system clipboard, or bogging down GPU rendering. It features instant one-click export to social video formats (1080p/4K, 9:16 vertical Reels/Shorts with background blur-fill, and configurable hold frames).

#### Key Features:

- **Zero Brush Latency:** Captures occur asynchronously via PhotoCraft's native socket protocol without interrupting drawing flow.
- **Ergonomic Session Control:** Dedicated Record, Pause, Stop, Snapshot, and Reset controls with real-time frame stats and animated toolstrip badge.
- **Clipboard Isolation:** Never accesses or overwrites the OS clipboard (`arboard`), keeping copied text and references safe.
- **Social Video Presets:** Fast MP4 export formatted for TikTok, YouTube Shorts, and Instagram Reels (9:16 Blur-Fill, 16:9, and Original aspect).
- **Hold Last Frame:** Smooth configurable freeze (0.0 to 5.0 seconds) on the finished artwork before looping.

#### Use Cases:

1. **Speedpaint Video for Social Media:**
   - Open your illustration in PhotoCraft.
   - Launch **Timelapse Studio** and click **Record** (window auto-minimizes to keep your canvas clear).
   - Paint your sketch, lineart, and coloring naturally. Use **Pause** whenever taking a break, and **Record** to resume seamlessly.
   - Click **Stop**, then **Export MP4**, choose the _Reels / TikTok 9:16 (Blur)_ preset, and save your ready-to-publish video. Click **Open Folder** to reveal the file in Explorer/Finder.
2. **Step-by-Step Educational Art Tutorials:**
   - Record concept art iterations, digital portrait shading, or character design workflows.
   - Take manual **Snapshot** frames at key artistic milestones.
   - Export 1080p/4K high-resolution videos showcasing your creative process layer by layer for students and patrons.
3. **Client Proof & Creative Archive:**
   - Archive high-speed process videos to prove original authorship and document the evolution of commercial projects.
   - Click **Reset** when starting a new canvas session to clear previous cache cleanly.

---

**Русский:**
**Студия таймлапсов (Timelapse Studio)** — легковесный инструмент фоновой записи процесса рисования (спидпейнтов) для PhotoCraft Companion с нулевой задержкой пера и полным сохранением отзывчивости интерфейса. Плагин захватывает снимки холста через прямой сетевой протокол без затирания системного буфера обмена и экспортирует готовые ролики в один клик.

#### Основные возможности:

- **Нулевая задержка кисти:** Фоновый захват не блокирует рендеринг холста и не вызывает подтормаживаний кисти или планшета.
- **Эргономичное управление сессией:** Раздельные кнопки «Запись», «Пауза», «Стоп», «Снимок» и «Сброс» с выводом счетчика кадров и анимированным бейджем в тулбаре.
- **Изоляция буфера обмена ОС:** Системный буфер обмена Windows, macOS и Linux остается на 100% нетронутым во время всей сессии рисования.
- **Форматы для соцсетей:** Автоматическая адаптация роликов под вертикальный формат 9:16 (Reels, Shorts, TikTok) с элегантным размытием краев (_Blur-Fill_) и классический 16:9.
- **Фиксация финального кадра:** Настраиваемая пауза (от 0 до 5 секунд) на готовой работе в конце видео.

#### Практическое применение:

1. **Публикация спидпейнтов в соцсети (Reels, TikTok, Shorts):**
   - Откройте холст в PhotoCraft, запустите **Студию таймлапсов** и нажмите **Запись** (панель автоматически сворачивается, освобождая холст).
   - Создавайте иллюстрацию, наносите мазки и детализируйте слои в привычном темпе. Используйте **Пауза** при необходимости и **Запись** для продолжения сессии.
   - По завершении нажмите **Стоп**, затем **Экспорт MP4**, выберите пресет _Reels / TikTok 9:16 (Размытие)_ и сохраните видео. Нажмите **Открыть папку**, чтобы моментально подсветить файл в проводнике.
2. **Создание обучающих мастер-классов:**
   - Записывайте процесс цифровой живописи, скетчинга или концепт-арта для курсов, YouTube или портфолио.
   - Делайте ручные кадры кнопкой **Снимок** на ключевых этапах иллюстрации.
   - Экспортируйте ролики в разрешении 1080p или 4K с плавной частотой 30–60 кадров/сек.
3. **Фиксация авторства и чистый перезапуск:**
   - Сохраняйте этапы работы над коммерческими заказами для наглядной демонстрации заказчикам.
   - Нажмите **Сброс** при начале новой сессии для полной очистки кэша и подготовки к новой записи.

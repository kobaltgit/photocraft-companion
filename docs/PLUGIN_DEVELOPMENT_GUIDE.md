# Руководство разработчика плагинов PhotoCraft (Plugin Development Guide)

Добро пожаловать в руководство по разработке расширений и плагинов для графического редактора **PhotoCraft** и его экосистемы **PhotoCraft Companion**.

---

## 1. Архитектурная модель плагинов PhotoCraft

Графический редактор **PhotoCraft** построен по принципу *Engine-First*:
1. Все графические манипуляции, меню, фильтры и инструменты в PhotoCraft представляют собой **декларативные команды** из глобального реестра (более 500 встроенных команд).
2. Любая команда может выполняться из графического интерфейса, через командную строку (`photocraft-cli`), через протокол MCP (Model Context Protocol) или через локальный сокет управления `--control` (порт 18492, JSON-RPC 2.0).
3. **Встроенные Wasm-плагины PhotoCraft** выполняются в изолированной песочнице WebAssembly: они не имеют прямого доступа к локальной файловой системе, сети и тензорным ядрам видеокарты (GPU).
4. **PhotoCraft Companion** выступает нативным хостом-компаньоном: он подключается к сокету редактора, обеспечивая плагинам прямой доступ к аппаратным возможностям ПК:
   - **DirectML / DirectX 12 / Metal GPU:** локальный инференс современных нейросетей (Bria AI RMBG, исправление шумов, апскейлеры) через ONNX Runtime (`ort`).
   - **In-Place пиксельный конвейер:** чтение пикселей активного слоя холста в оперативную память и вставка прозрачного результата обратно с субпиксельной точностью.
   - **Глобальные хоткеи и плавающий оверлей:** привязка горячих клавиш ОС и вынесение кнопок в компактный тулбар поверх холста редактора.

---

## 2. Форматы плагинов

Экосистема поддерживает два типа плагинов для распространения:

```text
my_plugin/
├── plugin.json              # Главный декларативный манифест
├── icon.svg                 # Векторная иконка для палитры тулбара (SVG)
└── models/                  # (Опционально) Веса локальных ONNX-моделей
    └── my_model.onnx
```

### А. No-Code макросы (`plugin_type: "no_code"`)
Не требуют навыков программирования и компиляции кода. Описывают последовательную цепочку команд PhotoCraft с аргументами в формате JSON.
- **Порог входа:** нулевой.
- **Применение:** пакетные цветокоррекции, наложение сеток фильтров, виньетирование, частотное разложение, сохранение и экспорт.

### Б. Нативные нейросетевые плагины (`plugin_type: "native_ai"`)
Используют локальные модели глубокого обучения (ONNX) и скомпилированные функции Rust/C++.
- **Возможности:** удаление фона, сегментация объектов, генеративная заливка, сверхразрешение (Super-Resolution), художественные стилизации.
- **Автономия:** веса моделей поставляются прямо в папке плагина либо автоматически загружаются по указанному URL при первом запуске.

---

## 3. Спецификация манифеста `plugin.json`

Каждый плагин обязан содержать файл `plugin.json` в корне своего каталога:

```json
{
  "id": "macro.developer.hdr_pop",
  "name": "HDR Film Pop",
  "version": "1.0.0",
  "description": "Создает эффект динамического диапазона: авто-уровни, повышение резкости и насыщенности.",
  "author": "YourName",
  "plugin_type": "no_code",
  "default_hotkey": "Ctrl+Shift+H",
  "icon": "adjust",
  "commands": [
    {
      "command": "layer.duplicate",
      "params": {},
      "description": "Дублирование исходного слоя"
    },
    {
      "command": "image.adjustments.levels",
      "params": {
        "inBlack": 10,
        "gamma": 1.15,
        "inWhite": 245
      },
      "description": "Тоновая коррекция уровней"
    },
    {
      "command": "filter.sharpen.unsharpMask",
      "params": {
        "amount": 75,
        "radius": 1.2,
        "threshold": 2
      },
      "description": "Контурная резкость"
    }
  ]
}
```

### Поля манифеста:

| Поле | Тип | Обязательное | Описание |
|---|---|---|---|
| `id` | `string` | Да | Уникальный обратный доменный идентификатор плагина (например, `com.author.plugin_name`) |
| `name` | `string` | Да | Отображаемое человекочитаемое название плагина |
| `version` | `string` | Да | Семантическая версия плагина (`1.0.0`) |
| `description` | `string` | Да | Краткое описание действия |
| `author` | `string` | Нет | Имя разработчика или команды |
| `plugin_type` | `string` | Нет | `"no_code"` (по умолчанию), `"native_ai"`, `"script"`, `"external_cli"` |
| `default_hotkey`| `string` | Нет | Сочетание клавиш по умолчанию (например, `"Ctrl+Shift+B"`, `"Alt+F8"`) |
| `icon` | `string` | Нет | Имя встроенной иконки (`"scissors"`, `"adjust"`, `"brush"`) либо путь к SVG-файлу (`"icon.svg"`) |
| `commands` | `array` | Для No-Code | Список шагов команды PhotoCraft |
| `models` | `array` | Для AI | Список требуемых нейросетевых моделей с URL и хэшем |

---

## 4. Спецификация секции `models` (для нейросетевых плагинов)

Если плагину требуются локальные веса нейросети, они описываются в массиве `models`:

```json
{
  "id": "ai.author.portrait_bokeh",
  "name": "Portrait Depth & Bokeh",
  "version": "1.0.0",
  "description": "Создает реалистичное размытие заднего плана на основе карты глубины.",
  "plugin_type": "native_ai",
  "action_id": "plugin:portrait_bokeh",
  "models": [
    {
      "name": "depth_anything_v2.onnx",
      "required": true,
      "size_mb": 98,
      "url": "https://huggingface.co/models/.../depth_v2.onnx",
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    }
  ]
}
```

Компаньон автоматически:
- Проверяет наличие файла в локальной папке `plugins/<id>/models/<name>`.
- Если файл отсутствует, в графическом менеджере плагинов появляется предупреждение и кнопка **`[ ⬇ Скачать модель ]`**.
- Проверяет целостность по SHA-256 после загрузки.

---

## 5. Справочник команд PhotoCraft для No-Code плагинов

Ядро графического редактора PhotoCraft предоставляет **851 команду**.
Полный структурированный каталог всех доступных команд ядра с параметрами JSON Schema, путями в меню и хоткеями вынесен в отдельный документ:
👉 **[Полный справочник команд PhotoCraft (851 команда)](PHOTOCRAFT_COMMANDS_REFERENCE.md)**

Ниже приведены самые востребованные команды для базовой автоматизации:

### Управление слоями (`layer.*`):
* `layer.duplicate` — дублировать активный слой (`params: {}`).
* `layer.delete` — удалить активный слой (`params: {}`).
* `layer.mergeDown` — объединить с нижним слоем.
* `layer.flattenImage` — свести все слои документа.
* `layer.setProps` — изменить свойства слоя:
  ```json
  { "opacity": 0.85, "blend": "Screen", "visible": true }
  ```
* `layer.createClippingMask` / `layer.releaseClippingMask` — управление обтравочной маской.

### Коррекции изображения (`image.adjustments.*`):
* `image.adjustments.desaturate` — обесцвечивание слоя (`{}`).
* `image.adjustments.invert` — инверсия цветов (`{}`).
* `image.adjustments.brightnessContrast` — яркость и контраст:
  ```json
  { "brightness": 20, "contrast": 15 }
  ```
* `image.adjustments.levels` — уровни:
  ```json
  { "inBlack": 5, "gamma": 1.1, "inWhite": 250, "outBlack": 0, "outWhite": 255 }
  ```
* `image.adjustments.curves` — кривые:
  ```json
  { "points": [[0, 0], [64, 48], [192, 212], [255, 255]] }
  ```
* `image.adjustments.blackWhite` — тонкая настройка ч/б по цветовым диапазонам:
  ```json
  { "reds": 40, "yellows": 60, "greens": 40, "cyans": 60, "blues": 20, "magentas": 80 }
  ```
* `image.adjustments.vibrance` — сочность:
  ```json
  { "vibrance": 25, "saturation": 10 }
  ```

### Фильтры (`filter.*`):
* `filter.blur.gaussianBlur` — размытие по Гауссу:
  ```json
  { "radius": 4.5 }
  ```
* `filter.sharpen.unsharpMask` — контурная резкость:
  ```json
  { "amount": 100, "radius": 1.0, "threshold": 0 }
  ```
* `filter.sharpen.smartSharpen` — умная резкость:
  ```json
  { "amount": 120, "radius": 1.5, "reduceNoise": 15 }
  ```
* `filter.other.highPass` — цветовой контраст (High Pass) для разделения частот:
  ```json
  { "radius": 3.0 }
  ```
* `filter.noise.reduceNoise` — шумоподавление:
  ```json
  { "strength": 6, "preserveDetails": 60, "reduceColorNoise": 45, "sharpenDetails": 25 }
  ```

---

## 6. Упаковка и дистрибуция: формат `.pcpl`

Для распространения плагина среди пользователей упакуйте его содержимое в стандартный ZIP-архив с расширением `.pcpl` (PhotoCraft Plugin Package):

```bash
# Пример создания пакета в терминале
zip -r my_super_plugin.pcpl plugin.json icon.svg
```

### Структура архива:
* `plugin.json` должен находиться строго в корне архива.
* `icon.svg` — векторная иконка размером 24×24 или 32×32 px.
* Папка `models/` (по желанию): если модель небольшая, её можно включить в архив, либо указать `url` в `plugin.json` для отложенной загрузки.

### Установка пользователем:
Пользователь открывает **Менеджер плагинов** в PhotoCraft Companion и нажимает кнопку:
**`[ + Установить плагин (.pcpl) ]`**. Компаньон автоматически проверит манипест, распакует файлы в директорию `plugins/<id>/` и мгновенно выведет кнопку на плавающий тулбар без перезапуска редактора!

---

## 7. Разработка нативных Rust-плагинов (`crates/core_contracts`)

Если вы пишете нативный код на Rust:
1. Подключите крейт `core_contracts`:
   ```toml
   [dependencies]
   core_contracts = { path = "../core_contracts" }
   async-trait = "0.1"
   image = "0.25"
   ```
2. Реализуйте трейт `Plugin`:
   ```rust
   use async_trait::async_trait;
   use core_contracts::{Plugin, PluginContext, PluginResult, Result};

   pub struct MyCustomPlugin {
       manifest: core_contracts::PluginManifest,
   }

   #[async_trait]
   impl Plugin for MyCustomPlugin {
       fn id(&self) -> &str {
           &self.manifest.id
       }

       fn manifest(&self) -> &core_contracts::PluginManifest {
           &self.manifest
       }

       async fn execute(&self, ctx: &PluginContext) -> Result<PluginResult> {
           // 1. Получаем пиксели активного слоя
           let mut image = ctx.client.copy_active_layer_image().await?;

           // 2. Обрабатываем изображение (например, инвертируем каналы)
           for pixel in image.pixels_mut() {
               pixel[0] = 255 - pixel[0]; // R
               pixel[1] = 255 - pixel[1]; // G
               pixel[2] = 255 - pixel[2]; // B
           }

           // 3. Вставляем обратно в открытый холст PhotoCraft
           ctx.client.paste_image_in_place(&image).await?;

           Ok(PluginResult::ok("Слой успешно обработан!"))
       }
   }
   ```
3. Зарегистрируйте плагин в `PluginRegistry`:
   ```rust
   registry.register_native(Arc::new(MyCustomPlugin::new()));
   ```

---

## 8. Тестирование и отладка

1. Убедитесь, что PhotoCraft запущен с флагом `--control` (по умолчанию в версии 0.6.0+ сокет `127.0.0.1:18492` активен автоматически).
2. Запустите Companion:
   ```bash
   cargo run --package companion_app
   ```
3. Откройте вкладку **`[ Plugins ]`** в окне менеджера, чтобы убедиться, что ваш плагин успешно распознан и валидирован.
4. Проверьте выполнение команды по клику на кнопку плавающей панели или по назначенному хоткею.

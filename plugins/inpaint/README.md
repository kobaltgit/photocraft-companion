# AI Generative Fill (LaMa Inpainting)

[![Plugin: AI Generative Fill](https://img.shields.io/badge/Plugin-AI_Generative_Fill-blue.svg)](plugin.json)
[![Model: LaMa Inpainting](https://img.shields.io/badge/Model-LaMa_ONNX-purple.svg)](https://huggingface.co/Carve/LaMa-ONNX)
[![Type: Native AI](https://img.shields.io/badge/Type-Native_AI_(ONNX)-green.svg)](plugin.json)

**English:**
Context-aware zero-prompt neural object removal and generative fill plugin for PhotoCraft based on **LaMa (Large Mask Inpainting)** using Fast Fourier Convolutions.
Select any unwanted object, watermark, or blemish with Lasso/Marquee in PhotoCraft, trigger the plugin, and LaMa reconstructs the background texture with seamless context-aware blending directly in place (`layer.mergeDown`).

### Use Cases:
- **Object & Distraction Removal:** Erasing photobombers, power lines, trash, and background clutter from travel photos.
- **Watermark & Timestamp Clean-up:** Removing date stamps, logos, and scratched artifacts without repetitive cloning stamp strokes.
- **Surface & Skin Retouching:** Seamless deletion of severe blemishes, sensor dust spots, and texture tears while maintaining natural surrounding grain.

### Model Weights:
- **Direct Download:** [lama_inpaint.onnx (lama_fp32.onnx)](https://huggingface.co/Carve/LaMa-ONNX/resolve/main/lama_fp32.onnx) (208 MB)
- **Hugging Face Repository:** [Carve/LaMa-ONNX](https://huggingface.co/Carve/LaMa-ONNX)

---

**Русский:**
Плагин умного удаления объектов и генеративной заливки без текстовых промптов для PhotoCraft на базе архитектуры **LaMa (Large Mask Inpainting)**.
Выделите нежелательный объект, провод или дефект инструментом Лассо или Прямоугольная область в PhotoCraft, нажмите кнопку плагина — и нейросеть заполнит выделенную область окружающей текстурой и геометрией с автослиянием в активный слой (`layer.mergeDown`).

### Практическое применение:
- **Удаление лишних деталей и людей из кадра:** Бесшовное стирание случайных прохожих, проводов, мусора и дорожных знаков на пейзажных и тревел-фотографиях.
- **Очистка от водяных знаков и штампов даты:** Быстрое удаление логотипов и дат съемки без утомительной ручной работы штампом и лечащей кистью.
- **Ретушь пыли матриц и пятен:** Удаление царапин на отсканированных пленках и пылевых точек сенсора камеры с сохранением оригинального зерна и градиента неба.

### Ссылки на веса модели:
- **Прямое скачивание:** [lama_inpaint.onnx (lama_fp32.onnx)](https://huggingface.co/Carve/LaMa-ONNX/resolve/main/lama_fp32.onnx) (208 МБ)
- **Репозиторий Hugging Face:** [Carve/LaMa-ONNX](https://huggingface.co/Carve/LaMa-ONNX)


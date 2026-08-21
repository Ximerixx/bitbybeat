---
name: bitbybeat rust port
overview: Полная архитектура порта TouchDesigner-прототипа аудиоанализатора на Rust (egui + cpal + OSC), описанная в наборе md-файлов в папке md_plans/. План воспроизводит DSP-тракт, механизм полуавтономного адаптивного управления (dual-gain, lag, сигмоиды) и OSC-выход порт 7700.
todos:
  - id: overview
    content: Написать md_plans/00_overview.md (цели, стек, глоссарий TD->Rust, ограничения дампа)
    status: pending
  - id: arch
    content: Написать md_plans/01_architecture.md (потоковая модель, control-rate 60Гц, обмен данными)
    status: pending
  - id: dsp
    content: Написать md_plans/02_dsp_pipeline.md (порт сигнального графа с параметрами из дампа)
    status: pending
  - id: control
    content: Написать md_plans/03_adaptive_control.md (dual-gain, lag, ControlGain/Threshold, сигмоида)
    status: pending
  - id: osc
    content: Написать md_plans/04_osc_output.md (OSC схема порт 7700, пост-процессоры, адреса)
    status: pending
  - id: gui
    content: Написать md_plans/05_gui.md (egui layout, редактор сигмоиды, метры, спектр, пресеты)
    status: pending
  - id: params
    content: Написать md_plans/06_params_and_presets.md (таблица дефолтов из дампа, сериализация)
    status: pending
  - id: structure
    content: Написать md_plans/07_project_structure.md (крейт, модули, Cargo, этапы)
    status: pending
isProject: false
---

# Архитектура Rust-порта bitbybeat (аудиоанализатор → OSC)

Цель: заменить прожорливый TouchDesigner-прототип автономной Rust-тулзой с GUI (egui), аудио I/O (cpal), полуавтономным адаптивным управлением и OSC-сервером (порт 7700). Дамп `project1_dump.json` (1597 нод) реверс-инжинирится: ~60-80 нод — реальный DSP, остальное — GUI TouchDesigner.

## Что делаю
Создаю папку `md_plans/` и пишу туда 8 md-файлов (без изменения кода — это планировочные документы). Ниже — суть каждого.

## Ключевые решения (по ответам)
- Кроссплатформенно (Linux приоритет + Windows) через `cpal`.
- VST3-хостинг — заложить как опциональную будущую фичу (низкий приоритет), сейчас не реализуем; в дампе он в байпасе (`vstbypass=True`, `switch_neutone=0`).
- GUI максимально функциональный: контролы DSP + механизм адаптивного управления (сигмоида), метры, спектр, выбор устройств, опциональный мониторинг-выход.

## Разобранный сигнальный тракт (факт из дампа)
```mermaid
flowchart TD
  src["audiodevin / audiofilein"] --> monoDsp["math1 preoff0.1 gain2.28 (ГЕЙН В DSP)"]
  monoDsp --> comp["audiodyna: compressor thr-20.6 ratio0.638 gain6.9"]
  comp --> vst["audiovst Neutone FX (BYPASS)"]
  vst --> sw["switch_neutone"]
  sw --> lp["audiofilter1 LP 150Hz roll20 -> low"]
  sw --> bp["audiofilter2 BP 800Hz roll20 -> mid"]
  sw --> hp["audiofilter3 HP 3500Hz res0.8 -> high"]
  sw --> spec["audiospect -> shuffle 440 бинов"]
  lp --> arms1["analyze rmspower -> low_"]
  bp --> arms2["analyze rmspower -> mid_"]
  hp --> arms3["analyze rmspower -> high_"]
  arms1 --> kick["limit->logic->trigger retrig0.08 -> Kickdrum"]
  arms3 --> snare["limit->logic -> Snare"]
  spec --> rhythm["Rythm_ / centroid / fms / sms"]
```

Отдельная адаптивная ветвь (полуавтономное управление):
```mermaid
flowchart TD
  inm["InMerge"] --> rms["rms rmspower"]
  rms --> m2["math2 gain0.64 (ГЕЙН КОРРЕКЦИИ, отдельный от DSP)"]
  m2 --> lag["lagClearRythmIn_ lag1=2 lag2=4 accel2=3"]
  lag --> lg["low/mid/highControlGain (math: preoff/gain/postoff)"]
  lag --> kt["kick/snare/rythmControlThreshold_math -> express (СИГМОИДА)"]
  lg --> dspctl["правит Gain полос в audioAnalysis"]
  kt --> dspctl2["правит Threshold kick/snare/rythm"]
```

## Файлы в md_plans/
- `00_overview.md` — цели/не-цели, стек и обоснование крейтов (`eframe/egui`, `cpal`, `rustfft`+`realfft`, `biquad`, `rosc`, `ringbuf`/`triple_buffer`, `serde`+`ron`), глоссарий TouchDesigner→Rust (CHOP=буфер каналов, math/lag/limit/trigger/analyze/audiofilter/audiospect → их DSP-эквиваленты), ограничение дампа (формулы `express` не сохранены → сигмоида задаётся параметрически).
- `01_architecture.md` — потоковая модель: RT audio-callback (cpal) → lock-free ring buffer → DSP/analysis worker (control-rate 60 Гц, как `rate:60` в дампе) → атомарный снапшот метрик → GUI (egui) и OSC-sender. Диаграммы потоков и передачи данных, backpressure, отделение аудио-rate от control-rate.
- `02_dsp_pipeline.md` — точный порт графа: source→mono(avg)→compressor(`audiodyna`)→[vst bypass]→3 биквад-фильтра (LP150/BP800/HP3500 с rolloff/resonance)→RMS по полосам→детект kick/snare (limit→logic→trigger, retrigger 0.08с)→спектр (FFT, shuffle 440 бинов)→rhythm/spectral centroid/flux. Таблица «нода TD → Rust-функция → параметры из дампа».
- `03_adaptive_control.md` — сердце тулзы: dual-gain (`math1`→DSP vs `math2`→коррекция), `lag` сглаживание, `*ControlGain`/`*ControlThreshold` как аффинные мапперы (preoff/gain/postoff/torange из дампа), конфигурируемая сигмоида (замена `express`) с настраиваемыми коэффициентами, разводка выходов в контролы DSP. Мульти-поточные контролы для файнтюна под конкретный трек.
- `04_osc_output.md` — OSC-схема: порт 7700, `timeslice`/sample-format, пост-процессоры `Low_Base`/`Mid_Base`/`High_Mid`/`Jumper_trigger`/`Zig_Zagger`/`Count_Analysis` (счётчики kick/snare по 4/8/16, zigzag, jumper), список OSC-адресов и типов, частота отправки.
- `05_gui.md` — egui layout: выбор устройств вход/выход, глобальные dual-gain, панели low/mid/high (gain/threshold/smooth/add), панели kick/snare/rythm (threshold + сигмоида-редактор с превью кривой), метры уровней, спектр-визуализатор, OSC-монитор/статус, пресеты под трек (save/load).
- `06_params_and_presets.md` — полная таблица параметров с дефолтами из дампа (частоты фильтров, компрессор, gains, retrigger, lag, limit, диапазоны math) для точного воспроизведения; модель конфигурации и сериализация пресетов (serde/ron), совместимость профилей.
- `07_project_structure.md` — раскладка крейта и модулей (`audio/`, `dsp/`, `control/`, `osc/`, `gui/`, `config/`), `Cargo.toml` зависимости, этапы реализации (MVP: вход→полосы→OSC; далее адаптив, спектр, GUI, пресеты, опц. VST) и критерии готовности.

## Риски / допущения
- Сигмоидные формулы `express` в дампе редуцированы до чисел — воспроизводим как параметрическую сигмоиду с калибровкой под слух/трек.
- Точные значения `torange`/скрытых параметров некоторых `math` берём из дампа, где присутствуют; недостающие помечаем как «подобрать при калибровке».
- `audiospect`/`shuffle 440` и spectral flux/centroid реализуем через `realfft`; численное совпадение с TD не гарантируется 1:1, но поведенчески эквивалентно.
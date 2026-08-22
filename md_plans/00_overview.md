# 00 — Обзор проекта и карта инвентаризации

> **Эталон (source of truth):** `Analysis 2.2_v6_calibrated.9.toe.dir/` — раскрытый через `toeexpand` проект TouchDesigner.
> При любом расхождении этот дамп считается истиной; старый `project1_dump.json` и предыдущие предположения игнорируются.
> В `.toe.dir` параметры лежат в `*.parm` **вместе с выражениями** (`expr`), DAT-логика — в `*.text`/`*.table`/`*.logic`. Это то, чего не было в JSON.

## Что это за проект
Полуавтономный аудиоанализатор на TouchDesigner: берёт аудио (живой вход ASIO или файл), раскладывает на 3 полосы (low/mid/high), детектит удары (kick/snare/rythm), считает спектральные признаки (centroid, fms, sms), гоняет счётчики долей (4/8/16) и набор пост-обработчиков-«генераторов» (Low_Base, Mid_Base, High_Mid, Jumper, Zig_Zagger), а результат шлёт по **OSC на порт 7700**. Плюс механизм **адаптивного управления**: отдельная RMS-ветвь через сглаживание и сигмоиды подстраивает гейны полос и пороги детекции под текущий уровень сигнала.

## Главная цель порта (правки заказчика)
**Не просто повторить проект, а сделать его максимально конфигурируемым** с GUI, где настраивается вся обработка вживую: тумблеры включения/bypass на каждой ступени (мёртвые ноды тоже — их включают по ситуации), крутилки на все гейны и сигмоиды, переключаемая сигмоида в порогах, выбор источника (приоритет — аудиоустройства, в т.ч. monitor-выход). Детали — в `10_customer_requirements.md`; при конфликте с «оптимизациями» из `09` приоритет у требований `10`.

## Масштаб
- Всего нод: **757** (по `toe_model.json`). Полная перепись — в `08_node_census.md`.
- Реального DSP/логики: ~120–150 нод. Остальное — GUI (палитрный компонент AudioAnalysis: слайдеры, кнопки, индикаторы, текст) и служебные DAT-callbacks.
- Управление на control-rate: во всех активных CHOP стоит `timeslice=on`, логические дорожки помечены `rate = 60` → **60 Гц**.

## Две ветви (ключевая топология)
Проект физически разделён на два тракта, сходящихся только через **параметры-выражения**:

1. **Ветвь адаптивного управления** (верхний уровень `/project1`):
   `audiodevin1 (ASIO X-AIR, live)` → `InMerge` → `rms` → `math2 (gain 0.64)` → `lagClearRythmIn_` → аффинные мапперы `*ControlGain` / `*ControlThreshold` (+ 2 сигмоиды) → пишут значения в кастомные параметры компонента `audioAnalysis`.

2. **Ветвь анализа** (компонент `/project1/audioAnalysis`, палитра AudioAnalysis v6.1.1):
   `audiofilein1 (внутренний mp3)` → `in1` → `math1 (avg=моно)` → `audiodyna1` → `[audiovst Neutone — BYPASS]` → `switch_neutone` → 3 фильтра + спектр → RMS/детекция/признаки → `out1` → OSC.

> ⚠️ **Находка №1 (важно для порта).** В сохранённом состоянии вход компонента `audioAnalysis` **не подключён** снаружи (`inputs=[]`), поэтому анализ идёт по **внутреннему демо-mp3**, тогда как адаптивная RMS-ветвь читает **живой ASIO-вход**. Это два разных источника. В Rust-порте их надо **свести в один вход** (см. `09_optimizations.md`).

> ⚠️ **Находка №2.** Верхнеуровневый `audiodyna1` (компрессор `thr −20.6, ratio 0.638, gain 6.9`) **ни к чему не подключён на выходе** (мёртвая нода-монитор). Внутри `audioAnalysis` стоит второй `audiodyna1` с **дефолтными** параметрами. То есть «настоящий» компрессор в фактическом тракте анализа не применяется. См. `01_signal_pipeline.md`.

## Механизм «control → DSP» (как адаптив правит анализ)
Не через CHOP-export, а через **выражения в кастомных параметрах** компонента `audioAnalysis`:

| Параметр audioAnalysis | Выражение | Что задаёт |
|---|---|---|
| `Lowgain`  | `op('lowControlGain')[0]`  | гейн low-полосы |
| `Midgain`  | `op('midControlGain')[0]`  | гейн mid-полосы |
| `Highgain` | `op('highControlGain1')[0]` | гейн high-полосы (внимание: `highControlGain**1**`, gain 1.93) |
| `Kickthresh`  | `op('kickControlThreshold')[0]`  | порог kick (сигмоида) |
| `Snarethresh` | `op('snareControlThreshold')[0]` | порог snare (сигмоида) |
| `Rythmthresh` | `op('rythmControlThreshold')[0]` | порог rythm |
| `Low/Mid/High/Kickdetection/…` | `op('./low_')[0]` и т.п. | обратное чтение выходов (для GUI) |

Пороги `Lowthresh/Midthresh/Highthresh` — **статические** (0.116/0.052/0.116), адаптируется только **гейн** полос; у kick/snare/rythm наоборот — адаптируется **порог**.

## Карта документов инвентаризации
| Файл | Содержание |
|---|---|
| `00_overview.md` | этот обзор, две ветви, находки, механизм control→DSP |
| `01_signal_pipeline.md` | моно→компрессор→VST(bypass)→3 полосы→RMS→сглаживание→add, точные параметры |
| `02_beat_detection.md` | kick/snare/rythm: limit→logic→trigger, пороги, retrigger 0.08 |
| `03_spectral_features.md` | audiospect, shuffle 440, centroid, fms, sms |
| `04_adaptive_control.md` | dual-gain, lag, аффинные мапперы, 2 сигмоиды, разводка |
| `05_post_processors.md` | Count_Analysis (4/8/16), Low_Base, Mid_Base, High_Mid, Jumper_trigger, Zig_Zagger |
| `06_osc_output.md` | OSCOutMerge, список каналов, порт 7700, format sample |
| `07_params_defaults.md` | таблица дефолтов всех кастомных параметров |
| `08_node_census.md` | автоперепись всех 757 нод по контейнерам |
| `09_optimizations.md` | предложения по упрощению/оптимизации для Rust-порта |
| `10_customer_requirements.md` | **правки заказчика**: слой конфигурируемости (тумблеры, крутилки, источник, сигмоида-toggle) — приоритетный |

## Аудио-конфигурация из дампа
- **Вход (live):** `audiodevin1` — драйвер `asio`, устройство `X-AIR_ASIO_Driver`, каналы `1:In_1 2:In_2`, `format=stereo`, `ratemode=resample`.
- **Вход (файл):** `audiofilein1` — mp3 (в дампе путь на Windows-машину + fallback на `app.samplesFolder`).
- **Мониторинг-выход:** `audiodevout` (внутри audioAnalysis) — включается параметром `Audioout`, громкость `Listenvolume=0.612`, устройство FiiO KA3.
- **VST:** `audiovst` Neutone FX — в дампе **в байпасе** (`vstbypass = not Neutoneactive`, `Neutoneactive=off`).

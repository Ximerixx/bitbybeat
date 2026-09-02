# 02 — Детекция ударов: kick / snare / rythm

Все ноды внутри `/project1/audioAnalysis`. Общий паттерн детекции: **вычесть порог → clamp → logic (в бинарь) → trigger (импульс с retrigger)**.

## Kick (по low-полосе)
```
math2 (RMS low) ─► math8 ─► limit4 ─► logic1 ─► trigger1 ─► null_kickSignal ─► switch1 ─► Kickdrum
```
- `math8`: `preoff = -Kickthresh`  (дефолт `Kickthresh≈0.328`, приходит из `op('kickControlThreshold')[0]` — сигмоида).
- `limit4`: `type=clamp`, min=0, max=100.
- `logic1`: TD Logic CHOP (по умолчанию `convert=off` → «≠0 ? 1 : 0»), даёт гейт.
- `trigger1`: `retrigger=0.08` c, `attack=decay=sustain=release=0` → чистый импульс, не чаще ~12.5 раз/с.
- `switch1`: `index = op('kick').par.Active` (вкл/выкл детекции).
- Выход `Kickdrum` (null) → кастомный параметр `kickOutput` (см. `Kickdrum_export`).

## Snare (по high-полосе)
```
math16 (RMS high ×4) ─► math9 ─► limit5 ─► logic2 ─► nullsnareSignal ─► switch2 ─► Snare
```
- `math9`: `preoff = -Snarethresh` (дефолт `Snarethresh≈0.338`, из `op('snareControlThreshold')[0]`).
- `limit5`: clamp 0..100.
- `logic2` → `nullsnareSignal`.
- ⚠️ У snare, в отличие от kick, в дампе **нет отдельного trigger** между logic2 и null — импульс формирует сам logic2 (гейт). Retrigger-логика для snare реализуется в `Count_Analysis`/пост-обработке.
- `switch2`: `index = op('snare').par.Active`. Выход `Snare` → `snareOutput`.

## Rythm (по спектру)
```
… спектр … ─► math10 ─► limit6 ─► math11 ─► logic3 ─► switch10 ─► Rythm_
```
- `math10`: `preoff = -Rythmthresh`, `gain = 1000` (масштабирование спектральной меры).
- `limit6`: `type=clamp`, `min = Rythmthresh` (дефолт `Rythmthresh≈69.9`, из `op('rythmControlThreshold')[0]`), max=100.
- `math11`: `preoff = -op('limit6').par.min` (вычесть тот же порог обратно — нормировка к нулю).
- `logic3`: `convert=nonzero` → бинарь.
- `switch10`: `index = op('rythm').par.Active`. Выход `Rythm_` → `rythmOutput`.

## Экспорт-таблицы (какой выход → какой OSC-канал)
Из `*_export` DAT (map «path→parameter»):
| Контейнер | null-нода | параметр Output |
|---|---|---|
| `kick` | `Kickdrum` | `kickOutput` |
| `snare` | `Snare` | `snareOutput` |
| `rythm` | `Rythm_` | `rythmOutput` |
| `low` | `low_` | `lowOutput` |
| `mid` | `mid_` | `midOutput` |
| `high` | `high_` | `highOutput` |
| `smsd` | `sms` | `smsdOutput` |
| `fmsd` | `fms` | `fmsdOutput` |
| `spectralCentroid` | `centroid` | `spectralCentroidOutput` |

## Пороги (сводка)
| Детектор | Порог-параметр | Дефолт | Источник |
|---|---|---|---|
| kick   | `Kickthresh`  | 0.328 | сигмоида `kickControlThreshold` |
| snare  | `Snarethresh` | 0.338 | сигмоида `snareControlThreshold` |
| rythm  | `Rythmthresh` | 69.9  | `rythmControlThreshold` (аффинный) |

## Модель для порта
```
kick  = trigger( gate( clamp((rms_low  - Kickthresh )*?,0,100) ), retrigger=0.08 )
snare = gate( clamp((rms_high - Snarethresh)*?,0,100) )        # trigger — на этапе счётчиков
rythm = gate_nonzero( clamp(spectralMeasure - Rythmthresh, ...) )
```
Пороги kick/snare/rythm в реальном времени подменяются адаптивом (см. `04_adaptive_control.md`). Retrigger kick = 80 мс — держать как параметр.

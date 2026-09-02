# 01 — Сигнальный тракт анализа (audioAnalysis)

Все ноды ниже — внутри `/project1/audioAnalysis`, если не указано иное. Параметры взяты дословно из `*.parm`; выражения даны в обратных кавычках.

## 1. Источник → моно → компрессор → VST(bypass)
```
audiofilein1 (mp3)  ──► in1 ──► math1 (chanop=avg → МОНО)
                                   │
                                   ▼
                                audiodyna1 (компрессор, параметры ДЕФОЛТНЫЕ)
                                   │
                        merge2 (audiodyna1, audiodyna1)   ← оба входа = один и тот же audiodyna1
                                   │
                                   ▼
                                audiovst (Neutone FX)  ── vstbypass = not Neutoneactive  (BYPASS)
                                   │
                                math17 (chanop=avg)
                                   │
                                   ▼
                             switch_neutone   index = parent().par.Neutoneactive   (=0 → берёт math17, т.е. байпас)
```
- **math1**: `chanop=avg` — усреднение стерео в моно.
- **audiodyna1** (внутренний): без заданных параметров компрессии → фактически проходной. «Боевой» компрессор `thr −20.6 / ratio 0.638 / gain 6.9` — это **верхнеуровневый** `/project1/audiodyna1`, который **никуда не подключён** (см. Находку №2 в обзоре). ⚠️ несоответствие: калиброванные параметры компрессора в фактическом тракте не работают.
- **merge2** склеивает audiodyna1 сам с собой (артефакт настройки VST-входа).
- **audiovst**: Neutone FX VST3, `tempo=120`, 4 параметра (`Neutonebass/drums/vocals/other`), в дампе выключен (`Neutoneactive=off` → bypass). Порт: **опциональная будущая фича**, не в MVP.
- **switch_neutone**: выбирает между сухим (math17) и VST-сигналом. При `Neutoneactive=off` → сухой сигнал идёт дальше во все ветви.

`switch_neutone` — точка ветвления: его выход идёт в **3 фильтра**, **спектр** и мониторинг-выход `out2`/`audiodevout`.

## 2. Три полосы (биквад-фильтры TouchDesigner `audiofilter`)
| Нода | filter | cutoff | rolloff | resonance | rename→ |
|---|---|---|---|---|---|
| `audiofilter1` | lowpass (по умолчанию) | **150 Гц** (`cutofflog=2.17609`) | 20 dB/oct | — | `rename1` → `low` |
| `audiofilter2` | **bandpass** | **800 Гц** (`cutofflog=2.90309`) | 20 dB/oct | — | `rename2` → `mid` |
| `audiofilter3` | **highpass** | **3500 Гц** (`cutofflog=3.54407`) | 15 dB/oct | **0.8** | `rename3` → `high` |

`cutofflog = log10(freq)`. Все три с `timeslice=on`.

## 3. Уровни полос: RMS → gain/threshold → clamp → add → smooth
Для каждой полосы одинаковая структура (пример low, mid/high аналогичны):

```
low  ─► analyze1 (rmspower) ─► math2 ─► math3 ─► limit1 ─► addLow ─► filter1 ─► switch4 ─► low_ ─► (par Output)
mid  ─► analyze2 (rmspower) ─► math12(gain2) ─► math4 ─► limit2 ─► addMid ─► filter2 ─► switch5 ─► mid_
high ─► analyze3 (rmspower) ─► math16(gain4) ─► math5 ─► limit3 ─► addHigh ─► filter3 ─► switch6 ─► high_
```

Пооперационно (с реальными выражениями):

| Полоса | RMS | пред-гейн | threshold+gain (math) | clamp | add | smooth (filter width) |
|---|---|---|---|---|---|---|
| **low**  | `analyze1` rmspower | `math2` (без изм.) | `math3`: `preoff=-Lowthresh`, `gain=Lowgain` | `limit1` clamp 0..100 | `addLow`: `preoff=Lowadd` | `filter1` width=`Lowsmooth` |
| **mid**  | `analyze2` rmspower | `math12` gain=2 | `math4`: `preoff=-Midthresh`, `gain=Midgain` | `limit2` clamp 0..100 | `addMid`: `preoff=Midadd` | `filter2` width=`Midsmooth` |
| **high** | `analyze3` rmspower | `math16` gain=4 | `math5`: `preoff=-Highthresh`, `gain=Highgain` | `limit3` clamp 0..100 | `addHigh`: `preoff=Highadd` | `filter3` width=`Highsmooth` |

Где параметры компонента (дефолты из дампа):
- `Lowthresh=0.116  Midthresh=0.052  Highthresh=0.116` (статические)
- `Lowgain≈1.84  Midgain≈1.98  Highgain≈1.45` — **приходят из адаптива** (`op('lowControlGain')[0]` и т.д.)
- `Lowadd=-0.492  Midadd=-0.363  Highadd=-0.32`
- `Lowsmooth=0.276  Midsmooth=0.224  Highsmooth=0.093`
- `filterN`: `spike=0.1`, `speedcoeff=0` (TD Filter CHOP = скользящее сглаживание окном `width` секунд).

**Формула уровня полосы** (обобщённо, band ∈ {low,mid,high}):
```
rms_b   = RMS(bandFiltered)                     # low: как есть; mid: ×2; high: ×4 (пред-гейн)
lvl_b   = clamp( (rms_b - Thresh_b) * Gain_b , 0, 100 )
out_b   = smooth( lvl_b + Add_b , width = Smooth_b )
```
Выход `low_/mid_/high_` пробрасывается в кастомный параметр `Output` соответствующего под-контейнера и дальше в OSC.

Переключатели `switch4/5/6` (`index = op('low').par.Active` и т.д.) — включение/выключение полосы; при выкл. подаётся `constant_offlow`.

## 4. Мониторинг
- `out2 ← switch_neutone` (сырой сигнал наружу компонента).
- `audiodevout ← switch_neutone`: `active = parent().par.Audioout`, `volume = Listenvolume (0.612)`, устройство FiiO KA3. В порте — опциональный «listen»-выход.

## Резюме параметров тракта (для порта 1:1)
- Моно = среднее каналов.
- Компрессор: в фактическом тракте не активен; калиброванные значения (`−20.6 / 0.638 / +6.9 dB`) держать как **опциональный** пре-компрессор (по умолчанию соответствовать дампу — выкл./проходной, но значения сохранить).
- Фильтры: LP 150 (roll 20), BP 800 (roll 20), HP 3500 (roll 15, res 0.8). Реализация — биквады с указанным порядком (rolloff/6 ≈ число секций).
- Далее по каждой полосе: RMS → аффинное (порог/гейн) → clamp[0,100] → +add → сглаживание окном `Smooth` секунд.

# 04 — Адаптивное управление (сердце тулзы)

Живёт на верхнем уровне `/project1`. Идея: отдельная RMS-ветвь измеряет общий уровень входа, сглаживает его и через аффинные мапперы + сигмоиды подстраивает **гейны полос** и **пороги детекции** в компоненте `audioAnalysis`.

## Тракт управления
```
audiodevin1 (ASIO live) ─► InMerge ─► rms (rmspower)
                                        │
                                math2 (preoff=0.1, gain=0.64)          ← «гейн коррекции»
                                        │
                            lagClearRythmIn_ (lag1=2, lag2=4, accel1=1, accel2=3)   ← сглаживание
                                        │
        ┌───────────────┬───────────────┼───────────────┬───────────────┬──────────────┐
        ▼               ▼               ▼               ▼               ▼              ▼
 lowControlGain   midControlGain  highControlGain1  kickControl…_math  snareControl…_math  rythmControlThreshold
        │               │               │               │(→express)      │(→express)          │
     Lowgain         Midgain         Highgain       Kickthresh        Snarethresh         Rythmthresh
```
Параллельно `InMerge ─► math1 (preoff=0.1, gain=2.28)` — «DSP-гейн» (dual-gain: math1 для тракта, math2 для коррекции). ⚠️ В текущей разводке `math1` на верхнем уровне не имеет явного потребителя-соседа — держать как входной гейн живого сигнала в порте.

## Сглаживание (`lagClearRythmIn_`, TD Lag CHOP)
- `lag1 = 2`, `lag2 = 4` — время нарастания/спада (сек): реакция вверх быстрее (2 c), вниз медленнее (4 c).
- `accel1 = 1`, `accel2 = 3` — ограничение ускорения (сглаживание рывков), сильнее на спаде.
- `timeslice=on` → пересчёт на control-rate 60 Гц.
- В порте: асимметричный сглаживатель (attack/release + ограничение производной).
- ⚠️ **`10.R3`: lag — hardwired и stateful.** Bypass запрещён; при смене параметров внутреннее состояние (буфер сглаживания/инерция) **не сбрасывать** — нода «помнит последние значения», это критично.

## Аффинные мапперы (TD Math CHOP)
Порядок вычисления TD Math CHOP: `y = postoff + gain·(preoff + x)`, затем при заданном `torange` — линейный ремап `fromrange(0..1) → torange`. `chanop=add` для одноканального входа — no-op.

Пусть `L = lag(rms·0.64 + 0.064)` (выход `lagClearRythmIn_`). Тогда:

| Выход | Нода | preoff | gain | postoff | torange | Формула |
|---|---|---|---|---|---|---|
| **Lowgain**  | `lowControlGain`  | 0.2 | 4.57 | −0.2 | — | `4.57·(L+0.2) − 0.2` |
| **Midgain**  | `midControlGain`  | 0.5 | 4.05 | −0.6 | — | `4.05·(L+0.5) − 0.6` |
| **Highgain** | `highControlGain1`| 0   | 1.93 | 0    | — | `1.93·L` |
| **kick_x**   | `kickControlThreshold_math`  | 0   | 3.92 | −0.3 | 0..0.5  | `remap₀₁→₀‥₀.₅( 3.92·L − 0.3 )` |
| **snare_x**  | `snareControlThreshold_math` | 0.5 | 6.78 | −0.2 | 0..0.09 | `remap₀₁→₀‥₀.₀₉( 6.78·(L+0.5) − 0.2 )` |
| **Rythmthresh** | `rythmControlThreshold`   | 1.8 | 4.76 | −0.8 | 0..6    | `remap₀₁→₀‥₆( 4.76·(L+1.8) − 0.8 )` |

> ⚠️ **highControlGain vs highControlGain1.** Существуют обе ноды: `highControlGain` (gain 0.77, postoff +0.3) и `highControlGain1` (gain 1.93). Параметр `Highgain` берёт значение из **`highControlGain1`**. `highControlGain` — потребителей нет (запасной/старый). В порте использовать `highControlGain1`.

## Сигмоиды (восстановлены из дампа — главная находка)
Пороги kick/snare получаются прогоном аффинного выхода через логистическую функцию (`CHOP:express`):

```
Kickthresh  = 0.7 / (1 + exp(-5.4·(kick_x  − 0.3)))      # kickControlThreshold.expr0
Snarethresh = 0.9 / (1 + exp(-2.1·(snare_x − 0.5)))      # snareControlThreshold.expr0
```
- kick: потолок 0.7, крутизна 5.4, центр 0.3.
- snare: потолок 0.9, крутизна 2.1, центр 0.5.
- `me.inputVal` = выход соответствующего `*_math` (т.е. `kick_x`/`snare_x`).
- ⚠️ **`10.R4/R5`: сигмоида переключаема и настраиваема.** Тумблер «обрабатывать порог сигмоидой»: ON → `thresh = sigmoid(mapper_out)`; OFF → `thresh = mapper_out` напрямую. Параметры `ceil/k/center` — крутилки в GUI с превью кривой (для kick/snare, опц. rythm).

## Разводка в DSP (механизм «control → анализ»)
Через выражения кастомных параметров `audioAnalysis` (не CHOP-export):
```
audioAnalysis.par.Lowgain     = op('lowControlGain')[0]
audioAnalysis.par.Midgain     = op('midControlGain')[0]
audioAnalysis.par.Highgain    = op('highControlGain1')[0]
audioAnalysis.par.Kickthresh  = op('kickControlThreshold')[0]
audioAnalysis.par.Snarethresh = op('snareControlThreshold')[0]
audioAnalysis.par.Rythmthresh = op('rythmControlThreshold')[0]
```
Эти параметры далее входят в формулы полос/детекторов (см. `01`/`02`).

## Наблюдаемые дефолты (кэш eval из дампа)
`Lowgain≈1.84  Midgain≈1.98  Highgain≈1.45  Kickthresh≈0.328  Snarethresh≈0.338  Rythmthresh≈69.9`.

> ⚠️ **Несоответствие Rythmthresh.** Выражение параметра в дампе выглядит как `op('rythmControlThreshold')[0] op('rythmControlGain')[0]` (два оп-вызова, при этом `rythmControlGain` в проекте **отсутствует**). Диапазон `rythmControlThreshold` = 0..6, а кэш-значение параметра = 69.9. Значит либо строка выражения битая/устаревшая, либо порог rythm домножался на несуществующую ноду. **Для порта:** взять `Rythmthresh = rythmControlThreshold[0]` (0..6) и калибровать; значение 69.9 не воспроизводить вслепую.

## Модель для порта (control-rate 60 Гц)
```
lvl   = rms(input_mono)
c     = 0.64*lvl + 0.064               # math2
L     = lag_asym(c, up=2s, down=4s, accel_up=1, accel_down=3)
Lowgain  = 4.57*(L+0.2) - 0.2
Midgain  = 4.05*(L+0.5) - 0.6
Highgain = 1.93*L
kick_x   = clampmap01(3.92*L - 0.3) -> [0,0.5]
snare_x  = clampmap01(6.78*(L+0.5) - 0.2) -> [0,0.09]
Kickthresh  = 0.7/(1+exp(-5.4*(kick_x -0.3)))
Snarethresh = 0.9/(1+exp(-2.1*(snare_x-0.5)))
Rythmthresh = clampmap01(4.76*(L+1.8) - 0.8) -> [0,6]
```
Все коэффициенты — в конфиг/GUI (файнтюн под трек). Сигмоиды дать редактируемыми (потолок/крутизна/центр).

# 05 — Пост-процессоры и счётчики долей

Верхнеуровневые компоненты `/project1`, выходы которых сливаются в OSC (`OSCOutMerge`). Все на control-rate (`timeslice=on`).

## Count_Analysis — счётчики долей 4/8/16
Вход `in1` — сигнал с каналами `kick`/`snare` (`select2=kick`, `select3=snare`). Для каждого — три счётчика по модулю:

| Счётчик | limitmax | смысл |
|---|---|---|
| `count4k`/`count4s`  | 3  | цикл на 4 (0..3) |
| `count8k`/`count8s`  | 7  | цикл на 8 (0..7) |
| `count16k`/`count16s`| 15 | цикл на 16 (0..15) |

- `output=cl` (count → «cycle»), `resetvalue=1`, `timeslice=on`.
- `expressNk/Ns`: `if ($V == 1, 1, 0)` — импульс в момент, когда счётчик == 1 (начало цикла доли).
- `triggerNk/Ns`: чистые триггеры (все ADSR=0) по фронтам счётчика.
- `logicNk/Ns`: `preop=toggle` — переключатель-состояние по каждому удару доли.
- Всё переименовывается (`renameto=trigger4k`, `logic4k`, `express4k`, …) и сливается: `mergeall = merge1+merge2+merge3`, `out1 = mergeall`.
- В OSC уходят каналы `trigger4k/4s/8k/8s/16k/16s` (через верхнеуровневый `selectTriggers`).

Итог: метрономная сетка по каждому kick/snare — на каких долях 4/8/16-такта случился удар. Копии `Count_Analysis` встроены также в `Jumper_trigger` и `Zig_Zagger`.

## Low_Base — «раскачка» под low+kick
Компонент-генератор плавной огибающей, управляемой low-полосой и kick.
- Входные каналы: `low` (`select3`), `kick` (`select4`).
- Параметры: `Amplifylow`, `Gain`, `Lag`, `Offset`, `Speed`, `Wlength`.
- `math1`: `gain = Amplifylow` (усиление low).
- `math10`: `gain = Gain`.
- `math2/math3/math8`: ремапы с `torange` из `Lag` — динамическая длина сглаживания.
- Каскад `lag1/lag2/lag3`: времена лагов **сами модулируются kick** (`lag2['kick']`, `math8['kick']`) — т.е. на удары огибающая реагирует иначе.
- `math4`: `postoff = Offset`.
- `speed1` + `switch1` (`index=Speed`): опциональное интегрирование (Speed CHOP) вместо прямого значения.
- `trail1 wlength = Wlength`. Выход `rename1 → Low_Base`.

## Mid_Base — аналог для mid (с влиянием low и kick)
- Каналы: `low` (`select1`), `mid` (`select2`), `kick` (`select3`).
- Параметры: `Gain`, `Gain2`, `Defaultlag`, `Offset`, `Speed`, `Wlength`.
- `math1`: `chopop=mul`, `gain=Gain` (перемножение low-огибающей).
- `math2`: `preoff = math1['low']`, `gain = Gain2` (mid, смещённый low).
- Многоступенчатый lag (`lag1..lag5`), времена завязаны на `kick` и `Defaultlag` (`torange` = кратные `Defaultlag`: ×1, ×3, ×10).
- `speed1`+`switch1` как в Low_Base. Выход `Mid_Base`.

## High_Mid — аналог для high (с влиянием mid и snare)
- Каналы: `mid` (`select1`), `high` (`select2`), `snare` (`select3`).
- Параметры: `Midgain`, `Highgain`, `Snaregain`, `Defaultlag`, `Offset`, `Speed`, `Wlength`.
- `math1`: `mul`, `gain=Midgain`.
- `math2`: `preoff=math1['mid']`, `gain=Highgain`, `postoff=lag6['snare']`, `torange 0..0.5` — high, смещённый mid и модулированный snare.
- `math6`: `gain=Snaregain` — вклад snare. Лаги завязаны на `snare` и `Defaultlag`.
- Выход `High_Mid`.

## Jumper_trigger — «прыжковый» генератор по долям
Сложный модуль: берёт триггер доли из встроенного `Count_Analysis` (по умолчанию `trigger8k`, канал настраивается `par.Channames`) и полосы low/mid/high, и генерирует «прыгающее» значение.
- Параметры: `Audiofloat`, `Torange31`, `Torange32`, `Increments`, `Lagfloat2`, `Type`, `Mode`, `Channames`, `Generate`, `Seed`.
- `datto1`: читает DAT `parameter1` (par `Mode`) → режим работы; `switch1 index = datto1['Mode']` переключает две ветви (`math1` / `math13`).
- `randomCHOP1`: питон-расширение `RandExt` (Script CHOP) генерит случайную последовательность по `Seed`/`Samples`/`Range`/`Unique` — источник «прыжков».
- `lag2/lag3`: лаг с `overshoot` (`lag_Overshoot['onTrigger']`) — упругий отскок на триггере.
- `limit1`: clamp 0.8..1.1; `math5/math7`: ремапы вокруг 0.9..1.1.
- `chopexec1`: включает/выключает `par.Increments` по фронту `datto1`.
- Выход `rename1 → Jumper`.
- Семантика: на каждую заданную долю выдаёт случайно-модулированный «скачок», сглаженный с отскоком. Для порта — генератор событий/огибающих, опциональный.

## Zig_Zagger — зигзаг-генератор
- Вход: триггер доли из встроенного `Count_Analysis1` (`select1`, канал `par.Channames`, дефолт `kick`) + полоса (`select2`, `par.Channames2`, дефолт `low`; берётся `high` через `math_high` scope=high, ремап 0..4).
- `delete1`: чистит лишние каналы (`spectralCentroid kick kick1 rythm snare`, чётные индексы).
- Параметры: `Gain`, `Gain2`, `Triggerlag`, `Zigzaggerrangex`, `Zigzaggerrangey`.
- `lag1 lag2 = Triggerlag`; `math1 gain=Gain`, `math3 gain=Gain2`, `math4 add`.
- `limit1`: **`type=zigzag`**, min=`Zigzaggerrangex`, max=`Zigzaggerrangey` — пилообразно-отражённое значение в диапазоне.
- `speed1` интегрирует. Выход `rename1 → Zig_Zagger`.
- Семантика: непрерывный зигзаг между x/y, продвигаемый по долям. Для порта — опциональный генератор.

## Резюме
| Компонент | Вход | Управляется | Выход-канал OSC | Приоритет порта |
|---|---|---|---|---|
| `Count_Analysis` | kick/snare | — | trigger4k…16s | **высокий** (доли нужны) |
| `Low_Base`  | low, kick   | Amplifylow/Gain/Lag/Offset/Speed | `Low_Base`  | средний |
| `Mid_Base`  | low, mid, kick | Gain/Gain2/Defaultlag/… | `Mid_Base`  | средний |
| `High_Mid`  | mid, high, snare | Midgain/Highgain/Snaregain/… | `High_Mid` | средний |
| `Jumper_trigger` | доля+полосы | Mode/Type/rand/… | `Jumper` | низкий (спецэффект) |
| `Zig_Zagger` | доля+полоса | zigzag range/gain | `Zig_Zagger` | низкий (спецэффект) |

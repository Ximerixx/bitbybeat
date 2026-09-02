# 06 — OSC-выход

## Сборка и отправка
```
OSCOutMerge (merge) ← [ audioAnalysis/out1,
                        Low_Base/out1, Mid_Base/out1, High_Mid/out1,
                        Jumper_trigger/out1, Zig_Zagger/out1,
                        selectTriggers ]
        │  (srselect=first)
   timeslice1 (timeslice=on)
        │
     OSC_out  <CHOP:oscout>
```

## Параметры `OSC_out`
| Параметр | Значение |
|---|---|
| `port` | **7700** |
| `format` | **sample** (по одному значению на канал за кадр) |
| `maxsize` | 34 (макс. каналов) |
| `maxbytes` | 24394 |
| `sendrate` | off (шлёт покадрово на control-rate 60 Гц) |
| `timeslice` | on |
| `exportmethod` | autoname |
| `autoexportroot` | `parent()` |

Адрес OSC-сообщения = имя канала (TD OSC Out в режиме `sample`: `/<channelName>` со значением-float).

## Каналы (адреса) в OSC
### От `audioAnalysis/out1` (через `par2`: ops = low mid high kick snare rythm smsd fmsd spectralCentroid, `Output→*`)
| Канал | Источник | Тип |
|---|---|---|
| `low`  | огибающая low-полосы  | float 0..~100 (после clamp/add/smooth) |
| `mid`  | огибающая mid-полосы  | float |
| `high` | огибающая high-полосы | float |
| `kick` | импульс kick (retrig 0.08) | 0/1 |
| `snare`| гейт snare | 0/1 |
| `rythm`| гейт rythm | 0/1 |
| `smsd` | медленная спектр. мера | float 0..1 |
| `fmsd` | быстрая спектр. энергия | float 0..1 |
| `spectralCentroid` | центроид | float 0..1 |

### От пост-процессоров
| Канал | Источник |
|---|---|
| `Low_Base`  | генератор low-раскачки |
| `Mid_Base`  | генератор mid |
| `High_Mid`  | генератор high |
| `Jumper`    | прыжковый генератор |
| `Zig_Zagger`| зигзаг-генератор |

### От `selectTriggers` (из `Count_Analysis/out1`)
`channames = trigger4k trigger4s trigger8k trigger8s trigger16k trigger16s`
| Канал | Смысл |
|---|---|
| `trigger4k` / `trigger4s` | доля 1/4 по kick / snare |
| `trigger8k` / `trigger8s` | доля 1/8 |
| `trigger16k` / `trigger16s` | доля 1/16 |

Итого ~**20 каналов** (9 + 5 + 6), с запасом `maxsize=34`.

> Примечание: `select4` (в `/project1`, `in: audioAnalysis/out1`) — отдельный отбор, но в `OSCOutMerge` не входит; используется для GUI/иных целей.

## Модель для порта
- UDP OSC на `127.0.0.1:7700` (адрес сделать конфигурируемым).
- Отправка на control-rate (60 Гц): каждый кадр — по сообщению на канал `/<name> <float>` (или бандл всех каналов одним пакетом — эквивалентно, но эффективнее; см. `09_optimizations.md`).
- Крейт: `rosc`.
- Набор каналов сделать выбираемым (полосы/детекторы — must, генераторы/доли — опционально).

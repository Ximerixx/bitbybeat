# 03 — Спектр и спектральные признаки

Ноды внутри `/project1/audioAnalysis`. Источник спектра — `audiospecpt3` (TD `audiospect`) от `switch_neutone`.

## Получение спектра
```
switch_neutone ─► audiospecpt3 (audiospect, highfreqboost=1)   # магнитудный спектр
        │                    │
        │                    ├─► shuffle3 (splitn, nval=440) ─► shuffle4 (swap, nval=5, firstsample=on)
        │                    └─► analyze7 (sum)   # полная энергия спектра
```
- `audiospecpt3`: FFT-магнитуды, `highfreqboost=1` (поднятие ВЧ).
- `shuffle3 splitn 440`: разбить спектр на **440** бинов (переформатирование каналов→сэмплы).
- `shuffle4 swap 5 firstsample=on`: перестановка групп по 5 (переупорядочивание бинов).

## Спектральный центроид (`spectralCentroid` → `centroid`)
```
shuffle4 ─► pattern1 (ramp, amp=50, append) ─► math15 (mul) ─► analyze5 (sum)  =  Σ(i·mag)
shuffle4 ─────────────────────────────────────► analyze8 (sum)                 =  Σ(mag)
                                merge4[analyze5, analyze8] ─► math13 (div)       =  Σ(i·mag)/Σ(mag)
        ─► trail7 (wlength=4.31) ─► analyze11 ─► trail8 (wlength=1.99) ─► analyze12
        ─► math14 (fromrange 18..32 → 0..1) ─► switch9 ─► centroid
```
- Классический **спектральный центроид**: взвешенная суммой рампы (индекс×магнитуда) / сумма магнитуд.
- `pattern1` даёт линейный вес индекса (амплитуда рампы 50).
- Двойное сглаживание: `trail 4.31` → `trail 1.99` (окна в секундах на 60 Гц).
- `math14` ремап диапазона **18..32 → 0..1** (калибровочный рабочий диапазон центроида).
- `switch9 index = op('spectralCentroid').par.Active`.

## `fms` (fmsd) — «быстрая» спектральная энергия
```
audiospecpt3 ─► analyze7 (sum) ─► filter8 (width=0.5) ─► trail4 ─► analyze10 ─► math7 (fromrange 0..1000 → 0..1) ─► switch8 ─► fms
```
- Суммарная энергия спектра, сглаженная фильтром 0.5 c, ремап **0..1000 → 0..1**.
- Семантика: быстрый индикатор общей «наполненности» спектра (approx).

## `sms` (smsd) — «медленная» спектральная мера
```
filter8 ─► trail3 (wlength=10) ─► analyze9 ─► select2 (chan1) ─► math6 (fromrange 100..1800 → 0..1) ─► switch7 ─► sms
```
- Та же энергия, но с длинным сглаживанием (окно 10 c), ремап **100..1800 → 0..1**.
- Семантика: медленный «фон»/динамика уровня (approx).

## Питание rythm-детектора спектром
```
shuffle4 ─► analyze4 (nopeakvalue=1, allowstart/end=off) ─► trail5 (wlength=1) ─► math10 (gain=1000, preoff=-Rythmthresh) ─► limit6 ─► math11 ─► logic3 ─► Rythm_
```
- `analyze4` с `nopeakvalue=1` — измерение пиков/структуры спектра; далее в детектор rythm (см. `02_beat_detection.md`).

## ⚠️ Замечания по точности
- `fms`/`sms` — эвристические меры «магнитуды/движения спектра»; точное имя/семантика в TD не документированы в дампе. Диапазоны ремапа (`0..1000`, `100..1800`, центроид `18..32`) — **калибровочные под конкретный трек/вход**, в порте вынести в конфиг.
- `shuffle 440` + `swap 5` в TD переформатирует спектр; численное совпадение с FFT-реализацией на Rust (`realfft`) 1:1 не гарантируется, но поведенчески воспроизводимо: центроид считать напрямую по бинам `Σ(f_i·|X_i|)/Σ|X_i|`, диапазоны калибровать.

## Модель для порта
```
mag[]  = |FFT(frame)|            # + опц. highfreqboost
E      = Σ mag                   # полная энергия
cen    = Σ(i·mag)/Σ mag          # центроид (в единицах бинов)
centroid = remap(smooth2(cen), 18..32 → 0..1)
fms      = remap(smooth(E, 0.5s), 0..1000 → 0..1)
sms      = remap(smooth(E, 10s),  100..1800 → 0..1)
rythm_in = 1000 · peakMeasure(mag)   # → детектор rythm
```

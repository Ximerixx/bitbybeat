# 07 — Дефолты параметров (из дампа)

> Значения — кэш `eval()` из `*.parm` эталона. Выражения в обратных кавычках.

## AudioAnalysis (компонент анализа) — `/project1/audioAnalysis`

| Параметр | Значение | Выражение |
|---|---|---|
| `Lowthresh` | 0.116 |  |
| `Lowgain` | 1.84 | `op('lowControlGain')[0]` |
| `Highthresh` | 0.116 |  |
| `Highgain` | 1.45 | `op('highControlGain1')[0]` |
| `Lowsmooth` | 0.276 |  |
| `Highsmooth` | 0.093 |  |
| `Low` | 0 | `op('./low_')[0]` |
| `Mid` | 0 | `op('./mid_')[0]` |
| `High` | 0 | `op('./high_')[0]` |
| `Kickthresh` | 0.328 | `op('kickControlThreshold')[0]` |
| `Kickdetection` | 0 | `op('./Kickdrum')[0]` |
| `Snarethresh` | 0.338 | `op('snareControlThreshold')[0]` |
| `Snaredetection` | 0 | `op('./Snare')[0]` |
| `Rythmthresh` | 69.9 | `op('rythmControlThreshold')[0] op('rythmControlGain')[0]` |
| `Rythm` | 0 | `op('./Rythm_')[0]` |
| `Spectralcentroid` | 0 | `op('./centroid')[0]` |
| `Fmp` | 0.385 | `op('./fms')[0]` |
| `Smp` | 0 | `op('./sms')[0]` |
| `Midthresh` | 0.052 |  |
| `Midgain` | 1.98 | `op('midControlGain')[0]` |
| `Midsmooth` | 0.224 |  |
| `Lowadd` | -0.492 |  |
| `Midadd` | -0.363 |  |
| `Highadd` | -0.32 |  |
| `Version` | 6.1.1 |  |
| `Galmap` | off |  |
| `Lowactive` | on |  |
| `Midactive` | on |  |
| `Highactive` | on |  |
| `Kickactive` | on |  |
| `Snareactive` | on |  |
| `Rythmactive` | on |  |
| `Toxsavebuild` | 2023.11310 |  |
| `Neutonenotice` | See inside to install Nutone VST first |  |
| `Neutonebass` | 0 |  |
| `Neutonedrums` | 0 |  |
| `Neutonevocals` | 0 |  |
| `Neutoneother` | 0 |  |
| `Listenvolume` | 0.612 |  |
| `Ssdactive` | on |  |
| `Fsdactive` | on |  |
| `Scactive` | on |  |

## Low_Base — `/project1/Low_Base`

| Параметр | Значение | Выражение |
|---|---|---|
| `Amplifylow` | 2.4 |  |
| `Gain` | 3 |  |
| `Wlength` | 6 |  |

## Mid_Base — `/project1/Mid_Base`

| Параметр | Значение | Выражение |
|---|---|---|
| `Gain` | 0.888 |  |
| `Defaultlag` | 0.31 |  |
| `Gain2` | 1.3 |  |
| `Wlength` | 6 |  |

## High_Mid — `/project1/High_Mid`

| Параметр | Значение | Выражение |
|---|---|---|
| `Midgain` | 0.699 |  |
| `Defaultlag` | 0.201 |  |
| `Highgain` | 0.301 |  |
| `Snaregain` | 1.22 |  |
| `Wlength` | 3 |  |

## Jumper_trigger — `/project1/Jumper_trigger`

| Параметр | Значение | Выражение |
|---|---|---|
| `Torange31` | 0 |  |
| `Torange32` | 22 |  |
| `Audiofloat` | 0.43 |  |
| `Lagfloat2` | 0.4 |  |
| `Increments` | 22 |  |
| `Channames` | kick |  |
| `Mode` | 2 |  |

## Zig_Zagger — `/project1/Zig_Zagger`

| Параметр | Значение | Выражение |
|---|---|---|
| `Zigzaggerrangex` | 0 |  |
| `Zigzaggerrangey` | 3 |  |
| `Channames` | kick |  |
| `Channames2` | low |  |

## DSP / control ноды (числовые параметры)

| Нода | Тип | Параметры |
|---|---|---|
| `math1` | `CHOP:math` | preoff=0.1; gain=2.28 |
| `math2` | `CHOP:math` | preoff=0.1; gain=0.64 |
| `lagClearRythmIn_` | `CHOP:lag` | lag1=2; lag2=4; accel1=1; accel2=3; timeslice=on |
| `lowControlGain` | `CHOP:math` | chanop=add; preoff=0.2; gain=4.57; postoff=-0.2 |
| `midControlGain` | `CHOP:math` | chanop=add; preoff=0.5; gain=4.05; postoff=-0.6 |
| `highControlGain` | `CHOP:math` | chanop=add; gain=0.77; postoff=0.3 |
| `highControlGain1` | `CHOP:math` | chanop=add; gain=1.93 |
| `kickControlThreshold_math` | `CHOP:math` | chanop=add; gain=3.92; postoff=-0.3; torange1=0; torange2=0.5 |
| `kickControlThreshold` | `CHOP:express` | expr0expr=`0.7/(1+math.exp(-5.4*(me.inputVal-0.3)))` |
| `snareControlThreshold_math` | `CHOP:math` | chanop=add; preoff=0.5; gain=6.78; postoff=-0.2; torange1=0; torange2=0.09 |
| `snareControlThreshold` | `CHOP:express` | expr0expr=`0.9/(1+math.exp(-2.1*(me.inputVal-0.5)))` |
| `rythmControlThreshold` | `CHOP:math` | chanop=add; preoff=1.8; gain=4.76; postoff=-0.8; torange1=0; torange2=6 |
| `audiodyna1` | `CHOP:audiodyna` | enablecompressor=on; thresholdcompressor=-20.6; ratiocompressor=0.638; gaincompressor=6.9; timeslice=on |
| `audiofilter1` | `CHOP:audiofilter` | units=frequency; cutofflog=2.17609; cutofffrequency=150; rolloff=20; timeslice=on |
| `audiofilter2` | `CHOP:audiofilter` | filter=bandpass; units=frequency; cutofflog=2.90309; cutofffrequency=800; rolloff=20; timeslice=on |
| `audiofilter3` | `CHOP:audiofilter` | filter=highpass; units=frequency; cutofflog=3.54407; cutofffrequency=3500; resonance=0.8; rolloff=15; timeslice=on |
| `trigger1` | `CHOP:trigger` | retrigger=0.08; attack=0; decay=0; sustain=0; release=0; timeslice=on |
| `OSC_out` | `CHOP:oscout` | port=7700; maxsize=34; format=sample; sendrate=off; maxbytes=24394; timeslice=on; exportmethod=autoname |
| `audiodevin1` | `CHOP:audiodevin` | driver=asio; device=X-AIR_ASIO_Driver||6; inputnames=1:In_1 2:In_2; format=stereo; ratemode=resample; timeslice=on |

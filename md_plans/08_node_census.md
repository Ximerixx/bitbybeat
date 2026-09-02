# 08 — Полная перепись нод (авто-census)

> Источник истины: `Analysis 2.2_v6_calibrated.9.toe.dir` (toeexpand).
> Сгенерировано `gen_census.py` из `toe_model.json`. Всего нод: **757**.

## Сводка по типам (family:type)

| Кол-во | Тип |
|---:|---|
| 76 | `CHOP:math` |
| 38 | `COMP:base` |
| 38 | `CHOP:par` |
| 37 | `CHOP:rename` |
| 31 | `COMP:text` |
| 29 | `CHOP:out` |
| 27 | `COMP:container` |
| 26 | `DAT:table` |
| 24 | `DAT:text` |
| 24 | `CHOP:select` |
| 21 | `DAT:eval` |
| 19 | `CHOP:count` |
| 19 | `CHOP:trigger` |
| 18 | `CHOP:lag` |
| 18 | `DAT:script` |
| 17 | `CHOP:merge` |
| 15 | `DAT:panelexec` |
| 14 | `CHOP:express` |
| 14 | `CHOP:switch` |
| 14 | `COMP:button` |
| 14 | `CHOP:logic` |
| 13 | `TOP:rectangle` |
| 13 | `TOP:circle` |
| 12 | `CHOP:analyze` |
| 12 | `DAT:parexec` |
| 11 | `COMP:annotate` |
| 11 | `CHOP:null` |
| 11 | `CHOP:pattern` |
| 10 | `CHOP:trail` |
| 9 | `CHOP:in` |
| 9 | `CHOP:limit` |
| 9 | `TOP:constant` |
| 8 | `COMP:slider` |
| 7 | `TOP:ramp` |
| 7 | `TOP:comp` |
| 7 | `TOP:null` |
| 5 | `CHOP:constant` |
| 5 | `TOP:over` |
| 5 | `TOP:multiply` |
| 5 | `CHOP:lookup` |
| 5 | `TOP:lookup` |
| 5 | `TOP:transform` |
| 4 | `CHOP:speed` |
| 4 | `CHOP:filter` |
| 3 | `DAT:chopexec` |
| 3 | `CHOP:audiofilter` |
| 3 | `CHOP:delete` |
| 2 | `DAT:select` |
| 2 | `CHOP:audiofilein` |
| 2 | `CHOP:audiodyna` |
| 2 | `CHOP:shuffle` |
| 2 | `TOP:flip` |
| 1 | `COMP:time` |
| 1 | `DAT:filein` |
| 1 | `CHOP:beat` |
| 1 | `DAT:null` |
| 1 | `COMP:replicator` |
| 1 | `CHOP:timeslice` |
| 1 | `CHOP:oscout` |
| 1 | `CHOP:audiodevin` |
| 1 | `CHOP:datto` |
| 1 | `DAT:parameter` |
| 1 | `CHOP:script` |
| 1 | `CHOP:audiodevout` |
| 1 | `CHOP:audiospect` |
| 1 | `CHOP:audiovst` |
| 1 | `DAT:reorder` |
| 1 | `CHOP:info` |
| 1 | `DAT:info` |
| 1 | `DAT:substitute` |

## Перепись по контейнерам


### `/` — 3 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `local` | `COMP:base` | — | opviewer=./variables |
| `perform` | `COMP:container` | — | resizecomp=`/project1/audioAnalysis/low` |
| `project1` | `COMP:container` | — | w=600; h=1052; bottomanchor=0.223; vfillweight=0; sizefromwindow=on; cursor=pointer; resizecomp=`me`; repocomp=`.`; …(+7) |

### `/local` — 7 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `maps` | `COMP:base` | — | opviewer=./id1 |
| `master_beat` | `CHOP:beat` | — | updateglobal=on; timeslice=on |
| `midi` | `COMP:base` | — | opviewer=./device; loadondemand=on |
| `set_variables` | `DAT:table` | — |  |
| `shortcuts` | `DAT:filein` | — | file=`app.configFolder + '/PanelShortcuts.txt'`; converttable=on |
| `time` | `COMP:time` | — | play=0; rate=`cookRate()`; clone=/sys/local/time |
| `variables` | `DAT:null` | set_variables |  |

### `/local/maps` — 5 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `master` | `COMP:base` | — |  |
| `replicator1` | `COMP:replicator` | — | repsuffixstart=0; template=select2; namefromtable=colbyname; colname=id; opprefix=id; master=master; destination=`me.parent()`; callbacks=replicator1_callbacks |
| `replicator1_callbacks` | `DAT:text` | — |  |
| `select1` | `DAT:select` | — | dat=/local/midi/device; rowindexend=`me.inputs[0].numRows - 1`; extractcols=bynames; colindexend=`me.inputs[0].numCols - 1`; colnames=id definition |
| `select2` | `DAT:select` | select1 | extractrows=bynames; rowindexend=`me.inputs[0].numRows - 1`; rownames=?*; fromcol=1; colindexend=`me.inputs[0].numCols - 1` |

### `/project1` — 34 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `Count_Analysis` | `COMP:base` | — | opviewer=./mergeall |
| `High_Mid` | `COMP:base` | — | opviewer=./trail1; Midgain=0.699; Defaultlag=0.201; Highgain=0.301; Snaregain=1.22; Wlength=3; iop1shortcut=; iop1op= |
| `InMerge` | `CHOP:merge` | audiodevin1 |  |
| `Jumper_trigger` | `COMP:base` | — | opviewer=./trail1; enableexternaltox=off; Torange31=0; Torange32=22; Audiofloat=0.43; Lagfloat2=0.4; Increments=22; Channames=kick; …(+1) |
| `Low_Base` | `COMP:base` | — | opviewer=./trail1; Amplifylow=2.4; Gain=3; Wlength=6; iop1shortcut=; iop1op= |
| `Mid_Base` | `COMP:base` | — | opviewer=./trail1; Gain=0.888; Defaultlag=0.31; Gain2=1.3; Wlength=6; iop1shortcut=; iop1op= |
| `OSCOutMerge` | `CHOP:merge` | audioAnalysis/out1, Low_Base/out1, Mid_Base/out1, High_Mid/out1, Jumper_trigger/out1, Zig_Zagger/out1, selectTriggers | srselect=first |
| `OSC_out` | `CHOP:oscout` | timeslice1 | port=7700; maxsize=34; format=sample; sendrate=off; maxbytes=24394; timeslice=on; exportmethod=autoname |
| `Zig_Zagger` | `COMP:base` | — | opviewer=./trail1; Zigzaggerrangex=0; Zigzaggerrangey=3; Channames=kick; Channames2=low |
| `annotate3` | `COMP:annotate` | — | opviewer=./annotation; encloseops=`me.EncloseOPs if hasattr(me, 'EncloseOPs') else True`; includeinorder=on; layerzone=`0 if hasattr(me, 'EncloseOPs') and me.EncloseOPs else 1`; ext0object=op.TDAnnotate.mod.AnnotateExt.AnnotateExt(me); ext0name=; ext0promote=on; parentshortcut=Annotate; …(+22) |
| `annotate5` | `COMP:annotate` | — | opviewer=./annotation; encloseops=`me.EncloseOPs if hasattr(me, 'EncloseOPs') else True`; includeinorder=on; layerzone=`0 if hasattr(me, 'EncloseOPs') and me.EncloseOPs else 1`; ext0object=op.TDAnnotate.mod.AnnotateExt.AnnotateExt(me); ext0name=; ext0promote=on; parentshortcut=Annotate; …(+22) |
| `annotate6` | `COMP:annotate` | — | opviewer=./annotation; encloseops=`me.EncloseOPs if hasattr(me, 'EncloseOPs') else True`; includeinorder=on; layerzone=`0 if hasattr(me, 'EncloseOPs') and me.EncloseOPs else 1`; ext0object=op.TDAnnotate.mod.AnnotateExt.AnnotateExt(me); ext0name=; ext0promote=on; parentshortcut=Annotate; …(+22) |
| `annotate7` | `COMP:annotate` | — | opviewer=./annotation; encloseops=`me.EncloseOPs if hasattr(me, 'EncloseOPs') else True`; includeinorder=on; layerzone=`0 if hasattr(me, 'EncloseOPs') and me.EncloseOPs else 1`; ext0object=op.TDAnnotate.mod.AnnotateExt.AnnotateExt(me); ext0name=; ext0promote=on; parentshortcut=Annotate; …(+22) |
| `annotate8` | `COMP:annotate` | — | opviewer=./annotation; encloseops=`me.EncloseOPs if hasattr(me, 'EncloseOPs') else True`; includeinorder=on; layerzone=`0 if hasattr(me, 'EncloseOPs') and me.EncloseOPs else 1`; ext0object=op.TDAnnotate.mod.AnnotateExt.AnnotateExt(me); ext0name=; ext0promote=on; parentshortcut=Annotate; …(+22) |
| `audioAnalysis` | `COMP:container` | — | x=5; y=92; w=600; h=190; display=off; bgcolorr=0.1; bgcolorg=0.1; bgcolorb=0.1; …(+53) |
| `audiodevin1` | `CHOP:audiodevin` | — | driver=asio; device=X-AIR_ASIO_Driver\|\|6; inputnames=1:In_1 2:In_2; format=stereo; ratemode=resample; timeslice=on |
| `audiodyna1` | `CHOP:audiodyna` | InMerge | enablecompressor=on; thresholdcompressor=-20.6; ratiocompressor=0.638; gaincompressor=6.9; timeslice=on |
| `audiofilein1` | `CHOP:audiofilein` | — | file=`app.samplesFolder+'/Audio/JeremyCaulfield_www.dumb-unit.com.mp3'`; timeslice=on |
| `highControlGain` | `CHOP:math` | lagClearRythmIn_ | chanop=add; gain=0.77; postoff=0.3 |
| `highControlGain1` | `CHOP:math` | lagClearRythmIn_ | chanop=add; gain=1.93 |
| `kickControlThreshold` | `CHOP:express` | kickControlThreshold_math | expr0expr=`0.7/(1+math.exp(-5.4*(me.inputVal-0.3)))` |
| `kickControlThreshold_math` | `CHOP:math` | lagClearRythmIn_ | chanop=add; gain=3.92; postoff=-0.3; torange1=0; torange2=0.5 |
| `lagClearRythmIn_` | `CHOP:lag` | math2 | lag1=2; lag2=4; accel1=1; accel2=3; timeslice=on |
| `lowControlGain` | `CHOP:math` | lagClearRythmIn_ | chanop=add; preoff=0.2; gain=4.57; postoff=-0.2 |
| `math1` | `CHOP:math` | InMerge | preoff=0.1; gain=2.28 |
| `math2` | `CHOP:math` | rms | preoff=0.1; gain=0.64 |
| `midControlGain` | `CHOP:math` | lagClearRythmIn_ | chanop=add; preoff=0.5; gain=4.05; postoff=-0.6 |
| `rms` | `CHOP:analyze` | InMerge | function=rmspower |
| `rythmControlThreshold` | `CHOP:math` | lagClearRythmIn_ | chanop=add; preoff=1.8; gain=4.76; postoff=-0.8; torange1=0; torange2=6 |
| `select4` | `CHOP:select` | audioAnalysis/out1 |  |
| `selectTriggers` | `CHOP:select` | Count_Analysis/out1 | channames=trigger4k trigger4s trigger8k trigger8s trigger16k trigger16s |
| `snareControlThreshold` | `CHOP:express` | snareControlThreshold_math | expr0expr=`0.9/(1+math.exp(-2.1*(me.inputVal-0.5)))` |
| `snareControlThreshold_math` | `CHOP:math` | lagClearRythmIn_ | chanop=add; preoff=0.5; gain=6.78; postoff=-0.2; torange1=0; torange2=0.09 |
| `timeslice1` | `CHOP:timeslice` | OSCOutMerge | timeslice=on |

### `/project1/Count_Analysis` — 48 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `count16k` | `CHOP:count` | select2 | output=cl; limitmax=15; resetvalue=1; timeslice=on |
| `count16s` | `CHOP:count` | select3 | output=cl; limitmax=15; resetvalue=1; timeslice=on |
| `count4k` | `CHOP:count` | select2 | output=cl; limitmax=3; resetvalue=1; timeslice=on |
| `count4s` | `CHOP:count` | select3 | output=cl; limitmax=3; resetvalue=1; timeslice=on |
| `count8k` | `CHOP:count` | select2 | output=cl; limitmax=7; timeslice=on |
| `count8s` | `CHOP:count` | select3 | output=cl; limitmax=7; resetvalue=1; timeslice=on |
| `express4k` | `CHOP:express` | count4k | expr0expr=`if ($V == 1, 1, 0)` |
| `express4s` | `CHOP:express` | count4s | expr0expr=`if ($V == 1, 1, 0)` |
| `express8k` | `CHOP:express` | count8k | expr0expr=`if ($V == 1, 1, 0)` |
| `express8s` | `CHOP:express` | count8s | expr0expr=`if ($V == 1, 1, 0)` |
| `in1` | `CHOP:in` | — | timeslice=on |
| `logic16k` | `CHOP:logic` | trigger16k | preop=toggle; timeslice=on |
| `logic16s` | `CHOP:logic` | trigger16s | preop=toggle; timeslice=on |
| `logic4k` | `CHOP:logic` | trigger4k | preop=toggle; timeslice=on |
| `logic4s` | `CHOP:logic` | trigger4s | preop=toggle; timeslice=on |
| `logic8k` | `CHOP:logic` | trigger8k | preop=toggle; timeslice=on |
| `logic8s` | `CHOP:logic` | trigger8s | preop=toggle; timeslice=on |
| `merge1` | `CHOP:merge` | rename1, rename2, rename3, rename6, rename5, rename4 |  |
| `merge2` | `CHOP:merge` | rename7, rename8, rename9, rename12, rename11, rename10 |  |
| `merge3` | `CHOP:merge` | rename15, rename13, rename16, rename14 |  |
| `merge4` | `CHOP:merge` | select2, select3, merge1 |  |
| `mergeall` | `CHOP:merge` | merge1, merge2, merge3 |  |
| `out1` | `CHOP:out` | mergeall |  |
| `out2` | `CHOP:out` | merge4 |  |
| `rename1` | `CHOP:rename` | logic4k | renameto=logic4k |
| `rename10` | `CHOP:rename` | express8s | renameto=express8s |
| `rename11` | `CHOP:rename` | trigger8s | renameto=trigger8s |
| `rename12` | `CHOP:rename` | logic8s | renameto=logic8s |
| `rename13` | `CHOP:rename` | trigger16k | renameto=trigger16k |
| `rename14` | `CHOP:rename` | trigger16s | renameto=trigger16s |
| `rename15` | `CHOP:rename` | logic16k | renameto=logic16k |
| `rename16` | `CHOP:rename` | logic16s | renameto=logic16s |
| `rename2` | `CHOP:rename` | trigger4k | renameto=trigger4k |
| `rename3` | `CHOP:rename` | express4k | renameto=express4k |
| `rename4` | `CHOP:rename` | express4s | renameto=express4s |
| `rename5` | `CHOP:rename` | trigger4s | renameto=trigger4s |
| `rename6` | `CHOP:rename` | logic4s | renameto=logic4s |
| `rename7` | `CHOP:rename` | logic8k | renameto=logic8k |
| `rename8` | `CHOP:rename` | trigger8k | renameto=trigger8k |
| `rename9` | `CHOP:rename` | express8k | renameto=express8k |
| `select2` | `CHOP:select` | in1 | channames=kick |
| `select3` | `CHOP:select` | in1 | channames=snare |
| `trigger16k` | `CHOP:trigger` | count16k | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger16s` | `CHOP:trigger` | count16s | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger4k` | `CHOP:trigger` | count4k | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger4s` | `CHOP:trigger` | count4s | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger8k` | `CHOP:trigger` | count8k | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger8s` | `CHOP:trigger` | count8s | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |

### `/project1/High_Mid` — 21 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `Speed_Toggle` | `CHOP:par` | — | ops=`parent()`; parameters=Speed |
| `in1` | `CHOP:in` | — | timeslice=on |
| `lag1` | `CHOP:lag` | select1 | lag1=0.02; lag2=`op('lag5')['snare']`; snap=off; timeslice=on |
| `lag2` | `CHOP:lag` | select2 | lag1=0.02; lag2=`op('par1')['Defaultlag']`; snap=off; timeslice=on |
| `lag5` | `CHOP:lag` | math5 | lag1=0.02; lag2=`op('par1')['Defaultlag']`; snap=off; timeslice=on |
| `lag6` | `CHOP:lag` | math6 | lag1=0.02; lag2=`op('par1')['Defaultlag']`; snap=off; timeslice=on |
| `math1` | `CHOP:math` | lag1 | chopop=mul; gain=`parent().par.Midgain` |
| `math2` | `CHOP:math` | lag2 | preoff=`op('math1')['mid']`; gain=`parent().par.Highgain`; postoff=`op('lag6')['snare']`; torange1=0; torange2=0.5 |
| `math3` | `CHOP:math` | math2 | postoff=`op('par2')['Offset']` |
| `math5` | `CHOP:math` | select3 | torange1=`op('par1')['Defaultlag']`; torange2=1 |
| `math6` | `CHOP:math` | select3 | gain=`parent().par.Snaregain`; torange1=`op('par1')['Defaultlag']`; torange2=1 |
| `out1` | `CHOP:out` | rename1 |  |
| `par1` | `CHOP:par` | — | ops=`parent()`; parameters=Defaultlag |
| `par2` | `CHOP:par` | — | ops=`parent()`; parameters=Offset |
| `rename1` | `CHOP:rename` | switch1 | renameto=High_Mid |
| `select1` | `CHOP:select` | in1 | channames=mid |
| `select2` | `CHOP:select` | in1 | channames=high |
| `select3` | `CHOP:select` | in1 | channames=snare |
| `speed1` | `CHOP:speed` | math3 | timeslice=on |
| `switch1` | `CHOP:switch` | math3, speed1 | index=`op('Speed_Toggle')['Speed']` |
| `trail1` | `CHOP:trail` | out1 | wlength=`parent().par.Wlength`; timeslice=on |

### `/project1/Jumper_trigger` — 42 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `Count_Analysis` | `COMP:base` | — | opviewer=./mergeall |
| `LAG` | `CHOP:par` | — | ops=`parent()`; parameters=Lagfloat2 |
| `chopexec1` | `DAT:chopexec` | — | chop=datto1; offtoon=on; ontooff=on |
| `count1` | `CHOP:count` | rename2 | timeslice=on |
| `datto1` | `CHOP:datto` | — | dat=parameter1; extractrows=byindex; rowindexstart=1 |
| `in1` | `CHOP:in` | — | timeslice=on |
| `lag1` | `CHOP:lag` | select2 | lag1=0.02; lag2=0.2; snap=off; timeslice=on |
| `lag2` | `CHOP:lag` | randomCHOP1/out1 | lag1=`op('LAG')['Lagfloat2']`; lag2=`op('LAG')['Lagfloat2']`; overshoot1=`op('lag_Overshoot')['onTrigger']`; overshoot2=`op('lag_Overshoot')['onTrigger']`; snap=off; timeslice=on |
| `lag3` | `CHOP:lag` | math6 | lag1=`op('LAG')['Lagfloat2']`; lag2=`op('LAG')['Lagfloat2']`; overshoot1=`op('lag_Overshoot')['onTrigger']`; overshoot2=`op('lag_Overshoot')['onTrigger']`; snap=off; timeslice=on |
| `lag5` | `CHOP:lag` | math15 | lag1=0.02; lag2=0.1; snap=off; timeslice=on |
| `lag_Overshoot` | `CHOP:lag` | rename2 | lag1=0.04; lag2=0.1; snap=off; timeslice=on |
| `limit1` | `CHOP:limit` | math5 | type=clamp; min=0.8; max=1.1 |
| `limit2` | `CHOP:limit` | lag3 | type=`op('datto1')['Mode'] parent().par.Type`; min=`op('math2').par.torange1`; max=`op('math2').par.torange2` |
| `math1` | `CHOP:math` | math9, math8 | chopop=add; preoff=`op('math4')['low']`; gain=`op('math2').par.torange2 / op('math3')['Audiofloat'] if parent().par.Audiofloat > 0 else 0`; postoff=`op('math2')['chan1']` |
| `math10` | `CHOP:math` | par5 | chanop=sub; postop=square |
| `math11` | `CHOP:math` | math10 | preop=root |
| `math12` | `CHOP:math` | math11, math14 | chopop=mul |
| `math13` | `CHOP:math` | math8, math9 | chopop=add; gain=`op('par1')['Audiofloat']`; postoff=`op('limit2')['onTrigger']` |
| `math14` | `CHOP:math` | par4 | gain=0.01 |
| `math15` | `CHOP:math` | select3 | chopop=add; gain=`op('math7')['Torange32']/2` |
| `math2` | `CHOP:math` | lag2 | fromrange1=0; fromrange2=10; torange1=`parent().par.Torange31`; torange2=`parent().par.Torange32` |
| `math3` | `CHOP:math` | par1 | torange1=`parent().par.Torange32`; torange2=1 |
| `math5` | `CHOP:math` | par2 | fromrange1=0; fromrange2=`parent().par.Torange32`; torange1=0.9; torange2=1.1 |
| `math6` | `CHOP:math` | count1 | gain=`op('math12')['Torange31']` |
| `math7` | `CHOP:math` | limit1 | fromrange1=0.9; fromrange2=`parent().par.Torange32`; torange1=0.1; torange2=1 |
| `math8` | `CHOP:math` | select1 | gain=`op('math7')['Torange32']` |
| `math9` | `CHOP:math` | lag1, lag5 | chopop=add; gain=`op('math7')['Torange32']` |
| `out1` | `CHOP:out` | rename1 |  |
| `par1` | `CHOP:par` | — | ops=`parent()`; parameters=Audiofloat |
| `par2` | `CHOP:par` | — | ops=`parent()`; parameters=Torange32; renameto=`op('trail1')` |
| `par4` | `CHOP:par` | — | ops=`parent()`; parameters=Increments |
| `par5` | `CHOP:par` | — | ops=`parent()`; parameters=Torange31 Torange32 |
| `parameter1` | `DAT:parameter` | — | ops=`parent()`; parameters=Mode |
| `randomCHOP1` | `COMP:base` | — | ext0object=op('./RandExt').module.RandExt(me); ext0promote=on; Generate=`op('rename2')['onTrigger']`; Unique=on; Seed=666; Author=Function Store @function.str; Samples=1; Channelname=chan1 |
| `rename1` | `CHOP:rename` | switch1 | renameto=Jumper |
| `rename2` | `CHOP:rename` | select4 | renameto=onTrigger |
| `select1` | `CHOP:select` | in1 | channames=mid |
| `select2` | `CHOP:select` | in1 | channames=low |
| `select3` | `CHOP:select` | in1 | channames=high |
| `select4` | `CHOP:select` | Count_Analysis/out1 | channames=`parent().par.Channames` |
| `switch1` | `CHOP:switch` | math1, math13 | index=`op('datto1')['Mode']` |
| `trail1` | `CHOP:trail` | rename1 | timeslice=on |

### `/project1/Jumper_trigger/Count_Analysis` — 27 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `count16k` | `CHOP:count` | select2 | output=cl; limitmax=15; resetvalue=1; timeslice=on |
| `count16s` | `CHOP:count` | select3 | output=cl; limitmax=15; resetvalue=1; timeslice=on |
| `count4k` | `CHOP:count` | select2 | output=cl; limitmax=3; resetvalue=1; timeslice=on |
| `count4s` | `CHOP:count` | select3 | output=cl; limitmax=3; resetvalue=1; timeslice=on |
| `count8k` | `CHOP:count` | select2 | output=cl; limitmax=7; timeslice=on |
| `count8s` | `CHOP:count` | select3 | output=cl; limitmax=7; resetvalue=1; timeslice=on |
| `in1` | `CHOP:in` | — | timeslice=on |
| `merge1` | `CHOP:merge` | rename2, rename5 |  |
| `merge2` | `CHOP:merge` | rename8, rename11 |  |
| `merge3` | `CHOP:merge` | rename13, rename14, select2, select3 |  |
| `mergeall` | `CHOP:merge` | merge1, merge2, merge3 |  |
| `out1` | `CHOP:out` | mergeall |  |
| `out2` | `CHOP:out` | merge1 |  |
| `rename11` | `CHOP:rename` | trigger8s | renameto=trigger8s |
| `rename13` | `CHOP:rename` | trigger16k | renameto=trigger16k |
| `rename14` | `CHOP:rename` | trigger16s | renameto=trigger16s |
| `rename2` | `CHOP:rename` | trigger4k | renameto=trigger4k |
| `rename5` | `CHOP:rename` | trigger4s | renameto=trigger4s |
| `rename8` | `CHOP:rename` | trigger8k | renameto=trigger8k |
| `select2` | `CHOP:select` | in1 | channames=kick |
| `select3` | `CHOP:select` | in1 | channames=snare |
| `trigger16k` | `CHOP:trigger` | count16k | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger16s` | `CHOP:trigger` | count16s | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger4k` | `CHOP:trigger` | count4k | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger4s` | `CHOP:trigger` | count4s | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger8k` | `CHOP:trigger` | count8k | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger8s` | `CHOP:trigger` | count8s | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |

### `/project1/Jumper_trigger/randomCHOP1` — 6 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `RandExt` | `DAT:text` | — | language=python |
| `out1` | `CHOP:out` | script1 |  |
| `parexec1` | `DAT:parexec` | — | op=..; pars=Seed; onpulse=off |
| `parexec2` | `DAT:parexec` | — | op=..; pars=Samples Range* Intfloat Unique; onpulse=off |
| `script1` | `CHOP:script` | — | callbacks=script4_callbacks; exportmethod=datname; Randomize=`parent().par.Generate` |
| `script4_callbacks` | `DAT:text` | — | language=python |

### `/project1/Low_Base` — 22 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `in1` | `CHOP:in` | — | timeslice=on |
| `lag1` | `CHOP:lag` | math1 | lag1=`op('lag3')['kick']`; lag2=`op('lag2')['kick']`; snap=off; timeslice=on |
| `lag2` | `CHOP:lag` | math2 | lag1=0.1; lag2=`op('math8')['kick']`; snap=off; timeslice=on |
| `lag3` | `CHOP:lag` | math3 | lag1=0.1; lag2=0.1; snap=off; timeslice=on |
| `math1` | `CHOP:math` | select3 | gain=`op('par1')['Amplifylow']` |
| `math10` | `CHOP:math` | select4 | gain=`parent().par.Gain` |
| `math2` | `CHOP:math` | math10 | torange1=`op('par2')['Lag']`; torange2=1 |
| `math3` | `CHOP:math` | math10 | torange1=`op('par2')['Lag']`; torange2=0 |
| `math4` | `CHOP:math` | lag1 | postoff=`op('par3')['Offset']` |
| `math8` | `CHOP:math` | math10 | torange1=1; torange2=0.2 |
| `out1` | `CHOP:out` | rename1 |  |
| `par1` | `CHOP:par` | — | ops=`parent()`; parameters=Amplifylow |
| `par2` | `CHOP:par` | — | ops=`parent()`; parameters=Lag |
| `par3` | `CHOP:par` | — | ops=`parent()`; parameters=Offset |
| `par4` | `CHOP:par` | — | ops=`parent()`; parameters=Speed |
| `rename1` | `CHOP:rename` | switch1 | renameto=Low_Base |
| `select1` | `CHOP:select` | in1 |  |
| `select3` | `CHOP:select` | select1 | channames=low |
| `select4` | `CHOP:select` | select1 | channames=kick  |
| `speed1` | `CHOP:speed` | math4 | timeslice=on |
| `switch1` | `CHOP:switch` | math4, speed1 | index=`op('par4')['Speed']` |
| `trail1` | `CHOP:trail` | out1 | wlength=`parent().par.Wlength`; timeslice=on |

### `/project1/Mid_Base` — 22 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `Speed_Toggle` | `CHOP:par` | — | ops=`parent()`; parameters=Speed |
| `in1` | `CHOP:in` | — | timeslice=on |
| `lag1` | `CHOP:lag` | select1 | lag1=0.02; lag2=`op('lag5')['kick']`; snap=off; timeslice=on |
| `lag3` | `CHOP:lag` | math2 | lag1=`op('lag3')['kick']`; lag2=`op('lag4')['kick']`; snap=off; timeslice=on |
| `lag4` | `CHOP:lag` | math4 | lag1=0.02; lag2=`op('math8')['kick']`; snap=off; timeslice=on |
| `lag5` | `CHOP:lag` | math5 | lag1=0.02; lag2=`op('par1')['Defaultlag']`; snap=off; timeslice=on |
| `math1` | `CHOP:math` | lag1 | chopop=mul; gain=`parent().par.Gain` |
| `math2` | `CHOP:math` | select2 | preoff=`op('math1')['low']`; gain=`parent().par.Gain2` |
| `math3` | `CHOP:math` | lag3 | postoff=`op('par2')['Offset']` |
| `math4` | `CHOP:math` | select3 | torange1=`op('par1')['Defaultlag']`; torange2=`3*op('par1')['Defaultlag']` |
| `math5` | `CHOP:math` | select3 | torange1=`op('par1')['Defaultlag']`; torange2=1 |
| `math8` | `CHOP:math` | select3 | torange1=`10*op('par1')['Defaultlag']`; torange2=`op('par1')['Defaultlag']` |
| `out1` | `CHOP:out` | rename1 |  |
| `par1` | `CHOP:par` | — | ops=`parent()`; parameters=Defaultlag |
| `par2` | `CHOP:par` | — | ops=`parent()`; parameters=Offset |
| `rename1` | `CHOP:rename` | switch1 | renameto=Mid_Base |
| `select1` | `CHOP:select` | in1 | channames=low |
| `select2` | `CHOP:select` | in1 | channames=mid |
| `select3` | `CHOP:select` | in1 | channames=kick |
| `speed1` | `CHOP:speed` | math3 | timeslice=on |
| `switch1` | `CHOP:switch` | math3, speed1 | index=`op('Speed_Toggle')['Speed']` |
| `trail1` | `CHOP:trail` | out1 | wlength=`parent().par.Wlength`; timeslice=on |

### `/project1/Zig_Zagger` — 17 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `Count_Analysis1` | `COMP:base` | — | opviewer=./mergeall |
| `delete1` | `CHOP:delete` | in1 | delchannels=on; delscope=spectralCentroid kick kick1 rythm snare; selnumbers=0-100:2; legacypattern=on |
| `in1` | `CHOP:in` | — | timeslice=on |
| `lag1` | `CHOP:lag` | select1 | lag1=0; lag2=`op('par1')['Triggerlag']`; snap=off; timeslice=on |
| `limit1` | `CHOP:limit` | speed1 | type=zigzag; min=`op('par2')['Zigzaggerrangex']`; max=`op('par2')['Zigzaggerrangey']` |
| `math1` | `CHOP:math` | lag1 | gain=`parent().par.Gain` |
| `math3` | `CHOP:math` | math_high | gain=`parent().par.Gain2` |
| `math4` | `CHOP:math` | math1, math3 | chanop=add; chopop=add |
| `math_high` | `CHOP:math` | select2 | fromrange1=0; fromrange2=4; scope=high |
| `out1` | `CHOP:out` | rename1 |  |
| `par1` | `CHOP:par` | — | ops=`parent()`; parameters=Triggerlag |
| `par2` | `CHOP:par` | — | ops=`parent()`; parameters=Zigzaggerrangex Zigzaggerrangey |
| `rename1` | `CHOP:rename` | limit1 | renameto=Zig_Zagger |
| `select1` | `CHOP:select` | Count_Analysis1/out1 | channames=`parent().par.Channames` |
| `select2` | `CHOP:select` | delete1 | channames=`parent().par.Channames2` |
| `speed1` | `CHOP:speed` | math4 | timeslice=on |
| `trail1` | `CHOP:trail` | out1 | wlength=6; timeslice=on |

### `/project1/Zig_Zagger/Count_Analysis1` — 27 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `count16k` | `CHOP:count` | select2 | output=cl; limitmax=15; resetvalue=1; timeslice=on |
| `count16s` | `CHOP:count` | select3 | output=cl; limitmax=15; resetvalue=1; timeslice=on |
| `count4k` | `CHOP:count` | select2 | output=cl; limitmax=3; resetvalue=1; timeslice=on |
| `count4s` | `CHOP:count` | select3 | output=cl; limitmax=3; resetvalue=1; timeslice=on |
| `count8k` | `CHOP:count` | select2 | output=cl; limitmax=7; timeslice=on |
| `count8s` | `CHOP:count` | select3 | output=cl; limitmax=7; resetvalue=1; timeslice=on |
| `in1` | `CHOP:in` | — | timeslice=on |
| `merge1` | `CHOP:merge` | rename2, rename5 |  |
| `merge2` | `CHOP:merge` | rename8, rename11 |  |
| `merge3` | `CHOP:merge` | rename13, rename14, select2, select3 |  |
| `mergeall` | `CHOP:merge` | merge1, merge2, merge3 |  |
| `out1` | `CHOP:out` | mergeall |  |
| `out2` | `CHOP:out` | merge1 |  |
| `rename11` | `CHOP:rename` | trigger8s | renameto=trigger8s |
| `rename13` | `CHOP:rename` | trigger16k | renameto=trigger16k |
| `rename14` | `CHOP:rename` | trigger16s | renameto=trigger16s |
| `rename2` | `CHOP:rename` | trigger4k | renameto=trigger4k |
| `rename5` | `CHOP:rename` | trigger4s | renameto=trigger4s |
| `rename8` | `CHOP:rename` | trigger8k | renameto=trigger8k |
| `select2` | `CHOP:select` | in1 | channames=kick |
| `select3` | `CHOP:select` | in1 | channames=snare |
| `trigger16k` | `CHOP:trigger` | count16k | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger16s` | `CHOP:trigger` | count16s | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger4k` | `CHOP:trigger` | count4k | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger4s` | `CHOP:trigger` | count4s | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger8k` | `CHOP:trigger` | count8k | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |
| `trigger8s` | `CHOP:trigger` | count8s | attack=0; peaklen=0; decay=0; sustain=0; release=0; timeslice=on |

### `/project1/audioAnalysis` — 135 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `Kickdrum` | `CHOP:null` | switch1 | exporttable=Kickdrum_export |
| `Kickdrum_export` | `DAT:table` | — |  |
| `Rythm_` | `CHOP:null` | switch10 | exporttable=Rythm_export |
| `Rythm_export` | `DAT:table` | — | rows=1 |
| `Snare` | `CHOP:null` | switch2 | exporttable=Snare_export |
| `Snare_export` | `DAT:table` | — |  |
| `addHigh` | `CHOP:math` | limit3 | preoff=`parent().par.Highadd` |
| `addLow` | `CHOP:math` | limit1 | preoff=`parent().par.Lowadd` |
| `addMid` | `CHOP:math` | limit2 | preoff=`parent().par.Midadd` |
| `analyze1` | `CHOP:analyze` | rename1 | function=rmspower |
| `analyze10` | `CHOP:analyze` | trail4 |  |
| `analyze11` | `CHOP:analyze` | trail7 |  |
| `analyze12` | `CHOP:analyze` | trail8 |  |
| `analyze2` | `CHOP:analyze` | rename2 | function=rmspower |
| `analyze3` | `CHOP:analyze` | rename3 | function=rmspower |
| `analyze4` | `CHOP:analyze` | shuffle4 | allowstart=off; allowend=off; nopeakvalue=1 |
| `analyze5` | `CHOP:analyze` | math15 | function=sum |
| `analyze7` | `CHOP:analyze` | audiospecpt3 | function=sum |
| `analyze8` | `CHOP:analyze` | shuffle4 | function=sum |
| `analyze9` | `CHOP:analyze` | trail3 |  |
| `annotate1` | `COMP:annotate` | — | opviewer=./annotation; encloseops=`me.EncloseOPs if hasattr(me, 'EncloseOPs') else True`; includeinorder=on; layerzone=`0 if hasattr(me, 'EncloseOPs') and me.EncloseOPs else 1`; ext0object=op.TDAnnotate.mod.AnnotateExt.AnnotateExt(me); ext0name=; ext0promote=on; parentshortcut=Annotate; …(+23) |
| `annotate2` | `COMP:annotate` | — | opviewer=./annotation; encloseops=`me.EncloseOPs if hasattr(me, 'EncloseOPs') else True`; includeinorder=on; layerzone=`0 if hasattr(me, 'EncloseOPs') and me.EncloseOPs else 1`; ext0object=op.TDAnnotate.mod.AnnotateExt.AnnotateExt(me); ext0name=; ext0promote=on; parentshortcut=Annotate; …(+22) |
| `annotate3` | `COMP:annotate` | — | opviewer=./annotation; encloseops=`me.EncloseOPs if hasattr(me, 'EncloseOPs') else True`; includeinorder=on; layerzone=`0 if hasattr(me, 'EncloseOPs') and me.EncloseOPs else 1`; ext0object=op.TDAnnotate.mod.AnnotateExt.AnnotateExt(me); ext0name=; ext0promote=on; parentshortcut=Annotate; …(+22) |
| `annotate4` | `COMP:annotate` | — | opviewer=./annotation; encloseops=`me.EncloseOPs if hasattr(me, 'EncloseOPs') else True`; includeinorder=on; layerzone=`0 if hasattr(me, 'EncloseOPs') and me.EncloseOPs else 1`; ext0object=op.TDAnnotate.mod.AnnotateExt.AnnotateExt(me); ext0name=; ext0promote=on; parentshortcut=Annotate; …(+22) |
| `annotate5` | `COMP:annotate` | — | opviewer=./annotation; encloseops=`me.EncloseOPs if hasattr(me, 'EncloseOPs') else True`; includeinorder=on; layerzone=`0 if hasattr(me, 'EncloseOPs') and me.EncloseOPs else 1`; ext0object=op.TDAnnotate.mod.AnnotateExt.AnnotateExt(me); ext0name=; ext0promote=on; parentshortcut=Annotate; …(+22) |
| `annotate6` | `COMP:annotate` | — | opviewer=./annotation; encloseops=`me.EncloseOPs if hasattr(me, 'EncloseOPs') else True`; includeinorder=on; layerzone=`0 if hasattr(me, 'EncloseOPs') and me.EncloseOPs else 1`; ext0object=op.TDAnnotate.mod.AnnotateExt.AnnotateExt(me); ext0name=; ext0promote=on; parentshortcut=Annotate; …(+22) |
| `audiodevout` | `CHOP:audiodevout` | switch_neutone | active=`parent().par.Audioout`; device=﻿{0.0.0.00000000}.{96baecef-64ce-4f9e-9ed6-7b1e1f54a6c8}\|\|Наушники_(FiiO_KA3)\|\|1; volume=`parent.AudioAnalysis.par.Listenvolume`; timeslice=on |
| `audiodyna1` | `CHOP:audiodyna` | math1 | timeslice=on |
| `audiofilein1` | `CHOP:audiofilein` | — | timeslice=on |
| `audiofilter1` | `CHOP:audiofilter` | switch_neutone | units=frequency; cutofflog=2.17609; cutofffrequency=150; rolloff=20; timeslice=on |
| `audiofilter2` | `CHOP:audiofilter` | switch_neutone | filter=bandpass; units=frequency; cutofflog=2.90309; cutofffrequency=800; rolloff=20; timeslice=on |
| `audiofilter3` | `CHOP:audiofilter` | switch_neutone | filter=highpass; units=frequency; cutofflog=3.54407; cutofffrequency=3500; resonance=0.8; rolloff=15; timeslice=on |
| `audiospecpt3` | `CHOP:audiospect` | switch_neutone | highfreqboost=1 |
| `audiovst` | `CHOP:audiovst` | merge2 | file=`'C:/Program Files/Common Files/VST3'`; mode=Neutone FX; loadpluginstate=`parent().par.Neutoneactive`; regularparms=on; callbacks=audiovst1_callbacks; tempo=120; signature1=4; signature2=4; …(+6) |
| `audiovst1_callbacks` | `DAT:text` | — | language=python |
| `centroid` | `CHOP:null` | switch9 | exporttable=centroid_export |
| `centroid_export` | `DAT:table` | — |  |
| `chopexec` | `DAT:chopexec` | — | active=`parent().par.Galmap`; fromop=`me.parent()`; chop=par2 |
| `chopexec_initPars` | `DAT:chopexec` | — | chop=info2; channel=loaded; offtoon=on; language=python |
| `constant4` | `CHOP:constant` | — |  |
| `constant5` | `CHOP:constant` | — |  |
| `constant6` | `CHOP:constant` | — | const0name=chan0 |
| `constant_off` | `CHOP:constant` | — |  |
| `constant_offlow` | `CHOP:constant` | — | const0name=low |
| `docsHelper` | `COMP:container` | — | w=32; h=32; layer=1; display=off; topsmoothness=mipmap; borderover=off; ext0object=op('./DocsHelper').module.DocsHelper(me); ext0promote=on; …(+9) |
| `filter1` | `CHOP:filter` | addLow | width=`parent().par.Lowsmooth`; spike=0.1; speedcoeff=0; timeslice=on |
| `filter2` | `CHOP:filter` | addMid | width=`parent().par.Midsmooth`; spike=0.1; speedcoeff=0; timeslice=on |
| `filter3` | `CHOP:filter` | addHigh | width=`parent().par.Highsmooth`; spike=0.1; speedcoeff=0; timeslice=on |
| `filter8` | `CHOP:filter` | analyze7 | width=0.5; spike=0.1; speedcoeff=0; timeslice=on |
| `fms` | `CHOP:null` | switch8 | exporttable=fms_export |
| `fms_export` | `DAT:table` | — |  |
| `fmsd` | `COMP:container` | — | x=196; w=`43*4`; h=45; bgcolorr=0.1; bgcolorg=0.1; bgcolorb=0.1; bgalpha=1; borderover=off; …(+6) |
| `high` | `COMP:container` | — | x=10; y=-100; w=`43*4`; h=45; bottomanchor=1; vorigin=1; bgcolorr=0.1; bgcolorg=0.1; …(+13) |
| `high_` | `CHOP:null` | switch6 | exporttable=high__export |
| `high__export` | `DAT:table` | — |  |
| `in1` | `CHOP:in` | audiofilein1 |  |
| `info1` | `DAT:info` | — | op=audiovst; language=text |
| `info2` | `CHOP:info` | — | op=audiovst |
| `kick` | `COMP:container` | — | x=369; y=-10; w=`43*4`; h=45; bottomanchor=1; vorigin=1; bgcolorr=0.1; bgcolorg=0.1; …(+10) |
| `limit1` | `CHOP:limit` | math3 | type=clamp; min=0; max=100 |
| `limit2` | `CHOP:limit` | math4 | type=clamp; min=0; max=100 |
| `limit3` | `CHOP:limit` | math5 | type=clamp; min=0; max=100 |
| `limit4` | `CHOP:limit` | math8 | type=clamp; min=0; max=100 |
| `limit5` | `CHOP:limit` | math9 | type=clamp; min=0; max=100 |
| `limit6` | `CHOP:limit` | math10 | type=clamp; min=`parent().par.Rythmthresh`; max=100 |
| `logic1` | `CHOP:logic` | limit4 | timeslice=on |
| `logic2` | `CHOP:logic` | limit5 | timeslice=on |
| `logic3` | `CHOP:logic` | math11 | convert=nonzero; timeslice=on |
| `low` | `COMP:container` | — | x=10; y=-10; w=`43*4`; h=45; bottomanchor=1; vorigin=1; bgcolorr=0.1; bgcolorg=0.1; …(+13) |
| `low_` | `CHOP:null` | switch4 | exporttable=low__export |
| `low__export` | `DAT:table` | — |  |
| `map` | `DAT:table` | — | fill=setsize; rows=10; cols=2 |
| `map2` | `DAT:substitute` | map | before=/project1/; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `map3` | `DAT:reorder` | map | method=replacenames; before=src dest; after=dest src |
| `math1` | `CHOP:math` | in1 | chanop=avg |
| `math10` | `CHOP:math` | trail5 | preoff=`-parent().par.Rythmthresh`; gain=1000 |
| `math11` | `CHOP:math` | limit6 | preoff=`-op('limit6').par.min` |
| `math12` | `CHOP:math` | analyze2 | gain=2 |
| `math13` | `CHOP:math` | merge4 | chanop=div |
| `math14` | `CHOP:math` | analyze12 | fromrange1=18; fromrange2=32 |
| `math15` | `CHOP:math` | pattern1 | chanop=mul |
| `math16` | `CHOP:math` | analyze3 | gain=4 |
| `math17` | `CHOP:math` | audiovst | chanop=avg |
| `math2` | `CHOP:math` | analyze1 |  |
| `math3` | `CHOP:math` | math2 | preoff=`-parent().par.Lowthresh`; gain=`parent().par.Lowgain` |
| `math4` | `CHOP:math` | math12 | preoff=`-parent().par.Midthresh`; gain=`parent().par.Midgain` |
| `math5` | `CHOP:math` | math16 | preoff=`-parent().par.Highthresh`; gain=`parent().par.Highgain` |
| `math6` | `CHOP:math` | select2 | fromrange1=100; fromrange2=1800 |
| `math7` | `CHOP:math` | analyze10 | fromrange1=0; fromrange2=1000 |
| `math8` | `CHOP:math` | math2 | preoff=`-parent().par.Kickthresh` |
| `math9` | `CHOP:math` | math16 | preoff=`-parent().par.Snarethresh` |
| `merge2` | `CHOP:merge` | audiodyna1, audiodyna1 |  |
| `merge4` | `CHOP:merge` | analyze5, analyze8 |  |
| `mid` | `COMP:container` | — | x=10; y=-55; w=`43*4`; h=45; bottomanchor=1; vorigin=1; bgcolorr=0.1; bgcolorg=0.1; …(+13) |
| `mid_` | `CHOP:null` | switch5 | exporttable=mid__export |
| `mid__export` | `DAT:table` | — |  |
| `null_kickSignal` | `CHOP:null` | trigger1 |  |
| `nullsnareSignal` | `CHOP:null` | logic2 |  |
| `out1` | `CHOP:out` | par2 | label=`me.name` |
| `out2` | `CHOP:out` | switch_neutone | label=`me.name` |
| `par2` | `CHOP:par` | — | ops=low mid high kick snare rythm smsd fmsd spectralCentroid; parameters=Output; renamefrom=*:Output; renameto=* |
| `parexec1` | `DAT:parexec` | — | fromop=`me.parent()`; op=unmap; pars=Value1; onpulse=off |
| `parexec_help` | `DAT:parexec` | — | fromop=`me.parent()`; op=..; pars=Openpars Help Copymvspec Createengine Gotocuepause Gotocueplay Gototimepause Gototimeplay; valuechange=off; builtin=off |
| `pattern1` | `CHOP:pattern` | shuffle4 | wavetype=ramp; amp=50; combine=append |
| `readMeMapping` | `DAT:text` | — | wordwrap=on |
| `releaseHistory` | `DAT:text` | — | wordwrap=on |
| `rename1` | `CHOP:rename` | audiofilter1 | renameto=low |
| `rename2` | `CHOP:rename` | audiofilter2 | renameto=mid |
| `rename3` | `CHOP:rename` | audiofilter3 | renameto=high |
| `rythm` | `COMP:container` | — | x=369; y=-100; w=`43*4`; h=45; bottomanchor=1; vorigin=1; bgcolorr=0.1; bgcolorg=0.1; …(+10) |
| `select2` | `CHOP:select` | — | chops=analyze9; channames=chan1 |
| `showMap` | `COMP:button` | — | buttontype=momentary; scaletofit=onlyshrink; w=30; h=25; leftanchor=1; horigin=1; bottomanchor=1; vorigin=1; …(+22) |
| `shuffle3` | `CHOP:shuffle` | audiospecpt3 | method=splitn; nval=440 |
| `shuffle4` | `CHOP:shuffle` | shuffle3 | method=swap; nval=5; firstsample=on |
| `sms` | `CHOP:null` | switch7 | exporttable=sms_export |
| `sms_export` | `DAT:table` | — |  |
| `smsd` | `COMP:container` | — | x=9; w=`43*4`; h=45; bgcolorr=0.1; bgcolorg=0.1; bgcolorb=0.1; bgalpha=1; borderover=off; …(+6) |
| `spectralCentroid` | `COMP:container` | — | x=387; w=`43*4`; h=45; bgcolorr=0.1; bgcolorg=0.1; bgcolorb=0.1; bgalpha=1; borderover=off; …(+6) |
| `switch1` | `CHOP:switch` | constant_off, null_kickSignal | index=`op('kick').par.Active` |
| `switch10` | `CHOP:switch` | constant6, logic3 | index=`op('rythm').par.Active` |
| `switch2` | `CHOP:switch` | constant_off, nullsnareSignal | index=`op('snare').par.Active` |
| `switch4` | `CHOP:switch` | constant_offlow, filter1 | index=`op('low').par.Active` |
| `switch5` | `CHOP:switch` | constant_offlow, filter2 | index=`op('mid').par.Active` |
| `switch6` | `CHOP:switch` | constant_offlow, filter3 | index=`op('high').par.Active` |
| `switch7` | `CHOP:switch` | constant4, math6 | index=`op('smsd').par.Active` |
| `switch8` | `CHOP:switch` | constant4, math7 | index=`op('fmsd').par.Active` |
| `switch9` | `CHOP:switch` | constant5, math14 | index=`op('spectralCentroid').par.Active` |
| `switch_neutone` | `CHOP:switch` | math17 | index=`parent().par.Neutoneactive` |
| `trail3` | `CHOP:trail` | filter8 | wlength=10; timeslice=on |
| `trail4` | `CHOP:trail` | filter8 | timeslice=on |
| `trail5` | `CHOP:trail` | analyze4 | wlength=1; timeslice=on |
| `trail7` | `CHOP:trail` | math13 | wlength=4.31; timeslice=on |
| `trail8` | `CHOP:trail` | analyze11 | wlength=1.99; timeslice=on |
| `trigger1` | `CHOP:trigger` | logic1 | retrigger=0.08; attack=0; decay=0; sustain=0; release=0; timeslice=on |
| `unmap` | `COMP:button` | — | buttontype=momentary; scaletofit=onlyshrink; y=27; w=45; h=30; leftanchor=1; horigin=1; display=`parent().par.Galmap`; …(+20) |

### `/project1/audioAnalysis/fmsd` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `map` | `COMP:button` | — | buttontype=momentary; scaletofit=onlyshrink; x=333; w=30; h=25; layer=3; alignorder=6; display=`parent(2).par.Galmap`; …(+25) |

### `/project1/audioAnalysis/high` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `map` | `COMP:button` | — | buttontype=momentary; scaletofit=onlyshrink; x=333; w=30; h=25; layer=3; alignorder=6; display=`parent(2).par.Galmap`; …(+26) |

### `/project1/audioAnalysis/kick` — 4 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `active` | `COMP:button` | — | buttontype=momentary; scaletofit=onlyshrink; w=80; h=25; vmode=anchors; bottomanchor=0.2; topanchor=0.8; bgcolorr=`me.par.Bgcolorr`; …(+32) |
| `map` | `COMP:button` | — | buttontype=momentary; scaletofit=onlyshrink; x=333; w=30; h=25; layer=3; alignorder=6; display=`parent(2).par.Galmap`; …(+26) |
| `output` | `COMP:container` | — | w=30; h=30; vmode=anchors; bottomanchor=0.2; topanchor=0.8; alignorder=5; helpdat=./help; bgcolorr=`me.par.Bgcolorr`; …(+16) |
| `threshold` | `COMP:slider` | — | slidertype=slideruv; clampul=off; clampuh=off; clampvl=off; clampvh=off; w=45; vmode=fill; bottomanchor=0.5; …(+24) |

### `/project1/audioAnalysis/kick/active` — 14 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:constant` | — | colorr=0; colorg=0; colorb=0; alpha=0; combineinput=res; resolutionw=2; resolutionh=2 |
| `indicator` | `COMP:container` | — | x=0; w=`parent().height * me.par.Iconaspect`; layer=2; vmode=fill; display=`int(parent().par.Indicator!='text' or (parent().par.Behavior=='pulse' and parent().par.Labellocate!='none'))`; clickthrough=on; top=`me.par.Iconswitch.eval()`; borderaalpha=0; …(+8) |
| `iparBinary` | `COMP:base` | — | Flip=`not ((parent().par.Horizvert == 'horizontal') or ((parent().par.Horizvert == 'auto') and (parent().width >= parent().height)))` |
| `label` | `COMP:text` | — | text=`parent().par.Label if parent().par.Labellocate!='none' else ''`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=0.5; fontcolorg=0.5; fontcolorb=0.5; …(+25) |
| `out1` | `CHOP:out` | val |  |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=`''`; ontooff=on; valuechange=on |
| `parexec1` | `DAT:parexec` | — | fromop=`me.parent()`; op=..; pars=Value1; onpulse=off; builtin=off |
| `script1_callbacks` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `text` | `COMP:text` | — | text=`[parent().par.Textoff, parent().par.Texton, parent().par.Textoff, parent().par.Texton][int(math.ceil(op('tstate')[0]))]`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=`[.65,.8][parent().panel.select]`; fontcolorg=`me.par.fontcolorr`; fontcolorb=`me.par.fontcolorr`; …(+31) |
| `tstate` | `CHOP:express` | val | expr0expr=`me.inputVal + 2 * parent().panel.rollover` |
| `val` | `CHOP:par` | — | parameters=Value1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/kick/active/indicator` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `dotoff` | `TOP:circle` | — | radiusx=0.34; radiusy=0.34; fillcolorr=`[parent(2).par.Bgcolorr, .7][int(parent(2).par.Value1.eval())]`; fillcolorg=`[parent(2).par.Bgcolorg, .7][int(parent(2).par.Value1.eval())]`; fillcolorb=`[parent(2).par.Bgcolorb, .7][int(parent(2).par.Value1.eval())]`; borderr=0.7; borderg=0.7; borderb=0.7; …(+7) |

### `/project1/audioAnalysis/kick/active/text` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `textback` | `TOP:rectangle` | — | sizex=`parent().width-2`; sizey=`parent().height-2`; sizeunit=pixels; fillcolorr=0.23; fillcolorg=0.23; fillcolorb=0.23; fillalpha=`parent.binary.par.Behavior=='pulse'`; cornerradius=0.33; …(+6) |

### `/project1/audioAnalysis/kick/map` — 14 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:constant` | — | colorr=0; colorg=0; colorb=0; alpha=0; combineinput=res; resolutionw=2; resolutionh=2 |
| `indicator` | `COMP:container` | — | x=0; w=`parent().height * me.par.Iconaspect`; layer=2; vmode=fill; display=`int(parent().par.Indicator!='text' or (parent().par.Behavior=='pulse' and parent().par.Labellocate!='none'))`; clickthrough=on; top=`me.par.Iconswitch.eval()`; borderaalpha=0; …(+8) |
| `iparBinary` | `COMP:base` | — | Flip=`not ((parent().par.Horizvert == 'horizontal') or ((parent().par.Horizvert == 'auto') and (parent().width >= parent().height)))` |
| `label` | `COMP:text` | — | text=`parent().par.Label if parent().par.Labellocate!='none' else ''`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=0.5; fontcolorg=0.5; fontcolorb=0.5; …(+25) |
| `out1` | `CHOP:out` | val |  |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=`''`; ontooff=on; valuechange=on |
| `parexec1` | `DAT:parexec` | — | fromop=`me.parent()`; op=..; pars=Value1; onpulse=off; builtin=off |
| `script1_callbacks` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `text` | `COMP:text` | — | text=`[parent().par.Textoff, parent().par.Texton, parent().par.Textoff, parent().par.Texton][int(math.ceil(op('tstate')[0]))]`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=`[.65,.8][parent().panel.select]`; fontcolorg=`me.par.fontcolorr`; fontcolorb=`me.par.fontcolorr`; …(+31) |
| `tstate` | `CHOP:express` | val | expr0expr=`me.inputVal + 2 * parent().panel.rollover` |
| `val` | `CHOP:par` | — | parameters=Value1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/kick/map/indicator` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `dotoff` | `TOP:circle` | — | radiusx=0.34; radiusy=0.34; fillcolorr=`[parent(2).par.Bgcolorr, .7][int(parent(2).par.Value1.eval())]`; fillcolorg=`[parent(2).par.Bgcolorg, .7][int(parent(2).par.Value1.eval())]`; fillcolorb=`[parent(2).par.Bgcolorb, .7][int(parent(2).par.Value1.eval())]`; borderr=0.7; borderg=0.7; borderb=0.7; …(+7) |

### `/project1/audioAnalysis/kick/map/text` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `textback` | `TOP:rectangle` | — | sizex=`parent().width-2`; sizey=`parent().height-2`; sizeunit=pixels; fillcolorr=0.23; fillcolorg=0.23; fillcolorb=0.23; fillalpha=`parent.binary.par.Behavior=='pulse'`; cornerradius=0.33; …(+6) |

### `/project1/audioAnalysis/kick/output` — 5 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:constant` | — | colorr=0; colorg=0; colorb=0; alpha=0; combineinput=res; resolutionw=1; resolutionh=1 |
| `label` | `COMP:text` | — | text=`parent().par.Label if parent().par.Labellocate!='none' else ''`; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=0.5; fontcolorg=`me.par.fontcolorr`; fontcolorb=`me.par.fontcolorr`; textoffsetx=0; …(+23) |
| `script1_callbacks` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |

### `/project1/audioAnalysis/kick/threshold` — 29 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `circle1` | `TOP:circle` | — | radiusx=`parent().par.Arcdiameter*.5*2`; radiusy=`me.par.radiusx`; radiusunit=pixels; fillcolorr=0; fillcolorg=0; fillcolorb=0; fillalpha=0; borderr=1; …(+10) |
| `eval1` | `DAT:eval` | ramp3_keys | rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `field` | `COMP:text` | — | text=`parent().par.Value1`; type=float; formatting=number; editmode=editable; legacyfontselection=on; fontsize=20; fontcolorr=0.65; fontcolorg=0.65; …(+23) |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:null` | interior |  |
| `interior` | `TOP:comp` | transformy | tops=`'transformy' if (parent().par.Interiortop==None) else parent().par.Interiortop`; selectinput=`int((parent().par.Interiortop==None))`; operand=over; size=input1; px=0; py=0; legacyxform=on; inputfiltertype=mipmap |
| `logic1` | `CHOP:logic` | pattern2 | convert=bound; boundmin=0.3; boundmax=0.5 |
| `lookup2` | `CHOP:lookup` | logic1, pattern1 |  |
| `lookup3` | `TOP:lookup` | ramp3 | method=chop; darkuv1=0; darkuv2=0; lightuv1=1; lightuv2=0; chop=lookup2 |
| `math1` | `CHOP:math` | range1 | torange1=-240; torange2=60; exporttable=math1_export |
| `math1_export` | `DAT:table` | — |  |
| `math_export1` | `DAT:table` | — | rows=1 |
| `multiply2` | `TOP:multiply` | lookup3, circle1 | legacyxform=on |
| `out1` | `CHOP:out` | val | exportmethod=autoname |
| `over2` | `TOP:over` | rectangle, multiply2 | r=-160; legacyxform=on |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=`''`; whileon=on; valuechange=on |
| `pattern1` | `CHOP:pattern` | — | wavetype=ramp; length=2; torange1=`[parent().par.Arcbaser,parent().par.Arcbaseg,parent().par.Arcbaseb][me.chanIndex]`; torange2=`[parent().par.Arcdispr,parent().par.Arcdispg,parent().par.Arcdispb][me.chanIndex]`; channelname=r g b; left=hold; right=hold |
| `pattern2` | `CHOP:pattern` | — | wavetype=ramp; length=400 |
| `ramp3` | `TOP:ramp` | — | dat=eval1; color1=0; color2=0; color3=0; color4=1; type=radial; phase=`parent().par.Arcanglelow/360`; extendright=repeat; …(+2) |
| `ramp3_keys` | `DAT:table` | — | rows=5; cols=5 |
| `range1` | `CHOP:math` | val1 | fromrange1=`parent().par.Rangelow1`; fromrange2=`parent().par.Rangehigh1`; exporttable=math_export1 |
| `rectangle` | `TOP:rectangle` | — | sizex=`parent().par.Arcdiameter*.5 + me.par.sizey/2`; sizey=`parent().par.Handlesize`; sizeunit=pixels; centerx=`me.par.sizex/2 - me.par.sizey/2`; centery=0; centerunit=pixels; fillcolorr=`parent().par.Arcbaser`; fillcolorg=`parent().par.Arcbaseg`; …(+4) |
| `script1_callbacks` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `text` | `COMP:text` | — | text=`parent().par.Label`; callbacks=text1_callbacks; legacyfontselection=on; fontsize=20; fontcolorr=0.6; fontcolorg=0.6; fontcolorb=0.6; textoffsetx=0; …(+11) |
| `transformy` | `TOP:transform` | over2 | tx=0; ty=0; tunit=pixels |
| `val` | `CHOP:par` | — | parameters=Value1; builtin=on; renameto=`parent().par.Name1` |
| `val1` | `CHOP:par` | — | parameters=Value1 Default1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/low` — 7 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `active` | `COMP:button` | — | buttontype=momentary; scaletofit=onlyshrink; w=80; h=25; vmode=anchors; bottomanchor=0.2; topanchor=0.8; bgcolorr=`me.par.Bgcolorr`; …(+32) |
| `add` | `COMP:slider` | — | slidertype=slideruv; clampul=off; clampuh=off; clampvl=off; clampvh=off; w=45; vmode=fill; bottomanchor=0.5; …(+25) |
| `gain` | `COMP:slider` | — | slidertype=slideruv; clampul=off; clampuh=off; clampvl=off; clampvh=off; w=45; vmode=fill; bottomanchor=0.5; …(+25) |
| `map` | `COMP:button` | — | buttontype=momentary; scaletofit=onlyshrink; x=333; w=30; h=25; layer=3; alignorder=6; display=`parent(2).par.Galmap`; …(+26) |
| `output` | `COMP:slider` | — | slidertype=slideru; zonel=0; zoner=1; clampul=off; clampuh=off; w=43; vmode=anchors; bottomanchor=0.1; …(+28) |
| `smooth` | `COMP:slider` | — | slidertype=slideruv; clampul=off; clampuh=off; clampvl=off; clampvh=off; x=`43*3`; w=45; vmode=fill; …(+26) |
| `threshold` | `COMP:slider` | — | slidertype=slideruv; clampul=off; clampuh=off; clampvl=off; clampvh=off; w=45; vmode=fill; bottomanchor=0.5; …(+24) |

### `/project1/audioAnalysis/low/active` — 14 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:constant` | — | colorr=0; colorg=0; colorb=0; alpha=0; combineinput=res; resolutionw=2; resolutionh=2 |
| `indicator` | `COMP:container` | — | x=0; w=`parent().height * me.par.Iconaspect`; layer=2; vmode=fill; display=`int(parent().par.Indicator!='text' or (parent().par.Behavior=='pulse' and parent().par.Labellocate!='none'))`; clickthrough=on; top=`me.par.Iconswitch.eval()`; borderaalpha=0; …(+8) |
| `iparBinary` | `COMP:base` | — | Flip=`not ((parent().par.Horizvert == 'horizontal') or ((parent().par.Horizvert == 'auto') and (parent().width >= parent().height)))` |
| `label` | `COMP:text` | — | text=`parent().par.Label if parent().par.Labellocate!='none' else ''`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=0.5; fontcolorg=0.5; fontcolorb=0.5; …(+25) |
| `out1` | `CHOP:out` | val |  |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=`''`; ontooff=on; valuechange=on |
| `parexec1` | `DAT:parexec` | — | fromop=`me.parent()`; op=..; pars=Value1; onpulse=off; builtin=off |
| `script1_callbacks` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `text` | `COMP:text` | — | text=`[parent().par.Textoff, parent().par.Texton, parent().par.Textoff, parent().par.Texton][int(math.ceil(op('tstate')[0]))]`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=`[.65,.8][parent().panel.select]`; fontcolorg=`me.par.fontcolorr`; fontcolorb=`me.par.fontcolorr`; …(+31) |
| `tstate` | `CHOP:express` | val | expr0expr=`me.inputVal + 2 * parent().panel.rollover` |
| `val` | `CHOP:par` | — | parameters=Value1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/low/active/indicator` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `dotoff` | `TOP:circle` | — | radiusx=0.34; radiusy=0.34; fillcolorr=`[parent(2).par.Bgcolorr, .7][int(parent(2).par.Value1.eval())]`; fillcolorg=`[parent(2).par.Bgcolorg, .7][int(parent(2).par.Value1.eval())]`; fillcolorb=`[parent(2).par.Bgcolorb, .7][int(parent(2).par.Value1.eval())]`; borderr=0.7; borderg=0.7; borderb=0.7; …(+7) |

### `/project1/audioAnalysis/low/active/text` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `textback` | `TOP:rectangle` | — | sizex=`parent().width-2`; sizey=`parent().height-2`; sizeunit=pixels; fillcolorr=0.23; fillcolorg=0.23; fillcolorb=0.23; fillalpha=`parent.binary.par.Behavior=='pulse'`; cornerradius=0.33; …(+6) |

### `/project1/audioAnalysis/low/add` — 29 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `circle1` | `TOP:circle` | — | radiusx=`parent().par.Arcdiameter*.5*2`; radiusy=`me.par.radiusx`; radiusunit=pixels; fillcolorr=0; fillcolorg=0; fillcolorb=0; fillalpha=0; borderr=1; …(+10) |
| `eval1` | `DAT:eval` | ramp3_keys | rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `field` | `COMP:text` | — | text=`parent().par.Value1`; type=float; formatting=number; editmode=editable; legacyfontselection=on; fontsize=20; fontcolorr=0.65; fontcolorg=0.65; …(+23) |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:null` | interior |  |
| `interior` | `TOP:comp` | transformy | tops=`'transformy' if (parent().par.Interiortop==None) else parent().par.Interiortop`; selectinput=`int((parent().par.Interiortop==None))`; operand=over; size=input1; px=0; py=0; legacyxform=on; inputfiltertype=mipmap |
| `logic1` | `CHOP:logic` | pattern2 | convert=bound; boundmin=0.3; boundmax=0.5 |
| `lookup2` | `CHOP:lookup` | logic1, pattern1 |  |
| `lookup3` | `TOP:lookup` | ramp3 | method=chop; darkuv1=0; darkuv2=0; lightuv1=1; lightuv2=0; chop=lookup2 |
| `math1` | `CHOP:math` | range1 | torange1=-240; torange2=60; exporttable=math1_export |
| `math1_export` | `DAT:table` | — |  |
| `math_export1` | `DAT:table` | — | rows=1 |
| `multiply2` | `TOP:multiply` | lookup3, circle1 | legacyxform=on |
| `out1` | `CHOP:out` | val | exportmethod=autoname |
| `over2` | `TOP:over` | rectangle, multiply2 | r=-160; legacyxform=on |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=`''`; whileon=on; valuechange=on |
| `pattern1` | `CHOP:pattern` | — | wavetype=ramp; length=2; torange1=`[parent().par.Arcbaser,parent().par.Arcbaseg,parent().par.Arcbaseb][me.chanIndex]`; torange2=`[parent().par.Arcdispr,parent().par.Arcdispg,parent().par.Arcdispb][me.chanIndex]`; channelname=r g b; left=hold; right=hold |
| `pattern2` | `CHOP:pattern` | — | wavetype=ramp; length=400 |
| `ramp3` | `TOP:ramp` | — | dat=eval1; color1=0; color2=0; color3=0; color4=1; type=radial; phase=`parent().par.Arcanglelow/360`; extendright=repeat; …(+2) |
| `ramp3_keys` | `DAT:table` | — | rows=5; cols=5 |
| `range1` | `CHOP:math` | val1 | fromrange1=`parent().par.Rangelow1`; fromrange2=`parent().par.Rangehigh1`; exporttable=math_export1 |
| `rectangle` | `TOP:rectangle` | — | sizex=`parent().par.Arcdiameter*.5 + me.par.sizey/2`; sizey=`parent().par.Handlesize`; sizeunit=pixels; centerx=`me.par.sizex/2 - me.par.sizey/2`; centery=0; centerunit=pixels; fillcolorr=`parent().par.Arcbaser`; fillcolorg=`parent().par.Arcbaseg`; …(+4) |
| `script1_callbacks` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `text` | `COMP:text` | — | text=`parent().par.Label`; callbacks=text1_callbacks; legacyfontselection=on; fontsize=20; fontcolorr=0.6; fontcolorg=0.6; fontcolorb=0.6; textoffsetx=0; …(+11) |
| `transformy` | `TOP:transform` | over2 | tx=0; ty=0; tunit=pixels |
| `val` | `CHOP:par` | — | parameters=Value1; builtin=on; renameto=`parent().par.Name1` |
| `val1` | `CHOP:par` | — | parameters=Value1 Default1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/low/gain` — 29 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `circle1` | `TOP:circle` | — | radiusx=`parent().par.Arcdiameter*.5*2`; radiusy=`me.par.radiusx`; radiusunit=pixels; fillcolorr=0; fillcolorg=0; fillcolorb=0; fillalpha=0; borderr=1; …(+10) |
| `eval1` | `DAT:eval` | ramp3_keys | rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `field` | `COMP:text` | — | text=`parent().par.Value1`; type=float; formatting=number; editmode=editable; legacyfontselection=on; fontsize=20; fontcolorr=0.65; fontcolorg=0.65; …(+23) |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:null` | interior |  |
| `interior` | `TOP:comp` | transformy | tops=`'transformy' if (parent().par.Interiortop==None) else parent().par.Interiortop`; selectinput=`int((parent().par.Interiortop==None))`; operand=over; size=input1; px=0; py=0; legacyxform=on; inputfiltertype=mipmap |
| `logic1` | `CHOP:logic` | pattern2 | convert=bound; boundmin=0.3; boundmax=0.5 |
| `lookup2` | `CHOP:lookup` | logic1, pattern1 |  |
| `lookup3` | `TOP:lookup` | ramp3 | method=chop; darkuv1=0; darkuv2=0; lightuv1=1; lightuv2=0; chop=lookup2 |
| `math1` | `CHOP:math` | range1 | torange1=-240; torange2=60; exporttable=math1_export |
| `math1_export` | `DAT:table` | — |  |
| `math_export1` | `DAT:table` | — | rows=1 |
| `multiply2` | `TOP:multiply` | lookup3, circle1 | legacyxform=on |
| `out1` | `CHOP:out` | val | exportmethod=autoname |
| `over2` | `TOP:over` | rectangle, multiply2 | r=-160; legacyxform=on |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=`''`; whileon=on; valuechange=on |
| `pattern1` | `CHOP:pattern` | — | wavetype=ramp; length=2; torange1=`[parent().par.Arcbaser,parent().par.Arcbaseg,parent().par.Arcbaseb][me.chanIndex]`; torange2=`[parent().par.Arcdispr,parent().par.Arcdispg,parent().par.Arcdispb][me.chanIndex]`; channelname=r g b; left=hold; right=hold |
| `pattern2` | `CHOP:pattern` | — | wavetype=ramp; length=400 |
| `ramp3` | `TOP:ramp` | — | dat=eval1; color1=0; color2=0; color3=0; color4=1; type=radial; phase=`parent().par.Arcanglelow/360`; extendright=repeat; …(+2) |
| `ramp3_keys` | `DAT:table` | — | rows=5; cols=5 |
| `range1` | `CHOP:math` | val1 | fromrange1=`parent().par.Rangelow1`; fromrange2=`parent().par.Rangehigh1`; exporttable=math_export1 |
| `rectangle` | `TOP:rectangle` | — | sizex=`parent().par.Arcdiameter*.5 + me.par.sizey/2`; sizey=`parent().par.Handlesize`; sizeunit=pixels; centerx=`me.par.sizex/2 - me.par.sizey/2`; centery=0; centerunit=pixels; fillcolorr=`parent().par.Arcbaser`; fillcolorg=`parent().par.Arcbaseg`; …(+4) |
| `script1_callbacks` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `text` | `COMP:text` | — | text=`parent().par.Label`; callbacks=text1_callbacks; legacyfontselection=on; fontsize=20; fontcolorr=0.6; fontcolorg=0.6; fontcolorb=0.6; textoffsetx=0; …(+11) |
| `transformy` | `TOP:transform` | over2 | tx=0; ty=0; tunit=pixels |
| `val` | `CHOP:par` | — | parameters=Value1; builtin=on; renameto=`parent().par.Name1` |
| `val1` | `CHOP:par` | — | parameters=Value1 Default1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/low/map` — 14 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:constant` | — | colorr=0; colorg=0; colorb=0; alpha=0; combineinput=res; resolutionw=2; resolutionh=2 |
| `indicator` | `COMP:container` | — | x=0; w=`parent().height * me.par.Iconaspect`; layer=2; vmode=fill; display=`int(parent().par.Indicator!='text' or (parent().par.Behavior=='pulse' and parent().par.Labellocate!='none'))`; clickthrough=on; top=`me.par.Iconswitch.eval()`; borderaalpha=0; …(+8) |
| `iparBinary` | `COMP:base` | — | Flip=`not ((parent().par.Horizvert == 'horizontal') or ((parent().par.Horizvert == 'auto') and (parent().width >= parent().height)))` |
| `label` | `COMP:text` | — | text=`parent().par.Label if parent().par.Labellocate!='none' else ''`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=0.5; fontcolorg=0.5; fontcolorb=0.5; …(+25) |
| `out1` | `CHOP:out` | val |  |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=`''`; ontooff=on; valuechange=on |
| `parexec1` | `DAT:parexec` | — | fromop=`me.parent()`; op=..; pars=Value1; onpulse=off; builtin=off |
| `script1_callbacks` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `text` | `COMP:text` | — | text=`[parent().par.Textoff, parent().par.Texton, parent().par.Textoff, parent().par.Texton][int(math.ceil(op('tstate')[0]))]`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=`[.65,.8][parent().panel.select]`; fontcolorg=`me.par.fontcolorr`; fontcolorb=`me.par.fontcolorr`; …(+31) |
| `tstate` | `CHOP:express` | val | expr0expr=`me.inputVal + 2 * parent().panel.rollover` |
| `val` | `CHOP:par` | — | parameters=Value1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/low/map/indicator` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `dotoff` | `TOP:circle` | — | radiusx=0.34; radiusy=0.34; fillcolorr=`[parent(2).par.Bgcolorr, .7][int(parent(2).par.Value1.eval())]`; fillcolorg=`[parent(2).par.Bgcolorg, .7][int(parent(2).par.Value1.eval())]`; fillcolorb=`[parent(2).par.Bgcolorb, .7][int(parent(2).par.Value1.eval())]`; borderr=0.7; borderg=0.7; borderb=0.7; …(+7) |

### `/project1/audioAnalysis/low/map/text` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `textback` | `TOP:rectangle` | — | sizex=`parent().width-2`; sizey=`parent().height-2`; sizeunit=pixels; fillcolorr=0.23; fillcolorg=0.23; fillcolorb=0.23; fillalpha=`parent.binary.par.Behavior=='pulse'`; cornerradius=0.33; …(+6) |

### `/project1/audioAnalysis/low/output` — 21 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `deadZone` | `COMP:container` | — | w=`[(parent().par.Borderlow-8), parent().width][ipar.Slider.Flip]`; h=`[parent().height, (parent().par.Borderlow-8)][ipar.Slider.Flip]`; display=`parent().par.Labellocate=='left'`; enable=off; topsmoothness=mipmap; borderover=off |
| `delete` | `CHOP:delete` | val | delchannels=on; discard=nonscoped; select=bynum; delscope=Default1; selnumbers=0; legacypattern=on |
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `field` | `COMP:text` | — | text=`parent().par.Value1`; type=float; formatting=number; precision=`parent().par.Decimals`; thousandsseparator=space; editmode=`['locked', 'editable'][parent().par.Numfield == 'edit']`; legacyfontselection=on; fontsize=`op('label').par.fontsize`; …(+28) |
| `flip` | `TOP:flip` | ramp | flipy=on; flop=bottomleft; filtertype=mipmap; npasses=`ipar.Slider.Flip` |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:null` | interior |  |
| `indicator` | `COMP:container` | — | x=`[(op('math')['v1'] - parent().par.Handlesize/2), 2][ipar.Slider.Flip]`; y=`[2, (op('math')['v1'] - parent().par.Handlesize/2)][ipar.Slider.Flip]`; w=`[parent().par.Handlesize, parent().width-4][ipar.Slider.Flip]`; h=`[parent().height-4, parent().par.Handlesize][ipar.Slider.Flip]`; layer=2; clickthrough=on; bgcolorr=0.7; bgcolorg=0.7; …(+3) |
| `interior` | `TOP:comp` | flip | tops=`'flip' if (parent().par.Interiortop==None) else parent().par.Interiortop`; selectinput=`int((parent().par.Interiortop==None))`; operand=over; size=input1; tx=0; ty=0; sx=1; sy=1; …(+6) |
| `iparSlider` | `COMP:base` | — | Flip=`not ((parent().par.Horizvert == 'horizontal') or ((parent().par.Horizvert == 'auto') and (parent().width >= parent().height)))` |
| `label` | `COMP:text` | — | text=`parent().par.Label if parent().par.Labellocate!='none' else ''`; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=0.5; fontcolorg=0.5; fontcolorb=0.5; textoffsetx=0; …(+19) |
| `math` | `CHOP:math` | val | fromrange1=`parent().par.Rangelow1`; fromrange2=`parent().par.Rangehigh1`; torange1=0; torange2=1 |
| `offsetbar` | `COMP:container` | — | x=`[min(op('math')[0], op('math')[1]), parent().width-4][ipar.Slider.Flip]`; y=`[2, min(op('math')[0], op('math')[1])][ipar.Slider.Flip]`; w=`[abs(op('math')[0] - op('math')[1]), 2][ipar.Slider.Flip]`; h=`[2, abs(op('math')[0] - op('math')[1])][ipar.Slider.Flip]`; layer=1; display=`parent().par.Showoffset`; clickthrough=on; bgcolorr=0.06666; …(+4) |
| `out1` | `CHOP:out` | delete | exportmethod=autoname |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=u; offtoon=off; valuechange=on |
| `ramp` | `TOP:ramp` | — | dat=`'ramp_keys'`; color1=0.17; color2=0.17; color3=0.17; color4=1; interpnotches=step; resolutionw=1000; resolutionh=1; …(+3) |
| `ramp_keys` | `DAT:script` | — | callbacks=script1_callbacks1 |
| `script1_callbacks` | `DAT:text` | — |  |
| `script1_callbacks1` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `val` | `CHOP:par` | — | parameters=Value1 Default1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/low/smooth` — 29 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `circle1` | `TOP:circle` | — | radiusx=`parent().par.Arcdiameter*.5*2`; radiusy=`me.par.radiusx`; radiusunit=pixels; fillcolorr=0; fillcolorg=0; fillcolorb=0; fillalpha=0; borderr=1; …(+10) |
| `eval1` | `DAT:eval` | ramp3_keys | rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `field` | `COMP:text` | — | text=`parent().par.Value1`; type=float; formatting=number; editmode=editable; legacyfontselection=on; fontsize=20; fontcolorr=0.65; fontcolorg=0.65; …(+23) |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:null` | interior |  |
| `interior` | `TOP:comp` | transformy | tops=`'transformy' if (parent().par.Interiortop==None) else parent().par.Interiortop`; selectinput=`int((parent().par.Interiortop==None))`; operand=over; size=input1; px=0; py=0; legacyxform=on; inputfiltertype=mipmap |
| `logic1` | `CHOP:logic` | pattern2 | convert=bound; boundmin=0.3; boundmax=0.5 |
| `lookup2` | `CHOP:lookup` | logic1, pattern1 |  |
| `lookup3` | `TOP:lookup` | ramp3 | method=chop; darkuv1=0; darkuv2=0; lightuv1=1; lightuv2=0; chop=lookup2 |
| `math1` | `CHOP:math` | range1 | torange1=-240; torange2=60; exporttable=math1_export |
| `math1_export` | `DAT:table` | — |  |
| `math_export1` | `DAT:table` | — | rows=1 |
| `multiply2` | `TOP:multiply` | lookup3, circle1 | legacyxform=on |
| `out1` | `CHOP:out` | val | exportmethod=autoname |
| `over2` | `TOP:over` | rectangle, multiply2 | r=-160; legacyxform=on |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=`''`; whileon=on; valuechange=on |
| `pattern1` | `CHOP:pattern` | — | wavetype=ramp; length=2; torange1=`[parent().par.Arcbaser,parent().par.Arcbaseg,parent().par.Arcbaseb][me.chanIndex]`; torange2=`[parent().par.Arcdispr,parent().par.Arcdispg,parent().par.Arcdispb][me.chanIndex]`; channelname=r g b; left=hold; right=hold |
| `pattern2` | `CHOP:pattern` | — | wavetype=ramp; length=400 |
| `ramp3` | `TOP:ramp` | — | dat=eval1; color1=0; color2=0; color3=0; color4=1; type=radial; phase=`parent().par.Arcanglelow/360`; extendright=repeat; …(+2) |
| `ramp3_keys` | `DAT:table` | — | rows=5; cols=5 |
| `range1` | `CHOP:math` | val1 | fromrange1=`parent().par.Rangelow1`; fromrange2=`parent().par.Rangehigh1`; exporttable=math_export1 |
| `rectangle` | `TOP:rectangle` | — | sizex=`parent().par.Arcdiameter*.5 + me.par.sizey/2`; sizey=`parent().par.Handlesize`; sizeunit=pixels; centerx=`me.par.sizex/2 - me.par.sizey/2`; centery=0; centerunit=pixels; fillcolorr=`parent().par.Arcbaser`; fillcolorg=`parent().par.Arcbaseg`; …(+4) |
| `script1_callbacks` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `text` | `COMP:text` | — | text=`parent().par.Label`; callbacks=text1_callbacks; legacyfontselection=on; fontsize=20; fontcolorr=0.6; fontcolorg=0.6; fontcolorb=0.6; textoffsetx=0; …(+11) |
| `transformy` | `TOP:transform` | over2 | tx=0; ty=0; tunit=pixels |
| `val` | `CHOP:par` | — | parameters=Value1; builtin=on; renameto=`parent().par.Name1` |
| `val1` | `CHOP:par` | — | parameters=Value1 Default1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/low/threshold` — 29 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `circle1` | `TOP:circle` | — | radiusx=`parent().par.Arcdiameter*.5*2`; radiusy=`me.par.radiusx`; radiusunit=pixels; fillcolorr=0; fillcolorg=0; fillcolorb=0; fillalpha=0; borderr=1; …(+10) |
| `eval1` | `DAT:eval` | ramp3_keys | rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `field` | `COMP:text` | — | text=`parent().par.Value1`; type=float; formatting=number; editmode=editable; legacyfontselection=on; fontsize=20; fontcolorr=0.65; fontcolorg=0.65; …(+23) |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:null` | interior |  |
| `interior` | `TOP:comp` | transformy | tops=`'transformy' if (parent().par.Interiortop==None) else parent().par.Interiortop`; selectinput=`int((parent().par.Interiortop==None))`; operand=over; size=input1; px=0; py=0; legacyxform=on; inputfiltertype=mipmap |
| `logic1` | `CHOP:logic` | pattern2 | convert=bound; boundmin=0.3; boundmax=0.5 |
| `lookup2` | `CHOP:lookup` | logic1, pattern1 |  |
| `lookup3` | `TOP:lookup` | ramp3 | method=chop; darkuv1=0; darkuv2=0; lightuv1=1; lightuv2=0; chop=lookup2 |
| `math1` | `CHOP:math` | range1 | torange1=-240; torange2=60; exporttable=math1_export |
| `math1_export` | `DAT:table` | — |  |
| `math_export1` | `DAT:table` | — | rows=1 |
| `multiply2` | `TOP:multiply` | lookup3, circle1 | legacyxform=on |
| `out1` | `CHOP:out` | val | exportmethod=autoname |
| `over2` | `TOP:over` | rectangle, multiply2 | r=-160; legacyxform=on |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=`''`; whileon=on; valuechange=on |
| `pattern1` | `CHOP:pattern` | — | wavetype=ramp; length=2; torange1=`[parent().par.Arcbaser,parent().par.Arcbaseg,parent().par.Arcbaseb][me.chanIndex]`; torange2=`[parent().par.Arcdispr,parent().par.Arcdispg,parent().par.Arcdispb][me.chanIndex]`; channelname=r g b; left=hold; right=hold |
| `pattern2` | `CHOP:pattern` | — | wavetype=ramp; length=400 |
| `ramp3` | `TOP:ramp` | — | dat=eval1; color1=0; color2=0; color3=0; color4=1; type=radial; phase=`parent().par.Arcanglelow/360`; extendright=repeat; …(+2) |
| `ramp3_keys` | `DAT:table` | — | rows=5; cols=5 |
| `range1` | `CHOP:math` | val1 | fromrange1=`parent().par.Rangelow1`; fromrange2=`parent().par.Rangehigh1`; exporttable=math_export1 |
| `rectangle` | `TOP:rectangle` | — | sizex=`parent().par.Arcdiameter*.5 + me.par.sizey/2`; sizey=`parent().par.Handlesize`; sizeunit=pixels; centerx=`me.par.sizex/2 - me.par.sizey/2`; centery=0; centerunit=pixels; fillcolorr=`parent().par.Arcbaser`; fillcolorg=`parent().par.Arcbaseg`; …(+4) |
| `script1_callbacks` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `text` | `COMP:text` | — | text=`parent().par.Label`; callbacks=text1_callbacks; legacyfontselection=on; fontsize=20; fontcolorr=0.6; fontcolorg=0.6; fontcolorb=0.6; textoffsetx=0; …(+11) |
| `transformy` | `TOP:transform` | over2 | tx=0; ty=0; tunit=pixels |
| `val` | `CHOP:par` | — | parameters=Value1; builtin=on; renameto=`parent().par.Name1` |
| `val1` | `CHOP:par` | — | parameters=Value1 Default1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/mid` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `map` | `COMP:button` | — | buttontype=momentary; scaletofit=onlyshrink; x=333; w=30; h=25; layer=3; alignorder=6; display=`parent(2).par.Galmap`; …(+26) |

### `/project1/audioAnalysis/rythm` — 2 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `map` | `COMP:button` | — | buttontype=momentary; scaletofit=onlyshrink; x=333; w=30; h=25; layer=3; alignorder=6; display=`parent(2).par.Galmap`; …(+26) |
| `threshold` | `COMP:slider` | — | slidertype=slideruv; clampul=off; clampuh=off; clampvl=off; clampvh=off; w=45; vmode=fill; bottomanchor=0.5; …(+25) |

### `/project1/audioAnalysis/showMap` — 14 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:constant` | — | colorr=0; colorg=0; colorb=0; alpha=0; combineinput=res; resolutionw=2; resolutionh=2 |
| `indicator` | `COMP:container` | — | x=0; w=`parent().height * me.par.Iconaspect`; layer=2; vmode=fill; display=`int(parent().par.Indicator!='text' or (parent().par.Behavior=='pulse' and parent().par.Labellocate!='none'))`; clickthrough=on; top=`me.par.Iconswitch.eval()`; borderaalpha=0; …(+8) |
| `iparBinary` | `COMP:base` | — | Flip=`not ((parent().par.Horizvert == 'horizontal') or ((parent().par.Horizvert == 'auto') and (parent().width >= parent().height)))` |
| `label` | `COMP:text` | — | text=`parent().par.Label if parent().par.Labellocate!='none' else ''`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=0.5; fontcolorg=0.5; fontcolorb=0.5; …(+25) |
| `out1` | `CHOP:out` | val |  |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=`''`; ontooff=on; valuechange=on |
| `parexec1` | `DAT:parexec` | — | fromop=`me.parent()`; op=..; pars=Value1; onpulse=off; builtin=off |
| `script1_callbacks` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `text` | `COMP:text` | — | text=`[parent().par.Textoff, parent().par.Texton, parent().par.Textoff, parent().par.Texton][int(math.ceil(op('tstate')[0]))]`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=`[.65,.8][parent().panel.select]`; fontcolorg=`me.par.fontcolorr`; fontcolorb=`me.par.fontcolorr`; …(+31) |
| `tstate` | `CHOP:express` | val | expr0expr=`me.inputVal + 2 * parent().panel.rollover` |
| `val` | `CHOP:par` | — | parameters=Value1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/showMap/indicator` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `dotoff` | `TOP:circle` | — | radiusx=0.34; radiusy=0.34; fillcolorr=`[parent(2).par.Bgcolorr, .7][int(parent(2).par.Value1.eval())]`; fillcolorg=`[parent(2).par.Bgcolorg, .7][int(parent(2).par.Value1.eval())]`; fillcolorb=`[parent(2).par.Bgcolorb, .7][int(parent(2).par.Value1.eval())]`; borderr=0.7; borderg=0.7; borderb=0.7; …(+7) |

### `/project1/audioAnalysis/showMap/text` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `textback` | `TOP:rectangle` | — | sizex=`parent().width-2`; sizey=`parent().height-2`; sizeunit=pixels; fillcolorr=0.23; fillcolorg=0.23; fillcolorb=0.23; fillalpha=`parent.binary.par.Behavior=='pulse'`; cornerradius=0.33; …(+6) |

### `/project1/audioAnalysis/smsd` — 3 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `active` | `COMP:button` | — | buttontype=momentary; scaletofit=onlyshrink; w=27; h=25; vmode=anchors; bottomanchor=0.2; topanchor=0.8; bgcolorr=`me.par.Bgcolorr`; …(+30) |
| `map` | `COMP:button` | — | buttontype=momentary; scaletofit=onlyshrink; x=333; w=30; h=25; layer=3; alignorder=6; display=`parent(2).par.Galmap`; …(+26) |
| `output` | `COMP:slider` | — | slidertype=slideru; zonel=0; zoner=1; clampul=off; clampuh=off; x=188; y=10; w=105; …(+24) |

### `/project1/audioAnalysis/smsd/active` — 14 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:constant` | — | colorr=0; colorg=0; colorb=0; alpha=0; combineinput=res; resolutionw=2; resolutionh=2 |
| `indicator` | `COMP:container` | — | x=0; w=`parent().height * me.par.Iconaspect`; layer=2; vmode=fill; display=`int(parent().par.Indicator!='text' or (parent().par.Behavior=='pulse' and parent().par.Labellocate!='none'))`; clickthrough=on; top=`me.par.Iconswitch.eval()`; borderaalpha=0; …(+8) |
| `iparBinary` | `COMP:base` | — | Flip=`not ((parent().par.Horizvert == 'horizontal') or ((parent().par.Horizvert == 'auto') and (parent().width >= parent().height)))` |
| `label` | `COMP:text` | — | text=`parent().par.Label if parent().par.Labellocate!='none' else ''`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=0.5; fontcolorg=0.5; fontcolorb=0.5; …(+25) |
| `out1` | `CHOP:out` | val |  |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=`''`; ontooff=on; valuechange=on |
| `parexec1` | `DAT:parexec` | — | fromop=`me.parent()`; op=..; pars=Value1; onpulse=off; builtin=off |
| `script1_callbacks` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `text` | `COMP:text` | — | text=`[parent().par.Textoff, parent().par.Texton, parent().par.Textoff, parent().par.Texton][int(math.ceil(op('tstate')[0]))]`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=`[.65,.8][parent().panel.select]`; fontcolorg=`me.par.fontcolorr`; fontcolorb=`me.par.fontcolorr`; …(+31) |
| `tstate` | `CHOP:express` | val | expr0expr=`me.inputVal + 2 * parent().panel.rollover` |
| `val` | `CHOP:par` | — | parameters=Value1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/smsd/active/indicator` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `dotoff` | `TOP:circle` | — | radiusx=0.34; radiusy=0.34; fillcolorr=`[parent(2).par.Bgcolorr, .7][int(parent(2).par.Value1.eval())]`; fillcolorg=`[parent(2).par.Bgcolorg, .7][int(parent(2).par.Value1.eval())]`; fillcolorb=`[parent(2).par.Bgcolorb, .7][int(parent(2).par.Value1.eval())]`; borderr=0.7; borderg=0.7; borderb=0.7; …(+7) |

### `/project1/audioAnalysis/smsd/active/text` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `textback` | `TOP:rectangle` | — | sizex=`parent().width-2`; sizey=`parent().height-2`; sizeunit=pixels; fillcolorr=0.23; fillcolorg=0.23; fillcolorb=0.23; fillalpha=`parent.binary.par.Behavior=='pulse'`; cornerradius=0.33; …(+6) |

### `/project1/audioAnalysis/smsd/map` — 14 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:constant` | — | colorr=0; colorg=0; colorb=0; alpha=0; combineinput=res; resolutionw=2; resolutionh=2 |
| `indicator` | `COMP:container` | — | x=0; w=`parent().height * me.par.Iconaspect`; layer=2; vmode=fill; display=`int(parent().par.Indicator!='text' or (parent().par.Behavior=='pulse' and parent().par.Labellocate!='none'))`; clickthrough=on; top=`me.par.Iconswitch.eval()`; borderaalpha=0; …(+8) |
| `iparBinary` | `COMP:base` | — | Flip=`not ((parent().par.Horizvert == 'horizontal') or ((parent().par.Horizvert == 'auto') and (parent().width >= parent().height)))` |
| `label` | `COMP:text` | — | text=`parent().par.Label if parent().par.Labellocate!='none' else ''`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=0.5; fontcolorg=0.5; fontcolorb=0.5; …(+25) |
| `out1` | `CHOP:out` | val |  |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=`''`; ontooff=on; valuechange=on |
| `parexec1` | `DAT:parexec` | — | fromop=`me.parent()`; op=..; pars=Value1; onpulse=off; builtin=off |
| `script1_callbacks` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `text` | `COMP:text` | — | text=`[parent().par.Textoff, parent().par.Texton, parent().par.Textoff, parent().par.Texton][int(math.ceil(op('tstate')[0]))]`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=`[.65,.8][parent().panel.select]`; fontcolorg=`me.par.fontcolorr`; fontcolorb=`me.par.fontcolorr`; …(+31) |
| `tstate` | `CHOP:express` | val | expr0expr=`me.inputVal + 2 * parent().panel.rollover` |
| `val` | `CHOP:par` | — | parameters=Value1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/smsd/map/indicator` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `dotoff` | `TOP:circle` | — | radiusx=0.34; radiusy=0.34; fillcolorr=`[parent(2).par.Bgcolorr, .7][int(parent(2).par.Value1.eval())]`; fillcolorg=`[parent(2).par.Bgcolorg, .7][int(parent(2).par.Value1.eval())]`; fillcolorb=`[parent(2).par.Bgcolorb, .7][int(parent(2).par.Value1.eval())]`; borderr=0.7; borderg=0.7; borderb=0.7; …(+7) |

### `/project1/audioAnalysis/smsd/map/text` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `textback` | `TOP:rectangle` | — | sizex=`parent().width-2`; sizey=`parent().height-2`; sizeunit=pixels; fillcolorr=0.23; fillcolorg=0.23; fillcolorb=0.23; fillalpha=`parent.binary.par.Behavior=='pulse'`; cornerradius=0.33; …(+6) |

### `/project1/audioAnalysis/smsd/output` — 21 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `deadZone` | `COMP:container` | — | w=`[(parent().par.Borderlow-8), parent().width][ipar.Slider.Flip]`; h=`[parent().height, (parent().par.Borderlow-8)][ipar.Slider.Flip]`; display=`parent().par.Labellocate=='left'`; enable=off; topsmoothness=mipmap; borderover=off |
| `delete` | `CHOP:delete` | val | delchannels=on; discard=nonscoped; select=bynum; delscope=Default1; selnumbers=0; legacypattern=on |
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `field` | `COMP:text` | — | text=`parent().par.Value1`; type=float; formatting=number; precision=`parent().par.Decimals`; thousandsseparator=space; editmode=`['locked', 'editable'][parent().par.Numfield == 'edit']`; legacyfontselection=on; fontsize=`op('label').par.fontsize`; …(+28) |
| `flip` | `TOP:flip` | ramp | flipy=on; flop=bottomleft; filtertype=mipmap; npasses=`ipar.Slider.Flip` |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:null` | interior |  |
| `indicator` | `COMP:container` | — | x=`[(op('math')['v1'] - parent().par.Handlesize/2), 2][ipar.Slider.Flip]`; y=`[2, (op('math')['v1'] - parent().par.Handlesize/2)][ipar.Slider.Flip]`; w=`[parent().par.Handlesize, parent().width-4][ipar.Slider.Flip]`; h=`[parent().height-4, parent().par.Handlesize][ipar.Slider.Flip]`; layer=2; clickthrough=on; bgcolorr=0.7; bgcolorg=0.7; …(+3) |
| `interior` | `TOP:comp` | flip | tops=`'flip' if (parent().par.Interiortop==None) else parent().par.Interiortop`; selectinput=`int((parent().par.Interiortop==None))`; operand=over; size=input1; tx=0; ty=0; sx=1; sy=1; …(+6) |
| `iparSlider` | `COMP:base` | — | Flip=`not ((parent().par.Horizvert == 'horizontal') or ((parent().par.Horizvert == 'auto') and (parent().width >= parent().height)))` |
| `label` | `COMP:text` | — | text=`parent().par.Label if parent().par.Labellocate!='none' else ''`; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=0.5; fontcolorg=0.5; fontcolorb=0.5; textoffsetx=0; …(+19) |
| `math` | `CHOP:math` | val | fromrange1=`parent().par.Rangelow1`; fromrange2=`parent().par.Rangehigh1`; torange1=0; torange2=1 |
| `offsetbar` | `COMP:container` | — | x=`[min(op('math')[0], op('math')[1]), parent().width-4][ipar.Slider.Flip]`; y=`[2, min(op('math')[0], op('math')[1])][ipar.Slider.Flip]`; w=`[abs(op('math')[0] - op('math')[1]), 2][ipar.Slider.Flip]`; h=`[2, abs(op('math')[0] - op('math')[1])][ipar.Slider.Flip]`; layer=1; display=`parent().par.Showoffset`; clickthrough=on; bgcolorr=0.06666; …(+4) |
| `out1` | `CHOP:out` | delete | exportmethod=autoname |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=u; offtoon=off; valuechange=on |
| `ramp` | `TOP:ramp` | — | dat=`'ramp_keys'`; color1=0.17; color2=0.17; color3=0.17; color4=1; interpnotches=step; resolutionw=1000; resolutionh=1; …(+3) |
| `ramp_keys` | `DAT:script` | — | callbacks=script1_callbacks1 |
| `script1_callbacks` | `DAT:text` | — |  |
| `script1_callbacks1` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `val` | `CHOP:par` | — | parameters=Value1 Default1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/snare` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `map` | `COMP:button` | — | buttontype=momentary; scaletofit=onlyshrink; x=333; w=30; h=25; layer=3; alignorder=6; display=`parent(2).par.Galmap`; …(+26) |

### `/project1/audioAnalysis/spectralCentroid` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `map` | `COMP:button` | — | buttontype=momentary; scaletofit=onlyshrink; x=333; w=30; h=25; layer=3; alignorder=6; display=`parent(2).par.Galmap`; …(+26) |

### `/project1/audioAnalysis/unmap` — 14 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `extend` | `COMP:base` | — | clone=`\"\" if parent().par.Extend==None else parent().par.Extend` |
| `help` | `DAT:eval` | — | expr=parent().par.Help; rowindexend=`me.inputs[0].numRows - 1`; colindexend=`me.inputs[0].numCols - 1` |
| `icon` | `TOP:constant` | — | colorr=0; colorg=0; colorb=0; alpha=0; combineinput=res; resolutionw=2; resolutionh=2 |
| `indicator` | `COMP:container` | — | x=0; w=`parent().height * me.par.Iconaspect`; layer=2; vmode=fill; display=`int(parent().par.Indicator!='text' or (parent().par.Behavior=='pulse' and parent().par.Labellocate!='none'))`; clickthrough=on; top=`me.par.Iconswitch.eval()`; borderaalpha=0; …(+8) |
| `iparBinary` | `COMP:base` | — | Flip=`not ((parent().par.Horizvert == 'horizontal') or ((parent().par.Horizvert == 'auto') and (parent().width >= parent().height)))` |
| `label` | `COMP:text` | — | text=`parent().par.Label if parent().par.Labellocate!='none' else ''`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=0.5; fontcolorg=0.5; fontcolorb=0.5; …(+25) |
| `out1` | `CHOP:out` | val |  |
| `panelexec` | `DAT:panelexec` | — | fromop=`me.parent()`; panels=`me.parent()`; panelvalue=`''`; ontooff=on; valuechange=on |
| `parexec1` | `DAT:parexec` | — | fromop=`me.parent()`; op=..; pars=Value1; onpulse=off; builtin=off |
| `script1_callbacks` | `DAT:text` | — |  |
| `script_export` | `DAT:script` | — | callbacks=script1_callbacks |
| `text` | `COMP:text` | — | text=`[parent().par.Textoff, parent().par.Texton, parent().par.Textoff, parent().par.Texton][int(math.ceil(op('tstate')[0]))]`; type=multiline; legacyfontselection=on; scaletofit=onlyshrink; fontsize=20; fontcolorr=`[.65,.8][parent().panel.select]`; fontcolorg=`me.par.fontcolorr`; fontcolorb=`me.par.fontcolorr`; …(+31) |
| `tstate` | `CHOP:express` | val | expr0expr=`me.inputVal + 2 * parent().panel.rollover` |
| `val` | `CHOP:par` | — | parameters=Value1; builtin=on; renameto=`parent().par.Name1` |

### `/project1/audioAnalysis/unmap/indicator` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `dotoff` | `TOP:circle` | — | radiusx=0.34; radiusy=0.34; fillcolorr=`[parent(2).par.Bgcolorr, .7][int(parent(2).par.Value1.eval())]`; fillcolorg=`[parent(2).par.Bgcolorg, .7][int(parent(2).par.Value1.eval())]`; fillcolorb=`[parent(2).par.Bgcolorb, .7][int(parent(2).par.Value1.eval())]`; borderr=0.7; borderg=0.7; borderb=0.7; …(+7) |

### `/project1/audioAnalysis/unmap/text` — 1 нод

| Нода | Тип | Входы | Параметры / выражения |
|---|---|---|---|
| `textback` | `TOP:rectangle` | — | sizex=`parent().width-2`; sizey=`parent().height-2`; sizeunit=pixels; fillcolorr=0.23; fillcolorg=0.23; fillcolorb=0.23; fillalpha=`parent.binary.par.Behavior=='pulse'`; cornerradius=0.33; …(+6) |

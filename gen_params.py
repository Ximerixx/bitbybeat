#!/usr/bin/env python3
"""Generate md_plans/07_params_defaults.md — custom parameter defaults per component."""
import json
m = json.load(open("toe_model.json", encoding="utf-8"))

# TD convention: custom parameters start with an uppercase letter.
def custom_pars(path):
    r = m.get(path, {})
    out = []
    for k, v in (r.get("pars") or {}).items():
        if k and k[0].isupper():
            out.append((k, v))
    return out

COMPS = [
    ("/project1/audioAnalysis", "AudioAnalysis (компонент анализа)"),
    ("/project1/Low_Base", "Low_Base"),
    ("/project1/Mid_Base", "Mid_Base"),
    ("/project1/High_Mid", "High_Mid"),
    ("/project1/Jumper_trigger", "Jumper_trigger"),
    ("/project1/Zig_Zagger", "Zig_Zagger"),
]

# key DSP / control nodes and their numeric params of interest
CTRL = [
    "/project1/math1", "/project1/math2", "/project1/lagClearRythmIn_",
    "/project1/lowControlGain", "/project1/midControlGain",
    "/project1/highControlGain", "/project1/highControlGain1",
    "/project1/kickControlThreshold_math", "/project1/kickControlThreshold",
    "/project1/snareControlThreshold_math", "/project1/snareControlThreshold",
    "/project1/rythmControlThreshold",
    "/project1/audiodyna1",
    "/project1/audioAnalysis/audiofilter1", "/project1/audioAnalysis/audiofilter2",
    "/project1/audioAnalysis/audiofilter3", "/project1/audioAnalysis/trigger1",
    "/project1/OSC_out", "/project1/audiodevin1",
]

L = ["# 07 — Дефолты параметров (из дампа)\n",
     "> Значения — кэш `eval()` из `*.parm` эталона. Выражения в обратных кавычках.\n"]

for path, title in COMPS:
    cps = custom_pars(path)
    if not cps:
        continue
    L.append(f"## {title} — `{path}`\n")
    L.append("| Параметр | Значение | Выражение |")
    L.append("|---|---|---|")
    for k, v in cps:
        expr = ("`" + v["expr"] + "`") if "expr" in v else ""
        L.append(f"| `{k}` | {v['val']} | {expr} |")
    L.append("")

L.append("## DSP / control ноды (числовые параметры)\n")
L.append("| Нода | Тип | Параметры |")
L.append("|---|---|---|")
for path in CTRL:
    r = m.get(path)
    if not r:
        L.append(f"| `{path}` | — | (нет в дампе) |")
        continue
    ps = []
    for k, v in (r.get("pars") or {}).items():
        if k in ("pageindex", "autoexportroot", "defaultreadencoding"):
            continue
        s = f"{k}={v['val']}"
        if "expr" in v:
            s = f"{k}=`{v['expr']}`"
        ps.append(s)
    typ = f"{r.get('family','')}:{r.get('type','')}"
    L.append(f"| `{r['name']}` | `{typ}` | {'; '.join(ps)} |".replace("\n", " "))
L.append("")

open("md_plans/07_params_defaults.md", "w", encoding="utf-8").write("\n".join(L))
print("wrote md_plans/07_params_defaults.md")

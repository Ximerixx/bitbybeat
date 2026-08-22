#!/usr/bin/env python3
"""Generate md_plans/08_node_census.md: full node inventory from toe_model.json."""
import json
from collections import Counter, defaultdict

m = json.load(open("toe_model.json", encoding="utf-8"))

def parent(p):
    return p.rsplit("/", 1)[0] or "/"

# skip these noisy boilerplate param keys in the per-node summary
SKIP = {"pageindex", "autoexportroot", "defaultreadencoding", "tile"}

def par_summary(r, limit=8):
    out = []
    for k, v in (r.get("pars") or {}).items():
        if k in SKIP:
            continue
        s = f"{k}={v['val']}"
        if "expr" in v:
            s = f"{k}=`{v['expr']}`"
        out.append(s)
    if len(out) > limit:
        out = out[:limit] + [f"…(+{len(out)-limit})"]
    return "; ".join(out)

by_parent = defaultdict(list)
for p, r in m.items():
    by_parent[parent(p)].append(r)

types = Counter((v.get("family") or "?") + ":" + (v.get("type") or "?") for v in m.values())

lines = []
lines.append("# 08 — Полная перепись нод (авто-census)\n")
lines.append("> Источник истины: `Analysis 2.2_v6_calibrated.9.toe.dir` (toeexpand).")
lines.append("> Сгенерировано `gen_census.py` из `toe_model.json`. Всего нод: **%d**.\n" % len(m))

lines.append("## Сводка по типам (family:type)\n")
lines.append("| Кол-во | Тип |")
lines.append("|---:|---|")
for t, c in types.most_common():
    lines.append(f"| {c} | `{t}` |")
lines.append("")

lines.append("## Перепись по контейнерам\n")
for cont in sorted(by_parent):
    nodes = by_parent[cont]
    lines.append(f"\n### `{cont or '/'}` — {len(nodes)} нод\n")
    lines.append("| Нода | Тип | Входы | Параметры / выражения |")
    lines.append("|---|---|---|---|")
    for r in sorted(nodes, key=lambda x: x["name"]):
        ins = ", ".join(r["inputs"]) if r["inputs"] else "—"
        ps = par_summary(r).replace("|", "\\|")
        typ = f"{r.get('family','')}:{r.get('type','')}"
        lines.append(f"| `{r['name']}` | `{typ}` | {ins} | {ps} |")
lines.append("")

open("md_plans/08_node_census.md", "w", encoding="utf-8").write("\n".join(lines))
print("wrote md_plans/08_node_census.md", len(lines), "lines")

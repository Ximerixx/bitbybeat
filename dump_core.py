#!/usr/bin/env python3
"""Readable dump of the functional core (CHOP/DAT) grouped by container."""
import json
import sys
from collections import defaultdict

model = json.load(open("toe_model.json", encoding="utf-8"))

def parent(path):
    return path.rsplit("/", 1)[0] or "/"

# focus families
FOCUS = {"CHOP", "DAT"}

groups = defaultdict(list)
for p, r in model.items():
    fam = r.get("family")
    if fam in FOCUS:
        groups[parent(p)].append(r)

# arg: optional substring filter on container path
flt = sys.argv[1] if len(sys.argv) > 1 else ""

def fmt_pars(r):
    out = []
    for k, v in (r.get("pars") or {}).items():
        if k in ("pageindex",):
            continue
        s = f"{k}={v['val']}"
        if "expr" in v:
            s += f"  ::expr:: {v['expr']}"
        out.append(s)
    return out

for cont in sorted(groups):
    if flt and flt not in cont:
        continue
    nodes = groups[cont]
    print(f"\n############ {cont or '/'}  ({len(nodes)} chop/dat) ############")
    for r in sorted(nodes, key=lambda x: x["name"]):
        ins = ", ".join(r["inputs"]) if r["inputs"] else "-"
        exps = (" exports:[" + ",".join(r["exports"]) + "]") if r["exports"] else ""
        print(f"  {r['name']}  <{r['type']}>  in:[{ins}]{exps}")
        for pk in fmt_pars(r):
            print(f"       {pk}")
        for ext in ("text", "table", "logic"):
            if ext in r:
                t = r[ext]
                if len(t) > 600:
                    t = t[:600] + " …[truncated]"
                pref = "       %s| " % ext
                print(pref + t.replace("\n", "\n" + pref))

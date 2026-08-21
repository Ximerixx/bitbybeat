import json

root = op('/project1')
out = {}

for o in root.findChildren(depth=10):
    rec = {"type": o.type, "path": o.path,
           "inputs": [c.path for c in o.inputs],
           "outputs": [c.path for c in o.outputs],
           "pars": {}}
    for p in o.pars():
        # сохраняем И значение, И режим, И выражение/экспорт
        entry = {"val": str(p.eval()), "mode": p.mode.name}
        if p.mode.name == "EXPRESSION":
            entry["expr"] = p.expr
        elif p.mode.name == "EXPORT":
            entry["export"] = p.exportSource.path if p.exportSource else None
        rec["pars"][p.name] = entry
    # текст DAT-нод (callbacks, tables, ramp keys)
    if hasattr(o, "text"):
        try: rec["text"] = o.text
        except: pass
    if o.type in ("table",) and hasattr(o, "numRows"):
        try: rec["table"] = [[o[r,c].val for c in range(o.numCols)] for r in range(o.numRows)]
        except: pass
    out[o.path] = rec

with open("project1_dump_full.json", "w", encoding="utf-8") as f:
    json.dump(out, f, ensure_ascii=False, indent=2)

print("saved", len(out), "nodes")
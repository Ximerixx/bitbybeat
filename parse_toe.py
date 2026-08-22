#!/usr/bin/env python3
"""Parse a toeexpand .toe.dir tree into a structured node model.

Source of truth: the expanded TouchDesigner project directory.
Emits toe_model.json with per-node: family, type, path, inputs, exports,
parameters (value + expression when present), and attached DAT text/table/logic.
"""
import json
import os
import re
import sys

ROOT = "Analysis 2.2_v6_calibrated.9.toe.dir"

# param line format in .parm:  name mode value [expr...]
# mode is an integer bitfield. When a value is followed by an expression,
# the expression is the remainder of the line (may be quoted).
PARM_RE = re.compile(r'^(\S+)\s+(\d+)\s+(.*)$')


def strip_quotes(s):
    s = s.strip()
    if len(s) >= 2 and s[0] == '"' and s[-1] == '"':
        return s[1:-1]
    return s


def parse_parm(path):
    parms = {}
    try:
        with open(path, encoding="utf-8", errors="replace") as f:
            lines = f.read().splitlines()
    except FileNotFoundError:
        return parms
    for ln in lines:
        if not ln.strip() or ln.strip() == "?":
            continue
        m = PARM_RE.match(ln)
        if not m:
            continue
        name, mode, rest = m.group(1), int(m.group(2)), m.group(3)
        rest = rest.strip()
        # rest is either "value" or "value expr". Value is first token (maybe quoted).
        # Heuristic: value is first whitespace-delimited token unless quoted.
        val = rest
        expr = None
        if rest.startswith('"'):
            # quoted value
            end = rest.find('"', 1)
            if end != -1:
                val = rest[1:end]
                expr = rest[end + 1:].strip() or None
        else:
            parts = rest.split(None, 1)
            val = parts[0] if parts else ""
            expr = parts[1].strip() if len(parts) > 1 else None
        entry = {"val": val, "mode": mode}
        if expr:
            entry["expr"] = strip_quotes(expr) if expr.startswith('"') else expr
        parms[name] = entry
    return parms


def parse_n(path):
    """Parse a .n node file -> family, type, inputs list, exports list, tile."""
    info = {"family": None, "type": None, "inputs": [], "exports": [], "tile": None}
    try:
        with open(path, encoding="utf-8", errors="replace") as f:
            lines = f.read().splitlines()
    except FileNotFoundError:
        return info
    if not lines:
        return info
    first = lines[0].strip()
    if ":" in first:
        fam, typ = first.split(":", 1)
        info["family"], info["type"] = fam, typ
    else:
        info["type"] = first
    section = None
    for ln in lines[1:]:
        s = ln.strip()
        if s.startswith("tile "):
            info["tile"] = s[5:].strip()
            continue
        if s == "inputs":
            section = "inputs"
            continue
        if s == "exports":
            section = "exports"
            continue
        if s == "{":
            continue
        if s == "}":
            section = None
            continue
        if section == "inputs":
            # "0 \t nodename"
            parts = s.split(None, 1)
            if len(parts) == 2 and parts[0].isdigit():
                info["inputs"].append(parts[1].strip())
            elif parts and parts[0] not in ("*",):
                info["inputs"].append(s)
        elif section == "exports":
            if s and s != "*":
                info["exports"].append(s)
    return info


def read_text(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as f:
            return f.read()
    except FileNotFoundError:
        return None


def main():
    base = os.path.join(os.getcwd(), ROOT)
    model = {}
    for dirpath, dirnames, filenames in os.walk(base):
        for fn in filenames:
            if not fn.endswith(".n"):
                continue
            stem = fn[:-2]
            full = os.path.join(dirpath, fn)
            rel = os.path.relpath(full, base)
            node_path = "/" + os.path.dirname(rel).replace(os.sep, "/")
            if node_path == "/.":
                node_path = ""
            name = stem
            path = (node_path + "/" + name).replace("//", "/")
            n = parse_n(full)
            rec = {
                "path": path,
                "name": name,
                "family": n["family"],
                "type": n["type"],
                "tile": n["tile"],
                "inputs": n["inputs"],
                "exports": n["exports"],
            }
            pf = os.path.join(dirpath, stem + ".parm")
            if os.path.exists(pf):
                rec["pars"] = parse_parm(pf)
            for ext in ("text", "table", "logic"):
                ef = os.path.join(dirpath, stem + "." + ext)
                if os.path.exists(ef):
                    t = read_text(ef)
                    if t is not None and t.strip():
                        rec[ext] = t
            model[path] = rec

    with open("toe_model.json", "w", encoding="utf-8") as f:
        json.dump(model, f, ensure_ascii=False, indent=1)

    # stats
    from collections import Counter
    types = Counter((v.get("family") or "?") + ":" + (v.get("type") or "?") for v in model.values())
    print("nodes:", len(model))
    print("top types:")
    for t, c in types.most_common(40):
        print(f"  {c:4d}  {t}")


if __name__ == "__main__":
    main()

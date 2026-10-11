"""Fail if the top-level modules of src/ form a dependency cycle (layering guard)."""

import re
from graphlib import CycleError, TopologicalSorter
from pathlib import Path

SRC = Path("src")
graph: dict[str, set[str]] = {}
for p in SRC.rglob("*.rs"):
    rel = p.relative_to(SRC).parts
    top = rel[0].removesuffix(".rs")
    if top == "main":
        continue
    text = re.sub(
        r"#\[cfg\(test\)\]\s*mod tests.*", "", p.read_text(encoding="utf-8"), flags=re.S
    )
    deps = {d for d in re.findall(r"crate::(\w+)", text) if d != top}
    graph.setdefault(top, set()).update(deps)

for k in sorted(graph):
    print(f"{k:12} -> {', '.join(sorted(graph[k])) or '-'}")
try:
    print("order:", list(TopologicalSorter(graph).static_order()))
except CycleError as e:
    print("CYCLE:", e.args[1])
    raise SystemExit(1)

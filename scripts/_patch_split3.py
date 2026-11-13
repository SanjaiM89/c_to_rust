p = "scripts/20_analyze.py"
s = open(p).read()

old = '''    def order_within(area: str, members: list[str]) -> list[str]:
        """Topologically order the members of one area by the internal call graph."""
        member_set = set(members)
        indeg = {m: 0 for m in members}
        adj = {m: [] for m in members}
        for m in members:
            for c in graph.get(m, ()):
                if c in member_set:
                    adj[c].append(m)      # edge c -> m (c is a callee of m)
                    indeg[m] += 1
        ready = sorted([m for m in members if indeg[m] == 0])
        out: list[str] = []
        while ready:
            m = ready.pop(0)
            out.append(m)
            for nxt in adj[m]:
                indeg[nxt] -= 1
                if indeg[nxt] == 0:
                    ready.append(nxt)
                    ready.sort()
        # any member left over is part of a cycle inside the area
        out += sorted(m for m in members if m not in out)
        return out
'''

new = '''    def order_within(area: str, members: list[str]) -> list[list[str]]:
        """Order the members of one area into leaf-first *groups*.

        A strongly connected component is atomic: its functions are mutually
        recursive and must be translated together, so a chunk boundary is
        never placed inside one. The groups are then returned in leaf-first
        order (a group's callees come before it).
        """
        member_set = set(members)
        # SCCs restricted to this area
        sccs_local = tarjan_scc(sorted(member_set), graph)
        comp_of = {n: i for i, comp in enumerate(sccs_local) for n in comp}
        # condensation edges: comp(c) -> comp(m) when m calls c
        indeg = {i: 0 for i in range(len(sccs_local))}
        adj: dict[int, list[int]] = {i: [] for i in range(len(sccs_local))}
        for m in members:
            for c in graph.get(m, ()):
                if c in member_set and comp_of[c] != comp_of[m]:
                    if comp_of[m] not in adj[comp_of[c]]:
                        adj[comp_of[c]].append(comp_of[m])
                        indeg[comp_of[m]] += 1
        ready = sorted([i for i in range(len(sccs_local)) if indeg[i] == 0])
        groups: list[list[str]] = []
        while ready:
            i = ready.pop(0)
            groups.append(sccs_local[i])
            for nxt in adj[i]:
                indeg[nxt] -= 1
                if indeg[nxt] == 0:
                    ready.append(nxt)
                    ready.sort()
        # any SCC left over participates in a cycle across SCCs (should not
        # happen, but never drop a function)
        seen = {n for g in groups for n in g}
        for comp in sccs_local:
            if comp[0] not in seen:
                groups.append(comp)
        return groups
'''
assert old in s
s = s.replace(old, new)

old2 = '''        ordered = order_within(area, members)
        chunks = [ordered[i:i + MAX_UNIT_FUNCTIONS]
                  for i in range(0, len(ordered), MAX_UNIT_FUNCTIONS)]'''
new2 = '''        groups = order_within(area, members)
        # pack the SCC groups into chunks of at most MAX_UNIT_FUNCTIONS
        chunks: list[list[str]] = []
        cur: list[str] = []
        for g in groups:
            if cur and len(cur) + len(g) > MAX_UNIT_FUNCTIONS:
                chunks.append(cur)
                cur = []
            cur.extend(g)
        if cur:
            chunks.append(cur)'''
assert old2 in s
s = s.replace(old2, new2)

open(p, "w").write(s)
print("chunker is now SCC-aware")

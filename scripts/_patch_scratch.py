import re

p = "scripts/contract.py"
s = open(p).read()
s = s.replace(
    """thread_local! {
    /// The currently installed hooks, mirroring the C `global_hooks` global.
    static GLOBAL_HOOKS: std::cell::RefCell<Hooks> =""",
    """thread_local! {
    /// The currently installed hooks, mirroring the C `global_hooks` global.
    pub static GLOBAL_HOOKS: std::cell::RefCell<Hooks> =""")
open(p, "w").write(s)
print("contract: GLOBAL_HOOKS is now public")

p2 = "scripts/30_translate.py"
t = open(p2).read()
assert "def write_scratch" not in t

helper = '''def write_scratch(crate: str, unit_code: str, sigs: dict, callees: list[str]) -> str:
    """A crate with the contract, stubs for the external callees, and the unit.

    Compiling the unit *in isolation* stops a failure in an earlier unit from
    poisoning every later unit's compile rate. The stubs have exactly the
    target signatures, so the unit is compiled against the same interface it
    will meet in the final crate.
    """
    src = os.path.join(crate, "src")
    os.makedirs(src, exist_ok=True)
    with open(os.path.join(src, "contract.rs"), "w") as fh:
        fh.write(contract_mod.CONTRACT_RS)
    with open(os.path.join(src, "stubs.rs"), "w") as fh:
        fh.write("// generated stubs for callees that live in other units\\n")
        for c in callees:
            s = sigs[c]
            args = ", ".join(f"{p['name']}: {p['rust_type']}" for p in s["params"])
            fh.write(f"pub fn {s['rust_name']}({args}) -> {s['return_type']} "
                     f"{{ unimplemented!(\\"stub for {c}\\") }}\\n")
    mod = "unit_under_test"
    with open(os.path.join(src, mod + ".rs"), "w") as fh:
        fh.write("use crate::contract::*;\\nuse crate::stubs::*;\\n\\n")
        fh.write(unit_code + "\\n")
    with open(os.path.join(src, "lib.rs"), "w") as fh:
        fh.write("pub mod contract;\\npub mod stubs;\\n")
        fh.write(f"pub mod {mod};\\n")
    with open(os.path.join(crate, "Cargo.toml"), "w") as fh:
        fh.write("[package]\\nname = \\"scratch\\"\\nversion = \\"0.1.0\\"\\nedition = \\"2021\\"\\n\\n"
                 "[lib]\\nname = \\"scratch\\"\\npath = \\"src/lib.rs\\"\\n")
    return crate


def write_crate('''
t = t.replace("def write_crate(", helper, 1)
open(p2, "w").write(t)
print("30_translate.py: write_scratch added")

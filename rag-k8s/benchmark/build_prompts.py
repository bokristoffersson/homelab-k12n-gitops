
# build_prompts.py
import csv, json
from pathlib import Path

def selector_from(app, name_key, comp):
    if app: return f"app={app}"
    if name_key: return f"app.kubernetes.io/name={name_key}"
    if comp: return f"app.kubernetes.io/component={comp}"
    return None

def load_deployments(p: Path):
    out = []
    with p.open("r", encoding="utf-8") as f:
        r = csv.reader(f, delimiter="\t")
        for row in r:
            if not row or all(not c.strip() for c in row):
                continue
            ns, name = row[0].strip(), row[1].strip()
            app   = row[2].strip() if len(row) > 2 else ""
            comp  = row[3].strip() if len(row) > 3 else ""
            nameK = row[4].strip() if len(row) > 4 else ""
            sel = selector_from(app, nameK, comp)
            if name:
                # 5 prompt cases per deployment
                out.extend([
                    {
                        "intent":"restart","resource":"deployment","namespace":ns,
                        "target":f"deploy/{name}","selector":sel,
                        "goal":f"restart {name} in {ns}",
                    },
                    {
                        "intent":"diagnose","resource":"deployment","namespace":ns,
                        "target":f"deploy/{name}","selector":sel,
                        "goal":f"check rollout status for {name} in {ns}",
                    },
                    {
                        "intent":"logs","resource":"pod","namespace":ns,
                        "target":"pod/<newestReady>","selector":sel,
                        "goal":f"fetch logs for newest Ready pod of {name} in {ns} for 15m tail 300",
                    },
                    {
                        "intent":"diagnose","resource":"deployment","namespace":ns,
                        "target":f"deploy/{name}","selector":sel,
                        "goal":f"describe {name} and recent events in {ns}",
                    },
                    {
                        "intent":"scale","resource":"deployment","namespace":ns,
                        "target":f"deploy/{name}","selector":sel,
                        "goal":f"scale {name} in {ns} to 2 replicas",
                    },
                ])
    return out

def load_statefulsets(p: Path):
    out = []
    with p.open("r", encoding="utf-8") as f:
        r = csv.reader(f, delimiter="\t")
        for row in r:
            if not row or all(not c.strip() for c in row): continue
            ns, name = row[0].strip(), row[1].strip()
            app   = row[2].strip() if len(row) > 2 else ""
            nameK = row[3].strip() if len(row) > 3 else ""
            sel = selector_from(app, nameK, None)
            if name:
                out.append({
                    "intent":"diagnose","resource":"statefulset","namespace":ns,
                    "target":f"sts/{name}","selector":sel,
                    "goal":f"show status for sts {name} in {ns} and logs for index 0",
                })
    return out

def main():
    dep = Path("deployments.tsv"); ss = Path("statefulsets.tsv")
    prompts = []
    if dep.exists(): prompts += load_deployments(dep)
    if ss.exists():  prompts += load_statefulsets(ss)
    out_path = Path("prompts.json")
    out_path.write_text(json.dumps(prompts, indent=2), encoding="utf-8")
    print(f"Wrote {len(prompts)} prompts to {out_path}")
    print("Sample:", json.dumps(prompts[:5], indent=2))

if __name__ == "__main__":
    main()

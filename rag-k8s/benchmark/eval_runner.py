
# eval_runner.py
# Dependencies: python3, (optional) mlx-lm, jq in shell when executing commands
import csv, json, re, statistics, time
from pathlib import Path

# Optional MLX import (skip gracefully if not installed)
try:
    from mlx_lm import load, generate
    HAVE_MLX = True
except Exception:
    HAVE_MLX = False

# ---------- Config ----------
MODEL_ID = "mlx-community/Llama-3.2-3B-Instruct-4bit"  # set to your local MLX model
DECODING = dict(max_tokens=160, temp=0.2, top_p=0.9, repetition_penalty=1.1)

JSON_SCHEMA_KEYS = ["intent", "namespace", "target", "command", "summary"]  # selector optional

# Regex gold checks
RESTART_RE = re.compile(r"^kubectl\\s+rollout\\s+restart\\s+deploy\\/[a-z0-9-]+\\s+-n\\s+[a-z0-9-]+(\\s+--timeout=\\d+[smh])?$")
STATUS_RE  = re.compile(r"^kubectl\\s+rollout\\s+status\\s+deploy\\/[a-z0-9-]+\\s+-n\\s+[a-z0-9-]+(\\s+--timeout=\\d+[smh])?$")
SCALE_RE   = re.compile(r"^kubectl\\s+scale\\s+deploy\\/[a-z0-9-]+\\s+-n\\s+[a-z0-9-]+\\s+--replicas=\\d+$")
LOGS_RE    = re.compile(r"^kubectl\\s+logs\\s+\\$\\(kubectl\\s+get\\s+pod\\s+-n\\s+[a-z0-9-]+\\s+-l\\s+'.+'\\s+.*\\)\\s+-n\\s+[a-z0-9-]+\\s+--since=\\d+[smh]\\s+--tail=\\d+$")
STSF_RE    = re.compile(r"^kubectl\\s+get\\s+sts\\s+[a-z0-9-]+\\s+-n\\s+[a-z0-9-]+\\s+-o\\s+wide\\s+&&\\s+kubectl\\s+logs\\s+[a-z0-9-]+-0\\s+-n\\s+[a-z0-9-]+\\s+--since=\\d+[smh]\\s+--tail=\\d+$")

def selector_from(app, name_key, comp):
    if app:       return f"app={app}"
    if name_key:  return f"app.kubernetes.io/name={name_key}"
    if comp:      return f"app.kubernetes.io/component={comp}"
    return None

def load_deployments(tsv_path):
    cases = []
    with tsv_path.open("r", encoding="utf-8") as f:
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
                cases.append(dict(ns=ns, name=name, selector=sel, kind="deployment"))
    return cases

def load_statefulsets(tsv_path):
    cases = []
    with tsv_path.open("r", encoding="utf-8") as f:
        r = csv.reader(f, delimiter="\t")
        for row in r:
            if not row or all(not c.strip() for c in row):
                continue
            ns, name = row[0].strip(), row[1].strip()
            app   = row[2].strip() if len(row) > 2 else ""
            nameK = row[3].strip() if len(row) > 3 else ""
            sel = selector_from(app, nameK, None)
            if name:
                cases.append(dict(ns=ns, name=name, selector=sel, kind="statefulset"))
    return cases

def prompts_for_case(c):
    ns, name, sel, kind = c["ns"], c["name"], c["selector"], c["kind"]
    prompts = []
    if kind == "deployment":
        # Restart
        prompts.append({
            "intent": "restart",
            "namespace": ns,
            "target": f"deploy/{name}",
            "selector": sel,
            "command": f"kubectl rollout restart deploy/{name} -n {ns} --timeout=60s",
            "summary": "Restart issued; monitor rollout status."
        })
        # Rollout status
        prompts.append({
            "intent": "diagnose",
            "namespace": ns,
            "target": f"deploy/{name}",
            "selector": sel,
            "command": f"kubectl rollout status deploy/{name} -n {ns} --timeout=120s",
            "summary": "Rollout status checked."
        })
        # Logs (newest Ready pod)
        label_for_logs = sel or f"app.kubernetes.io/name={name}"  # fallback
        prompts.append({
            "intent": "logs",
            "namespace": ns,
            "target": "pod/<newestReady>",
            "selector": sel,
            "command": f"kubectl logs $(kubectl get pod -n {ns} -l '{label_for_logs}' --sort-by=.status.startTime -o json | jq -r '.items[] | select(.status.containerStatuses[]?.ready==true) | .metadata.name' | tail -n1) -n {ns} --since=15m --tail=300",
            "summary": "Recent logs retrieved."
        })
        # Describe + events
        prompts.append({
            "intent": "diagnose",
            "namespace": ns,
            "target": f"deploy/{name}",
            "selector": sel,
            "command": f"kubectl describe deploy/{name} -n {ns} && kubectl get events -n {ns} --sort-by=.lastTimestamp | tail -n 50",
            "summary": "Deployment state and recent events printed."
        })
        # Scale → 2
        prompts.append({
            "intent": "scale",
            "namespace": ns,
            "target": f"deploy/{name}",
            "selector": sel,
            "command": f"kubectl scale deploy/{name} -n {ns} --replicas=2",
            "summary": "Scaled to 2 replicas."
        })
    else:
        # StatefulSet status + logs index 0
        prompts.append({
            "intent": "diagnose",
            "namespace": ns,
            "target": f"sts/{name}",
            "selector": sel,
            "command": f"kubectl get sts {name} -n {ns} -o wide && kubectl logs {name}-0 -n {ns} --since=10m --tail=300",
            "summary": "StatefulSet status and index-0 logs printed."
        })
    return prompts

def json_valid(d):
    return all(k in d for k in JSON_SCHEMA_KEYS) and isinstance(d["command"], str) and d["command"].startswith("kubectl")

def regex_check(cmd):
    if RESTART_RE.match(cmd) or STATUS_RE.match(cmd) or SCALE_RE.match(cmd) or LOGS_RE.match(cmd) or STSF_RE.match(cmd):
        return True
    return False

def eval_mlx(model_id, prompt_text):
    model, tok = load(model_id)
    t0 = time.perf_counter()
    out = generate(model, tok, prompt_text, **DECODING)
    ttfb = time.perf_counter() - t0
    return out, ttfb

def main():
    dep_cases = load_deployments(Path("deployments.tsv"))
    ss_cases  = load_statefulsets(Path("statefulsets.tsv"))
    cases = dep_cases + ss_cases

    # Build prompts
    all_prompts = [p for c in cases for p in prompts_for_case(c)]
    print(json.dumps({"count": len(all_prompts), "sample": all_prompts[:10]}, indent=2))

    # Optional MLX evaluation of one prompt (schema-first)
    if HAVE_MLX:
        SYSTEM = ("You output ONLY a JSON object with keys: intent, namespace, target, selector?, command, summary. "
                  "Allowed verbs: get, describe, logs, scale, rollout restart. "
                  "Always include '-n <namespace>'. Prefer 'rollout restart' for deployments.")
        sample = all_prompts[0]
        # Build planning prompt for the same goal
        planning_prompt = f"{SYSTEM}\nContext:\nselector={sample.get('selector')}\nUser goal: {sample['intent']} {sample['target']} in {sample['namespace']}\nJSON:"
        out, ttfb = eval_mlx(MODEL_ID, planning_prompt)
        # Extract JSON block
        start = out.find("{"); end = out.rfind("}") + 1
        maybe_json = out[start:end] if start >= 0 and end > start else ""
        print("\\nMLX output TTFB(s):", round(ttfb, 3))
        if maybe_json:
            try:
                obj = json.loads(maybe_json)
                print("MLX JSON valid:", json_valid(obj))
                print("Command regex OK:", regex_check(obj.get("command","")))
                print(json.dumps(obj, indent=2))
            except Exception as e:
                print("Failed to parse MLX JSON:", e)

if __name__ == "__main__":
    main()

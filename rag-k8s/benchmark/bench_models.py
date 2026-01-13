
# bench_models.py
import json, re, time, statistics, argparse, csv, sys
from pathlib import Path

try:
    import yaml
except Exception:
    yaml = None

try:
    from mlx_lm import load, generate
    from mlx_lm.sample_utils import make_sampler, make_repetition_penalty
except Exception as e:
    print("mlx_lm not available. Please install MLX-LM. Error:", e)
    sys.exit(1)

# ---------- decoding + prompts ----------
# Create sampler with temperature and top_p
sampler = make_sampler(temp=0.2, top_p=0.9)
# Create repetition penalty logits processor
repetition_penalty = make_repetition_penalty(1.1)
DECODING = dict(max_tokens=256, sampler=sampler, logits_processors=[repetition_penalty])
SYSTEM = (
"Output a JSON object with: intent, namespace, target, command, summary.\n"
"Command must be a complete kubectl command starting with 'kubectl'.\n"
"Always include '-n <namespace>' in commands.\n"
"Examples:\n"
"restart: kubectl rollout restart deploy/name -n namespace\n"
"scale: kubectl scale deploy/name -n namespace --replicas=2\n"
"logs: kubectl logs pod-name -n namespace --since=15m --tail=300\n"
"diagnose: kubectl rollout status deploy/name -n namespace\n"
)

# ---------- JSON & regex checks ----------
REQ = ["intent","namespace","target","command"]  # summary is optional

# More flexible regex patterns - allow variations in spacing and format
# Normalize whitespace before matching
def normalize_cmd(cmd: str) -> str:
    """Normalize command string for regex matching"""
    # Remove extra whitespace, normalize to single spaces
    cmd = " ".join(cmd.split())
    return cmd.strip()

RESTART_RE = re.compile(r"^kubectl\s+rollout\s+restart\s+(?:deploy|deployment)[\/\s]+[a-z0-9-]+(?:\s+-n\s+[a-z0-9-]+)?(?:\s+--timeout=\d+[smh])?\s*$", re.IGNORECASE)
STATUS_RE  = re.compile(r"^kubectl\s+rollout\s+status\s+(?:deploy|deployment)[\/\s]+[a-z0-9-]+(?:\s+-n\s+[a-z0-9-]+)?(?:\s+--timeout=\d+[smh])?\s*$", re.IGNORECASE)
SCALE_RE   = re.compile(r"^kubectl\s+scale\s+(?:deploy|deployment)[\/\s]+[a-z0-9-]+(?:\s+-n\s+[a-z0-9-]+)?\s+--replicas=\d+\s*$", re.IGNORECASE)
LOGS_RE    = re.compile(r"^kubectl\s+logs\s+.*-n\s+[a-z0-9-]+.*--since=\d+[smh].*--tail=\d+.*$", re.IGNORECASE | re.DOTALL)
STSF_RE    = re.compile(r"^kubectl\s+get\s+sts\s+[a-z0-9-]+\s+-n\s+[a-z0-9-]+\s+-o\s+wide\s+&&\s+kubectl\s+logs\s+[a-z0-9-]+-0\s+-n\s+[a-z0-9-]+\s+--since=\d+[smh]\s+--tail=\d+\s*$", re.IGNORECASE)

def regex_ok(cmd: str)->bool:
    if not cmd:
        return False
    normalized = normalize_cmd(cmd)
    return any(r.match(normalized) for r in (RESTART_RE, STATUS_RE, SCALE_RE, LOGS_RE, STSF_RE))

def extract_json(txt: str)->dict|None:
    # Remove markdown code blocks if present
    txt = txt.strip()
    if txt.startswith("```"):
        # Remove ```json or ``` markers
        lines = txt.split("\n")
        if lines[0].startswith("```"):
            lines = lines[1:]
        if lines and lines[-1].strip() == "```":
            lines = lines[:-1]
        txt = "\n".join(lines)
    
    # Find JSON object
    s = txt.find("{")
    if s < 0:
        return None
    
    # Find matching closing brace
    brace_count = 0
    e = s
    for i in range(s, len(txt)):
        if txt[i] == "{":
            brace_count += 1
        elif txt[i] == "}":
            brace_count -= 1
            if brace_count == 0:
                e = i
                break
    
    if e <= s:
        return None
    
    chunk = txt[s:e+1]
    try:
        obj = json.loads(chunk)
        return obj
    except Exception as e:
        return None

def valid_json_shape(obj: dict)->bool:
    if not obj:
        return False
    return all(k in obj for k in REQ) and isinstance(obj.get("command"), str)

# ---------- tokenizer-aware length (optional) ----------
def token_count(tok, text:str)->int:
    try:
        # mlx_lm tokenizer supports encode
        return len(tok.encode(text))
    except Exception:
        return len(text.split())

# ---------- bench ----------
def eval_model(model_id: str, prompts: list[dict], limit: int|None=None, shuffle: bool=False, debug_log=None):
    import random
    model, tok = load(model_id)
    items = prompts[:]
    if shuffle:
        random.shuffle(items)
    if limit:
        items = items[:limit]

    results = []
    latencies = []
    
    # Debug: log model start
    def debug_print(msg):
        if debug_log:
            debug_log.write(f"[{model_id}] {msg}\n")
            debug_log.flush()
        else:
            print(f"   {msg}")
    
    debug_print(f"Starting evaluation of {len(items)} prompts")

    for i, p in enumerate(items, 1):
        # build planning prompt
        ctx = f"selector={p.get('selector')}" if p.get("selector") else "selector=<none>"
        user = f"{p['intent']} {p['target']} in {p['namespace']}"
        prompt_text = f"{SYSTEM}\nContext:\n{ctx}\nUser goal: {user}\nJSON:"

        t0 = time.perf_counter()
        out = generate(model, tok, prompt_text, **DECODING)
        t1 = time.perf_counter()

        obj = extract_json(out)
        json_ok = valid_json_shape(obj)
        cmd_ok = False
        
        # Debug: log all failures (not just first few)
        if not json_ok:
            debug_print(f"[Sample {i}] JSON extraction failed. Raw output (first 300 chars): {out[:300]}")
            if obj:
                debug_print(f"[Sample {i}] Extracted object keys: {list(obj.keys()) if obj else 'None'}")
                debug_print(f"[Sample {i}] Missing required keys: {[k for k in REQ if k not in obj]}")
        if json_ok:
            cmd = obj.get("command", "").strip()
            # Basic validation: command must start with "kubectl"
            if cmd and cmd.lower().startswith("kubectl"):
                cmd_ok = regex_ok(cmd)
            # Debug: log all failed commands
            if not cmd_ok:
                cmd_preview = cmd[:200] if cmd else "<empty>"
                debug_print(f"[Sample {i}] Command failed regex. Intent: {p['intent']}, Command: {cmd_preview}")
        out_tok = token_count(tok, out)

        lat = (t1 - t0) * 1000.0
        latencies.append(lat)

        results.append({
            "case_intent": p["intent"],
            "case_namespace": p["namespace"],
            "case_target": p["target"],
            "model": model_id,
            "latency_ms": round(lat, 2),
            "json_valid": int(json_ok),
            "cmd_regex_ok": int(cmd_ok),
            "output_tokens": out_tok,
            "generated_command": obj.get("command", "")[:200] if json_ok else ""  # Truncate for CSV
        })

    # aggregate
    agg = {
        "model": model_id,
        "n": len(results),
        "json_valid_rate": sum(r["json_valid"] for r in results) / len(results) if results else 0.0,
        "cmd_regex_rate": sum(r["cmd_regex_ok"] for r in results) / len(results) if results else 0.0,
        "lat_p50_ms": statistics.median(latencies) if latencies else 0.0,
        "lat_p95_ms": statistics.quantiles(latencies, n=20)[-1] if len(latencies) >= 20 else max(latencies) if latencies else 0.0,
        "avg_output_tokens": statistics.mean(r["output_tokens"] for r in results) if results else 0.0
    }
    
    # Debug: log summary
    debug_print(f"Completed evaluation: JSON valid: {agg['json_valid_rate']:.2%}, CMD regex ok: {agg['cmd_regex_rate']:.2%}, p50 latency: {agg['lat_p50_ms']:.1f}ms")
    
    return results, agg

def write_csv(rows: list[dict], path: Path):
    if not rows: return
    keys = list(rows[0].keys())
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=keys)
        w.writeheader(); w.writerows(rows)

def write_summary(aggs: list[dict], path: Path):
    with path.open("w", encoding="utf-8") as f:
        f.write("# Benchmark Summary\n\n")
        for a in aggs:
            f.write(f"## {a['model']}\n")
            f.write(f"- Samples: **{a['n']}**\n")
            f.write(f"- JSON validity: **{a['json_valid_rate']:.2%}**\n")
            f.write(f"- Command regex correctness: **{a['cmd_regex_rate']:.2%}**\n")
            f.write(f"- Latency p50 / p95 (ms): **{a['lat_p50_ms']:.1f} / {a['lat_p95_ms']:.1f}**\n")
            f.write(f"- Avg output tokens: **{a['avg_output_tokens']:.1f}**\n\n")

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--models", default="models.yaml")
    ap.add_argument("--prompts", default="prompts.json")
    ap.add_argument("--limit", type=int, default=None, help="subset per model (for quick runs). Omit to run all prompts.")
    ap.add_argument("--shuffle", action="store_true")
    ap.add_argument("--out", default="results.csv")
    ap.add_argument("--summary", default="summary.md")
    ap.add_argument("--debug-log", default=None, help="Write debug output to file")
    args = ap.parse_args()
    
    # Setup debug logging if requested
    debug_log = None
    if args.debug_log:
        debug_log = Path(args.debug_log).open("w", encoding="utf-8")

    prompts = json.loads(Path(args.prompts).read_text(encoding="utf-8"))
    if yaml is None:
        print("pyyaml not installed. Provide models via --models YAML or edit the script.")
        sys.exit(1)
    cfg = yaml.safe_load(Path(args.models).read_text(encoding="utf-8"))
    model_ids = cfg["models"]

    all_rows, all_aggs = [], []
    for mid in model_ids:
        print(f"==> Evaluating {mid} (limit={args.limit})")
        try:
            rows, agg = eval_model(mid, prompts, limit=args.limit, shuffle=args.shuffle, debug_log=debug_log)
            all_rows.extend(rows); all_aggs.append(agg)
            print(f"   {mid}: JSON {agg['json_valid_rate']:.2%}, CMD {agg['cmd_regex_rate']:.2%}, p50 {agg['lat_p50_ms']:.1f} ms")
        except Exception as e:
            print(f"   ERROR: Failed to evaluate {mid}: {e}")
            print(f"   Skipping {mid} and continuing with next model...")
            continue

    write_csv(all_rows, Path(args.out))
    write_summary(all_aggs, Path(args.summary))
    print(f"\nWrote per-sample: {args.out}")
    print(f"Wrote summary:    {args.summary}")
    if debug_log:
        debug_log.close()
        print(f"Wrote debug log:  {args.debug_log}")

if __name__ == "__main__":
    main()

# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "matplotlib",
# ]
# ///

import json
import glob
import os
import subprocess
import tempfile
from pathlib import Path
from os.path import splitext, basename
import matplotlib.pyplot as plt


def run_benchmark(name):
    print(f"Running benchmark for {name}", flush=True)
    # Read saved measurements directly to avoid cargo-criterion's Windows
    # socket shutdown error. Separate runs prevent mixing engines or stale data.
    result_root = Path("target/criterion-runs")
    result_root.mkdir(parents=True, exist_ok=True)
    result_dir = Path(tempfile.mkdtemp(prefix=f"{name}-", dir=result_root)).resolve()
    env = os.environ.copy()
    env.pop("CARGO_CRITERION_PORT", None)
    env["CRITERION_HOME"] = str(result_dir)
    subprocess.run(
        ["cargo", "bench", "--locked", "--bench", name, "--features", name,
         "--", "--noplot"],
        env=env,
        check=True,
    )
    res = []
    for path in result_dir.rglob("new/estimates.json"):
        estimates = json.loads(path.read_text(encoding="utf-8"))
        benchmark = json.loads(path.with_name("benchmark.json").read_text(encoding="utf-8"))
        typical = estimates.get("slope") or estimates["mean"]
        res.append({
            "id": benchmark["full_id"],
            "typical": {
                "estimate": typical["point_estimate"],
                "lower_bound": typical["confidence_interval"]["lower_bound"],
                "upper_bound": typical["confidence_interval"]["upper_bound"],
                "unit": "ns",
            },
            "estimates": estimates,
        })
    if not res:
        raise RuntimeError(f"No Criterion measurements found for {name} in {result_dir}")
    return res


# Compile webassembly modules
subprocess.run(
    "rustc --target wasm32-unknown-unknown -Cpanic=abort -O --crate-name sort_userdata scripts/sort_userdata.wasm.rs -o scripts/sort_userdata.wasm",
    shell=True,
    check=True,
)

benches = dict()
ids = set()
for f in glob.glob("benches/*.rs"):
    name = splitext(basename(f))[0]
    benches[name] = dict()
    results = run_benchmark(name)
    for res in results:
        benches[name][res["id"]] = res
        ids.add(res["id"])

with open("target/benchmark-results.json", "w") as f:
    json.dump(benches, f, indent=2)

for id in ids:
    fig, ax = plt.subplots()

    ymax = 0
    for (name, bench) in sorted(benches.items(), key=lambda x: x[1][id]["typical"]["estimate"]):
        val = round(bench[id]["typical"]["estimate"] / 1000000, 2)
        rect = ax.bar(name, val, width=0.3)
        ax.bar_label(rect, padding=3)
        ymax = max(ymax, val)

    ax.set_title("lower is better")
    ax.set_ylabel("time (ms)", fontweight="bold")
    ax.set_ylim(0, ymax * 1.2)
    fig.autofmt_xdate()
    fig.suptitle(id, fontsize=18)
    fig.tight_layout()

    plt.savefig(f"{id}.png", dpi=300)

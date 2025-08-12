#!/usr/bin/env python3
import os
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

CSV_PATH = "som-rs-5548.csv"

GLOBAL_OUT_ROOT = "output5-with-safe-tinystr"
ENV_OUT_ROOT = "output5-with-safe-tinystr/envs"
SUMMARY_ROOT = "output5-with-safe-tinystr/summaries"
PANEL_OUT = os.path.join(GLOBAL_OUT_ROOT, "panels")

BASELINE_EXE = "som-rs-bc-baseline"

SUBFOLDERS = {
    "time_ms":   {"criterion": "total",     "unit": "ms",    "xlabel": "Execution time (ms)", "fmt": "{:.2f} ms"},
    "gc_count":  {"criterion": "GC count",  "unit": "n",     "xlabel": "GC count (n)",        "fmt": "{:.0f}"},
    "gc_time_ms":{"criterion": "GC time",   "unit": "ms",    "xlabel": "GC time (ms)",        "fmt": "{:.2f} ms"},
    "bytes":     {"criterion": "Allocated", "unit": "bytes", "xlabel": "Allocated (bytes)",   "fmt": "{:.0f}"},
}

df = pd.read_csv(CSV_PATH)
sns.set(style="whitegrid")

os.makedirs(GLOBAL_OUT_ROOT, exist_ok=True)
os.makedirs(ENV_OUT_ROOT, exist_ok=True)
os.makedirs(SUMMARY_ROOT, exist_ok=True)
os.makedirs(PANEL_OUT, exist_ok=True)
for sub in SUBFOLDERS:
    os.makedirs(os.path.join(GLOBAL_OUT_ROOT, sub), exist_ok=True)

def bench_summary(bench_df: pd.DataFrame) -> pd.DataFrame:
    stats = (bench_df.groupby("exe")["value"]
             .agg(mean="mean", median="median", std="std", min="min", max="max", n="count")
             .reset_index())
    if BASELINE_EXE in stats["exe"].values:
        base_mean = stats.loc[stats["exe"] == BASELINE_EXE, "mean"].iloc[0]
        stats["pct_vs_baseline"] = base_mean / stats["mean"] - 1.0
    else:
        stats["pct_vs_baseline"] = np.nan
    stats["cov"] = stats["std"] / stats["mean"]
    return stats

def annotate_text_side(ax, y_idx, x_val, label, xmin, xmax):
    xspan = xmax - xmin
    offset = 0.02 * xspan
    edge_pad = 0.10 * xspan
    if x_val > xmax - edge_pad:
        x = x_val - offset; ha = "right"
    else:
        x = x_val + offset; ha = "left"
    ax.text(x, y_idx, label,
            va="center", ha=ha, fontsize=9, color="black",
            bbox=dict(facecolor="white", alpha=0.7, edgecolor="none", pad=1.5),
            clip_on=False)

def plot_metric(df_all, bench, xlabel, value_fmt, save_path, draw_baseline=True):
    bench_data = df_all[df_all["bench"] == bench]
    if bench_data.empty:
        return

    stats = bench_summary(bench_data)
    exe_order = (bench_data.groupby("exe")["value"]
                 .mean().sort_values(ascending=True).index.tolist())

    n_exe = max(1, len(exe_order))
    fig_h = max(3.2, 0.9 * n_exe + 1.2)
    fig, ax = plt.subplots(figsize=(10, fig_h), constrained_layout=True)

    sns.boxplot(
        data=bench_data, x="value", y="exe", order=exe_order,
        dodge=False, width=0.5, showmeans=False, showfliers=False, whis=(0, 100), ax=ax
    )

    if draw_baseline and BASELINE_EXE in stats["exe"].values:
        base_mean = stats.loc[stats["exe"] == BASELINE_EXE, "mean"].iloc[0]
        ax.axvline(base_mean, linestyle="--", linewidth=1, color="grey", alpha=0.9)
        ax.text(base_mean, -0.6, f"Baseline mean: {value_fmt.format(base_mean)}",
                ha="center", va="bottom", fontsize=8, color="grey",
                bbox=dict(facecolor="white", edgecolor="none", alpha=0.7, pad=1))

    xmin, xmax = ax.get_xlim()

    for i, exe in enumerate(exe_order):
        row = stats[stats["exe"] == exe].iloc[0]
        mean_val = row["mean"]; n = int(row["n"])
        if np.isfinite(row["pct_vs_baseline"]):
            pct = row["pct_vs_baseline"]
            sign = "+" if pct >= 0 else "−"
            pct_txt = f" ({sign}{abs(pct)*100:.1f}% vs base)"
        else:
            pct_txt = ""
        label = f"{value_fmt.format(mean_val)} {pct_txt}"
        annotate_text_side(ax, i, mean_val, label, xmin, xmax)

    envs = ", ".join(map(str, sorted(bench_data["envid"].unique())))
    ax.set_title(f"{bench} — Environment(s): {envs}")
    ax.set_xlabel(xlabel)
    ax.set_ylabel("Interpreter")
    ax.legend([], [], frameon=False)

    best_exe = stats.loc[stats["mean"].idxmin(), "exe"]
    overall_min, overall_max = bench_data["value"].min(), bench_data["value"].max()
    fig.text(0.01, 0.01,
             f"Best mean: {best_exe} - Range: {value_fmt.format(overall_min)}–{value_fmt.format(overall_max)}",
             ha="left", va="bottom", fontsize=8, color="#444")

    fig.savefig(save_path, dpi=300, bbox_inches="tight")
    plt.close(fig)

def save_summaries(df_scope, out_dir):
    os.makedirs(out_dir, exist_ok=True)
    for bench, bdf in df_scope.groupby("bench"):
        stats = bench_summary(bdf).sort_values("mean")
        stats.to_csv(os.path.join(out_dir, f"{bench}.csv"), index=False)

def save_master_summary(df_scope, out_csv_path):
    stats = (df_scope.groupby(["bench", "exe"])["value"]
             .agg(mean="mean", median="median", std="std", min="min", max="max", n="count")
             .reset_index())
    stats.to_csv(out_csv_path, index=False)

def box_no_dots(ax, data, order, xlabel):
    sns.boxplot(
        data=data, x="value", y="exe", order=order,
        dodge=False, width=0.5, showmeans=False, showfliers=False, whis=(0, 100), ax=ax
    )
    ax.set_xlabel(xlabel); ax.set_ylabel("")
    ax.legend([], [], frameon=False)

def plot_bench_panel(bench, out_path):
    slices = {}
    for key, spec in SUBFOLDERS.items():
        d = df[(df["criterion"] == spec["criterion"]) & (df["unit"] == spec["unit"]) & (df["bench"] == bench)]
        if not d.empty:
            slices[key] = (d.copy(), spec["xlabel"], spec["fmt"])
    if not slices:
        return

    order_src = slices.get("time_ms", next(iter(slices.values())))[0]
    exe_order = (order_src.groupby("exe")["value"].mean().sort_values().index.tolist())

    h = max(4.5, 0.75 * max(1, len(exe_order)) + 2)
    fig, axes = plt.subplots(2, 2, figsize=(13, h), constrained_layout=True)
    axes = axes.ravel()

    titles = {
        "time_ms": "Total time",
        "bytes": "Allocated bytes",
        "gc_count": "GC count",
        "gc_time_ms": "GC time",
    }

    i = 0
    for key in ["time_ms", "bytes", "gc_count", "gc_time_ms"]:
        if key not in slices:
            axes[i].axis("off"); i += 1; continue
        data, xlabel, fmt = slices[key]
        box_no_dots(axes[i], data, exe_order, xlabel)

        xmin, xmax = axes[i].get_xlim()
        for j, exe in enumerate(exe_order):
            mean_val = data.loc[data["exe"] == exe, "value"].mean()
            annotate_text_side(axes[i], j, mean_val, fmt.format(mean_val), xmin, xmax)
        axes[i].set_title(titles[key])
        i += 1

    envs = ", ".join(map(str, sorted(df[df["bench"] == bench]["envid"].unique())))
    fig.suptitle(f"{bench} — Environment(s): {envs}", fontsize=12)

    fig.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close(fig)

for folder, spec in SUBFOLDERS.items():
    crit, unit, xlabel, fmt = spec["criterion"], spec["unit"], spec["xlabel"], spec["fmt"]
    df_metric = df[(df["criterion"] == crit) & (df["unit"] == unit)]
    if df_metric.empty:
        continue

    for bench in sorted(df_metric["bench"].unique()):
        out_path = os.path.join(GLOBAL_OUT_ROOT, folder, f"{bench}.png")
        os.makedirs(os.path.dirname(out_path), exist_ok=True)
        plot_metric(df_metric, bench, xlabel, fmt, out_path, draw_baseline=True)

    save_summaries(df_metric, os.path.join(SUMMARY_ROOT, folder))
    save_master_summary(df_metric, os.path.join(SUMMARY_ROOT, f"MASTER_{folder}.csv"))

for bench in sorted(df["bench"].unique()):
    plot_bench_panel(bench, os.path.join(PANEL_OUT, f"{bench}.png"))

for env in sorted(df["envid"].unique()):
    env_df = df[df["envid"] == env]
    env_folder = os.path.join(ENV_OUT_ROOT, str(env))
    os.makedirs(env_folder, exist_ok=True)

    for folder, spec in SUBFOLDERS.items():
        crit, unit, xlabel, fmt = spec["criterion"], spec["unit"], spec["xlabel"], spec["fmt"]
        df_metric = env_df[(env_df["criterion"] == crit) & (env_df["unit"] == unit)]
        if df_metric.empty:
            continue

        metric_dir = os.path.join(env_folder, folder)
        os.makedirs(metric_dir, exist_ok=True)

        for bench in sorted(df_metric["bench"].unique()):
            out_path = os.path.join(metric_dir, f"{bench}.png")
            plot_metric(df_metric, bench, xlabel, fmt, out_path, draw_baseline=True)

        save_summaries(df_metric, os.path.join(env_folder, "summaries", folder))
        save_master_summary(df_metric, os.path.join(env_folder, "summaries", f"MASTER_{folder}.csv"))

    panel_dir = os.path.join(env_folder, "panels")
    os.makedirs(panel_dir, exist_ok=True)
    for bench in sorted(env_df["bench"].unique()):
        plot_bench_panel(bench, os.path.join(panel_dir, f"{bench}.png"))

def geometric_mean(x):
    x = np.asarray([v for v in x if np.isfinite(v) and v > 0])
    if x.size == 0:
        return np.nan
    return float(np.exp(np.mean(np.log(x))))

def overall_comparison(df_scope: pd.DataFrame, metric_name: str, outdir: str):
    os.makedirs(outdir, exist_ok=True)

    bench_means = (df_scope.groupby(["bench", "exe"])["value"]
                   .mean()
                   .reset_index())

    have_base = bench_means["bench"].isin(
        bench_means.loc[bench_means["exe"] == BASELINE_EXE, "bench"]
    )
    bench_means = bench_means.loc[have_base].copy()
    if bench_means.empty:
        print(f"[overall] No benches with baseline for {metric_name}.")
        return

    pivot = bench_means.pivot(index="bench", columns="exe", values="value")
    if BASELINE_EXE not in pivot.columns:
        print(f"[overall] Baseline '{BASELINE_EXE}' missing for {metric_name}.")
        return

    speedup = pivot.apply(lambda col: pivot[BASELINE_EXE] / col)

    # Geometric mean & median speedup across benches (skip NaN/inf)
    gmeans = speedup.apply(geometric_mean, axis=0)
    medians = speedup.median(axis=0, skipna=True)

    winners = pivot.eq(pivot.min(axis=1), axis=0)
    win_rates = winners.sum(axis=0) / winners.shape[0]

    summary = pd.DataFrame({
        "exe": speedup.columns,
        "geo_mean_speedup_vs_baseline": gmeans.values,
        "median_speedup_vs_baseline": medians.values,
        "win_rate": win_rates.values,
    }).sort_values(
        ["geo_mean_speedup_vs_baseline", "win_rate", "median_speedup_vs_baseline"],
        ascending=False
    ).reset_index(drop=True)
    summary.insert(0, "rank", np.arange(1, len(summary) + 1))

    out_csv = os.path.join(outdir, f"OVERALL_{metric_name}.csv")
    summary.to_csv(out_csv, index=False)

    print("\n=== OVERALL PERFORMANCE —", metric_name, "===")
    for _, r in summary.iterrows():
        ex = r["exe"]
        g = r["geo_mean_speedup_vs_baseline"]
        m = r["median_speedup_vs_baseline"]
        w = r["win_rate"]
        star = "  (baseline)" if ex == BASELINE_EXE else ""
        print(f"{int(r['rank']):>2}. {ex:<24} "
              f"gmean (×) vs base: {g:6.3f} | median (×): {m:6.3f} | best (winning) rate: {w*100:5.1f}% {star}")

overall_specs = [
    ("time_ms",   {"criterion": "total",     "unit": "ms"}),
    ("bytes",     {"criterion": "Allocated", "unit": "bytes"}),
    ("gc_time_ms",{"criterion": "GC time",   "unit": "ms"}),
    ("gc_count",  {"criterion": "GC count",  "unit": "n"}),
]

for metric_name, spec in overall_specs:
    scope = df[(df["criterion"] == spec["criterion"]) & (df["unit"] == spec["unit"])]
    if scope.empty:
        continue
    overall_comparison(scope, metric_name, SUMMARY_ROOT)

print("Done!")

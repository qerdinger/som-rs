#!/usr/bin/env python3
import os
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

# CSV file to be read
CSV_PATH = "som-rs-5553.csv"

# Output folder
GLOBAL_OUT_ROOT = "output7_5553-without-imm-float_v2"
ENV_OUT_ROOT = os.path.join(GLOBAL_OUT_ROOT, "envs")
SUMMARY_ROOT = os.path.join(GLOBAL_OUT_ROOT, "summaries")
PANEL_OUT = os.path.join(GLOBAL_OUT_ROOT, "panels")
SUMMARY_TXT = os.path.join(GLOBAL_OUT_ROOT, "summary.txt")

SUMMARY_IMG_TIME = os.path.join(GLOBAL_OUT_ROOT, "summary_time.png")
SUMMARY_IMG_BYTES = os.path.join(GLOBAL_OUT_ROOT, "summary_bytes.png")
SUMMARY_IMG_GCTIME = os.path.join(GLOBAL_OUT_ROOT, "summary_gc_time.png")
SUMMARY_IMG_GCCOUNT = os.path.join(GLOBAL_OUT_ROOT, "summary_gc_count.png")
SUMMARY_IMG_DASH = os.path.join(GLOBAL_OUT_ROOT, "summary_dashboard.png")

# Baseline
BASELINE_EXE = "som-rs-bc-baseline"

EXCLUDE_SUITES: list[str] = [
    # "interpreter",
    # "macro-awfy",
    # "somsom"
]

SUBFOLDERS = {
    "time_ms": {
        "criterion": "total",
        "unit": "ms",
        "xlabel": "Execution time (ms)",
        "fmt": "{:.2f} ms",
    },
    "gc_count": {
        "criterion": "GC count",
        "unit": "n",
        "xlabel": "GC count (n)",
        "fmt": "{:.0f}",
    },
    "gc_time_ms": {
        "criterion": "GC time",
        "unit": "ms",
        "xlabel": "GC time (ms)",
        "fmt": "{:.2f} ms",
    },
    "bytes": {
        "criterion": "Allocated",
        "unit": "bytes",
        "xlabel": "Allocated (bytes)",
        "fmt": "{:.0f}",
    },
}

# Read csv file
df = pd.read_csv(CSV_PATH)

# Show insights
print("Initial size : ", df.size)

df = df[~df["suite"].isin(EXCLUDE_SUITES)].copy()
print("After exclusion list size : ", df.size)

sns.set(style="whitegrid")

os.makedirs(GLOBAL_OUT_ROOT, exist_ok=True)
os.makedirs(ENV_OUT_ROOT, exist_ok=True)
os.makedirs(SUMMARY_ROOT, exist_ok=True)
os.makedirs(PANEL_OUT, exist_ok=True)
for sub in SUBFOLDERS:
    os.makedirs(os.path.join(GLOBAL_OUT_ROOT, sub), exist_ok=True)


def bench_summary(bench_df: pd.DataFrame) -> pd.DataFrame:
    stats = (
        bench_df.groupby("exe")["value"]
        .agg(mean="mean", median="median", std="std", min="min", max="max", n="count")
        .reset_index()
    )
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
        x = x_val - offset
        ha = "right"
    else:
        x = x_val + offset
        ha = "left"
    ax.text(
        x,
        y_idx,
        label,
        va="center",
        ha=ha,
        fontsize=9,
        color="black",
        bbox=dict(facecolor="white", alpha=0.7, edgecolor="none", pad=1.5),
        clip_on=False,
    )

def plot_metric(df_all, bench, xlabel, value_fmt, save_path, draw_baseline=True):
    bench_data = df_all[df_all["bench"] == bench]
    if bench_data.empty:
        return

    stats = bench_summary(bench_data)
    exe_order = (
        bench_data.groupby("exe")["value"]
        .mean()
        .sort_values(ascending=True)
        .index.tolist()
    )

    n_exe = max(1, len(exe_order))
    fig_h = max(3.2, 0.9 * n_exe + 1.2)
    fig, ax = plt.subplots(figsize=(10, fig_h), constrained_layout=True)

    sns.boxplot(
        data=bench_data,
        x="value",
        y="exe",
        order=exe_order,
        dodge=False,
        width=0.5,
        showmeans=False,
        showfliers=False,
        whis=(0, 100),
        ax=ax,
    )

    if draw_baseline and BASELINE_EXE in stats["exe"].values:
        base_mean = stats.loc[stats["exe"] == BASELINE_EXE, "mean"].iloc[0]
        ax.axvline(base_mean, linestyle="--", linewidth=1, color="grey", alpha=0.9)
        ax.text(
            base_mean,
            -0.6,
            f"Baseline mean: {value_fmt.format(base_mean)}",
            ha="center",
            va="bottom",
            fontsize=8,
            color="grey",
            bbox=dict(facecolor="white", edgecolor="none", alpha=0.7, pad=1),
        )

    xmin, xmax = ax.get_xlim()

    for i, exe in enumerate(exe_order):
        row = stats[stats["exe"] == exe].iloc[0]
        mean_val = row["mean"]
        if np.isfinite(row["pct_vs_baseline"]):
            pct = row["pct_vs_baseline"]
            sign = "+" if pct >= 0 else "−"
            pct_txt = f" ({sign}{abs(pct) * 100:.1f}% vs base)"
        else:
            pct_txt = ""
        label = f"{value_fmt.format(mean_val)} {pct_txt}"
        annotate_text_side(ax, i, mean_val, label, xmin, xmax)

    envs = ", ".join(map(str, sorted(bench_data["envid"].unique())))
    ax.set_title(f"{bench} | Environment(s): {envs}")
    ax.set_xlabel(xlabel)
    ax.set_ylabel("Interpreter")
    ax.legend([], [], frameon=False)

    best_exe = stats.loc[stats["mean"].idxmin(), "exe"]
    overall_min, overall_max = bench_data["value"].min(), bench_data["value"].max()
    fig.text(
        0.01,
        0.01,
        f"Best mean: {best_exe} - Range: {value_fmt.format(overall_min)}-{value_fmt.format(overall_max)}",
        ha="left",
        va="bottom",
        fontsize=8,
        color="#444",
    )

    fig.savefig(save_path, dpi=300, bbox_inches="tight")
    plt.close(fig)

def save_summaries(df_scope, out_dir):
    os.makedirs(out_dir, exist_ok=True)
    for bench, bdf in df_scope.groupby("bench"):
        stats = bench_summary(bdf).sort_values("mean")
        stats.to_csv(os.path.join(out_dir, f"{bench}.csv"), index=False)


def save_master_summary(df_scope, out_csv_path):
    stats = (
        df_scope.groupby(["bench", "exe"])["value"]
        .agg(mean="mean", median="median", std="std", min="min", max="max", n="count")
        .reset_index()
    )
    stats.to_csv(out_csv_path, index=False)


def box_no_dots(ax, data, order, xlabel):
    sns.boxplot(
        data=data,
        x="value",
        y="exe",
        order=order,
        dodge=False,
        width=0.5,
        showmeans=False,
        showfliers=False,
        whis=(0, 100),
        ax=ax,
    )
    ax.set_xlabel(xlabel)
    ax.set_ylabel("")
    ax.legend([], [], frameon=False)

def plot_bench_panel(bench, out_path):
    slices = {}
    for key, spec in SUBFOLDERS.items():
        d = df[
            (df["criterion"] == spec["criterion"])
            & (df["unit"] == spec["unit"])
            & (df["bench"] == bench)
        ]
        if not d.empty:
            slices[key] = (d.copy(), spec["xlabel"], spec["fmt"])
    if not slices:
        return

    order_src = slices.get("time_ms", next(iter(slices.values())))[0]
    exe_order = order_src.groupby("exe")["value"].mean().sort_values().index.tolist()

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
            axes[i].axis("off")
            i += 1
            continue
        data, xlabel, fmt = slices[key]
        box_no_dots(axes[i], data, exe_order, xlabel)

        xmin, xmax = axes[i].get_xlim()
        for j, exe in enumerate(exe_order):
            mean_val = data.loc[data["exe"] == exe, "value"].mean()
            annotate_text_side(axes[i], j, mean_val, fmt.format(mean_val), xmin, xmax)
        axes[i].set_title(titles[key])
        i += 1

    envs = ", ".join(map(str, sorted(df[df["bench"] == bench]["envid"].unique())))
    fig.suptitle(f"{bench} | Environment(s): {envs}", fontsize=12)

    fig.savefig(out_path, dpi=300, bbox_inches="tight")
    plt.close(fig)


for folder, spec in SUBFOLDERS.items():
    crit, unit, xlabel, fmt = (
        spec["criterion"],
        spec["unit"],
        spec["xlabel"],
        spec["fmt"],
    )
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
        crit, unit, xlabel, fmt = (
            spec["criterion"],
            spec["unit"],
            spec["xlabel"],
            spec["fmt"],
        )
        df_metric = env_df[(env_df["criterion"] == crit) & (env_df["unit"] == unit)]
        if df_metric.empty:
            continue

        metric_dir = os.path.join(env_folder, folder)
        os.makedirs(metric_dir, exist_ok=True)

        for bench in sorted(df_metric["bench"].unique()):
            out_path = os.path.join(metric_dir, f"{bench}.png")
            plot_metric(df_metric, bench, xlabel, fmt, out_path, draw_baseline=True)

        save_summaries(df_metric, os.path.join(env_folder, "summaries", folder))
        save_master_summary(
            df_metric, os.path.join(env_folder, "summaries", f"MASTER_{folder}.csv")
        )

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

    bench_means = df_scope.groupby(["bench", "exe"])["value"].mean().reset_index()

    have_base = bench_means["bench"].isin(
        bench_means.loc[bench_means["exe"] == BASELINE_EXE, "bench"]
    )
    bench_means = bench_means.loc[have_base].copy()
    if bench_means.empty:
        return None

    pivot = bench_means.pivot(index="bench", columns="exe", values="value")
    if BASELINE_EXE not in pivot.columns:
        return None

    speedup = pivot.apply(lambda col: pivot[BASELINE_EXE] / col)

    gmeans = speedup.apply(geometric_mean, axis=0)
    medians = speedup.median(axis=0, skipna=True)

    winners = pivot.eq(pivot.min(axis=1), axis=0)
    win_rates = winners.sum(axis=0) / winners.shape[0]

    summary = (
        pd.DataFrame(
            {
                "exe": speedup.columns,
                "geo_mean_speedup_vs_baseline": gmeans.values,
                "median_speedup_vs_baseline": medians.values,
                "win_rate": win_rates.values,
            }
        )
        .sort_values(
            ["geo_mean_speedup_vs_baseline", "win_rate", "median_speedup_vs_baseline"],
            ascending=False,
        )
        .reset_index(drop=True)
    )
    summary.insert(0, "rank", np.arange(1, len(summary) + 1))

    out_csv = os.path.join(outdir, f"OVERALL_{metric_name}.csv")
    summary.to_csv(out_csv, index=False)
    return summary


overall_specs = [
    ("time_ms", {"criterion": "total", "unit": "ms"}),
    ("bytes", {"criterion": "Allocated", "unit": "bytes"}),
    ("gc_time_ms", {"criterion": "GC time", "unit": "ms"}),
    ("gc_count", {"criterion": "GC count", "unit": "n"}),
]

overall_results: dict[str, pd.DataFrame | None] = {}
df_by_metric: dict[str, pd.DataFrame] = {}

for metric_name, spec in overall_specs:
    scope = df[(df["criterion"] == spec["criterion"]) & (df["unit"] == spec["unit"])]
    if scope.empty:
        continue
    df_by_metric[metric_name] = scope.copy()
    overall_results[metric_name] = overall_comparison(scope, metric_name, SUMMARY_ROOT)


def format_overall_block(metric_name: str, summary: pd.DataFrame) -> str:
    header = f"=== OVERALL PERFORMANCE | {metric_name} ==="
    if summary is None or summary.empty:
        return header + "\n(no result)\n"

    lines = [header]
    for _, r in summary.iterrows():
        ex = str(r["exe"])
        g = r["geo_mean_speedup_vs_baseline"]
        m = r["median_speedup_vs_baseline"]
        w = r["win_rate"]
        star = "   (baseline)" if ex == BASELINE_EXE else ""
        lines.append(
            f" {int(r['rank']):>2}. {ex:<24}"
            f" gmean (x) vs base: {g:6.3f} | median (x): {m:6.3f} |"
            f" best (winning) rate: {w * 100:6.1f}% {star}"
        )
    lines.append("")
    return "\n".join(lines)


blocks = []
for metric_name, _ in overall_specs:
    blocks.append(format_overall_block(metric_name, overall_results.get(metric_name)))

pretty_summary = "\n".join(blocks)

print(pretty_summary)
with open(SUMMARY_TXT, "w", encoding="utf-8") as f:
    f.write(pretty_summary)
print("Summary saved !")


def overall_raw_means(scope: pd.DataFrame) -> pd.Series:
    bench_means = scope.groupby(["bench", "exe"])["value"].mean().reset_index()
    return bench_means.groupby("exe")["value"].mean().sort_values(ascending=False)


def plot_overall_metric(
    metric_key: str,
    summary: pd.DataFrame | None,
    scope: pd.DataFrame | None,
    out_path: str,
    unit_label: str,
    title_suffix: str,
):
    os.makedirs(os.path.dirname(out_path), exist_ok=True)

    if summary is None or summary.empty:
        plt.figure(figsize=(7, 3))
        plt.axis("off")
        plt.text(0.5, 0.5, f"No data for {metric_key}", ha="center", va="center")
        plt.savefig(out_path, dpi=220, bbox_inches="tight")
        plt.close()
        return

    summary = summary.sort_values(
        "geo_mean_speedup_vs_baseline", ascending=True
    ).reset_index(drop=True)

    raw_means = None
    if scope is not None and not scope.empty:
        raw_means = overall_raw_means(scope)

    fig_h = 8.5 if raw_means is not None else 6.5
    fig = plt.figure(figsize=(10.5, fig_h))

    ax1 = fig.add_axes([0.10, 0.63, 0.80, 0.30])
    y = np.arange(len(summary))
    ax1.barh(y, summary["geo_mean_speedup_vs_baseline"].values)
    ax1.set_yticks(y, summary["exe"].values)
    ax1.set_xlabel("Geometric mean speedup vs baseline (x) | higher is better")
    ax1.set_title(f"{metric_key} | {title_suffix}")

    for i, v in enumerate(summary["geo_mean_speedup_vs_baseline"].values):
        ax1.text(v, i, f" {v:.3f}x", va="center")

    ax2 = fig.add_axes([0.10, 0.33, 0.80, 0.24])
    wins_pct = summary["win_rate"].values * 100.0
    ax2.barh(y, wins_pct)
    ax2.set_yticks(y, summary["exe"].values)
    ax2.set_xlabel("Winning rate across benches (%) | higher is better")
    for i, v in enumerate(wins_pct):
        ax2.text(v, i, f" {v:.1f}%", va="center")

    if raw_means is not None:
        ax3 = fig.add_axes([0.10, 0.08, 0.80, 0.20])
        ax3.axis("off")
        rows = []
        for ex in summary["exe"].values:
            val = raw_means.get(ex, np.nan)
            rows.append((ex, val))
        lines = [f"\n\nOverall mean (of bench means) by interpreter | {unit_label}"]
        for ex, v in rows:
            if np.isfinite(v):
                if unit_label == "bytes":
                    lines.append(f"- {ex:<24} : {v:,.0f} {unit_label}")
                elif unit_label == "n":
                    lines.append(f"- {ex:<24} : {v:.2f} {unit_label}")
                else:
                    lines.append(f"- {ex:<24} : {v:.3f} {unit_label}")
            else:
                lines.append(f"- {ex:<24} : n/a")
        text = "\n".join(lines)
        ax3.text(0, 1, text, va="top", ha="left", family="monospace")

    plt.savefig(out_path, dpi=220, bbox_inches="tight")
    plt.close()


plot_overall_metric(
    "time_ms",
    overall_results.get("time_ms"),
    df_by_metric.get("time_ms"),
    SUMMARY_IMG_TIME,
    unit_label="ms",
    title_suffix="speedups & wins (time)",
)
plot_overall_metric(
    "bytes",
    overall_results.get("bytes"),
    df_by_metric.get("bytes"),
    SUMMARY_IMG_BYTES,
    unit_label="bytes",
    title_suffix="speedups & wins (allocated bytes)",
)
plot_overall_metric(
    "gc_time_ms",
    overall_results.get("gc_time_ms"),
    df_by_metric.get("gc_time_ms"),
    SUMMARY_IMG_GCTIME,
    unit_label="ms",
    title_suffix="speedups & wins (GC time)",
)
plot_overall_metric(
    "gc_count",
    overall_results.get("gc_count"),
    df_by_metric.get("gc_count"),
    SUMMARY_IMG_GCCOUNT,
    unit_label="n",
    title_suffix="speedups & wins (GC count)",
)


def make_dashboard(out_path: str):
    fig, axes = plt.subplots(2, 2, figsize=(12, 8), constrained_layout=True)
    items = [
        ("time_ms", overall_results.get("time_ms")),
        ("bytes", overall_results.get("bytes")),
        ("gc_time_ms", overall_results.get("gc_time_ms")),
        ("gc_count", overall_results.get("gc_count")),
    ]
    for ax, (name, summ) in zip(axes.ravel(), items):
        if summ is None or summ.empty:
            ax.axis("off")
            ax.set_title(f"{name}: (no data)")
            continue
        show = summ.sort_values("geo_mean_speedup_vs_baseline", ascending=True)
        y = np.arange(len(show))
        ax.barh(y, show["geo_mean_speedup_vs_baseline"].values)
        ax.set_yticks(y, show["exe"].values, fontsize=8)
        ax.set_title(f"{name} | gmean speedup (x), higher is better")
        for i, v in enumerate(show["geo_mean_speedup_vs_baseline"].values):
            ax.text(v, i, f" {v:.2f}x", va="center", fontsize=8)
    plt.savefig(out_path, dpi=220, bbox_inches="tight")
    plt.close()


make_dashboard(SUMMARY_IMG_DASH)


def speedup_pivot_vs_baseline(scope: pd.DataFrame) -> pd.DataFrame | None:
    bench_means = scope.groupby(["bench", "exe"])["value"].mean().reset_index()

    have_base = bench_means["bench"].isin(
        bench_means.loc[bench_means["exe"] == BASELINE_EXE, "bench"]
    )
    bench_means = bench_means.loc[have_base].copy()
    if bench_means.empty:
        return None

    pivot = bench_means.pivot(index="bench", columns="exe", values="value")
    if BASELINE_EXE not in pivot.columns:
        return None

    speedup = pivot.apply(lambda col: pivot[BASELINE_EXE] / col)
    return speedup


def plot_overall_metric_detailed(
    metric_key: str,
    summary: pd.DataFrame | None,
    scope: pd.DataFrame | None,
    out_path: str,
    title_suffix: str,
):
    os.makedirs(os.path.dirname(out_path), exist_ok=True)

    if summary is None or summary.empty or scope is None or scope.empty:
        fig = plt.figure(figsize=(8, 3))
        plt.axis("off")
        plt.text(0.5, 0.5, f"No data for {metric_key}", ha="center", va="center")
        plt.savefig(out_path, dpi=220, bbox_inches="tight")
        plt.close()
        return

    summary = summary.sort_values(
        "geo_mean_speedup_vs_baseline", ascending=True
    ).reset_index(drop=True)
    exe_order = summary["exe"].tolist()

    speedup = speedup_pivot_vs_baseline(scope)
    if speedup is None:
        fig = plt.figure(figsize=(8, 3))
        plt.axis("off")
        plt.text(
            0.5,
            0.5,
            f"No baseline to compute speedups for {metric_key}",
            ha="center",
            va="center",
        )
        plt.savefig(out_path, dpi=220, bbox_inches="tight")
        plt.close()
        return

    present = [ex for ex in exe_order if ex in speedup.columns]
    if not present:
        fig = plt.figure(figsize=(8, 3))
        plt.axis("off")
        plt.text(
            0.5,
            0.5,
            f"No overlapping executables for {metric_key}",
            ha="center",
            va="center",
        )
        plt.savefig(out_path, dpi=220, bbox_inches="tight")
        plt.close()
        return

    fig_h = 12
    fig = plt.figure(figsize=(11.5, fig_h))

    ax1 = fig.add_axes([0.10, 0.705, 0.80, 0.23])
    y = np.arange(len(summary))
    ax1.barh(y, summary["geo_mean_speedup_vs_baseline"].values)
    ax1.set_yticks(y, summary["exe"].values)
    ax1.set_xlabel("Geometric mean speedup vs baseline (x) | higher is better")
    ax1.set_title(f"{metric_key} | {title_suffix}")
    for i, v in enumerate(summary["geo_mean_speedup_vs_baseline"].values):
        ax1.text(v, i, f" {v:.3f}x", va="center")

    ax2 = fig.add_axes([0.10, 0.405, 0.80, 0.23])
    ax2.barh(y, summary["median_speedup_vs_baseline"].values)
    ax2.set_yticks(y, summary["exe"].values)
    ax2.set_xlabel("Median speedup vs baseline (x) | higher is better")
    for i, v in enumerate(summary["median_speedup_vs_baseline"].values):
        ax2.text(v, i, f" {v:.3f}x", va="center")

    ax3 = fig.add_axes([0.10, 0.08, 0.80, 0.25])

    dist_rows = []
    for ex in present:
        vals = speedup[ex].dropna().values
        for s in vals:
            if np.isfinite(s):
                dist_rows.append({"exe": ex, "speedup": float(s)})

    if dist_rows:
        dist_df = pd.DataFrame(dist_rows)
        sns.boxplot(
            data=dist_df,
            x="speedup",
            y="exe",
            order=exe_order,
            dodge=False,
            width=0.5,
            showmeans=False,
            showfliers=False,
            whis=(0, 100),
            ax=ax3,
        )
        ax3.set_xlabel("Per-benchmark speedup vs baseline (x)")

        xmin, xmax = ax3.get_xlim()
        for j, ex in enumerate(exe_order):
            vals = dist_df.loc[dist_df["exe"] == ex, "speedup"].dropna()
            if vals.empty:
                continue
            stats_text = (
                f"min={vals.min():.3f}x  "
                f"mean={vals.mean():.3f}x  "
                f"median={vals.median():.3f}x  "
                f"max={vals.max():.3f}x"
            )
            ax3.text(
                xmax + 0.01 * (xmax - xmin),
                j,
                stats_text,
                va="center",
                ha="left",
                fontsize=7,
                color="#333",
                clip_on=False,
            )
    else:
        ax3.axis("off")
        ax3.text(
            0.5, 0.5, "No per-benchmark speedup distribution", ha="center", va="center"
        )

    plt.savefig(out_path, dpi=220, bbox_inches="tight")
    plt.close()


plot_overall_metric_detailed(
    "time_ms",
    overall_results.get("time_ms"),
    df_by_metric.get("time_ms"),
    SUMMARY_IMG_TIME.replace(".png", "_detailed.png"),
    title_suffix="GMean | Median | Distribution (time)",
)
plot_overall_metric_detailed(
    "bytes",
    overall_results.get("bytes"),
    df_by_metric.get("bytes"),
    SUMMARY_IMG_BYTES.replace(".png", "_detailed.png"),
    title_suffix="GMean | Median | Distribution (allocated bytes)",
)
plot_overall_metric_detailed(
    "gc_time_ms",
    overall_results.get("gc_time_ms"),
    df_by_metric.get("gc_time_ms"),
    SUMMARY_IMG_GCTIME.replace(".png", "_detailed.png"),
    title_suffix="GMean | Median | Distribution (GC time)",
)
plot_overall_metric_detailed(
    "gc_count",
    overall_results.get("gc_count"),
    df_by_metric.get("gc_count"),
    SUMMARY_IMG_GCCOUNT.replace(".png", "_detailed.png"),
    title_suffix="GMean | Median | Distribution (GC count)",
)

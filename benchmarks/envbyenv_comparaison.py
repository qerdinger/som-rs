import os
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

CSV_PATH = "som-rs-5523.csv"

GLOBAL_OUT_ROOT = "output2"
ENV_OUT_ROOT = "output2/envs"

SUBFOLDERS = {
    "time_ms":   {"criterion": "total",     "unit": "ms",    "xlabel": "Execution time (ms)", "fmt": "{:.2f} ms"},
    "gc_count":  {"criterion": "GC count",  "unit": "n",     "xlabel": "GC count (n)",        "fmt": "{:.0f}"},
    "gc_time_ms":{"criterion": "GC time",   "unit": "ms",    "xlabel": "GC time (ms)",        "fmt": "{:.2f} ms"},
    "bytes":     {"criterion": "Allocated", "unit": "bytes", "xlabel": "Allocated (bytes)",   "fmt": "{:.0f}"},
}

df = pd.read_csv(CSV_PATH)
sns.set(style="whitegrid")

os.makedirs(GLOBAL_OUT_ROOT, exist_ok=True)
for sub in SUBFOLDERS:
    os.makedirs(os.path.join(GLOBAL_OUT_ROOT, sub), exist_ok=True)

def plot_metric(df_all, bench, xlabel, value_fmt, save_path):
    bench_data = df_all[df_all["bench"] == bench]
    if bench_data.empty:
        return

    exe_order = (bench_data.groupby("exe")["value"]
                 .mean()
                 .sort_values(ascending=True)
                 .index.tolist())

    n_exe = max(1, len(exe_order))
    fig_h = max(3.0, 0.9 * n_exe + 1.2)
    fig, ax = plt.subplots(figsize=(9, fig_h), constrained_layout=True)

    sns.boxplot(
        data=bench_data,
        x="value",
        y="exe",
        order=exe_order,
        hue="exe",
        dodge=False,
        width=0.5,
        showmeans=False,
        showfliers=False,
        whis=(0, 100),
        ax=ax,
    )

    xmin, xmax = ax.get_xlim()
    xspan = xmax - xmin
    offset = 0.02 * xspan
    edge_pad = 0.10 * xspan

    for i, exe in enumerate(exe_order):
        avg = bench_data.loc[bench_data["exe"] == exe, "value"].mean()

        if avg > xmax - edge_pad:
            x = avg - offset
            ha = "right"
        elif avg < xmin + edge_pad:
            x = avg + offset
            ha = "left"
        else:
            x = avg + offset
            ha = "left"

        ax.text(
            x, i, value_fmt.format(avg),
            va="center", ha=ha, fontsize=9, color="black",
            bbox=dict(facecolor="white", alpha=0.7, edgecolor="none", pad=1.5),
            clip_on=False,
        )

    envs = ", ".join(map(str, sorted(bench_data["envid"].unique())))
    ax.set_title(f"{bench} | Environment(s): {envs}")
    ax.set_xlabel(xlabel)
    ax.set_ylabel("Interpreter")
    ax.legend([], [], frameon=False)

    fig.savefig(save_path, dpi=300, bbox_inches="tight")
    plt.close(fig)

for folder, spec in SUBFOLDERS.items():
    crit, unit, xlabel, fmt = spec["criterion"], spec["unit"], spec["xlabel"], spec["fmt"]
    df_metric = df[(df["criterion"] == crit) & (df["unit"] == unit)]
    if df_metric.empty:
        continue
    for bench in sorted(df_metric["bench"].unique()):
        out_path = os.path.join(GLOBAL_OUT_ROOT, folder, f"{bench}.png")
        plot_metric(df_metric, bench, xlabel, fmt, out_path)

os.makedirs(ENV_OUT_ROOT, exist_ok=True)
for env in sorted(df["envid"].unique()):
    env_df = df[df["envid"] == env]
    env_folder = os.path.join(ENV_OUT_ROOT, str(env))
    for sub in SUBFOLDERS:
        os.makedirs(os.path.join(env_folder, sub), exist_ok=True)

    for folder, spec in SUBFOLDERS.items():
        crit, unit, xlabel, fmt = spec["criterion"], spec["unit"], spec["xlabel"], spec["fmt"]
        df_metric = env_df[(env_df["criterion"] == crit) & (env_df["unit"] == unit)]
        if df_metric.empty:
            continue
        for bench in sorted(df_metric["bench"].unique()):
            out_path = os.path.join(env_folder, folder, f"{bench}.png")
            plot_metric(df_metric, bench, xlabel, fmt, out_path)

print(f"{GLOBAL_OUT_ROOT}/ | {ENV_OUT_ROOT}/")

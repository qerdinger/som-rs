import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import os

CSV_FILE = "som-rs-5548.csv"
SEPARATOR  = ","


# Load csv file
df = pd.read_csv(CSV_FILE, sep=SEPARATOR)

df.columns = [c.strip().lower().replace(" ", "_") for c in df.columns]

for col in ["value","inputsize","iteration","invocation"]:
    if col in df.columns:
        df[col] = pd.to_numeric(df[col], errors="coerce")


time_df = df[(df["criterion"].str.lower() == "total") & (df["unit"].str.lower() == "ms")].copy()

group_iter = ["bench","exe","suite","inputsize","invocation"]
per_invocation = (
    time_df.groupby(group_iter, dropna=False)["value"]
           .median()
           .rename("median_ms_per_invocation")
           .reset_index()
)

group_final = ["bench","exe","suite","inputsize"]
time_summary = (
    per_invocation.groupby(group_final, dropna=False)["median_ms_per_invocation"]
                  .median()
                  .rename("median_ms")
                  .reset_index()
)

print("\n***** Collapsed timing summary (per bench/exe/suite/inputsize) *****")
print(time_summary.sort_values(["suite","bench","inputsize","median_ms"]).to_string(index=False))

alloc_df = df[(df["criterion"].str.lower() == "allocated") & (df["unit"].str.lower() == "bytes")].copy()

alloc_per_invocation = (
    alloc_df.groupby(group_iter, dropna=False)["value"]
            .median()
            .rename("median_bytes_per_invocation")
            .reset_index()
)

alloc_summary = (
    alloc_per_invocation.groupby(group_final, dropna=False)["median_bytes_per_invocation"]
                        .median()
                        .rename("median_bytes")
                        .reset_index()
)

print("\n***** Allocation summary  *****")
print(alloc_summary.sort_values(["suite","bench","inputsize","median_bytes"]).to_string(index=False))

metrics = df.pivot_table(
    index=["bench","exe"],
    columns=df["criterion"].str.lower(),
    values="value",
    aggfunc="median",
).rename(columns={
    "total": "time_ms",
    "allocated": "bytes",
    "gc count": "gc_count",
    "gc time": "gc_time_ms",
})

os.makedirs('output', exist_ok=True)

for col, label in [
    ("time_ms",     "Time (ms)"),
    ("gc_count",    "GC count"),
    ("gc_time_ms",  "GC time (ms)"),
    ("bytes",       "Allocated bytes"),
]:
    pivot = metrics[col].unstack()
    ax = pivot.plot(kind='bar', figsize=(10, 6), title=label)
    ax.set_ylabel(label)
    plt.xticks(rotation=45, ha='right')
    plt.tight_layout()
    plt.savefig(os.path.join('output', f"{col}.png"))
    plt.close()

import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.ticker import MaxNLocator

df = pd.read_csv("stats.csv")
df["timestamp"] = pd.to_datetime(df["timestamp"], errors="coerce")

start_group = df.groupby("start_points")["time_to_finish_in_micros"].mean().reset_index()
obstacle_group = df.groupby("obstacles_amount")["time_to_finish_in_micros"].mean().reset_index()

y_min = min(start_group["time_to_finish_in_micros"].min(), obstacle_group["time_to_finish_in_micros"].min())
y_max = max(start_group["time_to_finish_in_micros"].max(), obstacle_group["time_to_finish_in_micros"].max())

fig, axes = plt.subplots(1, 2, figsize=(12, 5), sharey=True)

# --- Plot 1: Cost by Start Points ---
axes[0].plot(
    start_group["start_points"],
    start_group["time_to_finish_in_micros"],
    marker="o",
    label="Avg Time per Start Points"
)
axes[0].set_title("Computational Cost by Start Points")
axes[0].set_xlabel("Start Points")
axes[0].set_ylabel("Time to Finish (μs)")
axes[0].grid(True)
axes[0].yaxis.set_major_locator(MaxNLocator(integer=True))
axes[0].xaxis.set_major_locator(MaxNLocator(integer=True))
axes[0].set_ylim(y_min, y_max)

# --- Plot 2: Cost by Obstacles ---
axes[1].plot(
    obstacle_group["obstacles_amount"],
    obstacle_group["time_to_finish_in_micros"],
    marker="o",
    color="orange",
    label="Avg Time per Obstacles"
)
axes[1].set_title("Computational Cost by Obstacles")
axes[1].set_xlabel("Obstacles Amount")
axes[1].grid(True)
axes[1].yaxis.set_major_locator(MaxNLocator(integer=True))
axes[1].xaxis.set_major_locator(MaxNLocator(integer=True))
axes[1].set_ylim(y_min, y_max)

plt.tight_layout()
plt.show()

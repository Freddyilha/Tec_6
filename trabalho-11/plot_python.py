import pandas as pd
import matplotlib.pyplot as plt

# Load CSV
df = pd.read_csv(
    "stats.csv",
    parse_dates=["timestamp"]
)

# Sort just in case
df = df.sort_values("timestamp")

# Create derived metrics
df["extra_steps"] = df["how_many_steps_agents_made"] - df["total_agents_path_length"]
df["extra_steps"] = df["extra_steps"].clip(lower=0)


# --- Plot 1: Agent movement vs optimal path ---
plt.figure()
plt.plot(df["timestamp"], df["how_many_steps_agents_made"], label="Actual steps")
plt.plot(df["timestamp"], df["total_agents_path_length"], label="Baseline (A*)")

plt.xlabel("Time")
plt.ylabel("Steps")
plt.title("Passos X Caminho calculado")
plt.legend()
plt.xticks(rotation=45)
plt.figtext(
    0.5, -0.15,
    "Temporal evolution of collision detections and path recalculations. "
    "Recalculations remain bounded, indicating stable collision resolution behavior.",
    ha="center",
    wrap=True
)
plt.tight_layout()
plt.show()


# --- Plot 2: Extra steps over time ---
plt.figure()
plt.plot(df["timestamp"], df["extra_steps"])

plt.xlabel("Time")
plt.ylabel("Extra steps")
plt.title("Quantidade de passos extras")
plt.xticks(rotation=45)
plt.tight_layout()
plt.show()


# --- Plot 3: Recalculations vs detections ---
plt.figure()
plt.plot(df["timestamp"], df["how_many_recalculations"], label="Recalculations")
plt.plot(df["timestamp"], df["how_many_detections"], label="Collision detections")

plt.xlabel("Time")
plt.ylabel("Count")
plt.title("Colisoes X Recalculando passos")
plt.legend()
plt.xticks(rotation=45)
plt.tight_layout()
plt.show()

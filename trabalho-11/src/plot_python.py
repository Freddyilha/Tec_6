import pandas as pd
import matplotlib.pyplot as plt

plt.rcParams["figure.figsize"] = (10, 6)

df = pd.read_csv("stats.csv", parse_dates=["timestamp"])

df = df[df["method_name"].isin(["GRID", "PATH","ORCA"])]

final = df.groupby("method_name").tail(1).copy()

print("\nFinal results per method:")
print(final[[
    "method_name",
    "collisions",
    "recalculations",
    "total_steps",
    "total_path_length",
    "actual_distance",
    "reached_goal_count"
]])

# -------------------------------------------------
# 1️⃣ Collisions Comparison
# -------------------------------------------------

plt.figure()
plt.bar(final["method_name"], final["collisions"])
plt.xlabel("Method")
plt.ylabel("Total Collisions")
plt.title("Total Collisions Comparison")
plt.tight_layout()
plt.show()

# -------------------------------------------------
# 2️⃣ Computational Cost (steps)
# -------------------------------------------------

plt.figure()
plt.bar(final["method_name"], final["total_steps"])
plt.xlabel("Method")
plt.ylabel("Total Simulation Steps")
plt.title("Computational Cost Comparison")
plt.tight_layout()
plt.show()

# -------------------------------------------------
# 3️⃣ Path Efficiency
# -------------------------------------------------

final["path_efficiency"] = (
    final["total_path_length"] / final["actual_distance"]
)

plt.figure()
plt.bar(final["method_name"], final["path_efficiency"])
plt.ylabel("Path Efficiency (planned / actual)")
plt.title("Path Quality Comparison")
plt.tight_layout()
plt.show()

# -------------------------------------------------
# 4️⃣ Distance Traveled
# -------------------------------------------------

plt.figure()
plt.bar(final["method_name"], final["actual_distance"])
plt.ylabel("Actual Distance Traveled")
plt.title("Total Distance Traveled")
plt.tight_layout()
plt.show()

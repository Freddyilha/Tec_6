import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv("stats.csv")

df["timestamp"] = pd.to_datetime(df["timestamp"], errors="coerce")

plt.figure(figsize=(10, 5))
plt.plot(df["points_amount"], df["time_to_finish_in_micros"], marker="o", label="Minkowski Sum Time")

plt.xlabel("Points Amount")
plt.ylabel("Minkowski Sum Time to Finish")
plt.title("Minkowski Sum Time by Points Amount")
plt.legend()
plt.grid(True)
plt.show()

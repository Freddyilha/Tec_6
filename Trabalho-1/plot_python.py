import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv("stats.csv")

df["timestamp"] = pd.to_datetime(df["timestamp"], errors="coerce")

plt.figure(figsize=(10,5))
plt.plot(df["timestamp"], df["clicks_on_dots"], label="Clicks on Dots")
plt.plot(df["timestamp"], df["clicks_on_lines"], label="Clicks on Lines")
plt.plot(df["timestamp"], df["number_of_clicks"], label="Total Clicks")

plt.xlabel("Timestamp")
plt.ylabel("Clicks")
plt.title("Clicks over Time")
plt.legend()
plt.grid(True)
plt.show()


plt.figure(figsize=(6,6))
plt.plot(df["mouse_x"], df["mouse_y"], marker=".", markersize=3, linestyle="-")
plt.xlabel("Mouse X")
plt.ylabel("Mouse Y")
plt.title("Mouse Trajectory")
plt.grid(True)
plt.gca().invert_yaxis()
plt.show()

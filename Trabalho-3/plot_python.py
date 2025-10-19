import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv("stats.csv", parse_dates=['timestamp'])

fig, ax1 = plt.subplots(figsize=(12,6))

# Left y-axis: Points
ax1.set_xlabel('Timestamp')
ax1.set_ylabel('Points', color='blue')
ax1.plot(df['timestamp'], df['points_on_hull'], label='Points on Hull', color='blue', marker='o')
ax1.plot(df['timestamp'], df['points_inside_hull'], label='Points Inside Hull', color='cyan', marker='x')
ax1.tick_params(axis='y', labelcolor='blue')

# Right y-axis: Memory
ax2 = ax1.twinx()
ax2.set_ylabel('Memory (KB)', color='red')
ax2.plot(df['timestamp'], df['memory_kb'], label='Memory (KB)', color='red', marker='^')
ax2.tick_params(axis='y', labelcolor='red')

# Combine legends
lines_1, labels_1 = ax1.get_legend_handles_labels()
lines_2, labels_2 = ax2.get_legend_handles_labels()
ax1.legend(lines_1 + lines_2, labels_1 + labels_2, loc='upper left')

plt.title('Points and Memory Usage Over Time')
plt.xticks(rotation=45)
plt.tight_layout()
plt.show()

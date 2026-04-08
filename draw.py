import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from matplotlib.gridspec import GridSpec

# 1. 准备数据
df = pd.read_csv('selected_bench_overview_20.csv')
# 按类别排序，让颜色块更集中
df = df.sort_values('family').reset_index(drop=True)

# 2. 设置布局：断轴设计
# 使用 GridSpec 分配上下两部分，比例为 1:2
fig = plt.figure(figsize=(14, 8))
gs = GridSpec(2, 1, height_ratios=[1, 2], hspace=0.1)
ax_top = fig.add_subplot(gs[0])
ax_bottom = fig.add_subplot(gs[1])

# 3. 颜色方案：为不同类别分配专业学术配色
unique_families = df['family'].unique()
palette = sns.color_palette("Set2", len(unique_families))
color_map = dict(zip(unique_families, palette))

# 4. 绘图
# 上下两部分画同样的数据，后面通过 ylim 来截断
for ax in [ax_top, ax_bottom]:
    sns.barplot(
        data=df, x='api_pair_name', y='safe_over_unsafe_ratio',
        hue='family', palette=color_map, ax=ax, dodge=False
    )
    ax.legend_.remove() # 暂时移除自带图例

# 5. 设置“断轴”范围
# 根据你的 CSV：Encoding 在 10 左右，其他在 1-4 之间
ax_top.set_ylim(9.5, 11)    # 只显示顶部的极端值
ax_bottom.set_ylim(0, 4.5)  # 显示大部分数据及基准线

# 6. 添加基准线 (y=1)
ax_bottom.axhline(y=1, color='#d62728', linestyle='--', linewidth=1.5, alpha=0.8)
ax_bottom.text(-0.5, 1.1, 'Baseline (y=1)', color='#d62728', fontweight='bold')

# 7. 细节修饰 (消除中间的边框)
ax_top.spines['bottom'].set_visible(False)
ax_bottom.spines['top'].set_visible(False)
ax_top.xaxis.set_visible(False) # 隐藏顶部的横坐标刻度

# 绘制断轴标志 (斜杠)
d = .01 
kwargs = dict(transform=ax_top.transAxes, color='k', clip_on=False)
ax_top.plot((-d, +d), (-d, +d), **kwargs)        # 左上
ax_top.plot((1 - d, 1 + d), (-d, +d), **kwargs)  # 右上
kwargs.update(transform=ax_bottom.transAxes) 
ax_bottom.plot((-d, +d), (1 - d, 1 + d), **kwargs)  # 左下
ax_bottom.plot((1 - d, 1 + d), (1 - d, 1 + d), **kwargs)  # 右下

# 8. 标签与图例
plt.xticks(rotation=45, ha='right', fontsize=10)
ax_bottom.set_ylabel('') # 清除默认标签
ax_top.set_ylabel('')
fig.text(0.04, 0.5, 'Safe / Unsafe Performance Ratio', va='center', rotation='vertical', fontsize=12)

# 手动添加分类图例
from matplotlib.patches import Patch
legend_elements = [Patch(facecolor=color_map[f], label=f) for f in unique_families]
ax_top.legend(handles=legend_elements, title="API Categories", loc='upper left', ncol=3)

plt.show()
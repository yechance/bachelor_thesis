import matplotlib.pyplot as plt
import seaborn as sns

def coverage_chart(df_model, df_baselines):
    fig, axes = plt.subplots(ncols=2, figsize=(16, 8))
    sns.lineplot(df_model, x="percentile", y="success_rate", ax=axes[0])
    axes[0].set_xlabel("percentile")
    axes[0].set_ylabel("coverage")
    axes[0].grid()

    sns.barplot(df_baselines, x="name", y="success_rate", ax=axes[1])
    axes[1].set_xlabel("baseline")
    axes[1].set_ylabel("coverage")

    plt.show()

def mae_chart(df_model, df_baselines):
    fig, axes = plt.subplots(ncols=2, figsize=(16, 8))
    sns.lineplot(df_model, x="success_rate", y="mae", hue="name", ax=axes[0])
    sns.scatterplot(df_baselines.iloc[:3], x="success_rate", y="mae", hue="name", ax=axes[0])
    axes[0].set_xlabel("percentile")
    axes[0].set_ylabel("MAE")
    axes[0].grid()

    sns.barplot(df_baselines, x="name", y="mae", ax=axes[1])
    axes[1].set_xlabel("baseline")
    axes[1].set_ylabel("MAE")

    plt.show()

def overhead_chart(df_model, df_baselines):
    fig, axes = plt.subplots(ncols=2, figsize=(16, 8))
    sns.lineplot(df_model, x="success_rate", y="overhead", hue="name", ax=axes[0])
    sns.scatterplot(df_baselines.iloc[:], x="success_rate", y="overhead", hue="name", ax=axes[0])
    axes[0].set_xlabel("percentile")
    axes[0].set_ylabel("overhead (ms)")
    axes[0].grid()

    sns.barplot(df_baselines, x="name", y="overhead", ax=axes[1])
    axes[1].set_xlabel("baseline")
    axes[1].set_ylabel("overhead (ms)")

    plt.show()
def linechart(x, y_list, x_label, y_label):
    plt.figure()

    for y in y_list:
        plt.plot(x, y, label=y)

    plt.xlabel(x_label)
    plt.ylabel(y_label)
    plt.legend()

    plt.show()

def linechart_two_axis(x, y1, y2, label):
    # Matplotlib을 사용하여 그래프 그리기
    fig, ax1 = plt.subplots(figsize=(12, 8))

    label_x, label_y1, label_y2 = label

    color = 'tab:red'
    ax1.set_xlabel(label_x)
    ax1.set_ylabel(label_y1, color=color)
    ax1.plot(x, y1, color=color)
    ax1.tick_params(axis='y', labelcolor=color)

    # ax1과 x축을 공유하는 두 번째 y축 생성
    ax2 = ax1.twinx()

    color = 'tab:blue'
    ax2.set_ylabel(label_y2, color=color)  # 두 번째 y축 라벨
    ax2.plot(x, y2, color=color)
    ax2.tick_params(axis='y', labelcolor=color)

    ax1.legend(loc=0)
    ax1.grid()

    # 그래프 제목과 범례 설정
    fig.tight_layout()  # 오른쪽 라벨이 잘리지 않도록 조정
    # # plt.title('Double Y Axis Example')
    fig.legend([label_y1, label_y2], loc='upper right')
    plt.show()

def dist(data, xlabel):
    f, ax = plt.subplots(figsize=(10, 4))
    sns.histplot(data, bins='auto', kde=True, ax=ax)
    ax.set_xlabel(xlabel)
    plt.show()
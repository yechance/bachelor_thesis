import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns

sns.set_theme(style="darkgrid", font_scale=2)

palette = {"success rate": "#2ca02c", "mae": "#ff7f0e", "quantile loss": "#d62728", "overhead": "#1f77b4"}
color = {'Success Rate': '#006400', 'MAE': "#FF8C00", 'Quantile Loss': "#d62728", 'Overhead': "#1f77b4"}
unit = {
    'Message Size': " (KB)",
    'Latency': " (ms)",
    'RTT': " (ms)",
    'Serialization Delay': " (ms)",
    "Combination Delay": " (ms)",
    'Sending Rate': " (MB/s)",
    'Success Rate': " (%)",
    'MAE': ' (ms)',
    'Quantile Loss': ' (ms)',
    'Overhead': '(ms)',
    'Relative Overhead': " (%)",
}


class Visualization:
    def __init__(self):
        None

    # 1. 전송률의 변화
    def change_over_time(self, savefile, df, y, y_limit, lim=200):

      f, ax1 = plt.subplots(figsize=(20, 8))
      sns.lineplot(df[: lim], x='ID', y=y, ax=ax1, label=y)
      plt.axhline(y_limit, color='black', linestyle='--')
      ax1.set_xlabel('Message ID')
      ax1.set_ylabel(unit[y])

      plt.savefig(savefile, format='pdf', bbox_inches='tight')
      # plt.show()

    # 2. 마진 분포와 선형 회귀
    def jointplot_latency(self, savefile, df, f, step=5):
        sns.jointplot(df[10::step], x=f, y='Latency',
                      kind="reg", truncate=False, height=7)
        plt.xlabel(f + unit[f])
        plt.ylabel('Latency' + unit['Latency'])

        plt.savefig(savefile, format='pdf', bbox_inches='tight')
        # plt.show()

    # 3. 히스토리컬 크기의 영향
    # 3-1. 성공률
    def over_data_size(self, savefile, df_group, x, y1, y2, color1, color2, y1_lim=None, y2_lim=None):
        fig, ax1 = plt.subplots()

        ax1.set_xlabel(x)
        ax1.set_ylabel(y1 + unit[y1], color=color1)
        sns.lineplot(df_group, x=x, y=y1, ax=ax1, color=color1, errorbar=None)
        ax1.tick_params(axis='y', labelcolor=color1)

        # 두 번째 Y 축 설정 (트윈 Y 축)
        ax2 = ax1.twinx()
        ax2.set_ylabel(y2 + unit[y2], color=color2)
        sns.lineplot(df_group, x=x, y=y2, ax=ax2, color=color2, errorbar=None)
        ax2.tick_params(axis='y', labelcolor=color2)

        if y1_lim is not None:
            ax1.set_ylim(y1_lim)

        if y2_lim is not None:
            ax2.set_ylim(y2_lim)

        plt.savefig(savefile, format='pdf', bbox_inches='tight')
        # plt.show()

    # 4. 업데이트 주기
    # def luk_diff_over_quantiles(self, savefile, df_group, use_failure_rate=False):
    #     plt.figure(figsize=(20, 12))
    # 
    #     g = sns.lineplot(df_group, x='Quantile', y='Quantile Error', label='Quantile Error')
    #     g = sns.lineplot(df_group, x='Quantile', y='MAE', label='MAE')
    #     g = sns.lineplot(df_group, x='Quantile', y='Quantile Loss', label='Quantile Loss')
    #     g = sns.lineplot(df_group, x='Quantile', y='Overhead', label='Overhead')
    # 
    #     plt.xlabel('Quantile')
    #     plt.ylabel('Performance Difference')
    # 
    #     plt.ylim(-3, 3)
    # 
    #     g.legend(loc='upper left', fontsize="x-small")  # , bbox_to_anchor = (1, 0.5), ncol = 1)
    #     plt.axhline(0, color='black', linestyle='--')
    # 
    #     plt.savefig(savefile, format='pdf', bbox_inches='tight')
    #     plt.show()

    # 5. 분위수에 따른 평가
    def quantile_err_over_quantiles(self, savefile, df_group, use_failure_rate=False):
        # plt.figure(figsize=(10, 6))

        g = None
        if use_failure_rate:
            g = sns.scatterplot(df_group, x='Quantile', y='Quantile Error', hue='Method', size='Failure Rate', sizes=(20, 200), alpha=0.6)

        else:
            g = sns.lineplot(df_group, x='Quantile', y='Quantile Error', hue='Method')

        plt.xlabel('Quantile')
        plt.ylabel('Quantile Error')

        plt.ylim(-0.2, 0.2)

        g.legend(loc='upper right', fontsize="x-small", bbox_to_anchor=(1, 0.5), ncol=1)
        plt.axhline(0, color='black', linestyle='--')

        plt.savefig(savefile, format='pdf', bbox_inches='tight')
        # plt.show()

    def over_quantiles(self, savefile, df_group, y, color, y_lim=None):
        # plt.figure(figsize=(20, 10))
        g = sns.lineplot(data=df_group, x='Quantile', y=y, hue='Method', color=color)
        plt.xlabel('Quantile')
        plt.ylabel(unit[y])
        if y_lim is not None:
            plt.ylim(y_lim)

        g.legend(loc='upper right', fontsize="small")  # , bbox_to_anchor = (1, 0.5), ncol = 1)

        plt.savefig(savefile, format='pdf', bbox_inches='tight')
        # plt.show()

    def err_distibution(self, savefile, df, xlim=(-1000,300)):
        plt.figure(figsize=(12, 14))
        sns.set_theme(style="whitegrid", font_scale=2)
        sns.histplot(data=df, x='Error', hue='Method', kde=True)

        plt.xlabel("Error (ms)")
        plt.xlim(xlim)
        plt.axvline(0, color='black', linestyle='--')

        plt.savefig(savefile, format='pdf', bbox_inches='tight')
        # plt.show()


    def ecdf(self, savefile, df, q=0.99, xlim=(-1000,300)):
        sns.set_theme(style="whitegrid", font_scale=2)
        plt.figure(figsize=(12, 14))
        sns.ecdfplot(df, x='Error', hue='Method')

        plt.xlabel("Error (ms)")
        plt.xlim(xlim)
        plt.axhline(q, color='red', linestyle='--')
        plt.axvline(0, color='black', linestyle='--')

        plt.savefig(savefile, format='pdf', bbox_inches='tight')
        # plt.show()

    def barplot(self, savefile, df, y):
        x = 'Method'
        sns.set_theme(style="whitegrid", font_scale=1.8)
        plt.figure(figsize=(12, 12))

        x_label = ''
        y_label = unit[y]

        if y == 's':
            y = x
            x = 'Success Rate - Percentile'
            df[x] = df['Success Rate'] - df['Quantile'] * 100
            x_label = x
            y_label = ''

        g = sns.catplot(
            data=df, kind="bar",
            x=x, y=y, hue="Quantile",
            errorbar="sd", palette="dark", alpha=.6, height=6
        )
        plt.xlabel(x_label)
        plt.ylabel(y_label)

        plt.savefig(savefile, format='pdf', bbox_inches='tight')
        # plt.show()

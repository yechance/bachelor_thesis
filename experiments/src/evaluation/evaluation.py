import pandas as pd
import numpy as np


def calculate_mid_results(df_list):
    for df in df_list:
        # 에러 계산
        df['Error'] = df['Latency'] - df['Estimated Latency']
        df['MAE'] = np.abs(df['Error'])

        df['Quantile Loss'] = df.apply(
            lambda row: ((1 - row['Quantile']) * row['MAE'] if row['Error'] < 0 else row['Quantile'] * row['MAE']), axis=1
        )

        df['Relative Overhead'] = (df['Overhead'] / df['Latency']) * 100  # 퍼센트
        df['Success Rate'] = (df['Latency'] <= df['Estimated Latency']) * 100


def evaluate(df):
    eval = df[['Method', 'Quantile', 'Success Rate', 'MAE', 'Quantile Loss', 'Overhead', 'Relative Overhead']].groupby(
        by=['Method', 'Quantile']).mean()
    return eval.reset_index()


def group_count(df):
    df.loc[:, 'count'] = 0
    return df[['Method', 'Quantile', 'count']].groupby(by=['Method', 'Quantile']).count()


def evaluate_valid_prediction(df):
    # 예측된 경우와 예측 자체를 실패한 경우 나눈다
    df_correct = df[df[['Estimated Latency']] > 0]
    df_fail = df[df[['Estimated Latency']] < 0]

    # 예측된 경우만 성능 평가를 하고
    calculate_mid_results([df_correct])
    df_correct_group = evaluate(df_correct)

    # 예측 실패 비율을 구하여 합친다
    merged_data = pd.merge(
        group_count(df_fail),
        group_count(df),
        on=['Method', 'Quantile'],
        suffixes=('_1', '_2')
    )
    merged_data['Failure Rate'] = np.round((merged_data['count_1'] / merged_data['count_2']), 3)

    group = df_correct_group.merge(
        merged_data,
        on=['Method', 'Quantile'],
        how='left'
    )
    group.fillna(0, inplace=True)
    group = group.reset_index()
    return group
import pandas as pd

K = 1000
M = 1000000

K = 1_000

# ID,Message Size,Sending Rate,RTT,Latency
# ,Method,Quantile,Latency,Estimated Latency,Overhead,Historical Data Size
data_col = {
    'i': 'ID',
    'm': 'Message Size',
    's': 'Sending Rate',
    'r': 'RTT',
    'l': 'Latency',
    's_d': 'Serialization Delay',
    'c_d': 'Combination Delay',
    'ma_s': 'SMA Sending Rate',
    'ma_r': 'SMA RTT',
}

eval_col = {
    'm': 'Method',
    'q': 'Quantile',
    'l': 'Latency',
    'l_pred': 'Estimated Latency',
    'c_d': 'Combination Delay',
    'msg': 'Message Size',
    'ft': 'feature',
    'o': 'Overhead',
    'r_o': 'Relative Overhead',
    'h': 'Historical Data Size',
    's': 'Success Rate',
    'q_loss': 'Quantile Loss',
    'e': 'Error',
    'ae': 'Absolute Error',
    'mae': 'MAE',
    'p': 'Update Period',
}

unit = {
    'm' : " (KB)",
    's' : " (MB per sec)",
    'l' : " (ms)",
    'r' : " (ms)",
    's_d': " (ms)",
    "c_d": " (ms)",
    'ma_s': 'MB per sec',
    'ms': " (ms)",
    'KB': " (KB)",
    "MBps": " (MB per sec)",
}


def data_col_by_keys(keys):
    return [data_col[key] for key in keys]


def eval_col_by_keys(keys):
    return [eval_col[key] for key in keys]


def load_data(csv_file):
    return pd.read_csv(csv_file)


def preproc(df_origin):
    col_i, col_m, col_s, col_r, col_l, col_s_d, col_c_d = data_col_by_keys(['i', 'm', 's', 'r', 'l', 's_d', 'c_d'])
    # sort the dataframe by message id
    df = df_origin[[col_i, col_m, col_s, col_r, col_l]].sort_values(data_col['i'])

    # features
    # Message Size : B -> KB
    # Sending rate : Bps -> KBps -> KB per ms
    # RTT : micro sec -> ms
    df[col_m] = df[col_m] / K
    df[col_s] = (df[col_s] / K) / K
    df[col_r] = df[col_r] / K
    # label
    # latency : micro sec -> ms
    df[col_l] = df[col_l] / K

    return df



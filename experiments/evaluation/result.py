import numpy as np
import pandas as pd
from evaluation.preprocess import *

class ExperimentResult:
    def __init__(self, num_result, model="model"):
        self.model_name = model
        self.quantile = np.zeros(num_result)
        self.y = np.zeros(num_result)
        self.y_pred = np.zeros(num_result)
        self.overhead = np.zeros(num_result)
        self.msg_size = np.zeros(num_result)
        self.feature = np.zeros(num_result)

        self.num_result = num_result
        self.idx = 0

    def add(self, q, y, y_pred, overhead, msg_size, ft):
        if self.idx >= self.num_result:
            return None
        self.quantile[self.idx] = q
        self.y[self.idx] = y
        self.y_pred[self.idx] = y_pred
        self.overhead[self.idx] = overhead
        self.msg_size[self.idx] = msg_size
        self.feature[self.idx] = ft
        self.idx += 1

    def to_df(self, num_h_data=200):
        col_m, col_q, col_l , col_l_pred, col_o, col_h, col_msg, col_ft = eval_col_by_keys(['m', 'q', 'l', 'l_pred', 'o', 'h', 'msg', 'ft'])

        df = pd.DataFrame(
            data=np.stack([self.quantile, self.y, self.y_pred, self.overhead, self.msg_size, self.feature], axis=1),
            columns=[col_q, col_l , col_l_pred, col_o, col_msg, col_ft]
        )
        df[col_m] = self.model_name
        df[col_h] = num_h_data
        return df









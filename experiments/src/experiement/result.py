import numpy as np
import pandas as pd


class ExperimentResult:
    def __init__(self, num_result, model="model"):
        self.model_name = model
        self.quantile = np.zeros(num_result)
        self.y = np.zeros(num_result)
        self.y_pred = np.zeros(num_result)
        self.overhead = np.zeros(num_result)

        self.num_result = num_result
        self.idx = 0

    def add(self, q, y, y_pred, overhead):
        if self.idx >= self.num_result:
            return None
        self.quantile[self.idx] = q
        self.y[self.idx] = y
        self.y_pred[self.idx] = y_pred
        self.overhead[self.idx] = overhead
        self.idx += 1

    def to_df(self, num_h_data=200):
        df = pd.DataFrame(
            data=np.stack([self.quantile, self.y, self.y_pred, self.overhead], axis=1),
            columns=['Quantile', 'Latency', 'Estimated Latency', 'Overhead']
        )
        df['Method'] = self.model_name
        return df









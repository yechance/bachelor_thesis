import pandas as pd
import numpy as np
from sklearn.linear_model import QuantileRegressor
from prediction.historical_data import HistoricalData
from prediction.lookup_table import LookupTable


class PredictionModel:
    def __init__(self, has_luk=True, update_period=40, num_data=400, use_mean=True):
        self.historical_data = HistoricalData(num_data, use_mean)
        self.has_luk = has_luk
        self.luk = LookupTable(update_period) if has_luk else None

    def percentile_latency(self, quantile):
        combination_feature = self.historical_data.next_feature

        if self.has_luk:
            # try to get the parameters
            if self.luk.valid(quantile):
                params = self.luk.get_parameters(quantile)
                return params['intercept_'] + params['coef_'] * combination_feature

        quantile_regressor = QuantileRegressor(quantile=quantile, alpha=0, solver='highs')
        quantile_regressor.fit(self.historical_data.feature_vec(), self.historical_data.y_vec())

        if self.has_luk:
            self.luk.set_parameters(quantile, quantile_regressor.intercept_, quantile_regressor.coef_)

        return quantile_regressor.predict(np.array([combination_feature]).reshape(-1, 1))

    def add_feature(self, msg_size, sending_rate=1, rtt=0):
        self.historical_data.add_features(msg_size, sending_rate, rtt)

    def add_actual_latency(self, latency):
        self.historical_data.add_label(latency)
        if self.has_luk:
            self.luk.increase_update_timer()

import numpy as np

from sklearn.linear_model import LinearRegression
from prediction.historical_data import HistoricalData
def simple_latency_from_features(msg_size, sending_rate=1, rtt=0):
    return msg_size / sending_rate + rtt


class MeanBaseline:
    # features : msg_size, sending_rate, rtt
    def __init__(self):
        self.historical_data = HistoricalData()

    def mean_latency(self):
        return self.historical_data.label.mean()

    def mean_latency_message_size(self, msg_size):
        return self.historical_data.sample_latency(msg_size).mean()

    def combination_delays(self, msg_size, sending_rate, rtt):
        return self.historical_data.current_condition()

    def mean_latency_linear_regression(self, msg_size):
        # feature reduction
        combination_features = msg_size

        linear_regressor = LinearRegression()
        linear_regressor.fit(self.historical_data.feature.reshape(-1, 1), self.historical_data.label)

        return linear_regressor.predict(np.array(combination_features).reshape(-1, 1))

    def add_features(self, msg_size, sending_rate, rtt):
        self.historical_data.add_features(msg_size, sending_rate, rtt)

    def add_actual_latency(self, latency):
        self.historical_data.add_label(latency)


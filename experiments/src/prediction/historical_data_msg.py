import numpy as np


class HistoricalDataNaive:
    def __init__(self, num_data=400):
        self.num_data = num_data

        self.feature = np.zeros(num_data)
        self.label = np.zeros(num_data)
        self.idx = 0
        self.next_feature = 0

    def add_features(self, msg_size, sending_rate=1, rtt=0):
        self.feature[self.idx] = msg_size
        self.next_feature = msg_size

    def add_label(self, label):
        self.feature[self.idx] = self.next_feature
        self.label[self.idx] = label
        self.idx = (self.idx + 1) % self.num_data
        return self.idx

    def current_condition(self):
        return np.array([self.feature[self.idx]]).reshape(-1, 1)

    def feature_vec(self):
        return self.feature.reshape(-1, 1)

    def y_vec(self):
        return self.label

    def sample_latency(self, feature, interval_rate=0.2):
        min_interval = feature * (1 - interval_rate / 2)
        max_interval = feature * (1 + interval_rate / 2)

        condition = (self.feature > min_interval) & (self.feature < max_interval)
        indices = np.where(condition)

        return self.label[indices]


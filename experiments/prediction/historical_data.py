import numpy as np


class HistoricalData:
    def __init__(self, num_data, use_mean=True):
        self.num_data = num_data
        self.num_mean_data = 20
        self.use_mean = use_mean

        self.message_size = np.zeros(num_data)
        self.sending_rate = np.zeros(num_data)
        self.rtt = np.zeros(num_data)
        self.ma_sending_rate = np.ones(self.num_data)
        self.ma_rtt = np.zeros(self.num_data)

        self.idx_mean = 0
        self.past_sending_rate_for_mean = np.ones(self.num_mean_data)
        self.past_rtt_for_mean = np.zeros(self.num_mean_data)

        self.feature = np.zeros(num_data)
        self.label = np.zeros(num_data)
        self.idx = 0
        self.next_feature = 0

    def add_features(self, msg_size, sending_rate, rtt):
        self.message_size[self.idx] = msg_size
        self.sending_rate[self.idx] = sending_rate
        self.rtt[self.idx] = rtt

        if self.use_mean:
            # moving average
            self.past_sending_rate_for_mean[self.idx_mean] = sending_rate
            self.past_rtt_for_mean[self.idx_mean] = rtt
            self.idx_mean = (self.idx_mean + 1) % self.num_mean_data

            self.ma_sending_rate[self.idx] = self.past_sending_rate_for_mean.mean()
            self.ma_rtt[self.idx] = self.past_rtt_for_mean.mean()

            self.next_feature = msg_size / self.ma_sending_rate[self.idx] + self.ma_rtt[self.idx]
        else:
            self.ma_sending_rate[self.idx] = sending_rate
            self.ma_rtt[self.idx] = rtt
            self.next_feature = msg_size / sending_rate + rtt

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

    def sample_latency(self, feature, interval_rate=0.1):
        min_interval = feature * (1 - interval_rate / 2)
        max_interval = feature * (1 + interval_rate / 2)

        condition = (self.feature > min_interval) & (self.feature < max_interval)
        indices = np.where(condition)

        return self.label[indices]


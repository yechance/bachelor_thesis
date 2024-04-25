import numpy as np

from src.prediction.historical_data_msg import HistoricalDataNaive
from sklearn.mixture import GaussianMixture

import scipy.stats as stats


class QuantileBaseline:
    # features : msg_size
    def __init__(self, num_data=400):
        self.historical_data = HistoricalDataNaive(num_data)

    def gaussian(self, quantile, feature, use_sampling=False):
        try:
            # sample
            latency_sample = self.historical_data.sample_latency(feature) if use_sampling else self.historical_data.y_vec()
            if len(latency_sample) < 0:
                return -1
            mu, std = stats.norm.fit(latency_sample)
            return stats.norm.ppf(quantile, loc=mu, scale=std)
        except:
            return -1

    def lognorm(self, quantile, feature, use_sampling=False):
        try:
            # sample
            latency_sample = self.historical_data.sample_latency(feature) if use_sampling else self.historical_data.y_vec()

            if len(latency_sample) < 2:
                return -1
            shape, loc, scale = stats.lognorm.fit(latency_sample, floc=0)
            return stats.lognorm.ppf(quantile, s=shape, loc=loc, scale=scale)
        except:
            return -1

    def gmm(self, quantile, feature, use_sampling=False):
        try:
            # sample
            latency_sample = self.historical_data.sample_latency(feature) if use_sampling else self.historical_data.y_vec()
            if len(latency_sample) < 2:
                return -1
            gmm = GaussianMixture(n_components=2, random_state=42)
            gmm.fit(latency_sample.reshape(-1, 1))

            samples = gmm.sample(n_samples=10000)[0].flatten()
            return np.percentile(samples, 100 * quantile)
        except:
            return -1

    def add_features(self, msg_size, sending_rate, rtt):
        self.historical_data.add_features(msg_size, sending_rate, rtt)

    def add_actual_latency(self, latency):
        self.historical_data.add_label(latency)


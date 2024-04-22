import numpy as np

from prediction.historical_data import HistoricalData
from sklearn.ensemble import RandomForestClassifier
from sklearn.preprocessing import KBinsDiscretizer
from sklearn.mixture import GaussianMixture

import scipy.stats as stats


class QuantileBaseline:
    # features : msg_size, sending_rate, rtt
    def __init__(self, num_data=500):
        self.historical_data = HistoricalData(num_data)

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

    # def histogram(self, quantile, feature, num_bins):
    #     X = np.stack([
    #             self.historical_data.message_size,
    #             self.historical_data.ma_sending_rate,
    #             self.historical_data.ma_rtt
    #         ], axis=1)
    #     y = self.historical_data.y_vec()
    #
    #     # Discretize the dependent variable space for the histogram estimator
    #     discretizer = KBinsDiscretizer(n_bins=num_bins, encode='ordinal', strategy='uniform')
    #     y_binned = discretizer.fit_transform(y.reshape(-1, 1)).flatten()
    #
    #     # Train a Random Forest classifier to predict the probability of each bin
    #     classifier = RandomForestClassifier(n_estimators=100, random_state=42)
    #     classifier.fit(X, y_binned)
    #
    #     # class probabilities
    #     probabilities = classifier.predict_proba(X)
    #
    #     # Calculate conditional density from class probabilities
    #     bin_edges = discretizer.bin_edges_[0]
    #     bin_widths = np.diff(bin_edges)
    #     conditional_density = probabilities / bin_widths
    #
    #     # Calculate the CDF from the conditional density
    #     cdf = np.cumsum(conditional_density, axis=1)
    #
    #     index = np.argmax(cdf >= quantile, axis=1)
    #     return bin_edges[index]


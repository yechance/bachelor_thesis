import time

from src.prediction.prediction_model import PredictionModel
from src.prediction.baseline_percentile import *
from src.experiement.result import *

import pandas as pd

K = 1_000
M = 1_000_000

# ID,Message Size,Sending Rate,RTT,Latency
# ,Method,Quantile,Latency,Estimated Latency,Overhead,Historical Data Size
data_column = ['ID', 'Message Size', 'Sending Rate', 'RTT', 'Latency', 'Serialization Delay', 'Combination Delay']
eval_colum = ['Method', 'Quantile', 'Latency', 'Estimated Latency', 'Overhead']
metrics = ['Success Rate', 'MAE', 'Quantile Loss', 'Overhead', 'Relative Overhead']
eval_help_idx = ['Historical Data Size', 'Update Period', 'Error', 'Absolute Error', 'Quantile Error']


def load_data(csv_file):
    return pd.read_csv(csv_file)


def preproc(df_origin):
    # sort the dataframe by message id
    df = df_origin[['ID', 'Message Size', 'Sending Rate', 'RTT', 'Latency']].sort_values('ID')

    # features
    # Message Size : B -> KB
    # Sending rate : Bps -> KBps -> KB per ms
    # RTT : micro sec -> ms
    df['Message Size'] = df['Message Size'] / K
    df['Sending Rate'] = (df['Sending Rate'] / K) / K
    df['RTT'] = df['RTT'] / K
    # label
    # latency : micro sec -> ms
    df['Latency'] = df['Latency'] / K

    return df


class Experiment:
    def __init__(self, csv_file, num_data=5200, num_h_data=400, start=400):
        df = preproc(pd.read_csv(csv_file))
        self.measurements = df
        self.num_measurements = len(self.measurements)

        # data set for evaluations
        self.dataset = self.measurements.iloc[start:, :]
        self.num_dataset = len(self.dataset)
        self.num_test_data = self.num_dataset-num_h_data
        self.num_h_data = num_h_data

    def performance_different_data_size(self, data_size_arr, quantile=0.95):
        experimental_results = pd.DataFrame(columns=['Method', 'Quantile', 'Latency', 'Estimated Latency', 'Overhead'])

        for data_size in data_size_arr:
            prediction_model = PredictionModel(has_luk=False, update_period=0, num_data=data_size)
            # baseline = QuantileBaseline(data_size)

            for i in range(data_size):
                current_data = self.dataset.iloc[i]
                m, s, r, l = current_data[['Message Size', 'Sending Rate', 'RTT', 'Latency']]
                prediction_model.add_feature(m, s, r)
                prediction_model.add_actual_latency(l)

            result = ExperimentResult(self.num_test_data, 'Quantile Reg')

            for i in range(data_size, 5200):
                current_data = self.dataset.iloc[i]
                m, s, r, l = current_data[['Message Size', 'Sending Rate', 'RTT', 'Latency']]

                prediction_model.add_feature(m, s, r)

                start = time.time()
                estimated_latency = prediction_model.percentile_latency(quantile)
                prediction_model.add_actual_latency(l)
                result.add(quantile, l, estimated_latency, (time.time() - start) * 1000)

            experimental_results = pd.concat([
                experimental_results,
                result.to_df(data_size),
            ])

        return experimental_results

    def period_update(self, periods, quantile=0.5):
        experimental_results = pd.DataFrame(
            columns=['Method', 'Quantile', 'Latency', 'Estimated Latency', 'Overhead', 'Update Period']
        )

        for p in periods:
            prediction_model = PredictionModel(has_luk=True, update_period=p)
            for i in range(self.num_h_data):
                current_data = self.dataset.iloc[i]
                m, s, r, l = current_data[['Message Size', 'Sending Rate', 'RTT', 'Latency']]
                prediction_model.add_feature(m, s, r)
                prediction_model.add_actual_latency(l)

            result = ExperimentResult(self.num_test_data)
            for i in range(self.num_h_data, self.num_dataset):
                current_data = self.dataset.iloc[i]
                m, s, r, l = current_data[['Message Size', 'Sending Rate', 'RTT', 'Latency']]

                prediction_model.add_feature(m, s, r)
                ft = prediction_model.historical_data.next_feature

                start = time.time()
                estimated_latency = prediction_model.percentile_latency(quantile)
                overhead = (time.time() - start) * 1000  # ms
                prediction_model.add_actual_latency(l)
                result.add(quantile, l, estimated_latency, overhead)
            df_result = result.to_df()
            df_result['Update Period'] = p
            experimental_results = pd.concat([experimental_results, df_result])
        return experimental_results

    def model_by_percentile(self, list_quantile, has_luk=True, use_mixed_feature=True, name="model"):
        # Results
        experimental_results = pd.DataFrame(columns=['Method', 'Quantile', 'Latency', 'Estimated Latency', 'Overhead'])
        # Evaluate
        for quantile in list_quantile:
            # Prediction Model
            model = PredictionModel(has_luk=has_luk)
            # initiation
            for i in range(self.num_h_data):
                current_data = self.dataset.iloc[i]
                m, l = current_data[['Message Size', 'Latency']]
                s, r = current_data[['Sending Rate', 'RTT']] if use_mixed_feature else (1, 0)
                model.add_feature(m, s, r)
                model.add_actual_latency(l)

            result = ExperimentResult(self.num_test_data, "quantile regression")
            for i in range(self.num_h_data, self.num_h_data + self.num_test_data):
                current_data = self.dataset.iloc[i]
                m, l = current_data[['Message Size', 'Latency']]
                s, r = current_data[['Sending Rate', 'RTT']] if use_mixed_feature else (1, 0)
                model.add_feature(m, s, r)

                start = time.time()
                estimated_latency = model.percentile_latency(quantile)
                overhead = (time.time() - start) * 1000  # ms
                model.add_actual_latency(l)
                result.add(quantile, l, estimated_latency, overhead)

            experimental_results = pd.concat([experimental_results, result.to_df()])
        return experimental_results

    def compared_baselines_quantile(self, list_quantile, use_sampling=True):
        # Results
        experimental_results = pd.DataFrame(columns=['Method', 'Quantile', 'Latency', 'Estimated Latency', 'Overhead'])
        baseline = QuantileBaseline()

        # initiation
        for i in range(self.num_h_data):
            current_data = self.dataset.iloc[i]
            m, s, r, l = current_data[['Message Size', 'Sending Rate', 'RTT', 'Latency']]
            baseline.add_features(m, s, r)
            baseline.add_actual_latency(l)

        num_quantiles = len(list_quantile)

        # Evaluate
        result_gaussian = ExperimentResult(self.num_test_data * num_quantiles, 'gaussian')
        result_lognorm = ExperimentResult(self.num_test_data * num_quantiles, 'lognorm')
        result_gmm = ExperimentResult(self.num_test_data * num_quantiles, 'gmm')

        for i in range(self.num_h_data, self.num_h_data + self.num_test_data):
            current_data = self.dataset.iloc[i]
            m, s, r, l = current_data[['Message Size', 'Sending Rate', 'RTT', 'Latency']]
            baseline.add_features(m, s, r)

            feature = baseline.historical_data.next_feature

            for q in list_quantile:
                start = time.time()
                estimated_latency = baseline.gaussian(q, feature, use_sampling)
                result_gaussian.add(q, l, estimated_latency, (time.time() - start) * 1000)

                start = time.time()
                estimated_latency = baseline.lognorm(q, feature, use_sampling)
                result_lognorm.add(q, l, estimated_latency, (time.time() - start) * 1000)

                start = time.time()
                estimated_latency = baseline.gmm(q, feature, use_sampling)
                result_gmm.add(q, l, estimated_latency, (time.time() - start) * 1000)

            baseline.add_actual_latency(l)

        experimental_results = pd.concat([
            experimental_results,
            result_gaussian.to_df(),
            result_lognorm.to_df(),
            result_gmm.to_df()
        ])

        return experimental_results

    def pred_latency(self, quantile=0.99, use_sampling=False):
        model = PredictionModel(has_luk=True)
        baseline = QuantileBaseline(400)

        # initiation
        for i in range(self.num_h_data):
            current_data = self.dataset.iloc[i]
            m, s, r, l = current_data[['Message Size', 'Sending Rate', 'RTT', 'Latency']]
            model.add_feature(m, s, r)
            model.add_actual_latency(l)
            baseline.add_features(m, s, r)
            baseline.add_actual_latency(l)

        result = ExperimentResult(self.num_test_data, "Quantile Reg")
        result_gaussian = ExperimentResult(self.num_test_data, 'Gaussian')
        result_lognorm = ExperimentResult(self.num_test_data, 'Lognorm')
        result_gmm = ExperimentResult(self.num_test_data, 'Gmm')

        for i in range(self.num_h_data, self.num_dataset):
            current_data = self.dataset.iloc[i]
            m, s, r, l = current_data[['Message Size', 'Sending Rate', 'RTT', 'Latency']]
            model.add_feature(m, s, r)
            baseline.add_features(m, s, r)

            ft_baseline = baseline.historical_data.next_feature

            start = time.time()
            estimated_latency = model.percentile_latency(quantile)
            result.add(quantile, l, estimated_latency, (time.time() - start) * 1000)

            start = time.time()
            estimated_latency = baseline.gaussian(quantile, ft_baseline, use_sampling)
            result_gaussian.add(quantile, l, estimated_latency, (time.time() - start) * 1000)

            start = time.time()
            estimated_latency = baseline.lognorm(quantile, ft_baseline, use_sampling)
            result_lognorm.add(quantile, l, estimated_latency, (time.time() - start) * 1000)

            start = time.time()
            estimated_latency = baseline.gmm(quantile, ft_baseline, use_sampling)
            result_gmm.add(quantile, l, estimated_latency, (time.time() - start) * 1000)

            model.add_actual_latency(l)
            baseline.add_actual_latency(l)

        return pd.concat([
            result.to_df(),
            result_gaussian.to_df(),
            result_lognorm.to_df(),
            result_gmm.to_df()
        ])
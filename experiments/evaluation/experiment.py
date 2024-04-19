import time

import numpy as np
import pandas as pd

from evaluation.preprocess import *
from prediction.prediction_model import PredictionModel
from evaluation.evaluation import Evaluation
from evaluation.baseline import *
from evaluation.baseline_percentile import *
from evaluation.result import *

class Experiment:
    def __init__(self, csv_file, num_test_data=4900, num_h_data=200):
        df = preproc(pd.read_csv(csv_file))
        self.measurements = df
        self.num_measurements = len(self.measurements)

        # data set for evaluations
        self.dataset = self.measurements.iloc[:, :]
        self.num_dataset = num_h_data + num_test_data
        self.num_test_data = num_test_data
        self.num_h_data = num_h_data

    def performance_different_data_size(self, data_size_arr, quantile=0.9, use_sampling=True):
        experimental_results = pd.DataFrame(columns=eval_col_by_keys(['m', 'q', 'l', 'l_pred', 'o']))

        for data_size in data_size_arr:
            prediction_model = PredictionModel(has_luk=False, update_period=0, num_data=data_size)
            baseline = QuantileBaseline(data_size)

            for i in range(data_size):
                current_data = self.dataset.iloc[i]
                m, s, r, l = current_data[data_col_by_keys(['m', 's', 'r', 'l'])]
                prediction_model.add_feature(m, s, r)
                prediction_model.add_actual_latency(l)
                baseline.add_features(m, s, r)
                baseline.add_actual_latency(l)

            result = ExperimentResult(self.num_test_data, 'Quantile Reg')
            result_gaussian = ExperimentResult(self.num_test_data, 'Gaussian')
            result_lognorm = ExperimentResult(self.num_test_data, 'Lognorm')
            result_gmm = ExperimentResult(self.num_test_data, 'GMM')

            for i in range(data_size, self.num_dataset):
                current_data = self.dataset.iloc[i]
                m, s, r, l = current_data[data_col_by_keys(['m', 's', 'r', 'l'])]

                baseline.add_features(m, s, r)
                prediction_model.add_feature(m, s, r)

                start = time.time()
                estimated_latency = prediction_model.percentile_latency(quantile)
                overhead = (time.time() - start) * 1000  # ms
                prediction_model.add_actual_latency(l)
                result.add(quantile, l, estimated_latency, overhead)

                feature = baseline.historical_data.next_feature

                start = time.time()
                estimated_latency = baseline.gaussian(quantile, feature, use_sampling)
                result_gaussian.add(quantile, l, estimated_latency, (time.time() - start) * 1000)

                start = time.time()
                estimated_latency = baseline.lognorm(quantile, feature, use_sampling)
                result_lognorm.add(quantile, l, estimated_latency, (time.time() - start) * 1000)

                start = time.time()
                estimated_latency = baseline.gmm(quantile, feature, use_sampling)
                result_gmm.add(quantile, l, estimated_latency, (time.time() - start) * 1000)

                baseline.add_actual_latency(l)

            experimental_results = pd.concat([
                experimental_results,
                result.to_df(data_size),
                result_gaussian.to_df(data_size),
                result_lognorm.to_df(data_size),
                result_gmm.to_df(data_size)
            ])

        return experimental_results

    def period_update(self, periods, quantile=0.5):
        experimental_results = pd.DataFrame(columns=eval_col_by_keys(['m', 'q', 'l', 'l_pred', 'o', 'p']))

        for p in periods:
            prediction_model = PredictionModel(has_luk=True, update_period=p)
            for i in range(self.num_h_data):
                current_data = self.dataset.iloc[i]
                m, s, r, l = current_data[data_col_by_keys(['m', 's', 'r', 'l'])]
                prediction_model.add_feature(m, s, r)
                prediction_model.add_actual_latency(l)

            result = ExperimentResult(self.num_test_data)
            for i in range(self.num_h_data, self.num_dataset):
                current_data = self.dataset.iloc[i]
                m, s, r, l = current_data[data_col_by_keys(['m', 's', 'r', 'l'])]

                prediction_model.add_feature(m, s, r)
                start = time.time()
                estimated_latency = prediction_model.percentile_latency(quantile)
                overhead = (time.time() - start) * 1000  # ms
                prediction_model.add_actual_latency(l)
                result.add(quantile, l, estimated_latency, overhead)
            df_result = result.to_df()
            df_result[eval_col['p']] = p
            experimental_results = pd.concat([experimental_results, df_result])
        return experimental_results

    def performance_model_by_percentile(self, list_quantile, has_luk=True, use_mixed_feature=True, name="model"):
        # Results
        experimental_results = pd.DataFrame(columns=eval_col_by_keys(['m', 'q', 'l', 'l_pred', 'o', 'msg', 'ft']))
        # Evaluate
        for quantile in list_quantile:
            # Prediction Model
            model = PredictionModel(has_luk=has_luk)
            # initiation
            for i in range(self.num_h_data):
                current_data = self.dataset.iloc[i]
                m, l = current_data[data_col_by_keys(['m', 'l'])]
                s, r = current_data[data_col_by_keys(['s', 'r'])] if use_mixed_feature else (1, 0)
                model.add_feature(m, s, r)
                model.add_actual_latency(l)

            result = ExperimentResult(self.num_test_data, "quantile regression")
            for i in range(self.num_h_data, self.num_h_data + self.num_test_data):
                current_data = self.dataset.iloc[i]
                m, l = current_data[data_col_by_keys(['m', 'l'])]
                s, r = current_data[data_col_by_keys(['s', 'r'])] if use_mixed_feature else (1, 0)
                model.add_feature(m, s, r)
                ft = model.historical_data.next_feature

                start = time.time()
                estimated_latency = model.percentile_latency(quantile)
                overhead = (time.time() - start) * 1000  # ms
                model.add_actual_latency(l)
                result.add(quantile, l, estimated_latency, overhead, m, ft)

            experimental_results = pd.concat([experimental_results, result.to_df()])
        return experimental_results

    def performance_compared_baselines(self, quantile):
        # Results
        experimental_results = pd.DataFrame(columns=eval_col_by_keys(['m', 'q', 'l', 'l_pred', 'o', 'msg', 'ft']))
        baseline = MeanBaseline()

        # initiation
        for i in range(self.num_h_data):
            current_data = self.dataset.iloc[i]
            m, s, r, l = current_data[data_col_by_keys(['m', 's', 'r', 'l'])]
            baseline.add_features(m, s, r)
            baseline.add_actual_latency(l)

        # Evaluate
        result_mean_latency = ExperimentResult(self.num_test_data, 'mean_latency')
        result_mean_sampled_latency = ExperimentResult(self.num_test_data, 'mean_sampled_latency')
        result_serialization_delay = ExperimentResult(self.num_test_data, 'serialization_delay')
        result_linear_regression = ExperimentResult(self.num_test_data, 'linear_regression')
        for i in range(self.num_h_data, self.num_h_data + self.num_test_data):
            current_data = self.dataset.iloc[i]
            m, s, r, l = current_data[data_col_by_keys(['m', 's', 'r', 'l'])]

            baseline.add_features(m, s, r)
            ft = baseline.historical_data.feature
            start = time.time()
            estimated_latency = baseline.mean_latency()
            overhead = (time.time() - start) * 1000  # ms
            result_mean_latency.add_result(0, l, estimated_latency, overhead, m, ft)

            start = time.time()
            estimated_latency = baseline.mean_latency_message_size(m)
            overhead = (time.time() - start) * 1000  # ms
            result_mean_sampled_latency.add_result(0, l, estimated_latency, overhead, m, ft)

            start = time.time()
            estimated_latency = baseline.latency_by_throughput(m, s)
            overhead = (time.time() - start) * 1000  # ms
            result_serialization_delay.add_result(0, l, estimated_latency, overhead, m, ft)

            start = time.time()
            estimated_latency = baseline.mean_latency_linear_regression(m)
            overhead = (time.time() - start) * 1000  # ms
            result_linear_regression.add_result(0, l, estimated_latency, overhead, m, ft)

            baseline.add_actual_latency(l)

        experimental_results = pd.concat([
            experimental_results,
            result_mean_latency.to_df(),
            result_mean_sampled_latency.to_df(),
            result_serialization_delay.to_df(),
            result_linear_regression.to_df()
        ])

        return experimental_results

    def performance_compared_baselines_quantile(self, list_quantile, use_sampling=True):
        # Results
        experimental_results = pd.DataFrame(columns=eval_col_by_keys(['m', 'q', 'l', 'l_pred', 'o']))
        baseline = QuantileBaseline()

        # initiation
        for i in range(self.num_h_data):
            current_data = self.dataset.iloc[i]
            m, s, r, l = current_data[data_col_by_keys(['m', 's', 'r', 'l'])]
            baseline.add_features(m, s, r)
            baseline.add_actual_latency(l)

        num_quantiles = len(list_quantile)

        # Evaluate
        result_gaussian = ExperimentResult(self.num_test_data * num_quantiles, 'gaussian')
        result_lognorm = ExperimentResult(self.num_test_data * num_quantiles, 'lognorm')
        result_gmm = ExperimentResult(self.num_test_data * num_quantiles, 'gmm')
        for i in range(self.num_h_data, self.num_h_data + self.num_test_data):
            current_data = self.dataset.iloc[i]
            m, s, r, l = current_data[data_col_by_keys(['m', 's', 'r', 'l'])]
            baseline.add_features(m, s, r)

            feature = baseline.historical_data.next_feature

            for q in list_quantile:
                start = time.time()
                estimated_latency = baseline.gaussian(q, feature, use_sampling)
                result_gaussian.add(q, l, estimated_latency, (time.time() - start) * 1000, m, feature)

                start = time.time()
                estimated_latency = baseline.lognorm(q, feature, use_sampling)
                result_lognorm.add(q, l, estimated_latency, (time.time() - start) * 1000, m, feature)

                start = time.time()
                estimated_latency = baseline.gmm(q, feature, use_sampling)
                result_gmm.add(q, l, estimated_latency, (time.time() - start) * 1000, m, feature)

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
        baseline = QuantileBaseline()

        # initiation
        for i in range(self.num_h_data):
            current_data = self.dataset.iloc[i]
            m, s, r, l = current_data[data_col_by_keys(['m', 's', 'r', 'l'])]
            model.add_feature(m, s, r)
            model.add_actual_latency(l)
            baseline.add_features(m, s, r)
            baseline.add_actual_latency(l)

        result = ExperimentResult(self.num_test_data, "Quantile Reg (luk)")
        result_gaussian = ExperimentResult(self.num_test_data, 'Gaussian')
        result_lognorm = ExperimentResult(self.num_test_data, 'Lognorm')
        result_gmm = ExperimentResult(self.num_test_data, 'Gmm')

        for i in range(self.num_h_data, self.num_h_data + self.num_test_data):
            current_data = self.dataset.iloc[i]
            m, s, r, l = current_data[data_col_by_keys(['m', 's', 'r', 'l'])]
            model.add_feature(m, s, r)
            baseline.add_features(m, s, r)

            ft = model.historical_data.next_feature
            start = time.time()
            estimated_latency = model.percentile_latency(quantile)
            result.add(quantile, l, estimated_latency, (time.time() - start) * 1000, m, ft)

            start = time.time()
            estimated_latency = baseline.gaussian(quantile, ft, use_sampling)
            result_gaussian.add(quantile, 0, estimated_latency, (time.time() - start) * 1000, m, ft)

            start = time.time()
            estimated_latency = baseline.lognorm(quantile, ft, use_sampling)
            result_lognorm.add(quantile, 0, estimated_latency, (time.time() - start) * 1000, m, ft)

            start = time.time()
            estimated_latency = baseline.gmm(quantile, ft, use_sampling)
            result_gmm.add(quantile, 0, estimated_latency, (time.time() - start) * 1000, m, ft)

            model.add_actual_latency(l)
            baseline.add_actual_latency(l)

        c_l_pred, c_l= eval_col_by_keys(['l_pred', 'l'])

        df_model = result.to_df()[eval_col_by_keys(['m', 'l', 'l_pred'])]
        experimental_results = df_model[eval_col_by_keys(['m', 'l'])].copy()
        experimental_results[eval_col['m']] = "GT"
        df_model = df_model.drop(columns=[c_l]).rename(columns={c_l_pred: c_l})
        df_qaussian = result_gaussian.to_df()[eval_col_by_keys(['m', 'l_pred', 'o'])].rename(columns={c_l_pred: c_l})
        df_lognorm = result_lognorm.to_df()[eval_col_by_keys(['m', 'l_pred'])].rename(columns={c_l_pred: c_l})
        df_qmm = result_gmm.to_df()[eval_col_by_keys(['m', 'l_pred'])].rename(columns={c_l_pred: c_l})

        return pd.concat([
            experimental_results,
            df_model,
            df_qaussian,
            df_lognorm,
            df_qmm
        ])
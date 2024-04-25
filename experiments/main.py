import numpy as np
import pandas as pd

from src.evaluation.evaluation import calculate_mid_results, evaluate
from src.evaluation.visualization import Visualization
from src.experiement.experiment import Experiment, preproc


def find_optimum_h_data_period(file):
    exp = Experiment(file, 5000, 200)
    data_size = np.arange(50, 1000, 50)
    period = np.concatenate([np.arange(1, 10, 1), np.arange(10, 400, 10)])

    # 히스토리컬 데이터 사이즈
    exp.performance_different_data_size(data_size, 0.95, False).to_csv("./results/h_data.csv")
    exp.performance_different_data_size(data_size, 0.95, True).to_csv("./results/h_data_sample.csv")

    # 업데이트
    exp.period_update(period,0.95).to_csv("./results/period.csv")


def over_quantiles(file, quantiles):
    exp = Experiment(file, 5000, 200)
    # 분위수
    exp.compared_baselines_quantile(quantiles, False).to_csv("./results/baseline_q_tc.csv")
    exp.compared_baselines_quantile(quantiles, True).to_csv("./results/baseline_q_tc_sample.csv")

    exp.model_by_percentile(quantiles, True, True).to_csv("./results/model_q_tc.csv")
    exp.model_by_percentile(quantiles, True, False).to_csv("./results/model_msg_q_tc_sample.csv")
    exp.model_by_percentile(quantiles, False, True).to_csv("./results/model_non_luk_q_non_tc.csv")


def get_results(file, filename, percentiles, use_sampling=False):
    root = "./results/"
    exp = Experiment(file)
    results = pd.concat([exp.pred_latency(p / 100, use_sampling) for p in percentiles])
    results.to_csv(root + filename + '.csv')


def create_images_concepts(measurement_file, display_max_delay=300, display_max_rtt=50, data_step=5, id_max=1000, max_rate=125, add_delay=6):
    save_folder = './images/concepts/'
    data = preproc(pd.read_csv(measurement_file))

    vis = Visualization()

    vis.change_over_time(save_folder+'sending_rate.pdf', data, 'Sending Rate', max_rate, id_max)
    vis.change_over_time(save_folder+'rtt.pdf', data, 'RTT', add_delay, id_max)

    vis.jointplot_latency(save_folder+'message_size_latency.pdf', data, 'Message Size', 1)
    vis.jointplot_latency(save_folder+'serialization_delay_latency.pdf', data[data['Serialization Delay'] < display_max_delay], 'Serialization Delay',data_step)
    vis.jointplot_latency(save_folder+'combination_delay_latency.pdf', data[data['Combination Delay'] < display_max_delay], 'Combination Delay', data_step)
    vis.jointplot_latency(save_folder+'rtt_latency.pdf', data[data['RTT'] < display_max_rtt], 'RTT', data_step)


def create_images_performance(exp_result, exp_name='m0'):
    save_folder = './images/evaluation/'+exp_name
    exp_result.loc[exp_result['Method'] == 'Quantile Reg', 'Method'] = 'Model'
    calculate_mid_results([exp_result])
    eval_result = evaluate(exp_result)

    vis = Visualization()
    vis.barplot(save_folder+'_success_rate.pdf', eval_result, 'Success Rate')
    vis.barplot(save_folder+'_mae.pdf', eval_result, 'MAE')
    vis.barplot(save_folder+'_q_loss.pdf', eval_result, 'Quantile Loss')
    vis.barplot(save_folder+'_r_overhead.pdf', eval_result, 'Relative Overhead')

    return eval_result


def create_images_distribution(exp_result, exp_name='m0', quantile=0.99, lim1=(-3000, 200), lim2=(-2000,50)):
    save_folder = './images/evaluation/' + exp_name
    exp_result_by_quantile = exp_result[exp_result['Quantile'] == quantile]

    vis = Visualization()
    vis.err_distibution(
        save_folder+'_err_dist.pdf',
        exp_result_by_quantile[exp_result_by_quantile['Method'] == 'Model'],
        lim1
    )
    vis.ecdf(save_folder+'_ecdf.pdf', exp_result_by_quantile, quantile, lim2)


if __name__ == '__main__':
    percentiles = np.array([90, 95, 99])

    # (1) Measurements => Experiment Results
    get_results('./measurements/m0_500KB_3MB1.csv', 'm0_1', percentiles)
    get_results('./measurements/m0_500KB_3MB2.csv', 'm0_2', percentiles)
    get_results('./measurements/m0_500KB_3MB3.csv', 'm0_3', percentiles)
    get_results('./measurements/m0_500KB_3MB4.csv', 'm0_4', percentiles)
    #
    # get_results('./measurements/m1_3MB1.csv', 'm1_1', percentiles)
    # get_results('./measurements/m1_3MB2.csv', 'm1_2', percentiles)
    # get_results('./measurements/m1_3MB3.csv', 'm1_3', percentiles)
    # get_results('./measurements/m1_3MB4.csv', 'm1_4', percentiles)

    # (2) Experiment Results => Evaluation & Image Creation using create images
    scenario1 = pd.concat([
            pd.read_csv('./results/m0_1.csv'),
            pd.read_csv('./results/m0_2.csv'),
            pd.read_csv('./results/m0_3.csv'),
            pd.read_csv('./results/m0_4.csv'),
        ])
    scenario2 = pd.concat([
            pd.read_csv('./results/m1_1.csv'),
            pd.read_csv('./results/m1_2.csv'),
            pd.read_csv('./results/m1_3.csv'),
            pd.read_csv('./results/m1_4.csv'),
        ])
    create_images_performance(scenario1, 'm0')
    create_images_performance(scenario2, 'm1')
    create_images_distribution(scenario1, 'm0')
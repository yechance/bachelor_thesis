import numpy as np
import pandas as pd
from evaluation.experiment import Experiment
from evaluation.visualization import *

PERFORMANCE_BY_DATA_SIZE = "results/historical_data_size.csv"
PERIOD = "results/update_period.csv"

QUANTILES_CSV = "results/model.csv"
QUANTILES_NON_LUK_CSV = "results/model_without_luk.csv"
QUANTILES_MSG_CSV = "results/model_only_msg_size.csv"

BASELINE_QUANTILE_CSV = "results/quantile_baselines.csv"
BASELINE_QUANTILE_SAMPLING_CSV = "results/quantile_baselines_sampling.csv"

HISTORICAL_DATA_SIZE_CSV = "results/historical_data_size.csv"
HISTORICAL_DATA_SIZE_SAMPLING_CSV = "results/historical_data_size_sampling.csv"

PREDICTED_LATENCY_50 = "results/predicted_latency_50.csv"
PREDICTED_LATENCY_95 = "results/predicted_latency_95.csv"
PREDICTED_LATENCY_99 = "results/predicted_latency_99.csv"
EVALUATION_99 = "results/evaluation_99.csv"
PREDICTED_LATENCY_1M_SAMPLE = "results/predicted_latency_sample.csv"

PREDICTED_LATENCY_99_1M = "results/predicted_latency_99_1M.csv"
EVALUATION_99_1M = "results/evaluation_99_1M.csv"

# def do_experements(load_file, save_file):
#     experiment_baseline = Experiment(file)
#     # 백분위수 비교
#     list_percentile = np.arange(50, 100, 2)
#     # results = experiment_baseline.performance_model_by_percentile(list_percentile, save_path_percentiles)
#
#     # 모델 성능
#     # results = experiment_baseline.performance_model_by_percentile(list_percentile, save_path_percentiles)
#     results = experiment_baseline.performance_model_by_percentile(list_percentile, save_path_model_single_feature, True,
#                                                                   False, "model_single_feature")
#
#     # 베이스라인 성능
#     # results = experiment_baseline.performance_compared_baselines(save_path_baselines)


if __name__ == '__main__':
    measurement_1MB_5MB = "data/measurement_1MB_5MB_random.csv"
    measurement_1MB = "data/measurement_1MB_random.csv"

    experiment_baseline = Experiment(measurement_1MB, 5000, 200)
    # experiment_baseline = Experiment(measurement_1MB_5MB, 4900, 200)

    df = experiment_baseline.pred_latency()
    df.to_csv(PREDICTED_LATENCY_99_1M)
    #
    # df = experiment_baseline.pred_latency()
    # df.to_csv(PREDICTED_LATENCY_95)
    #
    # df = experiment_baseline.pred_latency()
    # df.to_csv(PREDICTED_LATENCY_50)

    df_model = experiment_baseline.performance_model_by_percentile(
            [0.99],
    )

    df_baseline = experiment_baseline.performance_compared_baselines_quantile(
            [0.99], False
    )
    evaluation_99 = pd.concat([df_model, df_baseline])
    evaluation_99.to_csv(EVALUATION_99_1M)

    # data_size = np.concatenate([np.arange(10, 100, 10), np.arange(100, 1000, 50)])
    # period = np.arange(1, 50, 1)
    #
    # quantiles = np.round(
    #     np.concatenate([np.arange(0.5, 1, 0.05), np.arange(0.951, 1, 0.01)]),
    #     3
    # )

    # df = experiment_baseline.performance_model_by_percentile(
    #     quantiles,
    #     True,
    #     True,
    # )
    # df.to_csv(QUANTILES_CSV)
    #
    # df = experiment_baseline.performance_model_by_percentile(
    #     quantiles,
    #     False,
    #     True,
    # )
    # df.to_csv(QUANTILES_NON_LUK_CSV)
    #
    # df = experiment_baseline.performance_model_by_percentile(
    #     quantiles,
    #     True,
    #     False,
    # )
    # df.to_csv(QUANTILES_MSG_CSV)

    # df = experiment_baseline.performance_compared_baselines_quantile(
    #     quantiles,
    #     False
    # )
    # df.to_csv(BASELINE_QUANTILE_CSV)
    #
    # df = experiment_baseline.performance_compared_baselines_quantile(
    #     quantiles,
    #     True
    # )
    # df.to_csv(BASELINE_QUANTILE_SAMPLING_CSV)

    # 히스토리컬 데이터 사이즈와 업데이트 주기에 따른 평가
    # df = experiment_baseline.performance_different_data_size(
    #     data_size,
    #     0.95,
    #     False
    # )
    # df.to_csv(HISTORICAL_DATA_SIZE_CSV)
    #
    # df = experiment_baseline.performance_different_data_size(
    #     data_size,
    #     0.95,
    #     True
    # )
    # df.to_csv(HISTORICAL_DATA_SIZE_SAMPLING_CSV)
    #
    # df = experiment_baseline.period_update(
    #     period,
    #     0.95
    # )
    # df.to_csv(PERIOD)


# import asyncio
# import os
# import docker
#
# # Traffic Control
# # 1. Network Delay : tc qdisc add dev eth0 root netem delay ()
# # 2. Packet Loss : tc qdisc add dev eth0 root netem loss 10%
# # 3. Packet Corruption : tc qdisc change dev eth0 root netem corrupt 5%
# # 4. Bandwidth limit : tc qdisc add dev eth0 root rate (1mbit) burst (32kbit) latency
#
# # async def reset_inter_server_interface(container_name:str):
# #     p = await asyncio.create_subprocess_exec("docker","exec", container_name,
# #                                              "tc", "qdisc", "del", "dev", "eth0", "root")
# #                                              # cwd=os.getcwd() + "/docker/hrmes-dqlite")
# #     await p.wait()
# #
# # async def set_outbound_server_latency_and_bandwidth(container_name:str, milliseconds:int, mbits:int):
# #     p = await asyncio.create_subprocess_exec("docker", "exec", container_name,
# #                                              "tc", "qdisc", "add", "dev", "eth0", "root", "netem", "delay",
# #                                              str(int(milliseconds))+"ms", "rate", str(mbits)+"mbits",
# #                                              # cwd=os.getcwd() + "/docker/hrmes-dqlite"
# #                                              )
# #
# # async def set_inter_server_latency_and_bandwidth(container_names:list[str], milliseconds:int, mbits:int):
# #     for cn in container_names:
# #         await reset_inter_server_interface(cn)
# #         await set_outbound_server_latency_and_bandwidth(cn, milliseconds * 0.5, mbits)
#
# if __name__ == '__main__':
#     latency=0
#     bandwidth=0
#
#     traffic_control_cmd_eth0 = [
#         "tc qdisc del dev eth0 root",
#         "tc qdisc add dev eth0 root netem delay {latency}ms rate {bandwidth}mbits".format(latency=latency, bandwidth=bandwidth)
#     ]
#
#     my_docker = docker.from_env()
#
#     # docker network
#     quiche_network = my_docker.networks.create(
#         name="quiche_bbr",
#         driver="bridge"
#     )
#     client_image = ""
#     server_image = ""
#     # Run Server Container
#     my_docker.containers.run(
#         image=server_image,
#         command=traffic_control_cmd_eth0,  # Configure the traffic control in the server container
#         auto_remove=True,
#         detach=False,
#         name="quic_server",
#         network=quiche_network,
#         ports={'4433/udp': '4433'},
#         stdout=True,
#         stderr=True,
#     )
#     # Run Client Container
#     my_docker.containers.run(
#         image=client_image,
#         command=traffic_control_cmd_eth0,  # Configure the traffic control in the server container
#         auto_remove=True,
#         detach=False,
#         # environment
#         name="quic_client",
#         network=quiche_network,
#         ports={'3344/udp': '3344'},
#         # volume=
#         stdout=True,
#         stderr=True,
#     )
#     # Training : Measurement
#     # Historical data :
#
#     # Create Prediction Model
#
#     # Testing :
#     # KL-Divergence
#
#     # Evaluation :
#
#     # Remove docker settings
#     quiche_network.remove()
#     None
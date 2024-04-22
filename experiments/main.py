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

HISTORICAL_DATA_SIZE_L_CSV = "results/historical_data_size_loopback.csv"
HISTORICAL_DATA_SIZE_SAMPLING_L_CSV = "results/historical_data_size_sampling_loopback.csv"
PERIOD_L = "results/update_period_loopback.csv"

HISTORICAL_DATA_SIZE_L_95_CSV = "results/historical_data_size_loopback_95.csv"
HISTORICAL_DATA_SIZE_SAMPLING_L_95_CSV = "results/historical_data_size_sampling_loopback_95.csv"
PERIOD_L_95 = "results/update_period_loopback_95.csv"

QUANTILES_L_CSV = "results/model_loopback.csv"
QUANTILES_NON_LUK_L_CSV = "results/model_without_luk_loopback.csv"
QUANTILES_MSG_L_CSV = "results/model_only_msg_size_loopback.csv"

BASELINE_QUANTILE_L_CSV = "results/quantile_baselines_loopback.csv"
BASELINE_QUANTILE_SAMPLING_L_CSV = "results/quantile_baselines_sampling_loopback.csv"

PREDICTED_LATENCY_L_99 = "results/predicted_latency_99_loopback.csv"
EVALUATION_L_99 = "results/evaluation_99_loopback.csv"
# PREDICTED_LATENCY_1M_SAMPLE = "results/predicted_latency_sample.csv"

# PREDICTED_LATENCY_99_1M = "results/predicted_latency_99_1M.csv"
# EVALUATION_99_1M = "results/evaluation_99_1M.csv"


if __name__ == '__main__':
    data_size = np.arange(50, 1000, 50)
    period = np.arange(10, 400, 10)
    quantiles = np.round(
        np.arange(900, 1000, 5) / 1000,
        3
    )

    measurement_1MB_5MB = "data/measurement_1MB_5MB_random.csv"
    measurement_1MB = "data/measurement_1MB_random.csv"
    measurement_L = "data/500KB_6MB_5ms_1000mbit.csv"

    eth_tc = "./docker_data/eth_tc_1000mbit_1ms.csv"
    eth_non_tc = "./docker_data/eth_non_tc.csv"

    exp_tc = Experiment(eth_tc, 5000, 200)
    exp_n_tc = Experiment(eth_non_tc, 5000, 200)

    # # 히스토리컬 데이터 사이즈
    # exp_tc.performance_different_data_size(data_size, 0.95, False).to_csv("./docker_results/h_data_tc.csv")
    exp_tc.performance_different_data_size(data_size, 0.95, True).to_csv("./docker_results/h_data_tc_sample.csv")
    # exp_n_tc.performance_different_data_size(data_size, 0.95, False).to_csv("./docker_results/h_data_non_tc.csv")
    # exp_n_tc.performance_different_data_size(data_size, 0.95, True).to_csv("./docker_results/h_data_non_tc_sample.csv")

    # # 업데이트
    # exp_tc.period_update(period,0.95).to_csv("./docker_results/period_tc.csv")
    # exp_n_tc.period_update(period, 0.95).to_csv("./docker_results/period_non_tc.csv")

    # 분위수
    # exp_tc.compared_baselines_quantile(quantiles, False).to_csv("./docker_results/baseline_q_tc.csv")
    # exp_tc.compared_baselines_quantile(quantiles, True).to_csv("./docker_results/baseline_q_tc_sample.csv")
    #
    # exp_n_tc.compared_baselines_quantile(quantiles, False).to_csv("./docker_results/baseline_q_non_tc.csv")
    # exp_n_tc.compared_baselines_quantile(quantiles, True).to_csv("./docker_results/baseline_q_non_tc_sample.csv")
    #
    # exp_tc.model_by_percentile(quantiles, True, True).to_csv("./docker_results/model_q_tc.csv")
    # exp_tc.model_by_percentile(quantiles, True, False).to_csv("./docker_results/model_msg_q_tc_sample.csv")
    # exp_tc.model_by_percentile(quantiles, False, True).to_csv("./docker_results/model_non_luk_q_non_tc.csv")
    #
    # exp_n_tc.model_by_percentile(quantiles, True, True).to_csv("./docker_results/model_q_tc.csv")
    # exp_n_tc.model_by_percentile(quantiles, True, False).to_csv("./docker_results/model_msg_q_tc_sample.csv")
    # exp_n_tc.model_by_percentile(quantiles, False, True).to_csv("./docker_results/model_non_luk_q_non_tc.csv")

    # 예측 레이턴시
    # 분위수 50, 95, 99
    # exp_tc.pred_latency(0.95, False).to_csv("./docker_results/q_95_tc.csv")
    # exp_n_tc.pred_latency(0.95, False).to_csv("./docker_results/q_95_non_tc.csv")

    # exp_tc.pred_latency(0.99, False).to_csv("./docker_results/q_99_tc.csv")
    # exp_n_tc.pred_latency(0.99, False).to_csv("./docker_results/q_99_non_tc.csv")
    #
    # exp_tc.pred_latency(0.5, False).to_csv("./docker_results/q_50_tc.csv")
    # exp_n_tc.pred_latency(0.5, False).to_csv("./docker_results/q_50_non_tc.csv")
    #
    # exp_tc.pred_latency(0.95, True).to_csv("./docker_results/q_95_tc_sample.csv")
    # exp_n_tc.pred_latency(0.95, True).to_csv("./docker_results/q_95_non_tc_sample.csv")
    #
    # exp_tc.pred_latency(0.99, True).to_csv("./docker_results/q_99_tc.csv")
    # exp_n_tc.pred_latency(0.99, True).to_csv("./docker_results/q_99_non_tc_sample.csv")
    #
    # exp_tc.pred_latency(0.5, True).to_csv("./docker_results/q_50_tc_sample.csv")
    # exp_n_tc.pred_latency(0.5, True).to_csv("./docker_results/q_50_non_tc_sample.csv")
    # df.to_csv(PREDICTED_LATENCY_99_1M)
    #
    # df = experiment_baseline.pred_latency()
    # df.to_csv(PREDICTED_LATENCY_95)
    #
    # df = experiment_baseline.pred_latency()
    # df.to_csv(PREDICTED_LATENCY_50)




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
import asyncio
import os
import docker

# Traffic Control
# 1. Network Delay : tc qdisc add dev eth0 root netem delay ()
# 2. Packet Loss : tc qdisc add dev eth0 root netem loss 10%
# 3. Packet Corruption : tc qdisc change dev eth0 root netem corrupt 5%
# 4. Bandwidth limit : tc qdisc add dev eth0 root rate (1mbit) burst (32kbit) latency

async def reset_inter_server_interface(container_name:str):
    p = await asyncio.create_subprocess_exec("docker","exec", container_name,
                                             "tc", "qdisc", "del", "dev", "eth0", "root")
                                             # cwd=os.getcwd() + "/docker/hrmes-dqlite")
    await p.wait()

async def set_outbound_server_latency_and_bandwidth(container_name:str, milliseconds:int, mbits:int):
    p = await asyncio.create_subprocess_exec("docker", "exec", container_name,
                                             "tc", "qdisc", "add", "dev", "eth0", "root", "netem", "delay",
                                             str(int(milliseconds))+"ms", "rate", str(mbits)+"mbits",
                                             # cwd=os.getcwd() + "/docker/hrmes-dqlite"
                                             )

async def set_inter_server_latency_and_bandwidth(container_names:list[str], milliseconds:int, mbits:int):
    for cn in container_names:
        await reset_inter_server_interface(cn)
        await set_outbound_server_latency_and_bandwidth(cn, milliseconds * 0.5, mbits)

if __name__ == '__main__':
    latency=0
    bandwidth=0

    traffic_control_cmd_eth0 = [
        "tc qdisc del dev eth0 root",
        "tc qdisc add dev eth0 root netem delay {latency}ms rate {bandwidth}mbits".format(latency=latency, bandwidth=bandwidth)
    ]

    my_docker = docker.from_env()

    # docker network
    quiche_network = my_docker.networks.create(
        name="quiche_bbr",
        driver="bridge"
    )
    client_image = ""
    server_image = ""
    # Run Server Container
    my_docker.containers.run(
        image=server_image,
        command=traffic_control_cmd_eth0,  # Configure the traffic control in the server container
        auto_remove=True,
        detach=False,
        name="quic_server",
        network=quiche_network,
        ports={'4433/udp': '4433'},
        stdout=True,
        stderr=True,
    )
    # Run Client Container
    my_docker.containers.run(
        image=client_image,
        command=traffic_control_cmd_eth0,  # Configure the traffic control in the server container
        auto_remove=True,
        detach=False,
        # environment
        name="quic_client",
        network=quiche_network,
        ports={'3344/udp': '3344'},
        # volume=
        stdout=True,
        stderr=True,
    )
    # Training : Measurement
    # Historical data :

    # Create Prediction Model

    # Testing :
    # KL-Divergence

    # Evaluation :

    # Remove docker settings
    quiche_network.remove()
    None
#!/bin/bash
#tc qdisc del root dev eth0
tc qdisc add dev eth0 root handle 1: netem delay 5ms
tc qdisc add dev eth0 parent 1:1 handle 10: tbf rate 1000mbit burst 16kbit latency 50ms

./client
tc qdisc del root dev eth0
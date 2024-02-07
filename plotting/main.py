import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns

"""
Data Load & Preprocessing
"""
def load_data(filepath, network=None):
    df = pd.read_csv(filepath)
    df.astype('uint64')
    return df.sort_values(by=["message_size", "actual rtt"])


def preprocess_data(records, filter=[]):

    return None


"""
Default Chat Axis
    x axis : message size
    y axis : Latency

Factor :
 1. delivery rate
 
Comparison :
 1. rtt
 2. min rtt
 
"""
def basic_chat(records):
    f, ax = plt.subplots(figsize=(8, 4)) #, gridspec_kw=dict(width_ratios=[4, 3]))
    ax.scatter(data=data, x="message_size", y="actual rtt", s=1)# hue

    # total_len = len(data.index)
    # ax.set_xticks(np.arange(0, total_len+1, 10))
    # ax.set_yticks(np.arange(0, total_len+1, 10))
    ax.set_xscale("log")
    ax.set_yscale("log")
    plt.xlabel('Message Size (B)')
    plt.ylabel('Latency (micro sec)')
    plt.title('Latency Measurement Based on Message Size')
    plt.show()

def plot_with_delivery_rate(records):
    data = records[["message_size", "actual rtt", "delivery_rate"]]

    f, ax = plt.subplots(figsize=(8, 4))  # , gridspec_kw=dict(width_ratios=[4, 3]))
    sns.scatterplot(data=data, x="message_size", y="actual rtt", hue="delivery_rate", ax=ax)  # hue

    plt.show()

def plot_with_rtt(records):
    data = records[["message_size", "actual rtt", "rtt"]]

    f, ax = plt.subplots(figsize=(8, 4))  # , gridspec_kw=dict(width_ratios=[4, 3]))
    sns.scatterplot(data=data, x="message_size", y="actual rtt", hue="rtt", ax=ax)  # hue

    plt.show()

# def plot_rtt_
def total_overview(records):

    return None

def print_hi(name):
    # Use a breakpoint in the code line below to debug your script.
    print(f'Hi, {name}')  # Press ⌘F8 to toggle the breakpoint.


# Press the green button in the gutter to run the script.
if __name__ == '__main__':
    filepath = "example3.csv"
    df = load_data(filepath)
    data = df[["message_size", "actual rtt"]]
    basic_chat(df)
    # print(data.groupby(['message_size']).describe())
    # plot_with_delivery_rate(df)
    # plot_with_rtt(df)

# See PyCharm help at https://www.jetbrains.com/help/pycharm/

import numpy as np
from sklearn.metrics import mean_absolute_error, mean_pinball_loss


class Evaluation:
    def __init__(self, q, latency, latency_pred, overhead):
        self.q = q
        self.y = latency
        self.y_pred = latency_pred
        self.overhead = overhead

    def evaluate(self):
        return {
            "success_rate": np.mean(self.y < self.y_pred),
            "mae": mean_absolute_error(self.y, self.y_pred),
            "overhead": self.overhead.mean(),
            "relative_overhead": np.mean(self.overhead / self.y),
            "quantile_loss": mean_pinball_loss(self.latency, self.latency_pred, alpha=self.q)
        }

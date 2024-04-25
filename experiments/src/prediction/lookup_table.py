class LookupTable:
    def __init__(self, update_period):
        self.table = {}
        self.update_period = update_period

    def valid(self, quantile):
        row = self.table.get(quantile)
        if row:
            return row['update_timer'] < self.update_period
        else:
            return False

    def increase_update_timer(self):
        for row in self.table.values():
            row['update_timer'] += 1

    def set_parameters(self, quantile, intercept, coef):
        self.table[quantile] = {"intercept_": intercept, "coef_": coef, "update_timer": 0}

    def get_parameters(self, quantile):
        return self.table.get(quantile)

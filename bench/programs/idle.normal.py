# trace: D155. The idle game of `idle.normal.wax` in Python's `int`.
def run(n):
    gold = 0
    earned = 0
    rate = 1
    cost = 10
    for _ in range(n):
        gold += rate
        earned += rate
        if gold >= cost:
            gold -= cost
            rate *= 2
            cost *= 2
    return len(str(earned))

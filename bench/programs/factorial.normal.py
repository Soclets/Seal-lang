# trace: D155. n factorial in Python's `int`, counted by its digits, as `factorial.normal.wax` does.
def run(n):
    product = 1
    for i in range(2, n + 1):
        product *= i
    return len(str(product))

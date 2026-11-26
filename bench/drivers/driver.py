
# The benchmark driver: checks nothing itself, only runs `run` and prints what it gave and how long it took.
import sys, time
sys.set_int_max_str_digits(0)
argument, warmups, runs = ARGUMENT, WARMUPS, RUNS
started = time.perf_counter()
result = run(argument)
first = time.perf_counter() - started
for _ in range(warmups):
    run(argument)
times = []
for _ in range(runs):
    before = time.perf_counter()
    result = run(argument)
    times.append("%.9f" % (time.perf_counter() - before))
print("result %d" % result)
print("first %.9f" % first)
print("times " + " ".join(times))

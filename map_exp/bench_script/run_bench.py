#!/usr/bin/env python3

import os, os.path, subprocess, sys, csv, multiprocessing

def get_rust_time (num_threads, bench_type):
        command = "perflock hwloc-bind --cpubind node:0 --membind node:0 cargo run --release -- -l r -m " + bench_type + " -t " + str(num_threads)
        output = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=None, shell=True, cwd="../rust").communicate()
        return int(output[0])

def main ():
	f = open("../results/rust_rwlock_ro_nobound.csv","wt");
	try:
		writer = csv.writer(f);
		writer.writerow(("NumThreads", "BenchType", "NumOpsIn5Secs"))
		for num_threads in [1, 2, 4, 8, 16]:
			for bench_type in ["r"]:
				for i in range(3):
					writer.writerow((num_threads, bench_type, get_rust_time(num_threads, bench_type)))
					f.flush()
	finally:
		f.close()

main()

#!/usr/bin/env python3

import os, os.path, subprocess, sys, csv, multiprocessing

def get_rust_time (num_threads, bench_type):
        if num_threads > 10:
                command = "perflock hwloc-bind --cpubind package:0 cargo run --release -- -m " + bench_type + " -t " + str(num_threads)
        else:
                command = "perflock hwloc-bind --physical --cpubind pu:0-18 ~package:1 cargo run --release -- -m " + bench_type + " -t " + str(num_threads)
        output = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=None, shell=True, cwd="../rust").communicate()
        return int(output[0])

def main ():
	f = open("../results/rust_mutex_bound_16_hyperthread.csv","wt");
	try:
		writer = csv.writer(f);
		writer.writerow(("NumThreads", "BenchType", "NumOpsIn10Secs"))
		for num_threads in range(1, 21):
			for bench_type in ["r", "w", "rw"]:
				for i in range(3):
					writer.writerow((num_threads, bench_type, get_rust_time(num_threads, bench_type)))
					f.flush()
	finally:
		f.close()

main()

#!/bin/bash

cargo build --release
rm results.log
for w in 1 2 4; do
	for d in uniform skewed; do
		for r in 1 2 4 8 16 32; do
			target/release/benchmark -r $r -w $w -d $d -c | tee -a results.log;
		done;
	done;
done

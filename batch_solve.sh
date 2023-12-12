#!/bin/bash
rm -rf output/
mkdir -p output/
cargo build --release
echo Running solver
./target/release/ant_colony --batch $@
cd output
echo Solver finished, converting dotfiles to images
for dir in *
do
	cd $dir
	dot solution.dot -Tpng -o solution.dot.png
	cd ..
done
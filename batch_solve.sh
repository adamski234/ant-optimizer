#!/bin/bash
if [ -n "$SCRIPT_MAKE_TMPFS" ]
then
	sudo umount ./output/
fi

rm -rf output/
mkdir -p output/

if [ -n "$SCRIPT_MAKE_TMPFS" ]
then
	sudo mount -t tmpfs -o size=4G tmpfs ./output/
fi

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
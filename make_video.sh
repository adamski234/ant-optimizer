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

mkdir output/images
cargo build --release
echo Running solver
./target/release/ant_colony --record $@ >| output/solution.dot
cd output
echo Solver finished, converting dotfiles to images
ls | grep ".dot" | parallel dot {} -Tpng -o ./images/{}.png
cd images
echo Making a video out of single images
if [ -x "$(nvidia-smi)" ]
then
	ffmpeg -y -framerate 30 -i %d.dot.png -vcodec libx264 -qp 15 out.mkv
else
	ffmpeg -y -framerate 30 -i %d.dot.png -vcodec hevc_nvenc -qp 15 out.mkv
fi
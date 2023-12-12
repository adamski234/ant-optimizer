#!/bin/bash
rm -rf output/
mkdir -p output/
cargo build --release
echo Running solver
./target/release/ant_colony --batch --record $@
cd output
echo Solver finished, converting dotfiles to images
for dir in *
do
	cd $dir
	mkdir images
	ls | grep ".dot" | parallel dot {} -Tpng -o ./images/{}.png
	echo Creating video for $dir
	cd images
	if [ -x "$(nvidia-smi)" ]
	then
		ffmpeg -y -framerate 30 -i %d.dot.png -vcodec libx264 -qp 15 out.mkv
	else
		ffmpeg -y -framerate 30 -i %d.dot.png -vcodec hevc_nvenc -qp 15 out.mkv
	fi
	cd ../..
done
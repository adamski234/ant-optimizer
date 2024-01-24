#!/bin/bash
rm -rf output/
mkdir -p output/
cargo build --release
echo Running solver
./target/release/ant_colony --batch "$@"
cd output
echo Solver finished, converting dotfiles to images
ls */*.dot | parallel dot {} -Goverlap=prism -Tpng -o "{//}.png"
rm -rf ./*.txt/
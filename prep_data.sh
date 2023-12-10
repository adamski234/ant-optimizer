#!/bin/bash
cd data_raw
for file in *
do
cut -c 2- $file > ../data/$file
done
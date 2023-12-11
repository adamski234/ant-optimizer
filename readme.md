How to run for single file and get a graph file:
1. Prepare data file
2. Build the binary: `cargo build --release`
3. Run the binary with your preferred parameters: `./target/release/ant_colony --path data/B-n31-k5.txt --ant-count 30 --iterations 1000 --evaporation-coeff 0.5 --random-choice-chance 0.3 --pheromone-weight 2 --heuristic-weight 1  >| outfile.dot`
4. Process output file with `dot`: `dot outfile.dot -Tpng >| outfile.png`. You can replace `png` with `svg` for vector output

How to run for a single file, get a graph and a video
1. Make sure you have `parallel`, `graphviz` and `ffmpeg`. An NVIDIA card with NVENC that supports HEVC encoding (Starts with GP107) would also come in handy. Free up some space on your drive (at least 2GB). You can also set the `SCRIPT_MAKE_TMPFS` variable to anything to create a 4GB `tmpfs` instead. This should speed up video creation a little bit and will not tire out your SSD as much. However, it requires root permissions gotten through `sudo` in the script.
2. Run `./make_video.sh --path data/B-n31-k5.txt --ant-count 30 --iterations 1000 --evaporation-coeff 0.5 --random-choice-chance 0.3 --pheromone-weight 2 --heuristic-weight 1`.
3. Now wait. This will take a while. 
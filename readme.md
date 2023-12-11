How to run for single file and get a graph file:
1. Prepare data file
2. Build the binary: `cargo build --release`
3. Run the binary with your preferred parameters: `./target/release/ant_colony --path data/B-n31-k5.txt --ant-count 30 --iterations 1000 --evaporation-coeff 0.5 --random-choice-chance 0.3 --pheromone-weight 2 --heuristic-weight 1  >| outfile.dot`
4. Process output file with `dot`: `dot outfile.dot -Tpng >| outfile.png`. You can replace `png` with `svg` for vector output
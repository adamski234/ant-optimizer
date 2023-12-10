use std::path::PathBuf;

use clap::Parser;


#[derive(Parser)]
struct Config {
	#[arg(short, long)]
	batch: bool, // for processing directories
	#[arg(short, long)]
	path: PathBuf,
	#[arg(long, name = "ant-count")]
	ant_count: usize,
	#[arg(long)]
	iterations: u32,
	#[arg(long, name = "evaporation")]
	evaporation_coeff: f64,
	#[arg(long, name = "random-chance")]
	random_choice_chance: f64,
	#[arg(long, name = "pheromone-weight")]
	pheromone_weight: f64,
	#[arg(long, name = "heuristic-weight")]
	heuristic_weight: f64,

}

// first trim the leading spaces from files with `cut -c 2-`

fn main() {
	let config = Config::parse();
	if config.batch {
		//
	} else {
		let mut reader = csv::ReaderBuilder::new().has_headers(false).delimiter(b' ').trim(csv::Trim::All).from_path(config.path).unwrap();
		let mut nodes = Vec::<ant_colony::GraphNode>::new();
		for result in reader.deserialize() {
			nodes.push(result.unwrap());
		}
		let mut solver = ant_colony::WorldState::new(nodes, config.ant_count, config.random_choice_chance, config.pheromone_weight, config.heuristic_weight, config.iterations, config.evaporation_coeff);
		solver.do_all_iterations();
		//println!("{:#?}", solver.ants);
		println!("Found solution with length {}: \n {:#?}", solver.best_solution_length, solver.best_solution);
	}
}

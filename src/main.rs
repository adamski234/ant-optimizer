#![allow(clippy::needless_return)]

use std::{path::{Path, PathBuf}, collections::HashMap, ops::AddAssign};

use ant_colony::GraphNode;
use clap::Parser;


#[derive(Parser, Clone)]
struct Config {
	#[arg(short, long)]
	batch: bool, // for processing directories
	#[arg(short, long)]
	path: PathBuf,
	#[arg(long, name = "ant-count-per-vehicle")]
	ant_count_per_vehicle: usize,
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
	#[arg(long, name = "try-count")]
	try_count: Option<u32>,
}

impl From<&Config> for ant_colony::ConfigData {
	fn from(value: &Config) -> Self {
		return Self {
			ant_count_per_vehicle: value.ant_count_per_vehicle,
			heuristic_weight: value.heuristic_weight,
			iteration_count: value.iterations,
			pheromone_evaporation_coefficient: value.evaporation_coeff,
			pheromone_weight: value.pheromone_weight,
			random_choice_chance: value.random_choice_chance,
		};
	}
}

struct BatchRunData {
	pub min_result: f64,
	pub max_result: f64,
	pub average: f64,
	pub run_count: u32,
}

impl BatchRunData {
	fn new() -> Self {
		return Self {
			min_result: f64::MAX,
			max_result: f64::MIN,
			average: 0.0,
			run_count: 0,
		};
	}
}


impl AddAssign for BatchRunData {
	fn add_assign(&mut self, other: Self) {
		if other.max_result > self.max_result {
			self.max_result = other.max_result;
		}
		if other.min_result < self.min_result {
			self.min_result = other.min_result;
		}
		let self_sum = self.average * self.run_count as f64;
		let other_sum = other.average * other.run_count as f64;
		self.run_count += other.run_count;
		self.average = (self_sum + other_sum) / self.run_count as f64;
	}
}

impl AddAssign<f64> for BatchRunData {
	fn add_assign(&mut self, rhs: f64) {
		if rhs > self.max_result {
			self.max_result = rhs;
		}
		if rhs < self.min_result {
			self.min_result = rhs;
		}
		let previous_sum = self.average * self.run_count as f64;
		self.run_count += 1;
		self.average = (previous_sum + rhs) / self.run_count as f64;
		
	}
}

// first trim the leading spaces from files with `cut -c 2-`

// returns string that was printed before
fn process_set_of_nodes(nodes: Vec::<ant_colony::GraphNode>, config: Config, weight_limit: u32) -> String {
	let world_config = ant_colony::ConfigData::from(&config);
	let vehicle_count = (nodes.len() as f64).sqrt().round() as u8; // TODO variable amount of vehicles
	let mut solver = ant_colony::WorldState::new(nodes, world_config, weight_limit, vehicle_count);
	if let Some(tries) = config.try_count {
		let tries_per_thread = (tries as usize).div_ceil(num_cpus::get());
		let mut threads = Vec::with_capacity(num_cpus::get());
		for _ in 0..num_cpus::get() {
			let mut thread_solver = solver.clone();
			threads.push(std::thread::spawn(move || {
				let mut run_stats = BatchRunData::new();
				for _ in 0..tries_per_thread {
					thread_solver.do_all_iterations();
					run_stats += thread_solver.best_solution_length;
					thread_solver.reset();
				}
				return run_stats;
			}));
		}

		let result = threads.into_iter().map(|handle| {
			return handle.join();
		}).reduce(|a, b| {
			let mut batch = a.unwrap();
			batch += b.unwrap();
			return Ok(batch);
		}).unwrap().unwrap();
		return format!("Finished {} runs. Longest found route is {}, shortest found route is {}. The average length is {}", result.run_count, result.max_result, result.min_result, result.average);
	} else {
		solver.do_all_iterations();
		eprintln!("Found solution with length {}", solver.best_solution_length);
		return format!("{}", solver.solution_to_graphviz());
	}
}

fn read_file(path: &PathBuf) -> Vec<GraphNode> {
	let mut reader = csv::ReaderBuilder::new().has_headers(false).delimiter(b' ').trim(csv::Trim::All).from_path(path).unwrap();
	let mut nodes = Vec::<ant_colony::GraphNode>::new();
	for result in reader.deserialize() {
		nodes.push(result.unwrap());
	}
	return nodes;
}

fn read_directory(path: &PathBuf) -> HashMap<String, Vec<GraphNode>> {
	let mut node_map = HashMap::new();
	for file in std::fs::read_dir(path).unwrap() {
		let file = file.unwrap();
		let nodes = read_file(&file.path());
		node_map.insert(file.file_name().into_string().unwrap(), nodes);
	}
	return node_map;
}

fn batch_process_files(directory: &PathBuf, config: Config, weight_limits: &HashMap<String, u32>) {
	let node_map = read_directory(&config.path);
	if config.try_count.is_some() {
		// only save statistics
		for (filename, nodes) in node_map {
			let output = process_set_of_nodes(nodes, config.clone(), *weight_limits.get(Path::new(&filename).file_stem().unwrap().to_str().unwrap()).unwrap()); // won't write anything anyway
			println!("File {}: {}", filename, output);
		}
	} else {
		// create directories for each output file
		let mut threads = Vec::new();
		for (filename, nodes) in node_map {
			let directory = directory.clone();
			let config = config.clone();
			let weight_limit = *weight_limits.get(Path::new(&filename).file_stem().unwrap().to_str().unwrap()).unwrap();
			threads.push(std::thread::spawn(move || {
				let directory = format!("{}/{}", directory.display(), filename);
				std::fs::create_dir(format!("./{}/", directory)).unwrap();
				let output = process_set_of_nodes(nodes, config, weight_limit);
				std::fs::write(format!("./{}/solution.dot", directory), output).unwrap();
			}));
		}
		for thread in threads {
			thread.join().unwrap();
		}
	}
}

fn main() {
	let config = Config::parse();
	let weight_limits: HashMap<String, u32> = serde_json::from_str(&std::fs::read_to_string("./data/capacities.json").unwrap()).unwrap();
	if config.batch {
		batch_process_files(&"output".into(), config, &weight_limits);
	} else {
		let nodes = read_file(&config.path);
		let weight_limit = *weight_limits.get(config.path.file_stem().unwrap().to_str().unwrap()).unwrap();
		let output = process_set_of_nodes(nodes, config, weight_limit);
		println!("{}", output);
	}
}
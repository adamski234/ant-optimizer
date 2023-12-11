#![allow(clippy::needless_return)]

use std::path::PathBuf;

use clap::Parser;


#[derive(Parser, Clone)]
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
	pheromone_weight: i32,
	#[arg(long, name = "heuristic-weight")]
	heuristic_weight: i32,
	#[arg(long, name = "try-count", conflicts_with = "record")]
	try_count: Option<u32>,
	#[arg(short, long, conflicts_with = "try-count")]
	record: bool,
}

impl From<&Config> for ant_colony::ConfigData {
	fn from(value: &Config) -> Self {
		return Self {
			ant_count: value.ant_count,
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

	fn add_run(&mut self, result: f64) {
		if result > self.max_result {
			self.max_result = result;
		}
		if result < self.min_result {
			self.min_result = result;
		}
		let previous_sum = self.average * self.run_count as f64;
		self.run_count += 1;
		self.average = (previous_sum + result) / self.run_count as f64;
	}

	fn add_batch(&mut self, other: Self) {
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

// first trim the leading spaces from files with `cut -c 2-`

fn main() {
	let config = Config::parse();
	if config.batch {
		//
	} else {
		let world_config = ant_colony::ConfigData::from(&config);
		let mut reader = csv::ReaderBuilder::new().has_headers(false).delimiter(b' ').trim(csv::Trim::All).from_path(config.path).unwrap();
		let mut nodes = Vec::<ant_colony::GraphNode>::new();
		for result in reader.deserialize() {
			nodes.push(result.unwrap());
		}
		let mut solver = ant_colony::WorldState::new(nodes, world_config);
		if let Some(tries) = config.try_count {
			let tries_per_thread = (tries as usize).div_ceil(num_cpus::get());
			let mut threads = Vec::with_capacity(num_cpus::get());
			for _ in 0..num_cpus::get() {
				let mut thread_solver = solver.clone();
				threads.push(std::thread::spawn(move || {
					let mut run_stats = BatchRunData::new();
					for _ in 0..tries_per_thread {
						thread_solver.do_all_iterations();
						run_stats.add_run(thread_solver.best_solution_length);
						thread_solver.reset();
					}
					return run_stats;
				}));
			}

			let result = threads.into_iter().map(|handle| {
				return handle.join();
			}).reduce(|a, b| {
				let mut batch = a.unwrap();
				batch.add_batch(b.unwrap());
				return Ok(batch);
			}).unwrap().unwrap();
			println!("Finished {} runs. Longest found route is {}, shortest found route is {}. The average length is {}", result.run_count, result.max_result, result.min_result, result.average);
		} else {
			solver.do_all_iterations();
			eprintln!("Found solution with length {}", solver.best_solution_length);
			println!("{:#?}", solver.solution_to_graphviz());
		}
	}
}
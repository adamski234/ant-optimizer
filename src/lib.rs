#![allow(clippy::needless_return)]

// TODO color solutions

use itertools::Itertools;
use rand::prelude::*;

#[derive(Debug, Clone, Copy, Eq, PartialOrd, Ord, serde::Deserialize)]
pub struct GraphNode {
	pub attraction_number: u8,
	pub x: i32,
	pub y: i32,
	pub demand: u32,
	pub ready_time: u32,
	pub due_time: u32,
	pub service_time: u32,
}

impl std::hash::Hash for GraphNode {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		state.write_u8(self.attraction_number);
	}
}

impl std::cmp::PartialEq for GraphNode {
	fn eq(&self, other: &Self) -> bool {
		return self.attraction_number == other.attraction_number;
	}
}

impl GraphNode {
	pub fn distance_to(&self, other: &GraphNode) -> f64 {
		return (((self.x - other.x).pow(2) + (self.y - other.y).pow(2)) as f64).sqrt();
	}

	pub fn to_graphviz(&self) -> String {
		return format!("{} [pos = \"{}, {}!\"]", self.attraction_number, self.x, self.y);
	}
}

enum AntError {
	NoNodesLeft,
}

#[derive(Debug, Clone)]
pub struct Ant {
	pub node_at: GraphNode,
	pub current_path: Vec<u8>,
	pub current_distance: f64,
	nodes_to_visit: Vec<GraphNode>,
	costs: Vec<f64>,
	time: f64,
	cargo_so_far: u32, // cargo lost so far - no partial deliveries
	time_weight: f64,
	random_choice_chance: f64,
}

impl Ant {
	fn new(random_choice_chance: f64, nodes: Vec<GraphNode>, time_weight: f64) -> Self {
		return Self {
			node_at: GraphNode { attraction_number: 0, x: 0, y: 0, demand: 0, ready_time: 0, due_time: 0, service_time: 0 }, // empty init
			current_path: Vec::with_capacity(nodes.len()),
			current_distance: 0.0,
			costs: Vec::with_capacity(nodes.len()),
			nodes_to_visit: nodes,
			time: 0.0,
			cargo_so_far: 0,
			time_weight,
			random_choice_chance,
		};
	}
	
	fn move_ant(&mut self, world: &mut WorldState, random_source: &mut ThreadRng) -> Result<(), AntError> {
		self.costs.clear();

		// we're done
		if self.nodes_to_visit.is_empty() {
			return Err(AntError::NoNodesLeft);
		}

		let mut can_visit = Vec::with_capacity(self.nodes_to_visit.len());
		// pick the next destination
		let next_node: GraphNode;
		let mut cost_sum = 0.0;
		for node in &mut self.nodes_to_visit {
			let weight_limit = world.weight_limit;
			let data = world.get_edge((self.node_at.attraction_number, node.attraction_number));
			let arrive_time = if self.time + data.length > node.ready_time as f64 { self.time + data.length } else { node.ready_time as f64 };
			if arrive_time > node.due_time as f64 {
				// check if there is time left
				continue;
			}
			if self.cargo_so_far + node.demand > weight_limit {
				// check if there is weight left
				continue;
			}
			// there are no zero length edges
			let time_cost = (1.0 - (arrive_time - self.time) / (node.due_time as f64 - self.time)).powf(self.time_weight);
			if data.pheromone_strength == 0.0 {
				let cost = data.length_cost * time_cost;
				self.costs.push(cost);
				can_visit.push(node);
				cost_sum += cost;
			} else {
				let cost = data.pheromone_cost * data.length_cost * time_cost;
				self.costs.push(cost);
				can_visit.push(node);
				cost_sum += cost;
			}
		}

		if can_visit.is_empty() {
			next_node = world.graph[0]; //return to depot and begin again
		} else {
			if random_source.gen::<f64>() > self.random_choice_chance {
				// roulette selection
				let number_to_match = random_source.gen::<f64>() * cost_sum;
				let mut cost_so_far = 0.0;
				let mut node_index = 0;
				for (index, item) in self.costs.iter().enumerate() {
					cost_so_far += item;
					if cost_so_far > number_to_match {
						node_index = index;
						break;
					}
				}
				next_node = *can_visit[node_index];
			} else {
				next_node = **can_visit.choose(random_source).unwrap();
			}
		}
		let edge = world.get_edge((self.node_at.attraction_number, next_node.attraction_number));
		self.current_path.push(self.node_at.attraction_number);
		self.current_distance += edge.length;
		if let Some(i) = self.nodes_to_visit.iter().position(|x| x == &next_node) {
			self.nodes_to_visit.swap_remove(i);
		}
		self.node_at = next_node;
		self.time += if self.time + edge.length > next_node.ready_time as f64 { edge.length } else { next_node.ready_time as f64 - self.time} + next_node.service_time as f64;
		self.cargo_so_far += next_node.demand;
		if self.node_at == world.graph[0] {
			// if vehicle is at depot, reset counters
			self.time = 0.0;
			self.cargo_so_far = 0;
		}

		return Ok(());
	}

	fn clear(&mut self) {
		self.current_path.clear();
		self.current_distance = 0.0;
	}
}

#[derive(Debug, Clone)]
pub struct EdgeData {
	pheromone_strength: f64,
	length: f64,
	length_cost: f64, // 0 if length is 0
	pheromone_cost: f64,
}

#[derive(Debug, Clone)]
pub struct ConfigData {
	pub ant_count: usize,
	pub pheromone_weight: f64,
	pub heuristic_weight: f64,
	pub iteration_count: u32,
	pub pheromone_evaporation_coefficient: f64,
	pub random_choice_chance: f64,
	pub time_weight: f64,
}

#[derive(Debug, Clone)]
pub struct WorldState {
	graph: Vec<GraphNode>,
	pub ants: Vec<Ant>,
	pub edges: Vec<EdgeData>,
	iteration_count: u32,
	pheromone_evaporation_coefficient: f64,
	pub best_solution: Vec<GraphNode>,
	pub best_solution_length: f64,
	pub heuristic_weight: f64,
	pub pheromone_weight: f64,
	pub time_weight: f64,
	weight_limit: u32,
}

impl WorldState {
	pub fn new(input_nodes: Vec<GraphNode>, config: ConfigData, weight_limit: u32) -> Self {
		let mut result = WorldState {
			graph: input_nodes,
			ants: Vec::with_capacity(config.ant_count),
			edges: Vec::with_capacity(0x1 << 16),
			iteration_count: config.iteration_count,
			pheromone_evaporation_coefficient: config.pheromone_evaporation_coefficient,
			best_solution: Vec::new(),
			best_solution_length: f64::MAX,
			heuristic_weight: config.heuristic_weight,
			pheromone_weight: config.pheromone_weight,
			time_weight: config.time_weight,
			weight_limit,
		};
		unsafe {
			result.edges.set_len(0x1 << 16);
		}

		for _ in 0..config.ant_count {
			result.ants.push(Ant::new(config.random_choice_chance, result.graph.clone(), config.time_weight));
		}

		result.init_edges();
		result.init_ants();

		return result;
	}

	fn init_edges(&mut self) {
		for (index, node) in self.graph.iter().enumerate() {
			for second_node in self.graph[index + 1 ..].iter() {
				//println!("{}, {} => {}", node.attraction_number, second_node.attraction_number, (((node.attraction_number as u16) << 8) | (second_node.attraction_number as u16)));
				let length = node.distance_to(second_node);
				let to_insert = EdgeData {
					length,
					pheromone_strength: 0.01,
					length_cost: if length != 0.0 { length.recip().powf(self.heuristic_weight) } else { 0.0 },
					pheromone_cost: 0.01_f64.powf(self.pheromone_weight),
				};
				let hash;
				if node.attraction_number > second_node.attraction_number {
					hash = ((node.attraction_number as u16) << 8) | (second_node.attraction_number as u16);
				} else {
					hash = ((second_node.attraction_number as u16) << 8) | (node.attraction_number as u16);
				}
				self.edges[hash as usize] = to_insert;
			}
		}
	}
	
	pub fn init_ants(&mut self) {
		for ant in &mut self.ants {
			ant.clear();
			ant.nodes_to_visit = self.graph.clone();
			ant.node_at = self.graph[0];
			ant.nodes_to_visit.swap_remove(ant.nodes_to_visit.iter().position(|x| *x == ant.node_at).unwrap());
		}
	}

	fn get_edge(&mut self, pair: (u8, u8)) -> &mut EdgeData {
		let hash;
		if pair.0 > pair.1 {
			hash = ((pair.0 as u16) << 8) | (pair.1 as u16);
		} else {
			hash = ((pair.1 as u16) << 8) | (pair.0 as u16);
		}
		return &mut self.edges[hash as usize];
	}

	// moves ants until they're all done
	fn move_ants(&mut self) {
		let mut random_source = rand::thread_rng();
		let mut temp = self.ants.clone(); // evil clone to get around the borrow checker
		for ant in &mut temp {
			while ant.move_ant(self, &mut random_source).is_ok() {
				//
			}
			ant.current_path.push(ant.node_at.attraction_number);
			ant.current_distance += self.get_edge((ant.current_path[ant.current_path.len() - 1].clone(), ant.node_at.attraction_number)).length;
			ant.current_path.push(self.graph[0].attraction_number);
			ant.node_at = self.graph[0];
			ant.current_distance += self.get_edge((ant.current_path[ant.current_path.len() - 1], ant.node_at.attraction_number)).length;
		}
		self.ants = temp;
	}

	fn update_pheromones(&mut self) {
		// evaporate pheromones
		for (index, node) in self.graph.clone().iter().enumerate() {
			for second_node in self.graph.clone()[index + 1 ..].iter() {
				self.get_edge((node.attraction_number, second_node.attraction_number)).pheromone_strength *= self.pheromone_evaporation_coefficient;
			}
		}

		// add pheromones
		let pheromone_weight = self.pheromone_weight;
		for ant in &self.ants.clone() {
			for pair in ant.current_path.windows(2) {
				let edge = self.get_edge((pair[0], pair[1]));
				edge.pheromone_strength += ant.current_distance.recip();
				edge.pheromone_cost = edge.pheromone_strength.powf(pheromone_weight);
			}
		}
	}

	fn update_best_solution(&mut self) {
		for ant in &self.ants {
			if ant.current_distance < self.best_solution_length {
				self.best_solution = ant.current_path.iter().map(|x| {
					return self.graph[*x as usize].clone();
				}).collect_vec();
				self.best_solution_length = ant.current_distance;
			}
		}
	}

	pub fn do_iteration(&mut self) {
		self.init_ants();
		self.move_ants();
		self.update_pheromones();
		self.update_best_solution();
	}

	pub fn do_all_iterations(&mut self) {
		for _ in 0..self.iteration_count {
			self.do_iteration();
		}
	}

	pub fn reset(&mut self) {
		self.init_ants();
		self.init_edges();
		self.best_solution = Vec::new();
		self.best_solution_length = f64::MAX;
	}

	pub fn nodes_to_graphviz(&self) -> String {
		let mut result = String::new();
		for node in &self.graph {
			result.push_str(&node.to_graphviz());
			result.push('\n');
		}
		return result;
	}

	pub fn solution_edges_to_graphviz(&self) -> (String, u32) {
		let mut result = String::new();
		let color_generator = random_color::RandomColor::new();
		let mut color = color_generator.to_hex();
		let mut car_count = 0;
		for pair in self.best_solution.windows(2) {
			result.push_str(&format!("{} -> {} [color = \"{}\"]\n", pair[0].attraction_number, pair[1].attraction_number, color));
			if pair[1].attraction_number == 0 {
				color = color_generator.to_hex();
				car_count += 1;
			}
		}
		return (result, car_count);
	}

	pub fn solution_to_graphviz(&self) -> String {
		let (edges, cars) = self.solution_edges_to_graphviz();
		return format!("digraph D {{\n\
			layout = \"neato\"\n\
			labelloc = \"t\"\n\
			label = \"Solution length is {}, with {} cars\"\n\
			{}\n\n\
			{}\
			}}", self.best_solution_length, cars, self.nodes_to_graphviz(), edges
		);
	}
}
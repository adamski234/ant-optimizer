#![allow(clippy::needless_return)]

// TODO how do you store solutions? Update them?
// TODO include time in cost calculation

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
	OutOfTime,
	CargoFull,
	AllNodesUsedByOtherColonies
}

#[derive(Debug, Clone)]
pub struct Ant {
	pub node_at: GraphNode,
	pub current_path: Vec<u8>,
	pub current_distance: f64,
	pub random_choice_chance: f64, // less than 1
	nodes_to_visit: Vec<GraphNode>,
	costs: Vec<f64>,
	pub group_id: u8,
	time: f64,
	cargo_so_far: u32, // cargo lost so far - no partial deliveries
}

impl Ant {
	fn new(random_choice_chance: f64, nodes: Vec<GraphNode>, group_id: u8) -> Self {
		return Self {
			node_at: GraphNode { attraction_number: 0, x: 0, y: 0, demand: 0, ready_time: 0, due_time: 0, service_time: 0 }, // empty init, randomize later
			current_path: Vec::with_capacity(nodes.len()),
			current_distance: 0.0,
			random_choice_chance,
			costs: Vec::with_capacity(nodes.len()),
			nodes_to_visit: nodes,
			group_id,
			time: 0.0,
			cargo_so_far: 0
		};
	}
	
	fn move_ant(&mut self, world: &mut WorldState, random_source: &mut ThreadRng) -> Result<(), AntError> {
		self.costs.clear();

		// we're done
		if self.nodes_to_visit.is_empty() {
			return Err(AntError::NoNodesLeft);
		}

		// pick the next destination
		let next_node;
		let mut cost_sum = 0.0;
		if random_source.gen::<f64>() < self.random_choice_chance {
			// random uniform selection
			next_node = self.nodes_to_visit.choose(random_source).unwrap().clone();
		} else {
			let mut to_remove = None;
			// create the costs table

			// preliminary checks
			let mut was_any_node_available_cargo = false;
			let mut was_any_node_available_time = false;
			let mut was_any_node_available_other_colonies = false;

			for node in &mut self.nodes_to_visit {
				let weight_limit = world.weight_limit;
				let data = world.get_edge((self.node_at.attraction_number, node.attraction_number));
				if (self.time + data.length + node.service_time as f64) < node.due_time as f64 {
					// check if there is time left
					was_any_node_available_time = true;
					break;
				}
				if self.cargo_so_far + node.demand < weight_limit {
					// check if there is weight left
					was_any_node_available_cargo = true;
					break;
				}
				if world.visitable_nodes[self.group_id as usize].contains(&node.attraction_number) {
					// check if other colony did not grab the node first
					was_any_node_available_other_colonies = true;
					break;
				}
			}
			if !was_any_node_available_cargo {
				return Err(AntError::CargoFull);
			}
			if !was_any_node_available_time {
				return Err(AntError::OutOfTime);
			}
			if !was_any_node_available_other_colonies {
				return Err(AntError::AllNodesUsedByOtherColonies);
			}

			for node in &mut self.nodes_to_visit {
				if world.visitable_nodes[self.group_id as usize].contains(&node.attraction_number) {
					// check if other colony did not grab the node first
					continue;
				}
				let weight_limit = world.weight_limit;
				let data = world.get_edge((self.node_at.attraction_number, node.attraction_number));
				let begin_time = if self.time + data.length > node.ready_time as f64 { self.time + data.length } else { node.ready_time as f64 };
				if begin_time + node.service_time as f64 > node.due_time as f64 {
					// check if there is time left
					continue;
				}
				if self.cargo_so_far + node.demand > weight_limit {
					// check if there is weight left
					continue;
				}
				if data.length == 0.0 {
					// zero distance means we jump straight there and ignore every other possibility
					// removes a node at no cost and it is always the most optimal solution
					// see 67 and 68 in A-n80-k10.txt
					self.current_path.push(self.node_at.attraction_number);
					self.node_at = node.clone();
					to_remove = Some(self.node_at.clone());
					self.time += if self.time + data.length > node.ready_time as f64 { data.length } else { node.ready_time as f64} + node.service_time as f64;
					self.cargo_so_far += node.demand;
				} else {
					if data.pheromone_strengths[self.group_id as usize] == 0.0 {
						let cost = data.length_cost;
						self.costs.push(cost);
						cost_sum += cost;
					} else {
						let cost = data.pheromone_costs[self.group_id as usize] * data.length_cost;
						self.costs.push(cost);
						cost_sum += cost;
					}
				}
			}
			if let Some(node) = to_remove {
				self.nodes_to_visit.swap_remove(self.nodes_to_visit.iter().position(|x| x == &node).unwrap());
				for (index, visitables) in world.visitable_nodes.iter_mut().enumerate() {
					if index == self.group_id as usize {
						continue;
					}
					visitables.swap_remove(self.nodes_to_visit.iter().position(|x| *x == node).unwrap());
				}
				return Ok(());
			}
			
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
			next_node = self.nodes_to_visit[node_index].clone();
		}

		let edge = world.get_edge((self.node_at.attraction_number, next_node.attraction_number));
		self.current_path.push(self.node_at.attraction_number);
		self.current_distance += edge.length;
		self.nodes_to_visit.swap_remove(self.nodes_to_visit.iter().position(|x| x == &next_node).unwrap());
		self.node_at = next_node;
		self.time += if self.time + edge.length > next_node.ready_time as f64 { edge.length } else { next_node.ready_time as f64} + next_node.service_time as f64;
		self.cargo_so_far += next_node.demand;
		for (index, visitables) in world.visitable_nodes.iter_mut().enumerate() {
			if index == self.group_id as usize {
				continue;
			}
			visitables.swap_remove(self.nodes_to_visit.iter().position(|x| *x == next_node).unwrap());
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
	first_node: GraphNode,
	second_node: GraphNode,
	pheromone_strengths: Vec<f64>, // for multiple groups/colonies
	length: f64,
	length_cost: f64, // 0 if length is 0
	pheromone_costs: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct ConfigData {
	pub ant_count_per_vehicle: usize,
	pub random_choice_chance: f64,
	pub pheromone_weight: f64,
	pub heuristic_weight: f64,
	pub iteration_count: u32,
	pub pheromone_evaporation_coefficient: f64,
}

#[derive(Debug, Clone)]
pub struct WorldState {
	graph: Vec<GraphNode>,
	pub ants: Vec<Ant>,
	//pub edges: fnv::FnvHashMap<(GraphNode, GraphNode), EdgeData>, // populate at init, key is ordered tuple simulating an unordered pair, with first node having lower att number
	pub edges: Vec<EdgeData>,
	iteration_count: u32,
	pheromone_evaporation_coefficient: f64,
	pub best_solution: Vec<GraphNode>,
	pub best_solution_length: f64,
	pub heuristic_weight: f64,
	pub pheromone_weight: f64,
	weight_limit: u32,
	vehicle_count: u8,
	visitable_nodes: Vec<Vec<u8>>, // group id is first index
}

impl WorldState {
	pub fn new(input_nodes: Vec<GraphNode>, config: ConfigData, weight_limit: u32, vehicle_count: u8) -> Self {
		let mut visitable_nodes = Vec::with_capacity(vehicle_count as usize);
		for _ in 0..vehicle_count {
			visitable_nodes.push(input_nodes.iter().map(|x| x.attraction_number).collect_vec());
		}
		let mut result = WorldState {
			graph: input_nodes,
			ants: Vec::with_capacity(config.ant_count_per_vehicle * (vehicle_count as usize)),
			edges: Vec::with_capacity(0x1 << 16),
			iteration_count: config.iteration_count,
			pheromone_evaporation_coefficient: config.pheromone_evaporation_coefficient,
			best_solution: Vec::new(),
			best_solution_length: f64::MAX,
			heuristic_weight: config.heuristic_weight,
			pheromone_weight: config.pheromone_weight,
			weight_limit,
			vehicle_count,
			visitable_nodes,
		};
		unsafe {
			result.edges.set_len(0x1 << 16);
		}

		for group in 0..vehicle_count {
			for _ in 0..config.ant_count_per_vehicle {
				result.ants.push(Ant::new(config.random_choice_chance, result.graph.clone(), group));
			}
		}

		result.init_edges();
		result.init_ants();

		return result;
	}

	fn init_edges(&mut self) {
		for (index, node) in self.graph.iter().enumerate() {
			for second_node in self.graph[index + 1 ..].iter() {
				//println!("{}, {} => {}", node.attraction_number, second_node.attraction_number, (((node.attraction_number as u16) << 8) | (second_node.attraction_number as u16)));
				let mut pheromone_strengths = Vec::with_capacity(self.vehicle_count as usize);
				let mut pheromone_costs = Vec::with_capacity(self.vehicle_count as usize);
				for _ in 0..self.vehicle_count {
					pheromone_strengths.push(0.01);
					pheromone_costs.push((0.01_f64).powf(self.pheromone_weight));
				}
				let length = node.distance_to(second_node);
				let to_insert = EdgeData {
					first_node: node.clone(),
					second_node: second_node.clone(),
					length,
					pheromone_strengths,
					length_cost: if length != 0.0 { length.recip().powf(self.heuristic_weight) } else { 0.0 },
					pheromone_costs,
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
		self.ants.shuffle(&mut thread_rng()); // prevents all ants from a single group from eating up all the nodes
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
			ant.current_distance += self.get_edge((ant.current_path[ant.current_path.len() - 2].clone(), ant.node_at.attraction_number)).length;
		}
		self.ants = temp;
	}

	fn update_pheromones(&mut self) {
		// evaporate pheromones
		let evap_coeff = self.pheromone_evaporation_coefficient;
		for (index, node) in self.graph.clone().iter().enumerate() {
			for second_node in self.graph.clone()[index + 1 ..].iter() {
				for group_pher in &mut self.get_edge((node.attraction_number, second_node.attraction_number)).pheromone_strengths {
					*group_pher *= *group_pher * evap_coeff;
				}
			}
		}

		// add pheromones
		let pheromone_weight = self.pheromone_weight;
		for ant in &self.ants.clone() {
			for pair in ant.current_path.windows(2) {
				let edge = self.get_edge((pair[0].clone(), pair[1].clone()));
				edge.pheromone_strengths[ant.group_id as usize] += ant.current_distance.recip();
				edge.pheromone_costs[ant.group_id as usize] = edge.pheromone_strengths[ant.group_id as usize].powf(pheromone_weight);
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
		for group_list in &mut self.visitable_nodes { // restart the lists of visitable nodes by each group
			*group_list = self.graph.iter_mut().map(|x| x.attraction_number).collect_vec();
		}
	}

	pub fn nodes_to_graphviz(&self) -> String {
		let mut result = String::new();
		for node in &self.graph {
			result.push_str(&node.to_graphviz());
			result.push('\n');
		}
		return result;
	}

	pub fn solution_edges_to_graphviz(&self) -> String {
		let mut result = String::new();
		for pair in self.best_solution.windows(2) {
			result.push_str(&format!("{} -> {}\n", pair[0].attraction_number, pair[1].attraction_number));
		}
		return result;
	}

	pub fn solution_to_graphviz(&self) -> String {
		return format!("digraph D {{\n\
			layout = \"neato\"\n\
			labelloc = \"t\"\n\
			label = \"Solution length is {}\"\n\
			{}\n\n\
			{}\
			}}", self.best_solution_length, self.nodes_to_graphviz(), self.solution_edges_to_graphviz()
		);
	}
}
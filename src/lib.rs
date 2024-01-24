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
	OutOfTime,
	CargoFull,
	AllNodesUsedByOtherColonies,
	IDoNotKnow
}

#[derive(Debug, Clone)]
pub struct Ant {
	pub node_at: GraphNode,
	pub current_path: Vec<u8>,
	pub current_distance: f64,
	nodes_to_visit: Vec<GraphNode>,
	costs: Vec<f64>,
	pub group_id: u8,
	time: f64,
	cargo_so_far: u32, // cargo lost so far - no partial deliveries
	time_weight: f64,
	random_choice_chance: f64,
}

impl Ant {
	fn new(random_choice_chance: f64, nodes: Vec<GraphNode>, group_id: u8, time_weight: f64) -> Self {
		return Self {
			node_at: GraphNode { attraction_number: 0, x: 0, y: 0, demand: 0, ready_time: 0, due_time: 0, service_time: 0 }, // empty init, randomize later
			current_path: Vec::with_capacity(nodes.len()),
			current_distance: 0.0,
			costs: Vec::with_capacity(nodes.len()),
			nodes_to_visit: nodes,
			group_id,
			time: 0.0,
			cargo_so_far: 0,
			time_weight,
			random_choice_chance,
		};
	}
	
	fn move_ant(&mut self, world: &mut WorldState, random_source: &mut ThreadRng, deterministic: bool) -> Result<(), AntError> {
		self.costs.clear();

		// we're done
		if self.nodes_to_visit.is_empty() {
			return Err(AntError::NoNodesLeft);
		}

		// preliminary checks
		let mut was_any_node_available_cargo = false;
		let mut was_any_node_available_time = false;
		let mut was_any_node_available_other_colonies = true;

		for node in &mut self.nodes_to_visit {
			let weight_limit = world.weight_limit;
			let data = world.get_edge((self.node_at.attraction_number, node.attraction_number));
			if (self.time + data.length) < node.due_time as f64 {
				// check if there is time left
				was_any_node_available_time = true;
			}
			if self.cargo_so_far + node.demand < weight_limit {
				// check if there is weight left
				was_any_node_available_cargo = true;
			}
			for (index, list) in world.visited_nodes.iter().enumerate() {
				// check if other colonies grabbed the node first
				if index == self.group_id as usize {
					continue;
				}
				if list.contains(&node.attraction_number) {
					was_any_node_available_other_colonies = true;
				}
			}
			if was_any_node_available_cargo && was_any_node_available_other_colonies && was_any_node_available_time {
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

		let mut can_visit = Vec::with_capacity(self.nodes_to_visit.len());
		// pick the next destination
		let next_node: GraphNode;
		let mut cost_sum = 0.0;
		'node_loop: for node in &mut self.nodes_to_visit {
			for (index, list) in world.visited_nodes.iter().enumerate() {
				// check if other colonies grabbed the node first
				if index == self.group_id as usize {
					continue;
				}
				if list.contains(&node.attraction_number) {
					continue 'node_loop;
				}
			}
			let weight_limit = world.weight_limit;
			let data = world.get_edge((self.node_at.attraction_number, node.attraction_number));
			let arrive_time = if self.time + data.length > node.ready_time as f64 { self.time + data.length } else { node.ready_time as f64 };
			if arrive_time > node.due_time as f64 {
				// check if there is time left
				//eprintln!("TIME {} after {:?} at {}, due time is {}", self.time, self.current_path, self.node_at.attraction_number, node.due_time);
				continue;
			}
			if self.cargo_so_far + node.demand > weight_limit {
				// check if there is weight left
				continue;
			}
			// there are no zero length edges
			let time_cost = (node.ready_time as f64).recip().powf(self.time_weight); //(arrive_time - self.time as f64).powf(self.time_weight); //((arrive_time - self.time) / (node.due_time as f64 - self.time)).powf(self.time_weight);
			//eprintln!("time cost is {}, for self.time = {}, arrive_time = {}, node.due_time = {}", time_cost, self.time, arrive_time, node.due_time);
			if data.pheromone_strengths[self.group_id as usize] == 0.0 {
				let cost = data.length_cost * time_cost;
				self.costs.push(cost);
				can_visit.push(node);
				cost_sum += cost;
			} else {
				let cost = data.pheromone_costs[self.group_id as usize] * data.length_cost * time_cost;
				self.costs.push(cost);
				can_visit.push(node);
				cost_sum += cost;
			}
		}

		if self.costs.is_empty() {
			return Err(AntError::IDoNotKnow);
		}

		if deterministic {
			can_visit.sort_unstable_by_key(|x| x.ready_time);
			next_node = *can_visit[0];
		} else {
			if random_source.gen::<f64>() > self.random_choice_chance {
				// roulette selection
				eprintln!("rouletting");
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
		self.nodes_to_visit.swap_remove(self.nodes_to_visit.iter().position(|x| x == &next_node).unwrap());
		self.node_at = next_node;
		//eprintln!("edgelen: {}", edge.length);
		self.time += if self.time + edge.length > next_node.ready_time as f64 { edge.length } else { next_node.ready_time as f64 - self.time} + next_node.service_time as f64;
		self.cargo_so_far += next_node.demand;

		// add note for other colonies to know
		world.visited_nodes[self.group_id as usize].push(next_node.attraction_number);

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
	pub best_solution: Vec<Vec<GraphNode>>, // first index is vehicle id
	pub best_solution_length: f64,
	pub heuristic_weight: f64,
	pub pheromone_weight: f64,
	pub time_weight: f64,
	weight_limit: u32,
	vehicle_count: u8,
	visited_nodes: Vec<Vec<u8>>, // group id is first index
}

impl WorldState {
	pub fn new(input_nodes: Vec<GraphNode>, config: ConfigData, weight_limit: u32, vehicle_count: u8) -> Self {
		let mut visited_nodes = Vec::with_capacity(vehicle_count as usize);
		for group_id in 0..vehicle_count {
			visited_nodes.push(Vec::with_capacity(input_nodes.len()));
			visited_nodes[group_id as usize].push(0);
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
			time_weight: config.time_weight,
			weight_limit,
			vehicle_count,
			visited_nodes,
		};
		unsafe {
			result.edges.set_len(0x1 << 16);
		}

		for _ in 0..config.ant_count_per_vehicle {
			for group in 0..vehicle_count {
				result.ants.push(Ant::new(config.random_choice_chance, result.graph.clone(), group, config.time_weight));
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
			for group_list in &mut self.visited_nodes {
				group_list.clear();
			}
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
	fn move_ants(&mut self, deterministic: bool) {
		let mut random_source = rand::thread_rng();
		let mut temp = self.ants.clone(); // evil clone to get around the borrow checker
		loop {
			let mut anything_changed = false;
			for ant in &mut temp {
				let result = ant.move_ant(self, &mut random_source, deterministic);
				//eprintln!("after move ant is at time {}", ant.time);
				match result {
					Ok(()) => {
						anything_changed = true;
					}
					Err(AntError::OutOfTime) => {
						//eprintln!("ant out of time in colony {}", ant.group_id);
					}
					Err(AntError::CargoFull) => {
						//eprintln!("ant out of cargo space in colony {}", ant.group_id);
					}
					Err(AntError::AllNodesUsedByOtherColonies) => {
						//eprintln!("ant out of untaken nodes in colony {}", ant.group_id);
					}
					Err(AntError::NoNodesLeft) => {
						//eprintln!("ant out of nodes to visit in colony {}", ant.group_id);
					}
					Err(AntError::IDoNotKnow) => {
						//eprintln!("things happened");
					}
				}
			}
			if !anything_changed {
				break;
			}
		}
		for ant in &mut temp {
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
			if ant.current_path.len() == 2 {
				continue;
			}
			for pair in ant.current_path.windows(2) {
				let edge = self.get_edge((pair[0], pair[1]));
				edge.pheromone_strengths[ant.group_id as usize] += ant.current_distance.recip();
				edge.pheromone_costs[ant.group_id as usize] = edge.pheromone_strengths[ant.group_id as usize].powf(pheromone_weight);
			}
		}
	}

	fn update_best_solution(&mut self) {
		// colonies have mutually exclusive routes - will never overlap nodes
		// find ant with best route in each colony
		/*eprintln!("{:#?}", self.ants.iter().map(|x| {
			return (x.group_id, x.current_distance, &x.current_path);
		}).collect_vec());*/
		let mut best_routes = Vec::with_capacity(self.vehicle_count as usize);
		let mut best_route_lengths = Vec::with_capacity(self.vehicle_count as usize);

		for vehicle_id in 0..self.vehicle_count {
			let mut best_route = Vec::new();
			let mut best_route_length = f64::MAX;
			for ant in &self.ants {
				if ant.group_id == vehicle_id && ant.current_path.len() > 2 && ant.current_distance < best_route_length {
					best_route_length = ant.current_distance;
					best_route = ant.current_path.iter().map(|x| {
						return self.graph[*x as usize].clone();
					}).collect_vec();
				}
			}
			best_routes.push(best_route);
			best_route_lengths.push(best_route_length);
		}

		/*for (index, path) in best_routes.iter().enumerate() {
			eprintln!("colony {} with path {:?}", index, path.iter().map(|x| x.attraction_number).collect_vec());
		}*/

		// Check if the best route candidates handle all the nodes when combined together, before actually updating the best solution
		let mut all_nodes = Vec::with_capacity(self.graph.len());
		for node in &self.graph {
			all_nodes.push(node.attraction_number);
		}
		for route in &best_routes {
			for node in route {
				//eprintln!("Removing {}", node.attraction_number);
				if let Some(i) = all_nodes.iter().position(|x| x == &node.attraction_number) {
					all_nodes.swap_remove(i);
				}
			}
		}
		if all_nodes.is_empty() {
			self.best_solution_length = best_route_lengths.into_iter().reduce(|acc, item| acc + item).unwrap();
			self.best_solution = best_routes;
		}
	}

	pub fn do_iteration(&mut self, deterministic: bool) {
		self.init_ants();
		self.move_ants(deterministic);
		self.update_pheromones();
		self.update_best_solution();
	}

	pub fn do_all_iterations(&mut self) -> Result<(), ()> {
		self.do_iteration(true);
		for _ in 0..self.iteration_count {
			self.do_iteration(false);
			if self.best_solution_length == 0.0 {
				return Err(());
			}
		}
		for (index, path) in self.best_solution.iter().enumerate() {
			eprintln!("colony {} with path {:?}", index, path.iter().map(|x| x.attraction_number).collect_vec());
		}
		return Ok(());
	}

	pub fn reset(&mut self) {
		self.init_ants();
		self.init_edges();
		self.best_solution = Vec::new();
		self.best_solution_length = f64::MAX;
		for group_list in &mut self.visited_nodes { // restart the lists of visited nodes by each group
			group_list.clear();
			group_list.push(0);
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
		for colony in &self.best_solution {
			for pair in colony.windows(2) {
				result.push_str(&format!("{} -> {}\n", pair[0].attraction_number, pair[1].attraction_number));
			}
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
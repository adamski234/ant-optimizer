#![allow(clippy::needless_return)]

use std::collections::HashMap;
use itertools::Itertools;
use rand::prelude::*;

#[derive(Debug, Clone, Eq, PartialOrd, Ord, serde::Deserialize)]
pub struct GraphNode {
	pub attraction_number: u8,
	pub x: i32,
	pub y: i32
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
	CannotMove,
}

#[derive(Debug, Clone)]
pub struct Ant {
	pub node_at: GraphNode,
	pub current_path: Vec<u8>, // Added assumption: attraction number is always one higher than index in the original 
	pub current_distance: f64,
	pub random_choice_chance: f64, // less than 1
	nodes_to_visit: Vec<GraphNode>,
	costs: Vec<f64>,
}

impl Ant {
	fn new(random_choice_chance: f64, nodes: Vec<GraphNode>) -> Self {
		return Self {
			node_at: GraphNode { attraction_number: 0, x: 0, y: 0 }, // empty init, randomize later
			current_path: Vec::with_capacity(nodes.len()),
			current_distance: 0.0,
			random_choice_chance,
			costs: Vec::with_capacity(nodes.len()),
			nodes_to_visit: nodes,
		};
	}
	
	fn move_ant(&mut self, world: &mut WorldState, random_source: &mut ThreadRng) -> Result<(), AntError> {
		self.costs.clear();

		// we're done
		if self.nodes_to_visit.is_empty() {
			return Err(AntError::CannotMove);
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
			for node in &mut self.nodes_to_visit {
				let data = world.get_edge((self.node_at.attraction_number, node.attraction_number));
				if data.length == 0.0 {
					// zero distance means we jump straight there and ignore every other possibility
					// removes a node at no cost and it is always the most optimal solution
					// see 67 and 68 in A-n80-k10.txt
					self.current_path.push(self.node_at.attraction_number);
					self.node_at = node.clone();
					to_remove = Some(self.node_at.clone());
				} else {
					if data.pheromone_strength == 0.0 {
						let cost = data.length_cost;
						self.costs.push(cost);
						cost_sum += cost;
					} else {
						let cost = data.pheromone_cost * data.length_cost;
						self.costs.push(cost);
						cost_sum += cost;
					}
				}
			}
			if let Some(node) = to_remove {
				self.nodes_to_visit.swap_remove(self.nodes_to_visit.iter().position(|x| x == &node).unwrap());
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

		self.current_path.push(self.node_at.attraction_number);
		self.current_distance += world.get_edge((self.node_at.attraction_number, next_node.attraction_number)).length;
		self.nodes_to_visit.swap_remove(self.nodes_to_visit.iter().position(|x| x == &next_node).unwrap());
		self.node_at = next_node;

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
	pheromone_strength: f64,
	length: f64,
	length_cost: f64, // 0 if length is 0
	pheromone_cost: f64
}

#[derive(Debug, Clone)]
pub struct ConfigData {
	pub ant_count: usize,
	pub random_choice_chance: f64,
	pub pheromone_weight: f64,
	pub heuristic_weight: f64,
	pub iteration_count: u32,
	pub pheromone_evaporation_coefficient: f64,
}

#[derive(Debug, Clone)]
pub struct SingleIterationEdgeList {
	edges: HashMap<(GraphNode, GraphNode), f64>,
	min_pheromones: f64,
	max_pheromones: f64,
}

#[derive(Debug, Clone)]
pub struct MultipleIterationGraphviz {
	edge_lists: Vec<HashMap<(GraphNode, GraphNode), f64>>,
	min_pheromones: f64,
	max_pheromones: f64,
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
}

impl WorldState {
	pub fn new(input_nodes: Vec<GraphNode>, config: ConfigData) -> Self {
		let mut result = WorldState {
			graph: input_nodes,
			ants: Vec::with_capacity(config.ant_count),
			//edges: fnv::FnvHashMap::with_capacity_and_hasher(attraction_count * (attraction_count - 1) / 2, Default::default()), // holds exactly as many edges as required
			edges: Vec::with_capacity(0x1 << 16),
			iteration_count: config.iteration_count,
			pheromone_evaporation_coefficient: config.pheromone_evaporation_coefficient,
			best_solution: Vec::new(),
			best_solution_length: f64::MAX,
			heuristic_weight: config.heuristic_weight,
			pheromone_weight: config.pheromone_weight,
		};
		unsafe {
			result.edges.set_len(0x1 << 16);
		}

		for _ in 0..config.ant_count {
			result.ants.push(Ant::new(config.random_choice_chance, result.graph.clone()));
		}

		result.init_edges();

		return result;
	}

	fn init_edges(&mut self) {
		for (index, node) in self.graph.iter().enumerate() {
			for second_node in self.graph[index + 1 ..].iter() {
				//println!("{}, {} => {}", node.attraction_number, second_node.attraction_number, (((node.attraction_number as u16) << 8) | (second_node.attraction_number as u16)));
				let length = node.distance_to(second_node);
				let to_insert = EdgeData {
					first_node: node.clone(),
					second_node: second_node.clone(),
					length: length,
					pheromone_strength: 0.01,
					length_cost: if length != 0.0 { length.recip().powf(self.heuristic_weight) } else { 0.0 },
					pheromone_cost: (0.01_f64).powf(self.pheromone_weight),
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
			ant.node_at = self.graph.choose(&mut rand::thread_rng()).unwrap().clone();
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
			ant.current_distance += self.get_edge((ant.current_path[ant.current_path.len() - 2].clone(), ant.node_at.attraction_number)).length;
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
				let edge = self.get_edge((pair[0].clone(), pair[1].clone()));
				edge.pheromone_strength += ant.current_distance.recip();
				edge.pheromone_cost = edge.pheromone_strength.powf(pheromone_weight);
			}
		}
	}

	fn update_best_solution(&mut self) {
		for ant in &self.ants {
			if ant.current_distance < self.best_solution_length {
				self.best_solution = ant.current_path.iter().map(|x| {
					return self.graph[(*x - 1) as usize].clone();
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

	pub fn do_all_iterations_with_edge_recording(&mut self) -> MultipleIterationGraphviz {
		let mut result = MultipleIterationGraphviz {
			edge_lists: Vec::with_capacity(self.iteration_count as usize),
			max_pheromones: f64::MIN,
			min_pheromones: f64::MAX,
		};
		for _ in 0..self.iteration_count {
			self.do_iteration();
			let single_result = self.edge_pheromones_to_list();
			result.edge_lists.push(single_result.edges);
			if single_result.max_pheromones > result.max_pheromones {
				result.max_pheromones = single_result.max_pheromones;
			}
			if single_result.min_pheromones < result.min_pheromones {
				result.min_pheromones = single_result.min_pheromones;
			}
		}
		return result;
	}

	pub fn do_all_iterations_with_graphviz_recording(&mut self, low_color: colorgrad::Color, high_color: colorgrad::Color) -> Vec<String> {
		let edge_recordings = self.do_all_iterations_with_edge_recording();
		let color_source = colorgrad::CustomGradient::new()
			.colors(&[high_color, low_color])
			.domain(&[edge_recordings.min_pheromones, edge_recordings.max_pheromones])
			.build().unwrap();
		let mut result = Vec::with_capacity(edge_recordings.edge_lists.len());

		for iteration_edges in edge_recordings.edge_lists {
			let mut iteration_edge_list = String::new();
			for (pair, value) in iteration_edges {
				iteration_edge_list.push_str(&format!("{} -- {} [color = \"{}\"]\n", pair.0.attraction_number, pair.1.attraction_number, color_source.at(value).to_hex_string()));
			}
			result.push(iteration_edge_list);
		}

		return result;
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

	pub fn edge_pheromones_to_list(&self) -> SingleIterationEdgeList {
		// create a graph with edges colored according to their pheromone strength
		let mut result = SingleIterationEdgeList {
			edges: HashMap::new(),
			max_pheromones: f64::MIN,
			min_pheromones: f64::MAX,
		};

		for edge in &self.edges {
			result.edges.insert((edge.first_node.clone(), edge.second_node.clone()), edge.pheromone_strength);
			if edge.pheromone_strength > result.max_pheromones {
				result.max_pheromones = edge.pheromone_strength;
			}
			if edge.pheromone_strength < result.min_pheromones {
				result.min_pheromones = edge.pheromone_strength;
			}
		}

		return result;
	}

	pub fn do_bruteforce(&mut self) {
		// it's not supposed to be quick but it has to create a solution
		for solution in self.graph.clone().into_iter().permutations(self.graph.len()).unique() {
			let mut sum = 0.0;
			for pair in solution.windows(2) {
				sum += self.get_edge((pair[0].attraction_number, pair[1].attraction_number)).length;
			}
			if sum < self.best_solution_length {
				self.best_solution_length = sum;
				self.best_solution = solution;
			}
		}
	}
}
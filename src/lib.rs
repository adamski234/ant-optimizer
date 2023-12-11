#![allow(clippy::needless_return)]

use std::collections::HashMap;
use rand::prelude::*;

#[derive(Debug, Clone, Eq, PartialOrd, Ord, serde::Deserialize)]
pub struct GraphNode {
	pub attraction_number: u32,
	pub x: i32,
	pub y: i32
}

impl std::hash::Hash for GraphNode {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		state.write_u32(self.attraction_number);
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
	pub current_path: Vec<GraphNode>,
	pub current_distance: f64,
	pub random_choice_chance: f64, // less than 1
	pub pheromone_weight: i32,
	pub heuristic_weight: i32
}

impl Ant {
	fn new(heuristic_weight: i32, pheromone_weight: i32, random_choice_chance: f64) -> Self {
		return Self {
			node_at: GraphNode { attraction_number: 0, x: 0, y: 0 }, // empty init, randomize later
			current_path: Vec::new(),
			current_distance: 0.0,
			heuristic_weight,
			pheromone_weight,
			random_choice_chance,
		};
	}
	
	fn move_ant(&mut self, world: &mut WorldState) -> Result<(), AntError> {
		// Generate all possible ways to go
		let mut possible_paths = Vec::with_capacity(world.edges.len());
		for node in &mut world.graph {
			if node != &self.node_at && !self.current_path.contains(node) {
				possible_paths.push(node.clone());
			}
		}
		// we're done
		if possible_paths.is_empty() {
			return Err(AntError::CannotMove);
		}

		// create the costs table
		let mut costs = Vec::with_capacity(possible_paths.len());
		for node in &mut possible_paths {
			let data = world.get_edge((self.node_at.clone(), node.clone()));
			if data.length == 0.0 {
				// zero distance means we jump straight there and ignore every other possibility
				// removes a node at no cost and it is always the most optimal solution
				// see 67 and 68 in A-n80-k10.txt
				self.current_path.push(self.node_at.clone());
				self.node_at = node.clone();
				return Ok(());
			} else {
				costs.push(data.pheromone_strength.powi(self.pheromone_weight) * data.length.recip().powi(self.heuristic_weight));
			}
		}

		// and pick the next destination
		let next_node;
		if rand::thread_rng().gen::<f64>() < self.random_choice_chance {
			// random uniform selection
			next_node = possible_paths.choose(&mut rand::thread_rng()).unwrap().clone();
		} else {
			// roulette selection
			next_node = possible_paths[rand::distributions::WeightedIndex::new(costs).unwrap().sample(&mut rand::thread_rng())].clone();
		}

		self.current_path.push(self.node_at.clone());
		self.current_distance += world.get_edge((self.node_at.clone(), next_node.clone())).length;
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
	pheromone_strength: f64,
	length: f64,
}

#[derive(Debug, Clone)]
pub struct ConfigData {
	pub ant_count: usize,
	pub random_choice_chance: f64,
	pub pheromone_weight: i32,
	pub heuristic_weight: i32,
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
	pub edges: fnv::FnvHashMap<(GraphNode, GraphNode), EdgeData>, // populate at init, key is ordered tuple simulating an unordered pair, with first node having lower att number
	iteration_count: u32,
	pheromone_evaporation_coefficient: f64,
	pub best_solution: Vec<GraphNode>,
	pub best_solution_length: f64,
}

impl WorldState {
	pub fn new(input_nodes: Vec<GraphNode>, config: ConfigData) -> Self {
		let attraction_count = input_nodes.len();
		let mut result = WorldState {
			graph: input_nodes,
			ants: Vec::with_capacity(config.ant_count),
			edges: fnv::FnvHashMap::with_capacity_and_hasher(attraction_count * (attraction_count - 1) / 2, Default::default()), // holds exactly as many edges as required
			iteration_count: config.iteration_count,
			pheromone_evaporation_coefficient: config.pheromone_evaporation_coefficient,
			best_solution: Vec::new(),
			best_solution_length: f64::MAX,
		};

		for _ in 0..config.ant_count {
			result.ants.push(Ant::new(config.heuristic_weight, config.pheromone_weight, config.random_choice_chance));
		}

		result.init_edges();

		return result;
	}

	fn init_edges(&mut self) {
		for (index, node) in self.graph.iter().enumerate() {
			for second_node in self.graph[index + 1 ..].iter() {
				let to_insert = EdgeData {
					length: node.distance_to(second_node),
					pheromone_strength: 0.01
				};
				if node.attraction_number > second_node.attraction_number {
					self.edges.insert((second_node.clone(), node.clone()), to_insert);
				} else {
					self.edges.insert((node.clone(), second_node.clone()), to_insert);
				}
			}
		}
	}
	
	pub fn init_ants(&mut self) {
		for ant in &mut self.ants {
			ant.clear();
			ant.node_at = self.graph.choose(&mut rand::thread_rng()).unwrap().clone();
		}
	}

	fn get_edge(&mut self, pair: (GraphNode, GraphNode)) -> &mut EdgeData {
		if pair.0.attraction_number > pair.1.attraction_number {
			return self.edges.get_mut(&(pair.1, pair.0)).unwrap();
		} else {
			return self.edges.get_mut(&pair).unwrap();
		}
	}

	// moves ants until they're all done
	fn move_ants(&mut self) {
		let mut temp = self.ants.clone(); // evil clone to get around the borrow checker
		for ant in &mut temp {
			while ant.move_ant(self).is_ok() {
				//
			}
			ant.current_path.push(ant.node_at.clone());
			ant.current_distance += self.get_edge((ant.current_path[ant.current_path.len() - 2].clone(), ant.node_at.clone())).length;
		}
		self.ants = temp;
	}

	fn update_pheromones(&mut self) {
		// evaporate pheromones
		for data in &mut self.edges.values_mut() {
			data.pheromone_strength *= self.pheromone_evaporation_coefficient;
		}

		// add pheromones
		for ant in &self.ants.clone() {
			for pair in ant.current_path.windows(2) {
				self.get_edge((pair[0].clone(), pair[1].clone())).pheromone_strength += ant.current_distance.recip();
			}
		}
	}

	fn update_best_solution(&mut self) {
		for ant in &self.ants {
			if ant.current_distance < self.best_solution_length {
				self.best_solution = ant.current_path.clone();
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

		for (key, data) in &self.edges {
			result.edges.insert(key.clone(), data.pheromone_strength);
			if data.pheromone_strength > result.max_pheromones {
				result.max_pheromones = data.pheromone_strength;
			}
			if data.pheromone_strength < result.min_pheromones {
				result.min_pheromones = data.pheromone_strength;
			}
		}

		return result;
	}
}
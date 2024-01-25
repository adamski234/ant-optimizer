use std::os::unix::fs::MetadataExt;

fn main() {
	let header = "ants,iterations,evap_coeff,rand_chance,pher_weight,heur_weight,time_weight,graph_name,shortest_route,longest_route,average_route,time_for_stat_run";
	let gex = regex::Regex::new(r"File (.*): .*is (-?\d*\.?\d*),.*is (-?\d*\.?\d*)\. .*is (-?\d*\.?\d*)").unwrap();
	//let gex = regex::Regex::new(r".*is (-?\d*\.?\d*),.*is (-?\d*\.?\d*)\. .*is (-?\d*\.?\d*)").unwrap();

	println!("{}", header);

	for filename in glob::glob("./output/*_run_data.csv").unwrap() {
		let filename = filename.unwrap();
		eprintln!("{}", filename.display());
		let run_data = std::fs::read_to_string(&filename).unwrap();
		let run_data = run_data.lines().nth(1).unwrap();
		let stat_file = filename.to_str().unwrap().replace("run_data.csv", "statistics.txt");
		let metadata = std::fs::metadata(&stat_file).unwrap();
		let runtime = metadata.ctime() as u64 - metadata.created().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
		let stat_data = std::fs::read_to_string(&stat_file).unwrap().lines().map(|line| {
			let captures = gex.captures(line).unwrap();
			return format!("{},{},{},{},{},{}", run_data, &captures[1], &captures[3], &captures[2], &captures[4], runtime);
		}).collect::<Vec<_>>().join("\n");
		println!("{}", stat_data);
	}
}

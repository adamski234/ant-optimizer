#!/bin/bash
ant_counts=(50) #1
iteration_counts=(1000) #1
evaporation_coefficients=(0.85) #1
random_choice_chances=(0.15) #1
pheromone_weights=(1) #1
heuristic_weights=(3) #1
time_weights=(0 0.2 0.5 1 1.3 1.7 2 2.3 2.7 3) #10
runs_per_set=64

rm -rf output
#rm -rf output_*
mkdir -p output

for ants in "${ant_counts[@]}"
do
	for iterations in "${iteration_counts[@]}"
	do
		for evaporation_coefficient in "${evaporation_coefficients[@]}"
		do
			for random_choice_chance in "${random_choice_chances[@]}"
			do
				for pheromone_weight in "${pheromone_weights[@]}"
				do
					for heuristic_weight in "${heuristic_weights[@]}"
					do
						for time_weight in "${time_weights[@]}"
						do
							./target/release/ant_colony --path data/r101.csv --ant-count $ants --iterations $iterations --evaporation $evaporation_coefficient --random-chance $random_choice_chance --pheromone-weight $pheromone_weight --heuristic-weight $heuristic_weight --time-weight $time_weight --try-count $runs_per_set > output/"$time_weight"_time_statistics.txt
							echo Finished $time_weight
							echo "ants,iterations,evap_coeff,rand_chance,pher_weight,heur_weight,time_weight" > output/"$time_weight"_time_run_data.csv
							echo "$ants,$iterations,$evaporation_coefficient,$random_choice_chance,$pheromone_weight,$heuristic_weight,$time_weight" >> output/"$time_weight"_time_run_data.csv
						done
					done
				done
			done
		done
	done
done

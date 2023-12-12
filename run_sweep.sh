#!/bin/bash
ant_counts=(10 30 50) #3
iteration_counts=(500 1000 2000) #3
evaporation_coefficients=(0 0.5 1) #2
random_choice_chances=(0.3) #1
pheromone_weights=(1 2 5) #3
heuristic_weights=(1 3 5) #3
runs_per_set=256

rm -rf output
rm -rf output_*
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
						mkdir -p output
						./batch_video.sh --path data/ --ant-count $ants --iterations $iterations --evaporation-coeff $evaporation_coefficient --random-choice-chance $random_choice_chance --pheromone-weight $pheromone_weight --heuristic-weight $heuristic_weight
						./target/release/ant_colony --batch --path data/ --ant-count $ants --iterations $iterations --evaporation-coeff $evaporation_coefficient --random-choice-chance $random_choice_chance --pheromone-weight $pheromone_weight --heuristic-weight $heuristic_weight --try-count $runs_per_set > output/statistics.txt
						echo "ants,iterations,evap_coeff,rand_chance,pher_weight,heur_weight" > output/run_data.csv
						echo "$ants,$iterations,$evaporation_coefficient,$random_choice_chance,$pheromone_weight,$heuristic_weight" >> output/run_data.csv
						for dir in output/*/
						do
							cd $dir
							mv images/out.mkv ./recording.mkv
							mv images/solution.dot.png ./solution.png
							rm -rf images/ ./*.dot
							cd ../..
						done
						mv output output_"$ants"_ants_"$iterations"_iters_"$evaporation_coefficient"_evapcoeff_"$random_choice_chance"_randchch_"$pheromone_weight"_pher_"$heuristic_weight"_heur
						#exit 1
					done
				done
			done
		done
	done
done

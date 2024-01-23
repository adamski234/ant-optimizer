#!/bin/bash
ant_counts=(15) #3
iteration_counts=(10 30 50 100 300) #1
evaporation_coefficients=(0.25 0.75) #3
random_choice_chances=(0.3) #2
pheromone_weights=(0.0 0.2 0.4 0.6 0.8 1.0 1.2 1.4 1.6 1.8 2.0 2.2 2.4 2.6 2.8 3.0 3.2 3.4 3.6 3.8 4.0 4.2 4.4 4.6 4.8 5.0 5.2 5.4 5.6 5.8 6.0 6.2 6.4 6.6 6.8 7.0 7.2 7.4 7.6 7.8 8.0 8.2 8.4 8.6 8.8 9.0 9.2 9.4 9.6 9.8 10.0) #3
#heuristic_weights=(0.0 0.1 0.2 0.3 0.4 0.5 0.6 0.7 0.8 0.9 1.0 1.1 1.2 1.3 1.4 1.5 1.6 1.7 1.8 1.9 2.0 2.1 2.2 2.3 2.4 2.5 2.6 2.7 2.8 2.9 3.0) #3
#heuristic_weights=(1 3 5) #3
runs_per_set=128

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
					for heuristic_weight in "${pheromone_weights[@]}"
					do
#						mkdir -p output
						#./batch_solve.sh --path data/ --ant-count $ants --iterations $iterations --evaporation-coeff $evaporation_coefficient --random-choice-chance $random_choice_chance --pheromone-weight $pheromone_weight --heuristic-weight $heuristic_weight
						echo Finished solving, running statistics
						./target/release/ant_colony --path data/P-n76-k5.txt --ant-count $ants --iterations $iterations --evaporation-coeff $evaporation_coefficient --random-choice-chance $random_choice_chance --pheromone-weight $pheromone_weight --heuristic-weight $heuristic_weight --try-count $runs_per_set > output/"$ants"_ants_"$iterations"_iters_"$evaporation_coefficient"_evapcoeff_"$random_choice_chance"_randchch_"$pheromone_weight"_pher_"$heuristic_weight"_heur_statistics.txt
						echo "ants,iterations,evap_coeff,rand_chance,pher_weight,heur_weight" > output/"$ants"_ants_"$iterations"_iters_"$evaporation_coefficient"_evapcoeff_"$random_choice_chance"_randchch_"$pheromone_weight"_pher_"$heuristic_weight"_heur_run_data.csv
						echo "$ants,$iterations,$evaporation_coefficient,$random_choice_chance,$pheromone_weight,$heuristic_weight" >> output/"$ants"_ants_"$iterations"_iters_"$evaporation_coefficient"_evapcoeff_"$random_choice_chance"_randchch_"$pheromone_weight"_pher_"$heuristic_weight"_heur_run_data.csv
						#mv output output_"$ants"_ants_"$iterations"_iters_"$evaporation_coefficient"_evapcoeff_"$random_choice_chance"_randchch_"$pheromone_weight"_pher_"$heuristic_weight"_heur
					done
				done
			done
		done
	done
done

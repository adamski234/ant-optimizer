import pandas
from matplotlib import pyplot
import os

input_vars = ["ants", "evap_coeff", "rand_chance", "pher_weight", "heur_weight"]
output_vars = ["shortest_route", "longest_route", "average_route", "time_for_stat_run"]


data_source = pandas.read_csv("./statistics.csv")

# ants,iterations,evap_coeff,rand_chance,pher_weight,heur_weight,graph_name,shortest_route,longest_route,average_route,time_for_stat_run
# 10,1000,0.5,0.3,1,1,A-n32-k5.txt,578.8555406057682,734.0665272183886,667.1761024279239,213

try:
	os.removedirs("./graphs/")
except:
	print("didn't remove")
os.makedirs("./graphs/", exist_ok=True)

#data_source = data_source[(data_source["evap_coeff"] != 0.0) & (data_source["rand_chance"] != 0.8)]

for file_name in data_source["graph_name"].unique():
	graph_data = data_source[data_source["graph_name"] == file_name].drop(columns=["graph_name", "iterations"])
	print(file_name)
	print(graph_data.sort_values("average_route"))
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
import pandas
import plotly
import plotly.graph_objects as graph_objects
from plotly.subplots import make_subplots

data = pandas.read_csv("./statistics_smaller.csv", delimiter=",")

# Create a 3D scatter plot

data_5ants_10iters = data[(data["ants"] == 5) & (data["iterations"] == 10)]
data_10ants_10iters = data[(data["ants"] == 10) & (data["iterations"] == 10)]
data_20ants_10iters = data[(data["ants"] == 20) & (data["iterations"] == 10)]
data_5ants_30iters = data[(data["ants"] == 5) & (data["iterations"] == 30)]
data_10ants_30iters = data[(data["ants"] == 10) & (data["iterations"] == 30)]
data_20ants_30iters = data[(data["ants"] == 20) & (data["iterations"] == 30)]
data_5ants_50iters = data[(data["ants"] == 5) & (data["iterations"] == 50)]
data_10ants_50iters = data[(data["ants"] == 10) & (data["iterations"] == 50)]
data_20ants_50iters = data[(data["ants"] == 20) & (data["iterations"] == 50)]
data_5ants_100iters = data[(data["ants"] == 5) & (data["iterations"] == 100)]
data_10ants_100iters = data[(data["ants"] == 10) & (data["iterations"] == 100)]
data_20ants_100iters = data[(data["ants"] == 20) & (data["iterations"] == 100)]
data_5ants_300iters = data[(data["ants"] == 5) & (data["iterations"] == 300)]
data_10ants_300iters = data[(data["ants"] == 10) & (data["iterations"] == 300)]
data_20ants_300iters = data[(data["ants"] == 20) & (data["iterations"] == 300)]

# fig = plt.figure()

# ax11 = fig.add_subplot(5, 3, (1, 1), projection='3d')
# ax11.plot_trisurf(data_5ants_10iters["pher_weight"], data_5ants_10iters["heur_weight"], data_5ants_10iters["shortest_route"])
# ax11.plot_trisurf(data_5ants_10iters["pher_weight"], data_5ants_10iters["heur_weight"], data_5ants_10iters["longest_route"])
# ax11.plot_trisurf(data_5ants_10iters["pher_weight"], data_5ants_10iters["heur_weight"], data_5ants_10iters["average_route"])
# ax11.set_xlabel('Pher')
# ax11.set_ylabel('Heur')
# ax11.set_zlabel('Route len')

# ax12 = fig.add_subplot(5, 3, (1, 2), projection='3d')
# ax12.plot_trisurf(data_10ants_10iters["pher_weight"], data_10ants_10iters["heur_weight"], data_10ants_10iters["shortest_route"])
# ax12.plot_trisurf(data_10ants_10iters["pher_weight"], data_10ants_10iters["heur_weight"], data_10ants_10iters["longest_route"])
# ax12.plot_trisurf(data_10ants_10iters["pher_weight"], data_10ants_10iters["heur_weight"], data_10ants_10iters["average_route"])
# ax12.set_xlabel('Pher')
# ax12.set_ylabel('Heur')
# ax12.set_zlabel('Route len')

# ax13 = fig.add_subplot(5, 3, (1, 3), projection='3d')
# ax13.plot_trisurf(data_20ants_10iters["pher_weight"], data_20ants_10iters["heur_weight"], data_20ants_10iters["shortest_route"])
# ax13.plot_trisurf(data_20ants_10iters["pher_weight"], data_20ants_10iters["heur_weight"], data_20ants_10iters["longest_route"])
# ax13.plot_trisurf(data_20ants_10iters["pher_weight"], data_20ants_10iters["heur_weight"], data_20ants_10iters["average_route"])
# ax13.set_xlabel('Pher')
# ax13.set_ylabel('Heur')
# ax13.set_zlabel('Route len')

# ax21 = fig.add_subplot(5, 3, (2, 1), projection='3d')
# ax21.plot_trisurf(data_5ants_30iters["pher_weight"], data_5ants_30iters["heur_weight"], data_5ants_30iters["shortest_route"])
# ax21.plot_trisurf(data_5ants_30iters["pher_weight"], data_5ants_30iters["heur_weight"], data_5ants_30iters["longest_route"])
# ax21.plot_trisurf(data_5ants_30iters["pher_weight"], data_5ants_30iters["heur_weight"], data_5ants_30iters["average_route"])
# ax21.set_xlabel('Pher')
# ax21.set_ylabel('Heur')
# ax21.set_zlabel('Route len')

# ax22 = fig.add_subplot(5, 3, (2, 2), projection='3d')
# ax22.plot_trisurf(data_10ants_30iters["pher_weight"], data_10ants_30iters["heur_weight"], data_10ants_30iters["shortest_route"])
# ax22.plot_trisurf(data_10ants_30iters["pher_weight"], data_10ants_30iters["heur_weight"], data_10ants_30iters["longest_route"])
# ax22.plot_trisurf(data_10ants_30iters["pher_weight"], data_10ants_30iters["heur_weight"], data_10ants_30iters["average_route"])
# ax22.set_xlabel('Pher')
# ax22.set_ylabel('Heur')
# ax22.set_zlabel('Route len')

# ax23 = fig.add_subplot(5, 3, (2, 3), projection='3d')
# ax23.plot_trisurf(data_20ants_30iters["pher_weight"], data_20ants_30iters["heur_weight"], data_20ants_30iters["shortest_route"])
# ax23.plot_trisurf(data_20ants_30iters["pher_weight"], data_20ants_30iters["heur_weight"], data_20ants_30iters["longest_route"])
# ax23.plot_trisurf(data_20ants_30iters["pher_weight"], data_20ants_30iters["heur_weight"], data_20ants_30iters["average_route"])
# ax23.set_xlabel('Pher')
# ax23.set_ylabel('Heur')
# ax23.set_zlabel('Route len')


# ax11.shareview(ax12)
# ax12.shareview(ax13)

# plt.tight_layout()

# plt.show()

fig = make_subplots(rows=5, cols=3, specs=[
	[{"type": "surface"}, {"type": "surface"}, {"type": "surface"}],
	[{"type": "surface"}, {"type": "surface"}, {"type": "surface"}],
	[{"type": "surface"}, {"type": "surface"}, {"type": "surface"}],
	[{"type": "surface"}, {"type": "surface"}, {"type": "surface"}],
	[{"type": "surface"}, {"type": "surface"}, {"type": "surface"}],
])

#fig.write_html("./dupa.html")

fig.add_trace(graph_objects.Surface(x=data_5ants_10iters["pher_weight"], y=data_5ants_10iters["heur_weight"], z=data_5ants_10iters["shortest_route"]), row=1, col=1)

fig.show(renderer="iframe")
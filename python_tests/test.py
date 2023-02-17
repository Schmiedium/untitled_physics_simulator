import untitled_physics_simulator as ps
import pyarrow
import polars as pl
import sys

#create simulation with 0.001 seconds per timestep, and a sim duration of 5.0 seconds. ~5000 steps
sim = ps.Simulation(0.001, 5.0)

geo = "/home/alex/Documents/3D_Geometry/OBJs/icosahedron.obj"

e1 = ps.Entity("Dynamic", "test_builder").add_transform(10.0, 15.0, 0.0).add_geometry(geo)

#create entities and add to the sim
#you will have to supply your own objs, be carefuul they don't overlap on spawn
sim.create_entity(index = 3, name = "test1", entity_type = "Dynamic", position = (10.0, 10.0, 0.0), velocity = (0.0, 0.0, 0.0), geometry = "/home/alex/Documents/3D_Geometry/OBJs/icosahedron.obj")
sim.create_entity(index = 14, name = "test12", entity_type = "Dynamic", position = (0.0, 20.0, -10.0), velocity = (0.0, 0.0, 0.0), geometry = "/home/alex/Documents/3D_Geometry/OBJs/icosahedron.obj")
sim.add_entity(e1, 15)

#run the simulation with a render, render can be turned off by using simulation_run_headless
#store the output data in a variable
result = ps.simulation_run_headless(sim)

#print the output
print(result)

#Optionally write dataframes to csv or parquet
#These are polars dataframes, so you an do whatever with them


# result['test12'].write_csv("test12.csv")
# result['test7'].write_parquet("test7.parquet")




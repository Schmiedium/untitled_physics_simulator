from untitled_physics_simulator._untitled_physics_simulator import Simulation, Entity, TestModel, Warhead, simulation_run_headless, simulation_run
import pyarrow
import polars as pl
import sys
from dask.distributed import Client


def do_simulation_things():
    #create simulation with 0.001 seconds per timestep, and a sim duration of 5.0 seconds. ~5000 steps
    sim = Simulation(0.001, 1.0)

    geo = "/home/alex/Documents/3D_Geometry/OBJs/icosahedron.obj"

    # e1 = ps.Entity("Dynamic", "test_builder").add_transform(10.0, 15.0, 0.0).add_geometry(geo)
    entities = []

    x= 3; y = 3; z = 3

    e = Entity("Dynamic", f"test_{x}_{y}_{z}").add_transform(float(x), float(y), float(z)).add_geometry(geo)
    
    wh = TestModel()
    e = e.add_component(wh)
    entities.append(e)

    i = 1
    for entity in entities:
        sim.add_entity(entity, i)
        i = i+1

    #create entities and add to the sim
    #you will have to supply your own objs, be carefuul they don't overlap on spawn
    # sim.create_entity(index = 3, name = "test1", entity_type = "Dynamic", position = (10.0, 10.0, 0.0), velocity = (0.0, 0.0, 0.0), geometry = "/home/alex/Documents/3D_Geometry/OBJs/icosahedron.obj")
    # sim.create_entity(index = 14, name = "test12", entity_type = "Dynamic", position = (0.0, 20.0, -10.0), velocity = (0.0, 0.0, 0.0), geometry = "/home/alex/Documents/3D_Geometry/OBJs/icosahedron.obj")
    # sim.add_entity(e1, 15)

    return simulation_run_headless(sim)


def main():
    # client = Client(threads_per_worker=4, n_workers=2)
    # client


    #run the simulation with a render, render can be turned off by using simulation_run_headless
    #store the output data in a variable
    # future = client.submit(do_simulation_things)
    print(do_simulation_things())
    # result = do_simulation_things()

    #print the output
    # print(future.result())

    #Optionally write dataframes to csv or parquet
    #These are polars dataframes, so you an do whatever with them
    # result['test12'].write_csv("test12.csv")
    # result['test7'].write_parquet("test7.parquet")


if __name__ == '__main__':
    main()

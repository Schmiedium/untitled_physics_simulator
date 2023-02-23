import untitled_physics_simulator as ps
import pyarrow
import polars as pl
import sys
from dask.distributed import Client


def do_simulation_things():
    #create simulation with 0.001 seconds per timestep, and a sim duration of 5.0 seconds. ~5000 steps
    sim = ps.Simulation(0.001, 1.0)

    geo = "/home/alex/Documents/3D_Geometry/OBJs/icosahedron.obj"

    # e1 = ps.Entity("Dynamic", "test_builder").add_transform(10.0, 15.0, 0.0).add_geometry(geo)
    entities = []
    for x in range(0, 20, 3):
        for y in range(2, 20, 3):
            for z in range(0, 20, 3):
                e = ps.Entity("Dynamic", f"test_{x}_{y}_{z}").add_transform(float(x), float(y), float(z)).add_geometry(geo)
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

    return ps.simulation_run_headless(sim)


def main():
    # client = Client(threads_per_worker=4, n_workers=2)
    # client

    #run the simulation with a render, render can be turned off by using simulation_run_headless
    #store the output data in a variable
    # future = client.submit(do_simulation_things)
    print(sys.getsizeof(do_simulation_things()))
    # result = do_simulation_things()

    #print the output
    # print(future.result())

    #Optionally write dataframes to csv or parquet
    #These are polars dataframes, so you an do whatever with them
    # result['test12'].write_csv("test12.csv")
    # result['test7'].write_parquet("test7.parquet")


if __name__ == '__main__':
    main()

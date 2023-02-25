from untitled_physics_simulator._untitled_physics_simulator import Simulation, Entity, TestModel, Warhead, simulation_run_headless, simulation_run
import pyarrow
import polars as pl


def do_simulation_things():
    #create simulation with 0.001 seconds per timestep, and a sim duration of 5.0 seconds. ~5000 steps
    sim = Simulation(0.001, 1.0)

    geo = "/home/alex/Documents/3D_Geometry/OBJs/icosahedron.obj"

    entities = []

    x= 3; y = 3; z = 3

    e = Entity("Dynamic", f"test_{x}_{y}_{z}").add_transform(float(x), float(y), float(z)).add_geometry(geo)
    
    wh = TestModel("test_string")
    e = e.add_component(wh)
    entities.append(e)

    i = 1
    for entity in entities:
        sim.add_entity(entity, i)
        i = i+1


    return simulation_run_headless(sim)


def main():

    #run the simulation with a render, render can be turned off by using simulation_run_headless
    #store the output data in a variable
    print(do_simulation_things())



    #Optionally write dataframes to csv or parquet
    #These are polars dataframes, so you an do whatever with them
    # result['test12'].write_csv("test12.csv")
    # result['test7'].write_parquet("test7.parquet")


if __name__ == '__main__':
    main()

from untitled_physics_simulator import Simulation, Entity, TestModel, Warhead, simulation_run_headless, simulation_run, Gun
import polars as pl


def do_simulation_things():
    #create simulation with 0.001 seconds per timestep, and a sim duration of 5.0 seconds. ~5000 steps
    sim = Simulation(0.001, 9.0, 15.0)

    geo = "/home/alex/Documents/3D_Geometry/OBJs/icosahedron.obj"

    entities = []

    x = 0
    z = 0
    y = 3


    e = Entity("Dynamic", f"test_{x}_{y}_{z}").add_transform(float(x), float(y), float(z))
    gun = Gun(4)
    e = e.add_component(gun)

    entities.append(e)



    sim.add_entities(entities)

    print(f"simulation constructed with {len(entities)} entities")
    # return simulation_run_headless(sim)
    return simulation_run(sim)
    # return sim.scene_to_ron()

def main():

    #run the simulation with a render, render can be turned off by using simulation_run_headless
    #store the output data in a variable
    result = do_simulation_things()
    print(result)

    bullet = result['Bullet']['Transform'].join(result['Bullet']['Physics'], on="Time", how="left")



    #Optionally write dataframes to csv or parquet
    #These are polars dataframes, so you an do whatever with them
    bullet.write_csv("bullet.csv")
    # result['test7'].write_parquet("test7.parquet")


if __name__ == '__main__':
    main()

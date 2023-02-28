from untitled_physics_simulator import Simulation, Entity, TestModel, Warhead, simulation_run_headless, simulation_run, Gun



def do_simulation_things():
    #create simulation with 0.001 seconds per timestep, and a sim duration of 5.0 seconds. ~5000 steps
    sim = Simulation(0.01, 5.0, 3600.0)

    geo = "/home/alex/Documents/3D_Geometry/OBJs/icosahedron.obj"

    entities = []

    for x in range(0, 33, 3):
        for y in range(9, 33, 3):
            for z in range(0, 33, 3):
                e = Entity("Dynamic", f"test_{x}_{y}_{z}").add_transform(float(x), float(y), float(z)).add_geometry(geo, "Trimesh")
                # gun = Gun(4)

                # e = e.add_component(gun)
                entities.append(e)

    sim.add_entities(entities)

    print(f"simulation constructed with {len(entities)} entities")
    return simulation_run(sim)


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

import untitled_physics_simulator as ps
import pyarrow
import polars as pl


sim = ps.Simulation(0.01, 5.0)
sim.create_entity(index = 3, name = "test", entity_type = "Dynamic", position = (10.0, 0.0, 0.0), velocity = (0.0, 0.0, 0.0))

result = ps.simulation_run(sim)


print(result)
# result['Ball'].write_csv("Ball.csv")
# result['Ball'].write_parquet("Ball.parquet")




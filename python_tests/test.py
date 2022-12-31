import untitled_physics_simulator as ps
import pyarrow
import polars as pl


sim = ps.Simulation()
sim.create_entity(3, "test", "Dynamic", (10.0, 0.0, 0.0), (0.0, 0.0, 0.0))

result = ps.simulation_run(0.01, 5.0, sim)


print(result)
# result['Ball'].write_csv("Ball.csv")
# result['Ball'].write_parquet("Ball.parquet")




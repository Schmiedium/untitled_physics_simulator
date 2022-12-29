import untitled_physics_simulator as ps
import pyarrow
import polars as pl




result = ps.simulation_run(0.01, 5.0)


print(result)
result['Ball'].write_csv("Ball.csv")
result['Ball'].write_parquet("Ball.parquet")




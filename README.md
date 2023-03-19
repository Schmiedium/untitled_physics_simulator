# Untitled_Physics_Simulator

I threw bevy and bevy_rapier3d into a python package using maturin, mostly for fun

To try it out, make a new python environment and run "pip install maturin polars[all] patchelf"

Then you should be able to run "maturin develop" with that environment to build and install the package. add the "-r" flag to build release

There is an included test python file to see what's available

you'll need to supply your own .obj files for now

This project assumes NUE coordinates, that is positive x-axis is North, positive y-axis is Up, and positive z-axis is West.

TODO list:

let PSComponent derive macro derive the necessary trait bounds as well, so we only need to derive the one trait
figure out cpu_thread limiter
document all the code
implement caching for convex decomposition of geometry and expose the option

figure out how to expose mass_properties for colliders (mainly inertia tensor and mass, also elasticity)
add aeroballistic force to all entities (going to be bad time)
add a way to parse input trajectories. Thinking C2 continuous splines, or maybe that quartic spline from the continuity of splines video?


I think if I want to add bullets as entities, they would need to be resources, like the simulation object. Then add a flag to the gun for which bullet to use, query for Bullet resources and get a matching one
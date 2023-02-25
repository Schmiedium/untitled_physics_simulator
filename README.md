# Untitled_Physics_Simulator

I threw bevy and bevy_rapier3d into a python package using maturin, mostly for fun

To try it out, make a new python environment and run "pip install maturin polars[all] patchelf"

Then you should be able to run "maturin develop" with that environment to build and install the package. add the "-r" flag to build release

There is an included test python file to see what's available

you'll need to supply your own .obj files for now

TODO list:

add derive macro for PSComponent
figure out cpu_thread limiter
document all the code
fix the namespacing issues
redo the entity indexing, add the binary search thing for ensuring all entities have unqiue indices
implement caching for convex decomposition of geometry and expose the option
figure out how to expose mass_properties for colliders (mainly inertia tensor and mass, also elasticity)
add drag force to all entities (going to be bad time)



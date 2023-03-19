use crate::framework::physics::aerodynamics::drag::*;
use bevy::app::{App, Plugin};

pub struct AerodynamicsPlugin;

impl Plugin for AerodynamicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DragCurve>();
        app.add_system(drag_force);
        app.add_system(magnus_force);
        app.add_system(lift_force);
        app.add_system(pitch_damping_force);
        app.add_system(spin_damping_moment);
        app.add_system(rolling_moment);
        app.add_system(overturning_moment);
        app.add_system(magnus_moment);
        app.add_system(pitch_damping_moment);
    }
}

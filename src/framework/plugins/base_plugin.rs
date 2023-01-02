use bevy::prelude::Plugin;

use crate::framework::py_modules::simulation_builder;

pub(super) struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<simulation_builder::Name>();
        app.register_type::<simulation_builder::Shape>();
        app.register_type::<simulation_builder::ColliderInitializer>();
        app.register_type::<simulation_builder::RecordInitializer>();
    }
}

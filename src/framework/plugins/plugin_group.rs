use crate::models::gun::gun::GunPlugin;
use crate::models::test::test_model::TestPlugin;

use super::base_plugin::BasePlugin;
use crate::framework::physics::aerodynamics::aero_plugin::AerodynamicsPlugin;
use bevy::app::PluginGroupBuilder;
use bevy::prelude::PluginGroup;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier3d::render::RapierDebugRenderPlugin;

/// Plugin group for the visualized run, relies on default plugins to be added to the app
pub struct UntitledPluginsGroup;

impl PluginGroup for UntitledPluginsGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(BasePlugin)
            .add(RapierPhysicsPlugin::<NoUserData>::default())
            .add(RapierDebugRenderPlugin::default())
            .add(GunPlugin)
            .add(TestPlugin)
            .add(AerodynamicsPlugin)
    }
}

/// Plugin group for running headless, relies on MinimalPlugins to be added to the app
pub struct UntitledPluginsGroupHeadless;

impl PluginGroup for UntitledPluginsGroupHeadless {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(bevy::asset::AssetPlugin::default())
            .add(bevy::transform::TransformPlugin::default())
            .add(bevy::scene::ScenePlugin::default())
            .add(BasePlugin)
            .add(RapierPhysicsPlugin::<NoUserData>::default())
            .add(GunPlugin)
            .add(TestPlugin)
            .add(AerodynamicsPlugin)
    }
}

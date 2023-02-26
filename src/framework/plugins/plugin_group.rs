use crate::models::test::test_model::TestPlugin;

use super::base_plugin::BasePlugin;
use bevy::app::PluginGroupBuilder;
use bevy::prelude::PluginGroup;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier3d::render::RapierDebugRenderPlugin;

pub struct UntitledPluginsGroup;

impl PluginGroup for UntitledPluginsGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(BasePlugin)
            .add(RapierPhysicsPlugin::<NoUserData>::default())
            .add(RapierDebugRenderPlugin::default())
            .add(TestPlugin)
    }
}

pub struct UntitledPluginsGroupHeadless;

impl PluginGroup for UntitledPluginsGroupHeadless {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(bevy::asset::AssetPlugin::default())
            .add(bevy::transform::TransformPlugin::default())
            .add(bevy::scene::ScenePlugin::default())
            .add(BasePlugin)
            .add(RapierPhysicsPlugin::<NoUserData>::default())
            .add(TestPlugin)
    }
}

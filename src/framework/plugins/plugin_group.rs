use bevy::app::PluginGroupBuilder;
use bevy::prelude::PluginGroup;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier3d::render::RapierDebugRenderPlugin;

use super::base_plugin::BasePlugin;

pub struct UntitledPluginsGroup;

impl PluginGroup for UntitledPluginsGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(BasePlugin)
            .add(RapierPhysicsPlugin::<NoUserData>::default())
            .add(RapierDebugRenderPlugin::default())
    }
}

pub struct UntitledPluginsGroupHeadless;

impl PluginGroup for UntitledPluginsGroupHeadless {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(bevy::asset::AssetPlugin::default())
            .add(bevy::transform::TransformPlugin::default())
            .add(bevy::window::WindowPlugin::default())
            .add(bevy::scene::ScenePlugin::default())
            .add(bevy::winit::WinitPlugin::default())
            .add(bevy::render::RenderPlugin::default())
            .add(bevy::render::texture::ImagePlugin::default())
            .add(BasePlugin)
            .add(RapierPhysicsPlugin::<NoUserData>::default())
    }
}

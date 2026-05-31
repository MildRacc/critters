use std::default;

use bevy::{dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig}, ecs::event::Trigger, input::mouse::AccumulatedMouseMotion, picking::window, prelude::*, render::{Render, RenderStartup, RenderSystems, extract_resource::ExtractResourcePlugin, render_graph::RenderGraph, storage::ShaderStorageBuffer}, sprite_render::Material2dPlugin, window::{CursorGrabMode, CursorOptions, PrimaryWindow}};

use crate::simulation::world::{ReadbackBuffer, ShaderState};
mod simulation;
mod dev;
mod camera;


fn main() {
    println!("Hello, world!");

    let mut app = App::new();


    app.add_plugins((
        DefaultPlugins,
        Material2dPlugin::<simulation::world::VisMaterial>::default(),
        FpsOverlayPlugin
        {
            config: FpsOverlayConfig
            {
                text_config: TextFont { font: default(), font_size: 16.0, font_smoothing: bevy::text::FontSmoothing::None, ..Default::default() },
                text_color: dev::fps::OverlayColor::RED,
                refresh_interval: core::time::Duration::from_millis(100),
                enabled: true,
                frame_time_graph_config: FrameTimeGraphConfig
                {
                    enabled: false,
                    min_fps: 30.0,
                    target_fps: 200.0
                },
            }
        },
        ExtractResourcePlugin::<ReadbackBuffer>::default(),
        ExtractResourcePlugin::<simulation::critter::CritterUniforms>::default(),
        ExtractResourcePlugin::<simulation::world::ShaderState>::default(),
        simulation::world::ComputePlugin
    ));



    app.add_systems(Startup, (
        camera::spawn_camera, 
        simulation::world::setup, 
        simulation::world::init_world.after(simulation::world::setup),
    )); 
    app.add_systems(Update, (simulation::world::player_look, simulation::world::update_resolution));
    app.add_systems(Update, (simulation::critter::critters_velocity, simulation::critter::move_critters));

    app.run();

}



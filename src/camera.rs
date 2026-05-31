use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*, window::{CursorGrabMode, CursorOptions, PrimaryWindow}};




#[derive(Component)]
pub struct CameraMarker;




pub fn spawn_camera(mut commands: Commands)
{
    let cam = Camera2d::default();
    commands.spawn((cam, CameraMarker)); 
}


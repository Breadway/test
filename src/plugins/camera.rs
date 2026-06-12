use bevy::prelude::*;
use crate::components::player::Player;
use crate::state::GameState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
           .add_systems(PostUpdate, camera_follow
               .run_if(in_state(GameState::Playing)));
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn camera_follow(
    player_q: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut cam_q: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(player_t) = player_q.get_single() else { return };
    let Ok(mut cam_t) = cam_q.get_single_mut() else { return };
    let target = player_t.translation;
    cam_t.translation = cam_t.translation.lerp(target, 0.1);
}

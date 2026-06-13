use bevy::prelude::*;
use bevy::window::WindowResolution;

mod characters;
mod components;
mod data;
mod events;
mod plugins;
mod resources;
mod state;

use characters::CharactersPlugin;
use components::player::CharacterSelection;
use events::*;
use plugins::camera::CameraPlugin;
use plugins::combat::CombatPlugin;
use plugins::enemy::EnemyPlugin;
use plugins::player::PlayerPlugin;
use plugins::room::RoomPlugin;
use plugins::ui::UIPlugin;
use plugins::upgrade::UpgradePlugin;
use plugins::visual::VisualPlugin;
use plugins::xp::XPPlugin;
use resources::*;
use state::GameState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Hazard Pay — Biosphere Management Group™".to_string(),
                    resolution: WindowResolution::new(1280.0, 720.0),
                    resizable: true,
                    ..default()
                }),
                ..default()
            })
            // Crisp pixel art: no texture filtering
            .set(ImagePlugin::default_nearest()))
        .insert_resource(ClearColor(Color::srgb(0.03, 0.03, 0.045)))
        // States
        .init_state::<GameState>()
        // Events
        .add_event::<EnemyKilled>()
        .add_event::<PlayerDamaged>()
        .add_event::<RoomCleared>()
        .add_event::<LevelUp>()
        .add_event::<PathBranchAvailable>()
        .add_event::<ProjectileHit>()
        .add_event::<RoomTransitionRequest>()
        .add_event::<BossPhaseChanged>()
        .add_event::<ShotFired>()
        // Resources
        .insert_resource(SpatialGrid::default())
        .insert_resource(GameManager::default())
        .insert_resource(GameRng::default())
        .insert_resource(SpawnQueue::default())
        .insert_resource(RoomState::default())
        .insert_resource(RoomGraph::default())
        .insert_resource(CharacterSelection::default())
        // Plugins
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(XPPlugin)
        .add_plugins(RoomPlugin)
        .add_plugins(UpgradePlugin)
        .add_plugins(UIPlugin)
        .add_plugins(CharactersPlugin)
        .add_plugins(VisualPlugin)
        .add_plugins(data::pixel_art::PixelArtPlugin)
        .run();
}

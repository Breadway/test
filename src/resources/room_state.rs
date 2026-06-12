use bevy::prelude::*;
use crate::data::enemies::WaveConfig;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoomType {
    Combat,
    Elite,
    Shop,
    Unknown,
    Boss,
    Rest,
}

#[derive(Clone, Debug)]
pub struct RoomNode {
    pub id: usize,
    pub room_type: RoomType,
    pub depth: usize,
    pub children: Vec<usize>,
    pub completed: bool,
    pub lore_name: String,
}

#[derive(Resource)]
pub struct RoomState {
    pub room_index: usize,
    pub room_type: RoomType,
    pub waves: Vec<WaveConfig>,
    pub wave_index: usize,
    pub spawn_timer: f32,
    pub spawn_queue: Vec<(crate::components::enemy::EnemyType, Vec2)>,
    pub spawns_this_wave: u32,
    pub bounds: Rect,
}

impl Default for RoomState {
    fn default() -> Self {
        Self {
            room_index: 0,
            room_type: RoomType::Combat,
            waves: Vec::new(),
            wave_index: 0,
            spawn_timer: 0.0,
            spawn_queue: Vec::new(),
            spawns_this_wave: 0,
            bounds: Rect::from_center_size(Vec2::ZERO, Vec2::new(960.0, 640.0)),
        }
    }
}

#[derive(Resource, Default)]
pub struct RoomGraph {
    pub nodes: Vec<RoomNode>,
    pub current_node: usize,
}

impl RoomGraph {
    pub fn current(&self) -> Option<&RoomNode> {
        self.nodes.get(self.current_node)
    }

    pub fn available_next(&self) -> Vec<usize> {
        if let Some(node) = self.current() {
            node.children.clone()
        } else {
            vec![]
        }
    }
}

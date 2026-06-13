use bevy::prelude::*;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use crate::data::enemies::WaveConfig;
use crate::data::lore::{ROOM_ADJECTIVES, ROOM_NOUNS};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoomType {
    Combat,
    Elite,
    Shop,
    Unknown,
    Boss,
}

#[derive(Clone, Debug)]
pub struct RoomNode {
    pub id: usize,
    pub room_type: RoomType,
    pub children: Vec<usize>,
    pub completed: bool,
    pub lore_name: String,
}

#[derive(Resource)]
pub struct RoomState {
    pub room_index: usize,
    pub waves: Vec<WaveConfig>,
    pub wave_index: usize,
    pub spawn_timer: f32,
    pub bounds: Rect,
}

impl Default for RoomState {
    fn default() -> Self {
        Self {
            room_index: 0,
            waves: Vec::new(),
            wave_index: 0,
            spawn_timer: 0.0,
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
        self.current().map(|n| n.children.clone()).unwrap_or_default()
    }
}

/// Pure graph generation so connectivity can be unit-tested.
pub fn generate_room_graph(rng: &mut ChaCha8Rng) -> Vec<RoomNode> {
    let mut nodes = Vec::with_capacity(25);
    for i in 0..25 {
        let adj = ROOM_ADJECTIVES[rng.gen_range(0..ROOM_ADJECTIVES.len())];
        let noun = ROOM_NOUNS[rng.gen_range(0..ROOM_NOUNS.len())];
        let lore_name = format!("{} {}", adj, noun);

        let room_type = match i {
            11 | 24 => RoomType::Boss,
            6 | 16 => RoomType::Shop,
            3 | 13 | 20 => RoomType::Elite,
            // Some rooms show as "???" on the map; they still run normal waves
            _ if rng.gen_bool(0.2) => RoomType::Unknown,
            _ => RoomType::Combat,
        };

        let children = if i < 24 {
            let next = i + 1;
            if i % 3 == 0 && i + 2 < 25 {
                vec![next, next + 1]
            } else {
                vec![next]
            }
        } else {
            vec![]
        };

        nodes.push(RoomNode {
            id: i,
            room_type,
            children,
            completed: false,
            lore_name,
        });
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn graph_reaches_final_room() {
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let nodes = generate_room_graph(&mut rng);
        assert_eq!(nodes.len(), 25);

        // BFS from node 0 must reach node 24
        let mut visited = vec![false; 25];
        let mut queue = std::collections::VecDeque::from([0usize]);
        visited[0] = true;
        while let Some(i) = queue.pop_front() {
            for &c in &nodes[i].children {
                if !visited[c] {
                    visited[c] = true;
                    queue.push_back(c);
                }
            }
        }
        assert!(visited[24], "node 24 must be reachable from node 0");
        assert!(nodes[24].children.is_empty(), "final node must be terminal");
        // Every non-final node must lead somewhere
        for n in &nodes[..24] {
            assert!(!n.children.is_empty(), "node {} is a dead end", n.id);
            for &c in &n.children {
                assert!(c < 25, "child index out of range");
            }
        }
    }
}

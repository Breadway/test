use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource)]
pub struct SpatialGrid {
    cells: HashMap<(i32, i32), Vec<Entity>>,
    pub cell_size: f32,
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self {
            cells: HashMap::with_capacity(256),
            cell_size: 128.0,
        }
    }
}

impl SpatialGrid {
    fn cell_key(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }

    pub fn insert(&mut self, entity: Entity, pos: Vec2) {
        let key = self.cell_key(pos);
        self.cells.entry(key).or_insert_with(Vec::new).push(entity);
    }

    pub fn query_radius(&self, pos: Vec2, radius: f32) -> Vec<Entity> {
        let mut result = Vec::new();
        let min_col = ((pos.x - radius) / self.cell_size).floor() as i32;
        let max_col = ((pos.x + radius) / self.cell_size).floor() as i32;
        let min_row = ((pos.y - radius) / self.cell_size).floor() as i32;
        let max_row = ((pos.y + radius) / self.cell_size).floor() as i32;
        let radius_sq = radius * radius;

        for row in min_row..=max_row {
            for col in min_col..=max_col {
                if let Some(entities) = self.cells.get(&(col, row)) {
                    for &entity in entities {
                        let cell_center = Vec2::new(
                            (col as f32 + 0.5) * self.cell_size,
                            (row as f32 + 0.5) * self.cell_size,
                        );
                        let dx = cell_center.x - pos.x;
                        let dy = cell_center.y - pos.y;
                        if dx * dx + dy * dy <= (radius + self.cell_size).powi(2) {
                            result.push(entity);
                        }
                    }
                }
            }
        }
        result
    }

    pub fn query_radius_precise(&self, pos: Vec2, radius: f32, transforms: &Query<&Transform>) -> Vec<Entity> {
        let candidates = self.query_radius(pos, radius);
        let radius_sq = radius * radius;
        candidates.into_iter().filter(|&e| {
            if let Ok(t) = transforms.get(e) {
                let dp = t.translation.truncate() - pos;
                dp.length_squared() <= radius_sq
            } else {
                false
            }
        }).collect()
    }

    pub fn clear(&mut self) {
        for v in self.cells.values_mut() {
            v.clear();
        }
    }
}

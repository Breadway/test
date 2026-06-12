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

    /// Returns candidate entities from all cells overlapping the radius.
    /// Callers must do a precise distance check against actual positions.
    pub fn query_radius(&self, pos: Vec2, radius: f32) -> Vec<Entity> {
        let mut result = Vec::new();
        let min_col = ((pos.x - radius) / self.cell_size).floor() as i32;
        let max_col = ((pos.x + radius) / self.cell_size).floor() as i32;
        let min_row = ((pos.y - radius) / self.cell_size).floor() as i32;
        let max_row = ((pos.y + radius) / self.cell_size).floor() as i32;

        for row in min_row..=max_row {
            for col in min_col..=max_col {
                if let Some(entities) = self.cells.get(&(col, row)) {
                    result.extend_from_slice(entities);
                }
            }
        }
        result
    }

    pub fn clear(&mut self) {
        for v in self.cells.values_mut() {
            v.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_returns_inserted_entities_in_range() {
        let mut grid = SpatialGrid::default();
        let near = Entity::from_raw(1);
        let far = Entity::from_raw(2);
        grid.insert(near, Vec2::new(10.0, 10.0));
        grid.insert(far, Vec2::new(1000.0, 1000.0));

        let result = grid.query_radius(Vec2::ZERO, 80.0);
        assert!(result.contains(&near));
        assert!(!result.contains(&far));
    }

    #[test]
    fn clear_empties_cells() {
        let mut grid = SpatialGrid::default();
        grid.insert(Entity::from_raw(1), Vec2::ZERO);
        grid.clear();
        assert!(grid.query_radius(Vec2::ZERO, 200.0).is_empty());
    }

    #[test]
    fn query_spanning_cell_boundary() {
        let mut grid = SpatialGrid::default();
        // 128.0 cell size: entity just across the boundary
        let e = Entity::from_raw(3);
        grid.insert(e, Vec2::new(130.0, 0.0));
        let result = grid.query_radius(Vec2::new(120.0, 0.0), 20.0);
        assert!(result.contains(&e));
    }
}

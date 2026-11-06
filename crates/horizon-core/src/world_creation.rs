use crate::{Chunk, ChunkPosition};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldType {
    Default,
    Superflat,
    LargeBiomes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldCreationOptions {
    pub world_name: String,
    pub game_mode: GameMode,
    pub difficulty: Difficulty,
    pub world_type: WorldType,
    pub seed: String,
    pub generate_structures: bool,
}

impl Default for WorldCreationOptions {
    fn default() -> Self {
        Self {
            world_name: String::from("New World"),
            game_mode: GameMode::Survival,
            difficulty: Difficulty::Normal,
            world_type: WorldType::Default,
            seed: String::new(),
            generate_structures: true,
        }
    }
}

impl WorldCreationOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_world_name(&mut self, name: String) {
        let trimmed = name.trim();

        if !trimmed.is_empty() {
            self.world_name = trimmed.to_string();
        }
    }

    pub fn set_seed(&mut self, seed: String) {
        self.seed = seed.trim().to_string();
    }

    pub fn resolved_seed(&self, random_seed: u64) -> u64 {
        let seed = self.seed.trim();

        if seed.is_empty() {
            return random_seed;
        }

        if let Ok(number) = seed.parse::<u64>() {
            return number;
        }

        let mut hash = 0xcbf29ce484222325_u64;

        for byte in seed.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }

        hash
    }

    // Generate terrain using the seed selected in the creation menu.
    pub fn generate_chunk(
        &self,
        random_seed: u64,
        position: ChunkPosition,
    ) -> Chunk {
        let seed = self.resolved_seed(random_seed);

        crate::world::generate_chunk(seed, position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::terrain_height;

    #[test]
    fn default_options_are_correct() {
        let options = WorldCreationOptions::new();

        assert_eq!(options.world_name, "New World");
        assert_eq!(options.game_mode, GameMode::Survival);
        assert_eq!(options.difficulty, Difficulty::Normal);
        assert_eq!(options.world_type, WorldType::Default);
        assert!(options.generate_structures);
    }

    #[test]
    fn world_name_can_be_changed() {
        let mut options = WorldCreationOptions::new();

        options.set_world_name("My World".to_string());

        assert_eq!(options.world_name, "My World");
    }

    #[test]
    fn empty_world_name_does_not_replace_name() {
        let mut options = WorldCreationOptions::new();

        options.set_world_name("   ".to_string());

        assert_eq!(options.world_name, "New World");
    }

    #[test]
    fn numeric_seed_is_supported() {
        let mut options = WorldCreationOptions::new();

        options.set_seed("12345".to_string());

        assert_eq!(options.resolved_seed(999), 12345);
    }

    #[test]
    fn text_seed_is_deterministic() {
        let mut options = WorldCreationOptions::new();

        options.set_seed("Horizon".to_string());

        assert_eq!(
            options.resolved_seed(999),
            options.resolved_seed(123)
        );
    }

    #[test]
    fn empty_seed_uses_provided_random_seed() {
        let options = WorldCreationOptions::new();

        assert_eq!(options.resolved_seed(54321), 54321);
    }

    #[test]
    fn selected_seed_controls_terrain_generation() {
        let mut options = WorldCreationOptions::new();

        options.set_seed("12345".to_string());

        let position = ChunkPosition::new(0, 0);

        let chunk = options.generate_chunk(999, position);

        let expected_height = terrain_height(12345, 5, 5);

        assert_eq!(
            chunk.get(5, expected_height, 5),
            Some(crate::Block::Grass)
        );
    }

    #[test]
    fn same_seed_generates_same_terrain() {
        let mut first = WorldCreationOptions::new();
        let mut second = WorldCreationOptions::new();

        first.set_seed("Horizon".to_string());
        second.set_seed("Horizon".to_string());

        let position = ChunkPosition::new(0, 0);

        let a = first.generate_chunk(100, position);
        let b = second.generate_chunk(200, position);

        for x in 0..crate::CHUNK_WIDTH {
            for z in 0..crate::CHUNK_WIDTH {
                for y in 0..crate::CHUNK_HEIGHT {
                    assert_eq!(
                        a.get(x, y, z),
                        b.get(x, y, z)
                    );
                }
            }
        }
    }
}

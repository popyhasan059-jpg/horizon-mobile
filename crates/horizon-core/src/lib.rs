pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 64;
pub const CHUNK_VOLUME: usize = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Air,
    Grass,
    Dirt,
    Stone,
    Water,
}

pub struct Chunk {
    blocks: Vec<Block>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: vec![Block::Air; CHUNK_VOLUME],
        }
    }

    fn index(x: usize, y: usize, z: usize) -> Option<usize> {
        if x >= CHUNK_WIDTH || y >= CHUNK_HEIGHT || z >= CHUNK_WIDTH {
            return None;
        }

        Some(y * CHUNK_WIDTH * CHUNK_WIDTH + z * CHUNK_WIDTH + x)
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<Block> {
        let index = Self::index(x, y, z)?;
        Some(self.blocks[index])
    }

    pub fn set(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        block: Block,
    ) -> bool {
        let Some(index) = Self::index(x, y, z) else {
            return false;
        };

        self.blocks[index] = block;
        true
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_starts_empty() {
        let chunk = Chunk::new();
        assert_eq!(chunk.get(0, 0, 0), Some(Block::Air));
    }

    #[test]
    fn blocks_can_be_changed() {
        let mut chunk = Chunk::new();
        assert!(chunk.set(2, 5, 3, Block::Stone));
        assert_eq!(chunk.get(2, 5, 3), Some(Block::Stone));
    }

    #[test]
    fn invalid_coordinates_are_rejected() {
        let mut chunk = Chunk::new();
        assert!(!chunk.set(16, 0, 0, Block::Stone));
        assert_eq!(chunk.get(16, 0, 0), None);
    }
          }

use crate::{
    biome::{biome_at, Biome},
    Block,
    Chunk,
    ChunkPosition,
    CHUNK_HEIGHT,
    CHUNK_WIDTH,
};

const SEA_LEVEL: usize = 12;
const NOISE_SCALE: i32 = 8;

fn hash_height(seed: u64, x: i32, z: i32) -> f32 {
    let mut value = seed;
    value ^= (x as i64 as u64)
        .wrapping_mul(0x9E3779B185EBCA87);
    value ^= (z as i64 as u64)
        .wrapping_mul(0xC2B2AE3D27D4EB4F);

    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58476D1CE4E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D049BB133111EB);
    value ^= value >> 31;

    (value as u32) as f32 / u32::MAX as f32
}

fn smooth_noise(seed: u64, x: i32, z: i32) -> f32 {
    let grid_x = x.div_euclid(NOISE_SCALE);
    let grid_z = z.div_euclid(NOISE_SCALE);

    let tx = x.rem_euclid(NOISE_SCALE) as f32
        / NOISE_SCALE as f32;
    let tz = z.rem_euclid(NOISE_SCALE) as f32
        / NOISE_SCALE as f32;

    let tx = tx * tx * (3.0 - 2.0 * tx);
    let tz = tz * tz * (3.0 - 2.0 * tz);

    let a = hash_height(seed, grid_x, grid_z);
    let b = hash_height(seed, grid_x + 1, grid_z);
    let c = hash_height(seed, grid_x, grid_z + 1);
    let d = hash_height(seed, grid_x + 1, grid_z + 1);

    let top = a + (b - a) * tx;
    let bottom = c + (d - c) * tx;

    top + (bottom - top) * tz
}

pub fn terrain_height(seed: u64, x: i32, z: i32) -> usize {
    let noise = smooth_noise(seed, x, z);
    (8.0 + noise * 16.0) as usize
}

fn surface_block(biome: Biome) -> Block {
    match biome {
        Biome::Plains => Block::Grass,
        Biome::Forest => Block::Grass,
        Biome::Desert => Block::Sand,
        Biome::SnowyTaiga => Block::Snow,
    }
}

pub fn generate_chunk(
    seed: u64,
    position: ChunkPosition,
) -> Chunk {
    let mut chunk = Chunk::new();

    let origin_x =
        position.x.saturating_mul(CHUNK_WIDTH as i32);
    let origin_z =
        position.z.saturating_mul(CHUNK_WIDTH as i32);

    for local_x in 0..CHUNK_WIDTH {
        for local_z in 0..CHUNK_WIDTH {
            let world_x =
                origin_x.saturating_add(local_x as i32);
            let world_z =
                origin_z.saturating_add(local_z as i32);

            let height = terrain_height(
                seed,
                world_x,
                world_z,
            )
            .min(CHUNK_HEIGHT - 1);

            let biome = biome_at(seed, world_x, world_z);
            let surface = surface_block(biome);

            for y in 0..=height {
                let block = if y == height {
                    surface
                } else if y + 3 >= height {
                    Block::Dirt
                } else {
                    Block::Stone
                };

                chunk.set(local_x, y, local_z, block);
            }

            // Fill low areas with water.
            if height < SEA_LEVEL {
                for y in (height + 1)..=SEA_LEVEL {
                    if y < CHUNK_HEIGHT {
                        chunk.set(
                            local_x,
                            y,
                            local_z,
                            Block::Water,
                        );
                    }
                }
            }
        }
    }

    chunk
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_chunk_contains_terrain() {
        let chunk =
            generate_chunk(12345, ChunkPosition::new(0, 0));

        let mut found_terrain = false;

        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                for y in 0..CHUNK_HEIGHT {
                    if chunk.get(x, y, z) != Some(Block::Air) {
                        found_terrain = true;
                    }
                }
            }
        }

        assert!(found_terrain);
    }

    #[test]
    fn same_seed_generates_same_height() {
        assert_eq!(
            terrain_height(12345, 10, 20),
            terrain_height(12345, 10, 20)
        );
    }

    #[test]
    fn different_seeds_can_generate_different_heights() {
        assert_ne!(
            terrain_height(12345, 10, 20),
            terrain_height(98765, 10, 20)
        );
    }

    #[test]
    fn generated_height_stays_inside_chunk() {
        for x in -100..100 {
            for z in -100..100 {
                assert!(terrain_height(42, x, z) < CHUNK_HEIGHT);
            }
        }
    }

    #[test]
    fn neighboring_chunks_use_world_coordinates() {
        let left =
            generate_chunk(42, ChunkPosition::new(0, 0));
        let right =
            generate_chunk(42, ChunkPosition::new(1, 0));

        for z in 0..CHUNK_WIDTH {
            let left_height = terrain_height(42, 15, z as i32);
            let right_height = terrain_height(42, 16, z as i32);

            assert_eq!(
                left.get(15, left_height, z),
                Some(surface_block(biome_at(42, 15, z as i32)))
            );

            assert_eq!(
                right.get(0, right_height, z),
                Some(surface_block(biome_at(42, 16, z as i32)))
            );
        }
    }

    #[test]
    fn desert_generates_sand_surface() {
        let seed = 12345;

        for x in 0..512 {
            for z in 0..512 {
                if biome_at(seed, x, z) == Biome::Desert {
                    let position = ChunkPosition::new(
                        x.div_euclid(CHUNK_WIDTH as i32),
                        z.div_euclid(CHUNK_WIDTH as i32),
                    );

                    let chunk = generate_chunk(seed, position);

                    let local_x =
                        x.rem_euclid(CHUNK_WIDTH as i32) as usize;
                    let local_z =
                        z.rem_euclid(CHUNK_WIDTH as i32) as usize;
                    let height = terrain_height(seed, x, z);

                    assert_eq!(
                        chunk.get(local_x, height, local_z),
                        Some(Block::Sand)
                    );
                    return;
                }
            }
        }

        panic!("No desert biome found in test area");
    }

    #[test]
    fn snowy_taiga_generates_snow_surface() {
        let seed = 12345;

        for x in 0..512 {
            for z in 0..512 {
                if biome_at(seed, x, z) == Biome::SnowyTaiga {
                    let position = ChunkPosition::new(
                        x.div_euclid(CHUNK_WIDTH as i32),
                        z.div_euclid(CHUNK_WIDTH as i32),
                    );

                    let chunk = generate_chunk(seed, position);

                    let local_x =
                        x.rem_euclid(CHUNK_WIDTH as i32) as usize;
                    let local_z =
                        z.rem_euclid(CHUNK_WIDTH as i32) as usize;
                    let height = terrain_height(seed, x, z);

                    assert_eq!(
                        chunk.get(local_x, height, local_z),
                        Some(Block::Snow)
                    );
                    return;
                }
            }
        }

        panic!("No snowy taiga biome found in test area");
    }
                    }

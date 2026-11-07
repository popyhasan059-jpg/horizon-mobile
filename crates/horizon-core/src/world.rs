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

fn place_leaves(
    chunk: &mut Chunk,
    x: usize,
    y: usize,
    z: usize,
    block: Block,
) {
    if x >= CHUNK_WIDTH
        || y >= CHUNK_HEIGHT
        || z >= CHUNK_WIDTH
    {
        return;
    }

    if chunk.get(x, y, z) == Some(Block::Air) {
        chunk.set(x, y, z, block);
    }
}

fn place_tree(
    chunk: &mut Chunk,
    x: usize,
    ground_y: usize,
    z: usize,
    snowy: bool,
) {
    let trunk_height = 4;
    if ground_y + trunk_height + 2 >= CHUNK_HEIGHT {
        return;
    }

    let leaf_block = if snowy {
        Block::SnowyLeaves
    } else {
        Block::Leaves
    };

    for offset in 1..=trunk_height {
        chunk.set(x, ground_y + offset, z, Block::Log);
    }

    // Lower leaf layer.
    for dx in -2i32..=2 {
        for dz in -2i32..=2 {
            if dx.abs() == 2 && dz.abs() == 2 {
                continue;
            }

            let lx = x as i32 + dx;
            let lz = z as i32 + dz;

            if lx >= 0 && lz >= 0 {
                place_leaves(
                    chunk,
                    lx as usize,
                    ground_y + 3,
                    lz as usize,
                    leaf_block,
                );
            }
        }
    }

    // Upper leaf layer.
    for dx in -1i32..=1 {
        for dz in -1i32..=1 {
            let lx = x as i32 + dx;
            let lz = z as i32 + dz;

            if lx >= 0 && lz >= 0 {
                place_leaves(
                    chunk,
                    lx as usize,
                    ground_y + 4,
                    lz as usize,
                    leaf_block,
                );
            }
        }
    }

    place_leaves(
        chunk,
        x,
        ground_y + 5,
        z,
        leaf_block,
    );
}

fn place_vegetation(
    seed: u64,
    position: ChunkPosition,
    chunk: &mut Chunk,
) {
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

            let ground_y = terrain_height(
                seed,
                world_x,
                world_z,
            )
            .min(CHUNK_HEIGHT - 1);

            if ground_y < SEA_LEVEL {
                continue;
            }

            let biome = biome_at(seed, world_x, world_z);
            let surface = surface_block(biome);

            if chunk.get(local_x, ground_y, local_z)
                != Some(surface)
            {
                continue;
            }

            let above = ground_y + 1;
            if above >= CHUNK_HEIGHT {
                continue;
            }

            match biome {
                Biome::Forest => {
                    // Keep tree canopies inside this chunk.
                    if local_x >= 2
                        && local_x < CHUNK_WIDTH - 2
                        && local_z >= 2
                        && local_z < CHUNK_WIDTH - 2
                        && hash_height(
                            seed ^ 0xA11CE,
                            world_x,
                            world_z,
                        ) < 0.025
                    {
                        place_tree(
                            chunk,
                            local_x,
                            ground_y,
                            local_z,
                            false,
                        );
                    }
                }

                Biome::SnowyTaiga => {
                    if local_x >= 2
                        && local_x < CHUNK_WIDTH - 2
                        && local_z >= 2
                        && local_z < CHUNK_WIDTH - 2
                        && hash_height(
                            seed ^ 0x5A0,
                            world_x,
                            world_z,
                        ) < 0.02
                    {
                        place_tree(
                            chunk,
                            local_x,
                            ground_y,
                            local_z,
                            true,
                        );
                    }
                }

                Biome::Desert => {
                    if hash_height(
                        seed ^ 0xCACT05,
                        world_x,
                        world_z,
                    ) < 0.012
                    {
                        let cactus_height =
                            2 + (hash_height(
                                seed ^ 0xCA7,
                                world_x,
                                world_z,
                            ) * 3.0) as usize;

                        for offset in 1..=cactus_height {
                            if ground_y + offset < CHUNK_HEIGHT {
                                chunk.set(
                                    local_x,
                                    ground_y + offset,
                                    local_z,
                                    Block::Cactus,
                                );
                            }
                        }
                    }
                }

                Biome::Plains => {
                    let plant = hash_height(
                        seed ^ 0xF10,
                        world_x,
                        world_z,
                    );

                    if plant < 0.025 {
                        chunk.set(
                            local_x,
                            above,
                            local_z,
                            Block::Flower,
                        );
                    } else if plant < 0.12 {
                        chunk.set(
                            local_x,
                            above,
                            local_z,
                            Block::TallGrass,
                        );
                    }
                }
            }
        }
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

    // Generate terrain first.
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

    // Generate plants and trees after terrain.
    place_vegetation(seed, position, &mut chunk);

    chunk
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk_has_block(chunk: &Chunk, target: Block) -> bool {
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                for y in 0..CHUNK_HEIGHT {
                    if chunk.get(x, y, z) == Some(target) {
                        return true;
                    }
                }
            }
        }

        false
    }

    #[test]
    fn generated_chunk_contains_terrain() {
        let chunk =
            generate_chunk(12345, ChunkPosition::new(0, 0));

        assert!(chunk_has_block(&chunk, Block::Stone));
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

    #[test]
    fn vegetation_generation_is_deterministic() {
        let first =
            generate_chunk(12345, ChunkPosition::new(3, 4));
        let second =
            generate_chunk(12345, ChunkPosition::new(3, 4));

        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                for y in 0..CHUNK_HEIGHT {
                    assert_eq!(
                        first.get(x, y, z),
                        second.get(x, y, z)
                    );
                }
            }
        }
    }

    #[test]
    fn vegetation_appears_in_generated_world() {
        let mut found_tree = false;
        let mut found_cactus = false;
        let mut found_grass = false;
        let mut found_flower = false;

        for chunk_x in 0..8 {
            for chunk_z in 0..8 {
                let chunk = generate_chunk(
                    12345,
                    ChunkPosition::new(chunk_x, chunk_z),
                );

                found_tree |= chunk_has_block(&chunk, Block::Log);
                found_cactus |= chunk_has_block(&chunk, Block::Cactus);
                found_grass |= chunk_has_block(&chunk, Block::TallGrass);
                found_flower |= chunk_has_block(&chunk, Block::Flower);
            }
        }

        assert!(found_tree, "No trees generated");
        assert!(found_cactus, "No cacti generated");
        assert!(found_grass, "No tall grass generated");
        assert!(found_flower, "No flowers generated");
    }
        }

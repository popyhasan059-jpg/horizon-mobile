const BIOME_SCALE: i32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Plains,
    Forest,
    Desert,
    SnowyTaiga,
}

fn biome_hash(seed: u64, x: i32, z: i32) -> f32 {
    let mut value = seed
        ^ (x as i64 as u64).wrapping_mul(0x9E3779B185EBCA87)
        ^ (z as i64 as u64).wrapping_mul(0xC2B2AE3D27D4EB4F);

    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58476D1CE4E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D049BB133111EB);
    value ^= value >> 31;

    (value as u32) as f32 / u32::MAX as f32
}

fn smooth_noise(seed: u64, x: i32, z: i32) -> f32 {
    let gx = x.div_euclid(BIOME_SCALE);
    let gz = z.div_euclid(BIOME_SCALE);

    let mut tx = x.rem_euclid(BIOME_SCALE) as f32
        / BIOME_SCALE as f32;
    let mut tz = z.rem_euclid(BIOME_SCALE) as f32
        / BIOME_SCALE as f32;

    tx = tx * tx * (3.0 - 2.0 * tx);
    tz = tz * tz * (3.0 - 2.0 * tz);

    let a = biome_hash(seed, gx, gz);
    let b = biome_hash(seed, gx + 1, gz);
    let c = biome_hash(seed, gx, gz + 1);
    let d = biome_hash(seed, gx + 1, gz + 1);

    let top = a + (b - a) * tx;
    let bottom = c + (d - c) * tx;

    top + (bottom - top) * tz
}

pub fn biome_at(seed: u64, x: i32, z: i32) -> Biome {
    let climate = smooth_noise(seed, x, z);

    if climate < 0.35 {
        Biome::Plains
    } else if climate < 0.60 {
        Biome::Forest
    } else if climate < 0.82 {
        Biome::Desert
    } else {
        Biome::SnowyTaiga
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_and_coordinates_give_same_biome() {
        assert_eq!(
            biome_at(12345, 100, 200),
            biome_at(12345, 100, 200)
        );
    }

    #[test]
    fn negative_coordinates_are_supported() {
        let _biome = biome_at(12345, -100, -200);
    }

    #[test]
    fn biome_generation_is_deterministic() {
        for x in -200..200 {
            for z in -200..200 {
                assert_eq!(
                    biome_at(42, x, z),
                    biome_at(42, x, z)
                );
            }
        }
    }

    #[test]
    fn all_initial_biomes_can_generate() {
        let mut plains = false;
        let mut forest = false;
        let mut desert = false;
        let mut snowy_taiga = false;

        for x in 0..512 {
            for z in 0..512 {
                match biome_at(12345, x, z) {
                    Biome::Plains => plains = true,
                    Biome::Forest => forest = true,
                    Biome::Desert => desert = true,
                    Biome::SnowyTaiga => snowy_taiga = true,
                }
            }
        }

        assert!(plains);
        assert!(forest);
        assert!(desert);
        assert!(snowy_taiga);
    }
  }

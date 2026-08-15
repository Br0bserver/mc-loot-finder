use crate::catalog::{Placement, SpreadType};
use crate::error::Error;
use crate::random::LegacyRandom48;

const REGION_X_MULTIPLIER: i64 = 341_873_128_712;
const REGION_Z_MULTIPLIER: i64 = 132_897_987_541;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Candidate {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub block_x: i32,
    pub block_z: i32,
    pub squared_distance: i64,
}

pub fn locate(
    world_seed: i64,
    center_x: i32,
    center_z: i32,
    radius: i32,
    placement: Placement,
) -> Result<Vec<Candidate>, Error> {
    if radius < 0 {
        return Err(Error::Placement("radius must be non-negative".to_owned()));
    }

    let min_block_x = i64::from(center_x) - i64::from(radius);
    let max_block_x = i64::from(center_x) + i64::from(radius);
    let min_block_z = i64::from(center_z) - i64::from(radius);
    let max_block_z = i64::from(center_z) + i64::from(radius);
    if min_block_x < i64::from(i32::MIN)
        || max_block_x > i64::from(i32::MAX)
        || min_block_z < i64::from(i32::MIN)
        || max_block_z > i64::from(i32::MAX)
    {
        return Err(Error::Placement(
            "search area exceeds the supported block coordinate range".to_owned(),
        ));
    }

    let min_chunk_x = min_block_x.div_euclid(16) as i32;
    let max_chunk_x = max_block_x.div_euclid(16) as i32;
    let min_chunk_z = min_block_z.div_euclid(16) as i32;
    let max_chunk_z = max_block_z.div_euclid(16) as i32;
    let min_region_x = min_chunk_x.div_euclid(placement.spacing);
    let max_region_x = max_chunk_x.div_euclid(placement.spacing);
    let min_region_z = min_chunk_z.div_euclid(placement.spacing);
    let max_region_z = max_chunk_z.div_euclid(placement.spacing);
    let radius_squared = i64::from(radius) * i64::from(radius);

    let mut candidates = Vec::new();
    for region_x in min_region_x..=max_region_x {
        for region_z in min_region_z..=max_region_z {
            let limit = placement.spacing - placement.separation;
            let placement_seed = world_seed
                .wrapping_add(i64::from(region_x).wrapping_mul(REGION_X_MULTIPLIER))
                .wrapping_add(i64::from(region_z).wrapping_mul(REGION_Z_MULTIPLIER))
                .wrapping_add(placement.salt);
            let mut random = LegacyRandom48::new(placement_seed);
            let mut offset_x = random.next_int(limit);
            let offset_z = match placement.spread {
                SpreadType::Triangular => {
                    offset_x = (offset_x + random.next_int(limit)) / 2;
                    (random.next_int(limit) + random.next_int(limit)) / 2
                }
                SpreadType::Linear => random.next_int(limit),
            };
            let chunk_x = region_x * placement.spacing + offset_x;
            let chunk_z = region_z * placement.spacing + offset_z;
            let block_x = i64::from(chunk_x) * 16 + 8;
            let block_z = i64::from(chunk_z) * 16 + 8;
            let dx = block_x - i64::from(center_x);
            let dz = block_z - i64::from(center_z);
            if dx.abs() <= i64::from(radius)
                && dz.abs() <= i64::from(radius)
                && dx * dx <= radius_squared - dz * dz
            {
                candidates.push(Candidate {
                    chunk_x,
                    chunk_z,
                    block_x: block_x as i32,
                    block_z: block_z as i32,
                    squared_distance: dx * dx + dz * dz,
                });
            }
        }
    }
    candidates.sort_by_key(|candidate| candidate.squared_distance);
    Ok(candidates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locates_seed_zero_ancient_city_candidates() {
        let candidates = locate(
            0,
            0,
            0,
            5_000,
            Placement {
                spacing: 24,
                separation: 8,
                salt: 20_083_232,
                spread: SpreadType::Linear,
            },
        )
        .unwrap();

        assert_eq!(candidates.len(), 537);
        assert_eq!((candidates[0].chunk_x, candidates[0].chunk_z), (9, 7));
        assert_eq!((candidates[1].chunk_x, candidates[1].chunk_z), (8, -16));
    }

    #[test]
    fn rejects_coordinate_overflow() {
        let error = locate(
            0,
            i32::MAX,
            0,
            1,
            Placement {
                spacing: 24,
                separation: 8,
                salt: 20_083_232,
                spread: SpreadType::Linear,
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("coordinate range"));
    }
}

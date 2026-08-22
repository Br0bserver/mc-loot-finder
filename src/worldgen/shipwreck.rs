use super::{
    Scan, Scanner, invalid_scan, structure_biomes,
    template_scan::{RandomPrefix, TemplatePlacement},
    terrain::TerrainSampler,
};
use crate::catalog::shipwreck_decoration;
use crate::decoration_seed::DecorationRandom;
use crate::error::Error;

use pumpkin_data::{Rotation, structures::Structure};
use pumpkin_util::{HeightMap, math::vector3::Vector3, random::RandomImpl};
use pumpkin_world::generation::{
    noise::router::multi_noise_sampler::MultiNoiseSampler,
    structure::{structures::create_chunk_random, template::get_template},
};

const PIVOT: Vector3<i32> = Vector3::new(4, 0, 15);

const BEACHED_TEMPLATES: &[&str] = &[
    "shipwreck/with_mast",
    "shipwreck/sideways_full",
    "shipwreck/sideways_fronthalf",
    "shipwreck/sideways_backhalf",
    "shipwreck/rightsideup_full",
    "shipwreck/rightsideup_fronthalf",
    "shipwreck/rightsideup_backhalf",
    "shipwreck/with_mast_degraded",
    "shipwreck/rightsideup_full_degraded",
    "shipwreck/rightsideup_fronthalf_degraded",
    "shipwreck/rightsideup_backhalf_degraded",
];

const OCEAN_TEMPLATES: &[&str] = &[
    "shipwreck/with_mast",
    "shipwreck/upsidedown_full",
    "shipwreck/upsidedown_fronthalf",
    "shipwreck/upsidedown_backhalf",
    "shipwreck/sideways_full",
    "shipwreck/sideways_fronthalf",
    "shipwreck/sideways_backhalf",
    "shipwreck/rightsideup_full",
    "shipwreck/rightsideup_fronthalf",
    "shipwreck/rightsideup_backhalf",
    "shipwreck/with_mast_degraded",
    "shipwreck/upsidedown_full_degraded",
    "shipwreck/upsidedown_fronthalf_degraded",
    "shipwreck/upsidedown_backhalf_degraded",
    "shipwreck/sideways_full_degraded",
    "shipwreck/sideways_fronthalf_degraded",
    "shipwreck/sideways_backhalf_degraded",
    "shipwreck/rightsideup_full_degraded",
    "shipwreck/rightsideup_fronthalf_degraded",
    "shipwreck/rightsideup_backhalf_degraded",
];

impl Scanner {
    pub(super) fn scan_shipwreck(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        sampler: &mut MultiNoiseSampler<'_>,
    ) -> Result<Scan, Error> {
        let min_x = chunk_x
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("shipwreck chunk x overflowed".to_owned()))?;
        let min_z = chunk_z
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("shipwreck chunk z overflowed".to_owned()))?;
        if min_x.checked_sub(32).is_none()
            || min_x.checked_add(32).is_none()
            || min_z.checked_sub(32).is_none()
            || min_z.checked_add(32).is_none()
        {
            return Err(Error::Worldgen(
                "shipwreck template coordinates overflowed".to_owned(),
            ));
        }
        let middle_x = min_x
            .checked_add(8)
            .ok_or_else(|| Error::Worldgen("shipwreck center x overflowed".to_owned()))?;
        let middle_z = min_z
            .checked_add(8)
            .ok_or_else(|| Error::Worldgen("shipwreck center z overflowed".to_owned()))?;

        let mut terrain = TerrainSampler::new(&self.generator);
        let Some(is_beached) = self.select_shipwreck_variant(
            chunk_x,
            chunk_z,
            middle_x,
            middle_z,
            &mut terrain,
            sampler,
        ) else {
            return Ok(invalid_scan());
        };

        let mut generation_random = create_chunk_random(self.world_seed, chunk_x, chunk_z);
        let rotation = Rotation::from_index(generation_random.next_bounded_i32(4) as u8);
        let templates = if is_beached {
            BEACHED_TEMPLATES
        } else {
            OCEAN_TEMPLATES
        };
        let template_name =
            templates[generation_random.next_bounded_i32(templates.len() as i32) as usize];
        let template = get_template(template_name).ok_or_else(|| {
            Error::Worldgen(format!("shipwreck template is missing: {template_name}"))
        })?;

        let initial =
            TemplatePlacement::new(&template, Vector3::new(min_x, 90, min_z), rotation, PIVOT);
        // Small shipwrecks defer height adjustment to their first intersecting chunk's
        // postProcess call. Vanilla region traversal reaches the minimum X/Z chunk first,
        // so a beached wreck consumes nextInt(3) from that decoration chunk's stream.
        let first_chunk = initial.first_intersecting_chunk();
        let decoration = shipwreck_decoration(is_beached);
        let random_prefix = is_beached.then_some(RandomPrefix {
            chunk: first_chunk,
            next_int_bound: 3,
        });
        let target_y = shipwreck_target_y(
            &mut terrain,
            &template,
            Vector3::new(min_x, 90, min_z),
            is_beached,
            first_chunk,
            self.world_seed,
            decoration,
        );
        let placement = TemplatePlacement::new(
            &template,
            Vector3::new(min_x, target_y, min_z),
            rotation,
            PIVOT,
        );
        let chests = placement.collect_chests(
            self.world_seed,
            (chunk_x, chunk_z),
            decoration,
            random_prefix,
            shipwreck_marker_loot_table,
        );

        Ok(Scan {
            valid_structure: true,
            chests,
        })
    }

    fn select_shipwreck_variant(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        middle_x: i32,
        middle_z: i32,
        terrain: &mut TerrainSampler<'_>,
        sampler: &mut MultiNoiseSampler<'_>,
    ) -> Option<bool> {
        // Vanilla's shipwreck structure set selects ocean first and beached second, with
        // equal weights. A biome failure removes the selected entry and retries the other.
        let mut remaining = vec![
            (false, Structure::SHIPWRECK),
            (true, Structure::SHIPWRECK_BEACHED),
        ];
        let mut selection_random = create_chunk_random(self.world_seed, chunk_x, chunk_z);
        while !remaining.is_empty() {
            let choice = selection_random.next_bounded_i32(remaining.len() as i32) as usize;
            let (is_beached, structure) = remaining.remove(choice);
            let heightmap = if is_beached {
                HeightMap::WorldSurfaceWg
            } else {
                HeightMap::OceanFloorWg
            };
            let y = terrain.height(heightmap, middle_x, middle_z) - 1;
            if self.biome_is_valid(
                Vector3::new(middle_x, y, middle_z),
                structure_biomes(&structure),
                sampler,
            ) {
                return Some(is_beached);
            }
        }
        None
    }
}

fn shipwreck_target_y(
    terrain: &mut TerrainSampler<'_>,
    template: &pumpkin_world::generation::structure::template::StructureTemplate,
    origin: Vector3<i32>,
    is_beached: bool,
    first_chunk: (i32, i32),
    world_seed: i64,
    decoration: crate::catalog::DecorationSeedSpec,
) -> i32 {
    let heightmap = if is_beached {
        HeightMap::WorldSurfaceWg
    } else {
        HeightMap::OceanFloorWg
    };
    let mut lowest = i32::MAX;
    let mut sum = 0_i64;
    for x in origin.x..origin.x + template.size.x {
        for z in origin.z..origin.z + template.size.z {
            let height = terrain.height(heightmap, x, z);
            lowest = lowest.min(height);
            sum += i64::from(height);
        }
    }
    if is_beached {
        let mut random =
            DecorationRandom::for_feature(world_seed, first_chunk.0, first_chunk.1, decoration);
        lowest - template.size.y / 2 - random.next_int(3)
    } else {
        let area = i64::from(template.size.x) * i64::from(template.size.z);
        i32::try_from(sum / area).expect("shipwreck mean terrain height must fit i32")
    }
}

fn shipwreck_marker_loot_table(marker: &str) -> Option<&'static str> {
    match marker {
        "map_chest" => Some("minecraft:chests/shipwreck_map"),
        "supply_chest" => Some("minecraft:chests/shipwreck_supply"),
        "treasure_chest" => Some("minecraft:chests/shipwreck_treasure"),
        _ => None,
    }
}

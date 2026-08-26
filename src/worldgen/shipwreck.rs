use glam::IVec3;
use steel_registry::{REGISTRY, RegistryExt};
use steel_utils::{Identifier, Rotation};
use steel_worldgen::structure::StructureGenerationContext;

use super::template_data::{TemplateContainerData, get_template_container_data};
use super::template_scan::{RandomPrefix, TemplatePlacement};
use super::{Scan, Scanner, ScannerContext, invalid_scan};
use crate::catalog::shipwreck_decoration;
use crate::decoration_seed::DecorationRandom;
use crate::error::Error;
use crate::random::Random;

const PIVOT: IVec3 = IVec3::new(4, 0, 15);

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
    pub(super) fn scan_shipwreck(&self, chunk_x: i32, chunk_z: i32) -> Result<Scan, Error> {
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

        let Some(is_beached) = self.select_shipwreck_variant(chunk_x, chunk_z, middle_x, middle_z)
        else {
            return Ok(invalid_scan());
        };

        let mut generation_random = self.chunk_random(chunk_x, chunk_z);
        let rotation = Rotation::get_random(&mut generation_random);
        let templates = if is_beached {
            BEACHED_TEMPLATES
        } else {
            OCEAN_TEMPLATES
        };
        let template_name =
            templates[generation_random.next_i32_bounded(templates.len() as i32) as usize];
        let template = get_template_container_data(template_name).ok_or_else(|| {
            Error::Worldgen(format!("shipwreck template is missing: {template_name}"))
        })?;

        let initial =
            TemplatePlacement::new(template, IVec3::new(min_x, 90, min_z), rotation, PIVOT);
        let first_chunk = initial.first_intersecting_chunk();
        let decoration = shipwreck_decoration(is_beached);
        let random_prefix = is_beached.then_some(RandomPrefix {
            chunk: first_chunk,
            next_int_bound: 3,
        });
        let mut ctx = self.generation_context(chunk_x, chunk_z);
        let target_y = self.shipwreck_target_y(
            &mut ctx,
            template,
            IVec3::new(min_x, 90, min_z),
            is_beached,
            first_chunk,
            decoration,
        );
        let placement = TemplatePlacement::new(
            template,
            IVec3::new(min_x, target_y, min_z),
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
    ) -> Option<bool> {
        let mut remaining = vec![
            (false, Identifier::new_static("minecraft", "shipwreck")),
            (
                true,
                Identifier::new_static("minecraft", "shipwreck_beached"),
            ),
        ];
        let mut selection_random = self.chunk_random(chunk_x, chunk_z);
        let mut ctx = self.generation_context(chunk_x, chunk_z);
        while !remaining.is_empty() {
            let choice = selection_random.next_i32_bounded(remaining.len() as i32) as usize;
            let (is_beached, structure_id) = remaining.remove(choice);
            let structure_data = REGISTRY.structures.by_key(&structure_id)?;
            let ocean_floor = !is_beached;
            let y = ctx.terrain_surface_height(middle_x, middle_z, ocean_floor) - 1;
            let biome = ctx.biome_at(middle_x, y, middle_z);
            if structure_data.allowed_biomes.contains(&biome.key) {
                return Some(is_beached);
            }
        }
        None
    }

    fn shipwreck_target_y(
        &self,
        ctx: &mut ScannerContext<'_>,
        template: &TemplateContainerData,
        origin: IVec3,
        is_beached: bool,
        first_chunk: (i32, i32),
        decoration: crate::catalog::DecorationSeedSpec,
    ) -> i32 {
        let ocean_floor = !is_beached;
        let mut lowest = i32::MAX;
        let mut sum = 0_i64;
        for x in origin.x..origin.x + template.size[0] {
            for z in origin.z..origin.z + template.size[2] {
                let height = ctx.terrain_surface_height(x, z, ocean_floor);
                lowest = lowest.min(height);
                sum += i64::from(height);
            }
        }
        if is_beached {
            let mut random = DecorationRandom::for_feature(
                self.world_seed,
                first_chunk.0,
                first_chunk.1,
                decoration,
            );
            lowest - template.size[1] / 2 - random.next_int(3)
        } else {
            let area = i64::from(template.size[0]) * i64::from(template.size[2]);
            i32::try_from(sum / area).expect("shipwreck mean terrain height must fit i32")
        }
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

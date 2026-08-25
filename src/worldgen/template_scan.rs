use glam::IVec3;
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;
use steel_utils::Rotation;

use super::Chest;
use super::template_data::TemplateContainerData;
use crate::catalog::DecorationSeedSpec;
use crate::decoration_seed::DecorationRandom;

#[derive(Clone, Copy)]
pub(super) struct RandomPrefix {
    pub chunk: (i32, i32),
    pub next_int_bound: i32,
}

pub(super) struct TemplatePlacement<'a> {
    template: &'a TemplateContainerData,
    origin: IVec3,
    rotation: Rotation,
    pivot: IVec3,
}

impl<'a> TemplatePlacement<'a> {
    pub(super) const fn new(
        template: &'a TemplateContainerData,
        origin: IVec3,
        rotation: Rotation,
        pivot: IVec3,
    ) -> Self {
        Self {
            template,
            origin,
            rotation,
            pivot,
        }
    }

    pub(super) fn first_intersecting_chunk(&self) -> (i32, i32) {
        let (min_x, _, min_z, _) = self.horizontal_bounds();
        (min_x.div_euclid(16), min_z.div_euclid(16))
    }

    pub(super) fn collect_chests(
        &self,
        world_seed: i64,
        structure_chunk: (i32, i32),
        decoration: DecorationSeedSpec,
        random_prefix: Option<RandomPrefix>,
        marker_loot_table: fn(&str) -> Option<&'static str>,
    ) -> Vec<Chest> {
        let mut random_by_chunk = FxHashMap::<(i32, i32), DecorationRandom>::default();

        // Vanilla template placement loads every randomizable container block entity first.
        // Loading injects and consumes a temporary LootTableSeed even when the template NBT has
        // no LootTable. Data markers run only after all template blocks have been placed.
        for container_pos in self.template.randomizable_containers {
            let position = self.world_position(IVec3::new(
                container_pos.x,
                container_pos.y,
                container_pos.z,
            ));
            let chunk = decoration_chunk(position);
            random_for_chunk(
                &mut random_by_chunk,
                world_seed,
                decoration,
                chunk,
                random_prefix,
            )
            .next_long();
        }

        let mut next_visible_ordinal = FxHashMap::<(i32, i32), i32>::default();
        let mut chests = Vec::new();
        for marker in self.template.markers {
            let Some(loot_table) = marker_loot_table(marker.metadata) else {
                continue;
            };

            let marker_pos = self.world_position(IVec3::new(marker.x, marker.y, marker.z));
            let position = IVec3::new(marker_pos.x, marker_pos.y - 1, marker_pos.z);
            let chunk = decoration_chunk(position);
            let ordinal = next_visible_ordinal.entry(chunk).or_default();
            let loot_seed = random_for_chunk(
                &mut random_by_chunk,
                world_seed,
                decoration,
                chunk,
                random_prefix,
            )
            .next_long();
            chests.push(Chest {
                structure_chunk_x: structure_chunk.0,
                structure_chunk_z: structure_chunk.1,
                x: position.x,
                y: position.y,
                z: position.z,
                loot_table: loot_table.to_owned(),
                ordinal: *ordinal,
                loot_seed,
            });
            *ordinal += 1;
        }
        chests
    }

    fn horizontal_bounds(&self) -> (i32, i32, i32, i32) {
        let max_x = self.template.size[0].saturating_sub(1);
        let max_z = self.template.size[2].saturating_sub(1);
        [(0, 0), (max_x, 0), (0, max_z), (max_x, max_z)]
            .into_iter()
            .map(|(x, z)| self.world_position(IVec3::new(x, 0, z)))
            .fold(
                (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
                |(min_x, max_x, min_z, max_z), position| {
                    (
                        min_x.min(position.x),
                        max_x.max(position.x),
                        min_z.min(position.z),
                        max_z.max(position.z),
                    )
                },
            )
    }

    fn world_position(&self, position: IVec3) -> IVec3 {
        let transformed = self.rotation.transform_pos(position, self.pivot);
        self.origin + transformed
    }
}

fn random_for_chunk(
    random_by_chunk: &mut FxHashMap<(i32, i32), DecorationRandom>,
    world_seed: i64,
    decoration: DecorationSeedSpec,
    chunk: (i32, i32),
    random_prefix: Option<RandomPrefix>,
) -> &mut DecorationRandom {
    match random_by_chunk.entry(chunk) {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(entry) => {
            let mut random =
                DecorationRandom::for_feature(world_seed, chunk.0, chunk.1, decoration);
            if let Some(prefix) = random_prefix
                && prefix.chunk == chunk
            {
                random.next_int(prefix.next_int_bound);
            }
            entry.insert(random)
        }
    }
}

fn decoration_chunk(position: IVec3) -> (i32, i32) {
    (position.x.div_euclid(16), position.z.div_euclid(16))
}

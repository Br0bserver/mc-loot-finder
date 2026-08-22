use std::collections::{HashMap, hash_map::Entry};

use pumpkin_data::Rotation;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::generation::structure::template::StructureTemplate;

use super::Chest;
use crate::catalog::DecorationSeedSpec;
use crate::decoration_seed::DecorationRandom;

#[derive(Clone, Copy)]
pub(super) struct RandomPrefix {
    pub chunk: (i32, i32),
    pub next_int_bound: i32,
}

pub(super) struct TemplatePlacement<'a> {
    template: &'a StructureTemplate,
    origin: Vector3<i32>,
    rotation: Rotation,
    pivot: Vector3<i32>,
}

impl<'a> TemplatePlacement<'a> {
    pub(super) const fn new(
        template: &'a StructureTemplate,
        origin: Vector3<i32>,
        rotation: Rotation,
        pivot: Vector3<i32>,
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
        let mut random_by_chunk = HashMap::<(i32, i32), DecorationRandom>::with_capacity(4);

        // Vanilla template placement loads every randomizable container block entity first.
        // Loading injects and consumes a temporary LootTableSeed even when the template NBT has
        // no LootTable. Data markers run only after all template blocks have been placed.
        for block in &self.template.blocks {
            let palette = &self.template.palette[block.state as usize];
            if block.nbt.is_none() || !is_randomizable_container(&palette.name) {
                continue;
            }
            let position = self.world_position(block.pos);
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

        let mut next_visible_ordinal = HashMap::<(i32, i32), i32>::with_capacity(4);
        let mut chests = Vec::new();
        for block in &self.template.blocks {
            let palette = &self.template.palette[block.state as usize];
            if palette.name != "minecraft:structure_block" {
                continue;
            }
            let Some(metadata) = block
                .nbt
                .as_ref()
                .and_then(|nbt| nbt.get_string("metadata"))
            else {
                continue;
            };
            let Some(loot_table) = marker_loot_table(metadata) else {
                continue;
            };

            let marker = self.world_position(block.pos);
            let position = Vector3::new(marker.x, marker.y - 1, marker.z);
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
        let max_x = self.template.size.x.saturating_sub(1);
        let max_z = self.template.size.z.saturating_sub(1);
        [(0, 0), (max_x, 0), (0, max_z), (max_x, max_z)]
            .into_iter()
            .map(|(x, z)| self.world_position(Vector3::new(x, 0, z)))
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

    fn world_position(&self, position: Vector3<i32>) -> Vector3<i32> {
        let (x, z) = rotate_around_pivot(
            self.rotation,
            position.x,
            position.z,
            self.pivot.x,
            self.pivot.z,
        );
        let transformed = Vector3::new(x, position.y, z);
        Vector3::new(
            self.origin.x + transformed.x,
            self.origin.y + transformed.y,
            self.origin.z + transformed.z,
        )
    }
}

pub(super) const fn rotate_around_pivot(
    rotation: Rotation,
    x: i32,
    z: i32,
    pivot_x: i32,
    pivot_z: i32,
) -> (i32, i32) {
    match rotation {
        Rotation::None => (x, z),
        Rotation::Clockwise90 => (pivot_x - z + pivot_z, pivot_z + x - pivot_x),
        Rotation::Rotate180 => (2 * pivot_x - x, 2 * pivot_z - z),
        Rotation::CounterClockwise90 => (pivot_x + z - pivot_z, pivot_z - x + pivot_x),
    }
}

fn random_for_chunk(
    random_by_chunk: &mut HashMap<(i32, i32), DecorationRandom>,
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

fn decoration_chunk(position: Vector3<i32>) -> (i32, i32) {
    (position.x.div_euclid(16), position.z.div_euclid(16))
}

fn is_randomizable_container(name: &str) -> bool {
    matches!(
        name,
        "minecraft:chest"
            | "minecraft:trapped_chest"
            | "minecraft:barrel"
            | "minecraft:dispenser"
            | "minecraft:dropper"
            | "minecraft:hopper"
    )
}

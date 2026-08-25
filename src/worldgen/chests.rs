use glam::IVec3;
use rustc_hash::FxHashMap;
use steel_registry::template_pool::PoolElement;
use steel_utils::Identifier;
use steel_worldgen::structure::{StructurePiece, StructurePiecePayload};

use super::Chest;
use super::template_data::get_template_container_data;
use crate::catalog::DecorationSeedSpec;
use crate::decoration_seed::container_loot_seed;
use crate::error::Error;

pub(super) struct RawChest {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub loot_table: String,
}

fn collect_element_chests(
    element: &PoolElement,
    position: IVec3,
    rotation: steel_utils::Rotation,
    output: &mut Vec<RawChest>,
) {
    match element {
        PoolElement::Single { location, .. } | PoolElement::LegacySingle { location, .. } => {
            collect_template_chests(location, position, rotation, output);
        }
        PoolElement::List { elements, .. } => {
            for sub_element in elements {
                collect_element_chests(sub_element, position, rotation, output);
            }
        }
        PoolElement::Empty | PoolElement::Feature { .. } => {}
    }
}

fn collect_template_chests(
    location: &Identifier,
    position: IVec3,
    rotation: steel_utils::Rotation,
    output: &mut Vec<RawChest>,
) {
    let Some(template_data) = get_template_container_data(&location.path) else {
        return;
    };
    for chest in template_data.chests {
        let local_pos = IVec3::new(chest.x, chest.y, chest.z);
        let world_pos = position + rotation.transform_pos(local_pos, IVec3::ZERO);
        output.push(RawChest {
            x: world_pos.x,
            y: world_pos.y,
            z: world_pos.z,
            loot_table: chest.loot_table.to_owned(),
        });
    }
}

pub(super) fn collect_stub_chests(pieces: &[StructurePiece]) -> Vec<RawChest> {
    let mut raw = Vec::new();
    for piece in pieces {
        let StructurePiecePayload::Jigsaw(data) = &piece.payload else {
            continue;
        };
        collect_element_chests(&data.pool_element, data.position, data.rotation, &mut raw);
    }
    raw
}

pub(super) fn dedup_and_seed_chests(
    world_seed: i64,
    raw: Vec<RawChest>,
    structure_chunk: (i32, i32),
    decoration: DecorationSeedSpec,
) -> Result<Vec<Chest>, Error> {
    let mut next_ordinal_by_chunk = FxHashMap::<(i32, i32), i32>::default();
    let mut visible = Vec::with_capacity(raw.len());
    let mut index_by_position = FxHashMap::<(i32, i32, i32), usize>::default();
    for chest in raw {
        let chest_chunk_x = chest.x.div_euclid(16);
        let chest_chunk_z = chest.z.div_euclid(16);
        let ordinal = next_ordinal_by_chunk
            .entry((chest_chunk_x, chest_chunk_z))
            .or_insert(0);
        let current_ordinal = *ordinal;
        *ordinal += 1;
        let loot_seed = container_loot_seed(
            world_seed,
            chest_chunk_x,
            chest_chunk_z,
            decoration,
            current_ordinal,
        )?;
        let prediction = Chest {
            structure_chunk_x: structure_chunk.0,
            structure_chunk_z: structure_chunk.1,
            x: chest.x,
            y: chest.y,
            z: chest.z,
            loot_table: chest.loot_table,
            ordinal: current_ordinal,
            loot_seed,
        };
        let key = (prediction.x, prediction.y, prediction.z);
        if let Some(idx) = index_by_position.get(&key).copied() {
            visible[idx] = prediction;
        } else {
            index_by_position.insert(key, visible.len());
            visible.push(prediction);
        }
    }
    Ok(visible)
}

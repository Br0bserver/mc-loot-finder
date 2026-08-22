use std::collections::HashMap;

use super::Chest;
use crate::catalog::DecorationSeedSpec;
use crate::decoration_seed::container_loot_seed;
use crate::error::Error;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::generation::structure::structures::jigsaw::PoolElementStructurePiece;
pub(super) struct RawChest {
    x: i32,
    y: i32,
    z: i32,
    loot_table: String,
}

pub(super) fn collect_piece_chests(piece: &PoolElementStructurePiece, output: &mut Vec<RawChest>) {
    let origin = piece.pos.0;
    piece.element.for_each_template(|_, _, _, template| {
        let (corner_x, corner_z) = piece.rotation.rotate_offset(
            template.size.x.saturating_sub(1),
            template.size.z.saturating_sub(1),
        );
        let placement_origin = Vector3::new(
            origin.x + corner_x.min(0),
            origin.y,
            origin.z + corner_z.min(0),
        );
        for block in &template.blocks {
            let palette = &template.palette[block.state as usize];
            if palette.name != "minecraft:chest" {
                continue;
            }
            let local = piece.rotation.transform_pos(block.pos, template.size);
            let world = Vector3::new(
                placement_origin.x + local.x,
                placement_origin.y + local.y,
                placement_origin.z + local.z,
            );
            let loot_table = block
                .nbt
                .as_ref()
                .and_then(|nbt| nbt.get_string("LootTable"))
                .unwrap_or_default()
                .to_owned();
            output.push(RawChest {
                x: world.x,
                y: world.y,
                z: world.z,
                loot_table,
            });
        }
    });
}

pub(super) fn collect_position_chests(
    position: &pumpkin_world::generation::structure::structures::StructurePosition,
    poison_context: &'static str,
) -> Result<Vec<RawChest>, Error> {
    let collector = position
        .collector
        .lock()
        .map_err(|_| Error::Worldgen(format!("{poison_context} piece collector was poisoned")))?;
    let mut raw = Vec::new();
    for piece in &collector.pieces {
        let Some(piece) = piece.as_any().downcast_ref::<PoolElementStructurePiece>() else {
            continue;
        };
        collect_piece_chests(piece, &mut raw);
    }
    Ok(raw)
}
pub(super) fn dedup_and_seed_chests(
    world_seed: i64,
    raw: Vec<RawChest>,
    structure_chunk: (i32, i32),
    decoration: DecorationSeedSpec,
) -> Result<Vec<Chest>, Error> {
    let mut next_ordinal_by_chunk = HashMap::with_capacity(4);
    let mut visible = Vec::with_capacity(raw.len());
    let mut index_by_position = HashMap::with_capacity(raw.len());
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

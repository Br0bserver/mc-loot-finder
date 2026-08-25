use glam::IVec3;
use rustc_hash::FxHashMap;
use steel_registry::template_pool::PoolElement;
use steel_utils::Identifier;
use steel_worldgen::structure::{StructurePiece, StructurePiecePayload};

use super::Chest;
#[cfg(test)]
use super::template_data::{TemplateBlockPos, TemplateChest};
use super::template_data::{TemplateContainerData, get_template_container_data};
use crate::catalog::DecorationSeedSpec;
use crate::decoration_seed::container_loot_seed;
use crate::error::Error;

enum RawContainer {
    Hidden {
        x: i32,
        y: i32,
        z: i32,
    },
    Visible {
        x: i32,
        y: i32,
        z: i32,
        loot_table: String,
    },
}

fn collect_element_containers(
    element: &PoolElement,
    position: IVec3,
    rotation: steel_utils::Rotation,
    output: &mut Vec<RawContainer>,
) {
    match element {
        PoolElement::Single { location, .. } | PoolElement::LegacySingle { location, .. } => {
            let Some(template_data) = get_template_container_data(&location.path) else {
                return;
            };
            collect_template_containers(template_data, position, rotation, output);
        }
        PoolElement::List { elements, .. } => {
            for sub_element in elements {
                collect_element_containers(sub_element, position, rotation, output);
            }
        }
        PoolElement::Empty | PoolElement::Feature { .. } => {}
    }
}

fn collect_template_containers(
    template_data: &TemplateContainerData,
    position: IVec3,
    rotation: steel_utils::Rotation,
    output: &mut Vec<RawContainer>,
) {
    let mut emitted_visible = vec![false; template_data.chests.len()];
    for container in template_data.randomizable_containers {
        let local_pos = IVec3::new(container.x, container.y, container.z);
        let world_pos = position + rotation.transform_pos(local_pos, IVec3::ZERO);
        let visible_index = template_data
            .chests
            .iter()
            .enumerate()
            .find_map(|(index, chest)| {
                (!emitted_visible[index]
                    && chest.x == container.x
                    && chest.y == container.y
                    && chest.z == container.z)
                    .then_some(index)
            });
        if let Some(index) = visible_index {
            let chest = &template_data.chests[index];
            emitted_visible[index] = true;
            output.push(RawContainer::Visible {
                x: world_pos.x,
                y: world_pos.y,
                z: world_pos.z,
                loot_table: chest.loot_table.to_owned(),
            });
        } else {
            output.push(RawContainer::Hidden {
                x: world_pos.x,
                y: world_pos.y,
                z: world_pos.z,
            });
        }
    }

    for (index, chest) in template_data.chests.iter().enumerate() {
        if emitted_visible[index] {
            continue;
        }
        let local_pos = IVec3::new(chest.x, chest.y, chest.z);
        let world_pos = position + rotation.transform_pos(local_pos, IVec3::ZERO);
        output.push(RawContainer::Visible {
            x: world_pos.x,
            y: world_pos.y,
            z: world_pos.z,
            loot_table: chest.loot_table.to_owned(),
        });
    }
}

pub(super) fn collect_stub_containers(pieces: &[StructurePiece]) -> Vec<RawContainer> {
    let mut raw = Vec::new();
    for piece in pieces {
        let StructurePiecePayload::Jigsaw(data) = &piece.payload else {
            continue;
        };
        collect_element_containers(&data.pool_element, data.position, data.rotation, &mut raw);
    }
    raw
}

pub(super) fn dedup_and_seed_chests(
    world_seed: i64,
    raw: Vec<RawContainer>,
    structure_chunk: (i32, i32),
    decoration: DecorationSeedSpec,
) -> Result<Vec<Chest>, Error> {
    let mut next_ordinal_by_chunk = FxHashMap::<(i32, i32), i32>::default();
    let mut visible = Vec::with_capacity(raw.len());
    let mut index_by_position = FxHashMap::<(i32, i32, i32), usize>::default();
    for container in raw {
        let (x, y, z, loot_table) = match container {
            RawContainer::Hidden { x, y, z } => (x, y, z, None),
            RawContainer::Visible {
                x,
                y,
                z,
                loot_table,
            } => (x, y, z, Some(loot_table)),
        };
        let chest_chunk_x = x.div_euclid(16);
        let chest_chunk_z = z.div_euclid(16);
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
        let Some(loot_table) = loot_table else {
            continue;
        };
        let prediction = Chest {
            structure_chunk_x: structure_chunk.0,
            structure_chunk_z: structure_chunk.1,
            x,
            y,
            z,
            loot_table,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::ContainerSeedShortcut;

    #[test]
    fn hidden_template_container_consumes_ordinal_before_visible_chest() {
        static CHESTS: &[TemplateChest] = &[TemplateChest {
            x: 1,
            y: 2,
            z: 3,
            loot_table: "minecraft:chests/ancient_city",
        }];
        static CONTAINERS: &[TemplateBlockPos] = &[
            TemplateBlockPos { x: 0, y: 2, z: 3 },
            TemplateBlockPos { x: 1, y: 2, z: 3 },
        ];
        let template = TemplateContainerData {
            size: [4, 4, 4],
            chests: CHESTS,
            markers: &[],
            randomizable_containers: CONTAINERS,
        };
        let mut raw = Vec::new();
        collect_template_containers(
            &template,
            IVec3::ZERO,
            steel_utils::Rotation::None,
            &mut raw,
        );
        let chests = dedup_and_seed_chests(
            0,
            raw,
            (0, 0),
            DecorationSeedSpec {
                structure_index: 0,
                step: 7,
                ordinal_offset: 0,
                shortcut: ContainerSeedShortcut::Direct,
            },
        )
        .unwrap();

        assert_eq!(chests.len(), 1);
        assert_eq!(chests[0].ordinal, 1);
        assert_eq!(
            chests[0].loot_seed,
            container_loot_seed(
                0,
                0,
                0,
                DecorationSeedSpec {
                    structure_index: 0,
                    step: 7,
                    ordinal_offset: 0,
                    shortcut: ContainerSeedShortcut::Direct,
                },
                1,
            )
            .unwrap()
        );
    }
}

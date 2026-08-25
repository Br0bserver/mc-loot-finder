use super::chests::{collect_stub_chests, dedup_and_seed_chests};
use super::{Scan, Scanner, invalid_scan};
use crate::catalog::{ContainerSeedShortcut, DecorationSeedSpec, ScanKind, VILLAGE_PLACEMENT};
use crate::error::Error;
use crate::placement;
use crate::random::{LegacyRandom48, Random};
use steel_registry::REGISTRY;
use steel_utils::Identifier;
use steel_worldgen::structure::Structure;
use steel_worldgen::structure::jigsaw::JigsawStructure;

const PILLAGER_FREQUENCY: f32 = 0.2;
const VILLAGE_EXCLUSION_RADIUS: i32 = 10;

fn has_village_nearby(world_seed: i64, chunk_x: i32, chunk_z: i32) -> bool {
    let radius = VILLAGE_EXCLUSION_RADIUS;
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            let other_x = chunk_x + dx;
            let other_z = chunk_z + dz;
            if placement::is_placement_chunk(world_seed, other_x, other_z, VILLAGE_PLACEMENT) {
                return true;
            }
        }
    }
    false
}

fn pillager_frequency_passes(world_seed: i64, chunk_x: i32, chunk_z: i32) -> bool {
    let i = chunk_x >> 4;
    let j = chunk_z >> 4;
    let combined = i64::from(i << 4 ^ j) ^ world_seed;
    let mut random = LegacyRandom48::from_seed(combined as u64);
    let _ = random.next_i32();
    let bound = (1.0 / PILLAGER_FREQUENCY) as i32;
    random.next_i32_bounded(bound) == 0
}

impl Scanner {
    pub(super) fn scan_jigsaw_structure(&self, chunk_x: i32, chunk_z: i32) -> Result<Scan, Error> {
        if self.kind == ScanKind::BastionRemnant
            && !self.bastion_reached_in_weighted_selection(chunk_x, chunk_z)?
        {
            return Ok(invalid_scan());
        }

        let structure_id = self.kind.identifier();
        let structure_data = REGISTRY.structures.get(&structure_id).ok_or_else(|| {
            Error::Worldgen(format!(
                "structure registry missing {}",
                self.structure.name
            ))
        })?;

        let mut ctx = self.generation_context(chunk_x, chunk_z);
        let mut rng = self.feature_random(chunk_x, chunk_z);

        let Some(stub) = JigsawStructure.find_generation_point(&mut ctx, structure_data, &mut rng)
        else {
            return Ok(invalid_scan());
        };

        let raw = collect_stub_chests(&stub.pieces);
        let visible =
            dedup_and_seed_chests(self.world_seed, raw, (chunk_x, chunk_z), self.decoration()?)?;

        Ok(Scan {
            valid_structure: true,
            chests: visible,
        })
    }

    /// Scans a village candidate chunk.
    pub(super) fn scan_village(&self, chunk_x: i32, chunk_z: i32) -> Result<Scan, Error> {
        let mut random = self.chunk_random(chunk_x, chunk_z);
        let mut remaining = vec![
            (Identifier::new_static("minecraft", "village_desert"), 21),
            (Identifier::new_static("minecraft", "village_plains"), 22),
            (Identifier::new_static("minecraft", "village_savanna"), 23),
            (Identifier::new_static("minecraft", "village_snowy"), 24),
            (Identifier::new_static("minecraft", "village_taiga"), 25),
        ];

        let mut selected_stub = None;
        let mut selected_index = 0;

        while !remaining.is_empty() {
            let choice = random.next_i32_bounded(remaining.len() as i32) as usize;
            let (structure_id, index) = remaining.swap_remove(choice);
            let structure_data = REGISTRY.structures.get(&structure_id).ok_or_else(|| {
                Error::Worldgen(format!("village structure registry missing {structure_id}"))
            })?;

            let mut ctx = self.generation_context(chunk_x, chunk_z);
            let mut rng = self.feature_random(chunk_x, chunk_z);

            if let Some(stub) =
                JigsawStructure.find_generation_point(&mut ctx, structure_data, &mut rng)
            {
                selected_stub = Some(stub);
                selected_index = index;
                break;
            }
        }

        let Some(stub) = selected_stub else {
            return Ok(invalid_scan());
        };

        let raw = collect_stub_chests(&stub.pieces);
        let visible = dedup_and_seed_chests(
            self.world_seed,
            raw,
            (chunk_x, chunk_z),
            DecorationSeedSpec {
                structure_index: selected_index,
                step: 4,
                ordinal_offset: 0,
                shortcut: ContainerSeedShortcut::Direct,
            },
        )?;

        Ok(Scan {
            valid_structure: true,
            chests: visible,
        })
    }

    pub(super) fn scan_pillager_outpost(&self, chunk_x: i32, chunk_z: i32) -> Result<Scan, Error> {
        if !pillager_frequency_passes(self.world_seed, chunk_x, chunk_z) {
            return Ok(invalid_scan());
        }
        if has_village_nearby(self.world_seed, chunk_x, chunk_z) {
            return Ok(invalid_scan());
        }

        let structure_id = self.kind.identifier();
        let structure_data = REGISTRY.structures.get(&structure_id).ok_or_else(|| {
            Error::Worldgen("pillager outpost structure registry missing".to_owned())
        })?;

        let mut ctx = self.generation_context(chunk_x, chunk_z);
        let mut rng = self.feature_random(chunk_x, chunk_z);

        let Some(stub) = JigsawStructure.find_generation_point(&mut ctx, structure_data, &mut rng)
        else {
            return Ok(invalid_scan());
        };

        let raw = collect_stub_chests(&stub.pieces);
        let visible =
            dedup_and_seed_chests(self.world_seed, raw, (chunk_x, chunk_z), self.decoration()?)?;

        Ok(Scan {
            valid_structure: true,
            chests: visible,
        })
    }

    fn bastion_reached_in_weighted_selection(
        &self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Result<bool, Error> {
        let mut selection_random = self.chunk_random(chunk_x, chunk_z);
        let bastion_selected_first = selection_random.next_i32_bounded(5) >= 2;
        if bastion_selected_first {
            return Ok(true);
        }

        let block_x = chunk_x.checked_mul(16).ok_or_else(|| {
            Error::Worldgen("fortress biome probe x coordinate overflowed".to_owned())
        })?;
        let block_z = chunk_z.checked_mul(16).ok_or_else(|| {
            Error::Worldgen("fortress biome probe z coordinate overflowed".to_owned())
        })?;

        let mut ctx = self.generation_context(chunk_x, chunk_z);
        let biome = ctx.biome_at(block_x, 64, block_z);
        let fortress_data = REGISTRY
            .structures
            .get(&Identifier::new_static("minecraft", "fortress"))
            .ok_or_else(|| Error::Worldgen("fortress registry entry missing".to_owned()))?;

        Ok(!fortress_data.allowed_biomes.contains(&biome.key))
    }
}

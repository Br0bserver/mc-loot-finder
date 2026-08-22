use crate::error::Error;
pub mod candidates;
pub mod chests;
pub mod container_seed;
pub mod explain;
pub mod find;
pub mod loot;

use crate::catalog::CandidateStructure;
use crate::placement::{self, Candidate};
use crate::worldgen::{self, Scan};

/// Locate placement candidates and verify them with a full worldgen scan.
pub(crate) fn locate_and_scan(
    world_seed: i64,
    structure: &'static CandidateStructure,
    center_x: i32,
    center_z: i32,
    radius: i32,
) -> Result<(Vec<Candidate>, Vec<Scan>), Error> {
    let scanner = worldgen::Scanner::for_structure(structure, world_seed)?;
    let candidates =
        placement::locate(world_seed, center_x, center_z, radius, structure.placement)?;
    let scans = scanner.scan_many(candidates.iter().map(|c| (c.chunk_x, c.chunk_z)))?;
    Ok((candidates, scans))
}

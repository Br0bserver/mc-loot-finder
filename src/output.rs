use serde::Serialize;

/// Format an integer with thousands separators.
pub fn grouped(value: i64) -> String {
    let negative = value < 0;
    let digits = value.unsigned_abs().to_string();
    let mut result = String::with_capacity(digits.len() + digits.len() / 3 + usize::from(negative));
    if negative {
        result.push('-');
    }
    for (index, character) in digits.chars().enumerate() {
        if index != 0 && (digits.len() - index).is_multiple_of(3) {
            result.push(',');
        }
        result.push(character);
    }
    result
}

/// Format a count with a singular or plural label.
pub fn quantity(count: i64, singular: &str) -> String {
    format!(
        "{} {singular}{}",
        grouped(count),
        if count == 1 { "" } else { "s" }
    )
}

/// Print a JSON value followed by a newline.
pub fn print_json<T: Serialize>(value: &T) {
    println!(
        "{}",
        serde_json::to_string(value).expect("json serialization cannot fail")
    );
}

/// Distance rounded to three decimal places, matching the legacy "{:.3}" display.
pub fn rounded_distance(squared_distance: i64) -> f64 {
    let distance = (squared_distance as f64).sqrt();
    (distance * 1000.0).round() / 1000.0
}

#[derive(Debug, Serialize)]
pub struct FindOutput {
    pub version: &'static str,
    pub structure: &'static str,
    pub seed: i64,
    pub item: String,
    pub placement_candidates: usize,
    pub valid_structures: usize,
    pub checked_chests: usize,
    pub hits: usize,
    pub unpredictable_zero_seeds: usize,
    pub matches: Vec<FindMatch>,
}

#[derive(Debug, Serialize)]
pub struct FindMatch {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub item_count: i32,
    pub loot_table: String,
    pub loot_seed: i64,
    pub start_chunk_x: i32,
    pub start_chunk_z: i32,
}

#[derive(Debug, Serialize)]
pub struct ChestsOutput {
    pub version: &'static str,
    pub structure: &'static str,
    pub seed: i64,
    pub placement_candidates: usize,
    pub valid_structures: usize,
    pub chest_count: usize,
    pub chests: Vec<ChestJson>,
}

#[derive(Debug, Serialize)]
pub struct ChestJson {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub loot_table: String,
    pub loot_seed: i64,
    pub start_chunk_x: i32,
    pub start_chunk_z: i32,
    pub ordinal: i32,
}

#[derive(Debug, Serialize)]
pub struct CandidatesOutput {
    pub version: &'static str,
    pub structure: &'static str,
    pub seed: i64,
    pub status: &'static str,
    pub candidates: Vec<CandidateJson>,
}

#[derive(Debug, Serialize)]
pub struct CandidateJson {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub block_x: i32,
    pub block_z: i32,
    pub distance: f64,
}

#[derive(Debug, Serialize)]
pub struct LootOutput {
    pub version: &'static str,
    pub loot_table: String,
    pub loot_seed: i64,
    pub items: Vec<LootStackJson>,
}

#[derive(Debug, Serialize)]
pub struct LootStackJson {
    pub item: String,
    pub count: i32,
}

#[derive(Debug, Serialize)]
pub struct ContainerSeedOutput {
    pub version: &'static str,
    pub structure: &'static str,
    pub world_seed: i64,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub structure_index: i32,
    pub step: i32,
    pub ordinal: i32,
    pub loot_table_seed: i64,
}

#[derive(Debug, Serialize)]
pub struct ExplainListingOutput {
    pub version: &'static str,
    pub structures: Vec<ExplainStructureJson>,
}

#[derive(Debug, Serialize)]
pub struct ExplainStructureJson {
    pub name: &'static str,
    pub dimension: &'static str,
    pub full_scan: bool,
    pub default_item: &'static str,
    pub loot_tables: usize,
}

#[derive(Debug, Serialize)]
pub struct ExplainDetailOutput {
    pub version: &'static str,
    pub name: &'static str,
    pub structure_id: &'static str,
    pub dimension: &'static str,
    pub full_scan: bool,
    pub default_item: &'static str,
    pub placement: PlacementJson,
    pub decoration_step: i32,
    pub decoration_index: i32,
    pub scanner: &'static str,
    pub container_seed_shortcut: &'static str,
    pub loot_tables: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct PlacementJson {
    pub spacing: i32,
    pub separation: i32,
    pub salt: i64,
    pub spread: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_grouped_numbers() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(5_000), "5,000");
        assert_eq!(grouped(-1_234_567), "-1,234,567");
    }

    #[test]
    fn formats_quantities() {
        assert_eq!(quantity(1, "chest"), "1 chest");
        assert_eq!(quantity(3, "chest"), "3 chests");
    }

    #[test]
    fn find_output_keeps_wire_format() {
        let output = FindOutput {
            version: "26.1.2",
            structure: "ancient_city",
            seed: 114_514,
            item: "minecraft:silence_armor_trim_smithing_template".to_owned(),
            placement_candidates: 1,
            valid_structures: 1,
            checked_chests: 1,
            hits: 1,
            unpredictable_zero_seeds: 0,
            matches: vec![FindMatch {
                x: 1450,
                y: -35,
                z: 137,
                item_count: 1,
                loot_table: "minecraft:chests/ancient_city".to_owned(),
                loot_seed: 1_392_286_922_750_350_146,
                start_chunk_x: 96,
                start_chunk_z: 5,
            }],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert_eq!(
            json,
            r#"{"version":"26.1.2","structure":"ancient_city","seed":114514,"item":"minecraft:silence_armor_trim_smithing_template","placement_candidates":1,"valid_structures":1,"checked_chests":1,"hits":1,"unpredictable_zero_seeds":0,"matches":[{"x":1450,"y":-35,"z":137,"item_count":1,"loot_table":"minecraft:chests/ancient_city","loot_seed":1392286922750350146,"start_chunk_x":96,"start_chunk_z":5}]}"#
        );
    }

    #[test]
    fn chests_output_keeps_wire_format() {
        let output = ChestsOutput {
            version: "26.1.2",
            structure: "ancient_city",
            seed: 0,
            placement_candidates: 1,
            valid_structures: 1,
            chest_count: 1,
            chests: vec![ChestJson {
                x: 180,
                y: 80,
                z: -233,
                loot_table: "minecraft:chests/bastion_bridge".to_owned(),
                loot_seed: 1_335_123_538_721_756_194,
                start_chunk_x: 11,
                start_chunk_z: -14,
                ordinal: 0,
            }],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert_eq!(
            json,
            r#"{"version":"26.1.2","structure":"ancient_city","seed":0,"placement_candidates":1,"valid_structures":1,"chest_count":1,"chests":[{"x":180,"y":80,"z":-233,"loot_table":"minecraft:chests/bastion_bridge","loot_seed":1335123538721756194,"start_chunk_x":11,"start_chunk_z":-14,"ordinal":0}]}"#
        );
    }

    #[test]
    fn rounds_distance_to_three_decimals() {
        assert_eq!(rounded_distance(207_936), 456.0);
        assert_eq!(rounded_distance(0), 0.0);
    }
}

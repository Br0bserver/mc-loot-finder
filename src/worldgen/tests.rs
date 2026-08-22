use super::*;
use crate::catalog::CANDIDATE_STRUCTURES;
use crate::decoration_seed::container_loot_seed;
use std::collections::HashSet;

fn assert_static_seed_contract(scanner: &Scanner, scans: &[Scan]) {
    let decoration = scanner
        .structure
        .decoration
        .expect("scanner must have static decoration metadata");
    for chest in scans.iter().flat_map(|scan| &scan.chests) {
        let predicted = container_loot_seed(
            scanner.world_seed,
            chest.x.div_euclid(16),
            chest.z.div_euclid(16),
            decoration,
            chest.ordinal,
        )
        .expect("recompute chest seed from catalog metadata");
        assert_eq!(
            predicted, chest.loot_seed,
            "{} seed contract failed for chest {:?}",
            scanner.structure.name, chest
        );
    }
}

#[test]
fn scans_known_26_1_2_cities() {
    let scanner = Scanner::new(114514, ScanKind::AncientCity);
    let scans = scanner
        .scan_many([(96, 5), (244, 171)])
        .expect("scan known cities");
    let first = &scans[0];
    assert!(first.valid_structure);
    assert!(first.chests.iter().any(|chest| {
        chest.x == 1450
            && chest.y == -35
            && chest.z == 137
            && chest.loot_table == "minecraft:chests/ancient_city"
            && chest.loot_seed == 1_392_286_922_750_350_146
            && chest.ordinal == 0
    }));

    let second = &scans[1];
    assert!(second.valid_structure);
    assert!(second.chests.iter().any(|chest| {
        chest.x == 3965
            && chest.y == -37
            && chest.z == 2755
            && chest.loot_table == "minecraft:chests/ancient_city"
            && chest.loot_seed == -5_503_126_436_529_563_106
    }));
    assert_static_seed_contract(&scanner, &scans);
}

#[test]
fn scans_known_26_1_2_bastions() {
    let scanner = Scanner::new(0, ScanKind::BastionRemnant);
    let scans = scanner
        .scan_many([(11, -14), (-27, -10), (62, 32)])
        .expect("scan known bastions");
    assert!(scans.iter().all(|scan| scan.valid_structure));
    assert_eq!(
        scans
            .iter()
            .map(|scan| scan.chests.len())
            .collect::<Vec<_>>(),
        [3, 11, 6]
    );
    assert_eq!(
        scans[0].chests.first(),
        Some(&Chest {
            structure_chunk_x: 11,
            structure_chunk_z: -14,
            x: 180,
            y: 80,
            z: -233,
            loot_table: "minecraft:chests/bastion_bridge".to_owned(),
            ordinal: 0,
            loot_seed: 1_335_123_538_721_756_194,
        })
    );
    assert_eq!(
        scans[1].chests.first(),
        Some(&Chest {
            structure_chunk_x: -27,
            structure_chunk_z: -10,
            x: -428,
            y: 35,
            z: -189,
            loot_table: "minecraft:chests/bastion_other".to_owned(),
            ordinal: 0,
            loot_seed: -5_513_880_696_554_537_352,
        })
    );
    assert_eq!(
        scans[2].chests.first(),
        Some(&Chest {
            structure_chunk_x: 62,
            structure_chunk_z: 32,
            x: 1011,
            y: 35,
            z: 496,
            loot_table: "minecraft:chests/bastion_treasure".to_owned(),
            ordinal: 0,
            loot_seed: -6_403_023_197_147_397_919,
        })
    );

    let tables = scans
        .iter()
        .flat_map(|scan| scan.chests.iter())
        .map(|chest| chest.loot_table.as_str())
        .collect::<HashSet<_>>();
    assert_eq!(
        tables,
        HashSet::from([
            "minecraft:chests/bastion_bridge",
            "minecraft:chests/bastion_hoglin_stable",
            "minecraft:chests/bastion_other",
            "minecraft:chests/bastion_treasure",
        ])
    );
    assert_static_seed_contract(&scanner, &scans);
}

#[test]
fn scans_known_26_1_2_desert_pyramids() {
    let scanner = Scanner::new(0, ScanKind::DesertPyramid);
    // Seed 0: three valid pyramids and six candidates rejected by the
    // vanilla sea-level corner check (some corner below the sea level).
    let scans = scanner
        .scan_many([
            (0, -188),
            (77, -213),
            (81, -254),
            (22, -146),
            (-30, -160),
            (-10, -178),
            (98, -155),
            (102, -224),
            (105, -246),
        ])
        .expect("scan desert pyramid candidates");
    for scan in &scans[3..] {
        assert!(!scan.valid_structure, "candidate must be rejected");
        assert!(scan.chests.is_empty(), "rejected candidate has chests");
    }
    let scans = &scans[..3];
    let expected = [
        (
            "minecraft:chests/desert_pyramid",
            [
                (10, 59, -2996, -5_568_029_752_813_165_272),
                (12, 59, -2998, 8_612_763_612_274_328_067),
                (10, 59, -3000, 410_913_108_922_281_890),
                (8, 59, -2998, -6_529_954_051_122_263_735),
            ],
        ),
        (
            "minecraft:chests/desert_pyramid",
            [
                (1244, 60, -3398, 192_079_748_099_134_926),
                (1242, 60, -3396, -369_207_723_137_014_054),
                (1240, 60, -3398, 1_366_626_509_293_417_282),
                (1242, 60, -3400, 2_864_047_697_517_889_560),
            ],
        ),
        (
            "minecraft:chests/desert_pyramid",
            [
                (1304, 52, -4054, 8_475_396_442_896_426_591),
                (1306, 52, -4052, -164_227_586_464_969_558),
                (1308, 52, -4054, -6_884_729_539_475_924_943),
                (1306, 52, -4056, 5_000_275_533_034_043_386),
            ],
        ),
    ];
    assert_eq!(scans.len(), expected.len());
    for (scan, (loot_table, chests)) in scans.iter().zip(expected) {
        assert!(scan.valid_structure);
        let actual = scan
            .chests
            .iter()
            .map(|chest| {
                (
                    chest.loot_table.as_str(),
                    (chest.x, chest.y, chest.z, chest.loot_seed),
                )
            })
            .collect::<Vec<_>>();
        let wanted = chests
            .iter()
            .map(|(x, y, z, seed)| (loot_table, (*x, *y, *z, *seed)))
            .collect::<Vec<_>>();
        assert_eq!(actual, wanted);
    }
    assert_static_seed_contract(&scanner, &scans);
}

#[test]
fn scans_known_26_1_2_igloos() {
    let scanner = Scanner::new(0, ScanKind::Igloo);
    // Seed 0: three igloos with basements (chest vectors from the vanilla
    // 26.1.2 placement run) and three valid igloos without a basement.
    let scans = scanner
        .scan_many([
            (98, 192),
            (238, -29),
            (-110, 246),
            (-12, 231),
            (-46, 242),
            (-214, 141),
        ])
        .expect("scan igloo candidates");
    let expected_chests = [
        ((1569, 122, 3076), -7_862_992_963_971_781_551),
        ((3813, 48, -458), -3_865_222_752_920_655_871),
        ((-1755, 50, 3942), 1_861_016_387_536_410_190),
    ];
    for (scan, ((x, y, z), seed)) in scans.iter().take(3).zip(expected_chests) {
        assert!(scan.valid_structure);
        assert_eq!(scan.chests.len(), 1);
        let chest = &scan.chests[0];
        assert_eq!((chest.x, chest.y, chest.z), (x, y, z));
        assert_eq!(chest.loot_seed, seed);
        assert_eq!(chest.loot_table, "minecraft:chests/igloo_chest");
        assert_eq!(chest.ordinal, 0);
    }
    for scan in &scans[3..] {
        assert!(scan.valid_structure);
        assert!(
            scan.chests.is_empty(),
            "igloo without basement must have no chests"
        );
    }
    assert_static_seed_contract(&scanner, &scans);
}

#[test]
fn scans_known_26_1_2_snowy_village() {
    // Chunk (-114,290): a snowy village on a ravine edge. The free-space
    // region must use the village's max-distance (80) Y extent so ravine
    // candidates at y24 are rejected; the interior-expansion house
    // snowy_small_house_6 (y72-80) then attaches and its chest appears.
    let scanner = Scanner::new(0, ScanKind::Village);
    let scans = scanner
        .scan_many([(-114i32, 290i32)])
        .expect("scan village");
    assert_eq!(scans.len(), 1);
    assert!(scans[0].valid_structure);
    // Chest order follows the vanilla piece/block scan order.
    let expected_positions = [
        (-1823, 133, 4655),
        (-1803, 131, 4655),
        (-1787, 131, 4681),
        (-1845, 73, 4613),
    ];
    let known_seeds = [
        8_698_000_211_807_766_173,
        -6_504_443_412_966_562_273,
        2_903_910_160_224_306_417,
    ];
    assert_eq!(scans[0].chests.len(), expected_positions.len());
    for (chest, (x, y, z)) in scans[0].chests.iter().zip(expected_positions) {
        assert_eq!((chest.x, chest.y, chest.z), (x, y, z));
        assert_eq!(
            chest.loot_table,
            "minecraft:chests/village/village_snowy_house"
        );
        assert_eq!(chest.ordinal, 0);
    }
    for (chest, seed) in scans[0].chests.iter().zip(known_seeds) {
        assert_eq!(chest.loot_seed, seed);
    }
}

#[test]
fn scans_known_26_1_2_villages() {
    let scanner = Scanner::new(0, ScanKind::Village);
    // Seed 0: a savanna village at (38,45) with five chests and a plains
    // village at (17,59) with two chests (vectors from the vanilla
    // 26.1.2 placement run, including variant indices 23 and 22).
    let scans = scanner
        .scan_many([(38, 45), (17, 59)])
        .expect("scan villages");
    // Chest order follows the vanilla piece/block scan order (not
    // coordinate order): (613,75,715) is the first cartographer chest.
    let expected = [
        vec![
            (613, 75, 715, 7_894_754_405_246_038_683),
            (629, 74, 724, 7_967_509_563_249_290_458),
            (634, 74, 736, 412_383_135_729_192_107),
            (613, 72, 749, -5_741_561_169_428_246_725),
            (566, 72, 715, -8_360_261_121_396_786_299),
        ],
        vec![
            (283, 69, 944, 1_415_882_058_948_058_970),
            (290, 75, 900, -8_102_066_785_630_644_298),
        ],
    ];
    for (scan, chests) in scans.iter().zip(expected) {
        assert!(scan.valid_structure);
        let actual = scan
            .chests
            .iter()
            .map(|chest| (chest.x, chest.y, chest.z, chest.loot_seed))
            .collect::<Vec<_>>();
        assert_eq!(actual, chests);
    }
}

#[test]
fn scans_known_26_1_2_pillager_outpost() {
    // World-level valid pillager for seed 0: chunk (-51,70) passes
    // frequency (legacy_type_1, 0.2) and the 10-chunk village
    // exclusion, then generates one chest. This vector was captured
    // from the Rust scanner with placement filters enabled and matches
    // the Java direct generation for the same chunk (except the
    // placement filters).
    let scanner = Scanner::new(0, ScanKind::PillagerOutpost);
    let scans = scanner.scan_many([(-51, 70)]).expect("scan pillager");
    assert_eq!(scans.len(), 1);
    let scan = &scans[0];
    assert!(scan.valid_structure, "pillager should be valid");
    assert_eq!(scan.chests.len(), 1);
    let chest = &scan.chests[0];
    assert_eq!(chest.loot_table, "minecraft:chests/pillager_outpost");
    assert_eq!(
        (chest.x, chest.y, chest.z, chest.loot_seed, chest.ordinal),
        (-826, 77, 1110, -638836315418230144, 1),
        "pillager chest vector: {:?}",
        chest
    );
    assert_static_seed_contract(&scanner, &scans);
}

#[test]
fn pillager_outpost_36_103_is_filtered_by_frequency() {
    // Java's `VillageAndFortressIntegrationTest` probes pillager at
    // 36,103 via direct `generateSelectedStructure`, bypassing the
    // placement frequency check, and finds one chest at (566,84,1657).
    // For world scans `isStructurePlacementChunk` rejects 36,103
    // (legacy_type_1), so the Rust scanner must report it as invalid.
    let scanner = Scanner::new(0, ScanKind::PillagerOutpost);
    let scans = scanner.scan_many([(36, 103)]).expect("scan pillager");
    assert_eq!(scans.len(), 1);
    assert!(
        !scans[0].valid_structure,
        "36,103 should be invalid for world scan due to frequency"
    );
    assert!(scans[0].chests.is_empty());
}
#[test]
fn catalog_scan_support_matches_scanner_construction() {
    for structure in CANDIDATE_STRUCTURES {
        match structure.support {
            ScanSupport::CandidatesOnly => {
                assert!(!structure.supports_full_scan());
                let Err(error) = Scanner::for_structure(structure, 0) else {
                    panic!("{} unexpectedly constructed a full scanner", structure.name);
                };
                assert!(matches!(error, Error::Structure(_)));
                assert!(error.to_string().contains(structure.name));
            }
            ScanSupport::Full(kind) => {
                assert!(structure.supports_full_scan());
                let scanner = Scanner::for_structure(structure, 0)
                    .expect("full catalog entry must construct a scanner");
                assert_eq!(scanner.kind, kind);
                assert!(structure.decoration.is_some() || kind == ScanKind::Village);
            }
        }
    }
}

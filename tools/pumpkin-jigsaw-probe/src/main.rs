use pumpkin_data::structures::StructureKeys;
use pumpkin_world::generation::structure::structures::jigsaw::{
    JigsawGenerator, PoolElementStructurePiece,
};
use pumpkin_world::generation::structure::structures::{
    StructureGenerator, StructureGeneratorContext, create_chunk_random,
};

fn main() {
    let generator = JigsawGenerator::new("minecraft:ancient_city/city_center", 7)
        .with_start_jigsaw("minecraft:city_anchor");
    let context = StructureGeneratorContext {
        seed: 114_514,
        chunk_x: 244,
        chunk_z: 171,
        random: create_chunk_random(114_514, 244, 171),
        sea_level: 63,
        min_y: -64,
        height_sampler: None,
        structure_key: Some(StructureKeys::AncientCity),
    };
    let position = generator
        .get_structure_position(context)
        .expect("ancient city layout");
    let mut collector = position.collector.lock().expect("collector");
    let bounds = collector.get_bounding_box();
    let first = collector.pieces[0]
        .as_any()
        .downcast_ref::<PoolElementStructurePiece>()
        .expect("first jigsaw piece");
    let mut first_templates = Vec::new();
    first.element.for_each_template(|name, _, _, _| {
        first_templates.push(name.to_owned());
    });

    println!(
        "PUMPKIN start=({}, {}, {}) first_pos=({}, {}, {}) rotation={:?} templates={:?} first_box=({}, {}, {})..({}, {}, {}) pieces={} box=({}, {}, {})..({}, {}, {})",
        position.start_pos.0.x,
        position.start_pos.0.y,
        position.start_pos.0.z,
        first.pos.0.x,
        first.pos.0.y,
        first.pos.0.z,
        first.rotation,
        first_templates,
        first.piece.bounding_box.min.x,
        first.piece.bounding_box.min.y,
        first.piece.bounding_box.min.z,
        first.piece.bounding_box.max.x,
        first.piece.bounding_box.max.y,
        first.piece.bounding_box.max.z,
        collector.pieces.len(),
        bounds.min.x,
        bounds.min.y,
        bounds.min.z,
        bounds.max.x,
        bounds.max.y,
        bounds.max.z,
    );
}


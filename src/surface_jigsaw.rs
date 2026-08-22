//! Vanilla 26.1.2 surface jigsaw placement for villages and pillager outposts.
//!
//! The fork's `JigsawPlacement` cannot place village houses: the
//! `generate_structure_position` dispatch never enables the expansion hack for
//! villages (only pillager outposts), and the interior collision space is
//! bounded by the *unexpanded* piece box, so a house that sits fully inside a
//! street's expansion box is rejected. This module ports the vanilla algorithm
//! exactly:
//!
//! - template pools are expanded by element weight, then shuffled with
//!   Fisher-Yates; this ordering and its RNG consumption match vanilla;
//! - the expansion hack grows the candidate collision box by
//!   `max(childPoolMaxY, childFallbackMaxY) + 1`;
//! - the interior collision space is bounded by the *expanded* source
//!   collision box, so a child fully contained in the source's expansion box
//!   is accepted (vanilla `ONLY_SECOND` on the source shape);
//! - same-priority pieces are processed FIFO (vanilla `SequencedPriorityIterator`);
//! - start-piece jigsaw blocks are ordered by (y, x, z) like vanilla's
//!   block-entity ordering.

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

use pumpkin_data::Mirror;
use pumpkin_data::Rotation;
use pumpkin_util::math::block_box::BlockBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::RandomImpl;
use pumpkin_world::generation::structure::piece::StructurePieceType;
use pumpkin_world::generation::structure::structures::jigsaw::PoolElementStructurePiece;
use pumpkin_world::generation::structure::structures::jigsaw::{
    JigsawBlock, JigsawJointType, JigsawJunction, JigsawProjection, PoolElement, PoolElementKind,
    TemplatePool,
};
use pumpkin_world::generation::structure::structures::{
    StructureGeneratorContext, StructurePiece, StructurePiecesCollector, StructurePosition,
};
use pumpkin_world::generation::structure::template::get_template;

/// Maximum build height above `min_y` used for world-limit checks.
const WORLD_HEIGHT: i32 = 320;

#[derive(Clone, Copy)]
struct PieceState {
    piece_idx: usize,
    depth: i32,
    collision_space: usize,
}

/// Vanilla `SequencedPriorityIterator`: highest priority first, FIFO within a
/// priority level.
#[derive(Default)]
struct PriorityQueue {
    queues: BTreeMap<i32, VecDeque<usize>>,
}

impl PriorityQueue {
    fn new() -> Self {
        Self::default()
    }

    fn add(&mut self, priority: i32, piece_idx: usize) {
        self.queues
            .entry(priority)
            .or_default()
            .push_back(piece_idx);
    }

    fn pop(&mut self) -> Option<usize> {
        loop {
            let mut entry = self.queues.last_entry()?;
            let queue = entry.get_mut();
            let piece_idx = queue.pop_front();
            if queue.is_empty() {
                entry.remove();
            }
            if piece_idx.is_some() {
                return piece_idx;
            }
        }
    }
}

struct CollisionSpace {
    bounds: BlockBox,
    occupied: Vec<BlockBox>,
}

/// Vanilla 26.1.2 `StructureTemplatePool.getShuffledTemplates`: the pool's
/// templates are pre-expanded by weight at load time and
/// `Util.shuffledCopy` runs Fisher-Yates over that expanded list.
fn shuffled_templates(pool: &TemplatePool, random: &mut impl RandomImpl) -> Vec<PoolElement> {
    let mut elements = pool
        .elements
        .iter()
        .flat_map(|element| std::iter::repeat_n(element.clone(), element.weight as usize))
        .collect::<Vec<_>>();
    for index in (1..elements.len()).rev() {
        let other = random.next_bounded_i32(index as i32 + 1) as usize;
        elements.swap(index, other);
    }
    elements
}

fn get_jigsaw_blocks(
    template: &pumpkin_world::generation::structure::template::StructureTemplate,
) -> Vec<JigsawBlock> {
    let mut jigsaws = Vec::new();
    for block in &template.blocks {
        if let Some(jigsaw) =
            JigsawBlock::from_template_block(block, &template.palette[block.state as usize])
        {
            jigsaws.push(jigsaw);
        }
    }
    // Vanilla groups block entities and orders them by Y, X, Z.
    jigsaws.sort_by_key(|jigsaw| (jigsaw.pos.0.y, jigsaw.pos.0.x, jigsaw.pos.0.z));
    jigsaws
}

fn rotate_pos(pos: Vector3<i32>, rotation: Rotation) -> Vector3<i32> {
    let (x, z) = rotation.rotate_offset(pos.x, pos.z);
    Vector3::new(x, pos.y, z)
}

fn rotated_box(origin: BlockPos, size: Vector3<i32>, rotation: Rotation) -> BlockBox {
    let corner = rotate_pos(Vector3::new(size.x - 1, size.y - 1, size.z - 1), rotation);
    BlockBox::new(
        origin.0.x.min(origin.0.x + corner.x),
        origin.0.y,
        origin.0.z.min(origin.0.z + corner.z),
        origin.0.x.max(origin.0.x + corner.x),
        origin.0.y + corner.y,
        origin.0.z.max(origin.0.z + corner.z),
    )
}

fn get_element_size(element: &PoolElement) -> Option<Vector3<i32>> {
    fn size_for_kind(kind: &PoolElementKind) -> Option<Vector3<i32>> {
        match kind {
            PoolElementKind::Empty => None,
            PoolElementKind::Feature(_) => Some(Vector3::new(1, 1, 1)),
            PoolElementKind::Single { template, .. } => get_template(template).map(|t| t.size),
            PoolElementKind::List(elements) => {
                let mut result = Vector3::new(0, 0, 0);
                let mut found = false;
                for size in elements.iter().filter_map(size_for_kind) {
                    result.x = result.x.max(size.x);
                    result.y = result.y.max(size.y);
                    result.z = result.z.max(size.z);
                    found = true;
                }
                found.then_some(result)
            }
        }
    }
    size_for_kind(&element.kind)
}

fn get_element_jigsaw_blocks(element: &PoolElement) -> Vec<JigsawBlock> {
    fn jigsaws_for_kind(kind: &PoolElementKind) -> Vec<JigsawBlock> {
        match kind {
            PoolElementKind::Single { template, .. } => get_template(template)
                .map_or_else(Vec::new, |template| get_jigsaw_blocks(&template)),
            PoolElementKind::List(elements) => {
                elements.first().map_or_else(Vec::new, jigsaws_for_kind)
            }
            PoolElementKind::Feature(_) => vec![JigsawBlock {
                pos: BlockPos::new(0, 0, 0),
                name: "minecraft:bottom".to_string(),
                target: "minecraft:empty".to_string(),
                pool: "minecraft:empty".to_string(),
                final_state: "minecraft:air".to_string(),
                joint: JigsawJointType::Rollable,
                facing: pumpkin_util::BlockDirection::Down,
                up: pumpkin_util::BlockDirection::South,
                selection_priority: 0,
                placement_priority: 0,
            }],
            PoolElementKind::Empty => Vec::new(),
        }
    }
    jigsaws_for_kind(&element.kind)
}

fn can_attach(source: &JigsawBlock, target: &JigsawBlock, target_rotation: Rotation) -> bool {
    if source.target != target.name {
        return false;
    }
    let rotated_target_facing = rotate_direction(target.facing, target_rotation);
    if source.facing.opposite() != rotated_target_facing {
        return false;
    }
    if source.joint == JigsawJointType::Aligned {
        let rotated_target_up = rotate_direction(target.up, target_rotation);
        return source.up == rotated_target_up;
    }
    true
}

const fn rotate_direction(
    dir: pumpkin_util::BlockDirection,
    rotation: Rotation,
) -> pumpkin_util::BlockDirection {
    use pumpkin_util::BlockDirection;
    match rotation {
        Rotation::None => dir,
        Rotation::Clockwise90 => match dir {
            BlockDirection::North => BlockDirection::East,
            BlockDirection::East => BlockDirection::South,
            BlockDirection::South => BlockDirection::West,
            BlockDirection::West => BlockDirection::North,
            _ => dir,
        },
        Rotation::Rotate180 => match dir {
            BlockDirection::North => BlockDirection::South,
            BlockDirection::South => BlockDirection::North,
            BlockDirection::West => BlockDirection::East,
            BlockDirection::East => BlockDirection::West,
            _ => dir,
        },
        Rotation::CounterClockwise90 => match dir {
            BlockDirection::North => BlockDirection::West,
            BlockDirection::West => BlockDirection::South,
            BlockDirection::South => BlockDirection::East,
            BlockDirection::East => BlockDirection::North,
            _ => dir,
        },
    }
}

fn get_pool_max_y_size(pool_id: &str) -> i32 {
    let Some(pool) = TemplatePool::discover(pool_id) else {
        return 0;
    };
    let mut max_y = 0;
    for element in &pool.elements {
        if element.is_empty() {
            continue;
        }
        if let Some(size) = get_element_size(element) {
            max_y = max_y.max(size.y);
        }
    }
    max_y
}

const fn is_box_inside(outer: &BlockBox, inner: &BlockBox) -> bool {
    inner.min.x >= outer.min.x
        && inner.max.x <= outer.max.x
        && inner.min.y >= outer.min.y
        && inner.max.y <= outer.max.y
        && inner.min.z >= outer.min.z
        && inner.max.z <= outer.max.z
}

const fn boxes_intersect(a: &BlockBox, b: &BlockBox) -> bool {
    // Matches vanilla's AABB + 0.25 deflate semantics for integer boxes:
    // adjacent boxes (sharing no block) do not collide, boxes sharing at
    // least one block do.
    a.max.x >= b.min.x
        && a.min.x <= b.max.x
        && a.max.y >= b.min.y
        && a.min.y <= b.max.y
        && a.max.z >= b.min.z
        && a.min.z <= b.max.z
}

#[derive(Clone, Copy)]
struct SourceAttachment {
    piece_idx: usize,
    depth: i32,
    collision_space: usize,
    box_: BlockBox,
    collision_box: BlockBox,
    projection: JigsawProjection,
    rigid: bool,
}

#[derive(Clone, Copy)]
struct TargetAttachment<'a> {
    element: &'a PoolElement,
    size: Vector3<i32>,
    rotation: Rotation,
    jigsaw: &'a JigsawBlock,
    all_jigsaws: &'a [JigsawBlock],
}

struct AttachmentContext<'a, 'world> {
    interior_collision_space: &'a mut Option<usize>,
    collision_spaces: &'a mut Vec<CollisionSpace>,
    generator: &'a mut StructureGeneratorContext<'world>,
    use_expansion_hack: bool,
}

struct AttachmentPlacement {
    target_box: BlockBox,
    target_collision_box: BlockBox,
    target_pos: BlockPos,
    target_projection: JigsawProjection,
    target_rigid: bool,
    delta_y: i32,
    source_jigsaw_base_height: i32,
    source_jigsaw_local_y: i32,
    target_jigsaw_local_y: i32,
    target_box_y: i32,
    collision_space: usize,
}

/// Consumes exactly one bounded random value, matching vanilla's weighted
/// start-element selection.
fn select_start_element(pool: &TemplatePool, random: &mut impl RandomImpl) -> Option<PoolElement> {
    let total_weight: u32 = pool.elements.iter().map(|element| element.weight).sum();
    let mut remaining = random.next_bounded_i32(total_weight as i32) as u32;
    for element in &pool.elements {
        if remaining < element.weight {
            return Some(element.clone());
        }
        remaining -= element.weight;
    }
    None
}

/// Pure geometry helper; does not consume random values.
fn initial_collision_space(
    center_x: i32,
    center_y: i32,
    center_z: i32,
    max_distance_from_center: i32,
    min_y: i32,
    start_box: BlockBox,
) -> CollisionSpace {
    CollisionSpace {
        bounds: BlockBox::new(
            center_x - max_distance_from_center,
            (center_y - max_distance_from_center).max(min_y),
            center_z - max_distance_from_center,
            center_x + max_distance_from_center,
            (center_y + max_distance_from_center + 1).min(min_y + WORLD_HEIGHT),
            center_z + max_distance_from_center,
        ),
        occupied: vec![start_box],
    }
}

/// Shuffles the primary pool when depth permits and always shuffles the
/// fallback pool. These are the only random draws performed by this helper.
fn attachment_candidates(
    target_pool: &TemplatePool,
    depth: i32,
    max_depth: i32,
    random: &mut impl RandomImpl,
) -> Vec<PoolElement> {
    let mut elements = Vec::new();
    if depth < max_depth {
        elements.extend(shuffled_templates(target_pool, random));
    }
    if let Some(fallback_pool) = TemplatePool::discover(&target_pool.fallback) {
        elements.extend(shuffled_templates(&fallback_pool, random));
    }
    elements
}

/// Pure expansion-hack geometry; does not consume random values.
fn expanded_collision_box(
    mut collision_box: BlockBox,
    target_box: BlockBox,
    target_size: Vector3<i32>,
    target_rotation: Rotation,
    target_jigsaws: &[JigsawBlock],
    use_expansion_hack: bool,
) -> BlockBox {
    if !use_expansion_hack || (target_box.max.y - target_box.min.y + 1) > 16 {
        return collision_box;
    }

    let mut expand_to = 0;
    for target_jigsaw in target_jigsaws {
        let facing = rotate_direction(target_jigsaw.facing, target_rotation);
        let rotated_pos = rotate_pos(target_jigsaw.pos.0, target_rotation);
        let rotated_target_pos = rotated_pos.add(&facing.to_vector());
        let unexpanded_box = rotated_box(BlockPos::new(0, 0, 0), target_size, target_rotation);
        if !unexpanded_box.contains(
            rotated_target_pos.x,
            rotated_target_pos.y,
            rotated_target_pos.z,
        ) {
            continue;
        }
        let child_pool_id = &target_jigsaw.pool;
        let child_pool_max_y = get_pool_max_y_size(child_pool_id);
        let child_fallback_max_y = TemplatePool::discover(child_pool_id)
            .map_or(0, |pool| get_pool_max_y_size(&pool.fallback));
        expand_to = expand_to.max(child_pool_max_y).max(child_fallback_max_y);
    }
    if expand_to > 0 {
        let max_y_offset = (expand_to + 1).max(collision_box.max.y - collision_box.min.y);
        collision_box.max.y = collision_box.min.y + max_y_offset;
    }
    collision_box
}

/// Computes and reserves one attachment without consuming random values.
fn try_attach_piece(
    source: SourceAttachment,
    source_jigsaw: &JigsawBlock,
    target: TargetAttachment<'_>,
    context: AttachmentContext<'_, '_>,
) -> Option<AttachmentPlacement> {
    if !can_attach(source_jigsaw, target.jigsaw, target.rotation) {
        return None;
    }

    let target_projection = target.element.projection;
    let target_rigid = target_projection == JigsawProjection::Rigid;
    let source_facing = source_jigsaw.facing;
    let source_jigsaw_pos = source_jigsaw.pos;
    let target_jigsaw_pos = source_jigsaw_pos.add(
        source_facing.to_vector().x,
        source_facing.to_vector().y,
        source_facing.to_vector().z,
    );
    let source_jigsaw_local_y = source_jigsaw_pos.0.y - source.box_.min.y;
    let target_jigsaw_local_pos = rotate_pos(target.jigsaw.pos.0, target.rotation);
    let target_jigsaw_local_y = target_jigsaw_local_pos.y;
    let delta_y = source_jigsaw_local_y - target_jigsaw_local_y + source_facing.to_vector().y;
    let mut source_jigsaw_base_height = i32::MIN;
    let target_box_y = if source.rigid && target_rigid {
        source.box_.min.y + delta_y
    } else {
        source_jigsaw_base_height = context
            .generator
            .height_sampler
            .as_mut()
            .map_or(source_jigsaw_pos.0.y, |sampler| {
                sampler.estimate_height(source_jigsaw_pos.0.x, source_jigsaw_pos.0.z)
            });
        source_jigsaw_base_height - target_jigsaw_local_y
    };
    let raw_target_pos = BlockPos::new(
        target_jigsaw_pos.0.x - target_jigsaw_local_pos.x,
        target_jigsaw_pos.0.y - target_jigsaw_local_pos.y,
        target_jigsaw_pos.0.z - target_jigsaw_local_pos.z,
    );
    let mut target_pos = raw_target_pos;
    target_pos.0.y += target_box_y - raw_target_pos.0.y;
    let target_box = rotated_box(target_pos, target.size, target.rotation);
    let target_collision_box = expanded_collision_box(
        target_box,
        target_box,
        target.size,
        target.rotation,
        target.all_jigsaws,
        context.use_expansion_hack,
    );

    let collision_space = if source.collision_box.contains(
        target_jigsaw_pos.0.x,
        target_jigsaw_pos.0.y,
        target_jigsaw_pos.0.z,
    ) {
        *context.interior_collision_space.get_or_insert_with(|| {
            context.collision_spaces.push(CollisionSpace {
                bounds: source.collision_box,
                occupied: Vec::new(),
            });
            context.collision_spaces.len() - 1
        })
    } else {
        source.collision_space
    };
    let space = &context.collision_spaces[collision_space];
    if !is_box_inside(&space.bounds, &target_collision_box)
        || space
            .occupied
            .iter()
            .any(|box_| boxes_intersect(box_, &target_collision_box))
    {
        return None;
    }
    context.collision_spaces[collision_space]
        .occupied
        .push(target_collision_box);

    Some(AttachmentPlacement {
        target_box,
        target_collision_box,
        target_pos,
        target_projection,
        target_rigid,
        delta_y,
        source_jigsaw_base_height,
        source_jigsaw_local_y,
        target_jigsaw_local_y,
        target_box_y,
        collision_space,
    })
}

#[derive(Clone, Copy)]
pub struct SurfaceJigsawConfig<'a> {
    pub start_pool: &'a str,
    pub size: i32,
    pub start_y: i32,
    pub project_start_to_heightmap: bool,
    pub max_distance_from_center: i32,
    pub use_expansion_hack: bool,
}

/// Generates a surface jigsaw structure with the vanilla 26.1.2 algorithm and
/// collects the resulting pieces into a `StructurePosition`.
pub fn generate_surface_jigsaw_position(
    config: SurfaceJigsawConfig<'_>,
    context: &mut StructureGeneratorContext<'_>,
) -> Option<StructurePosition> {
    let SurfaceJigsawConfig {
        start_pool,
        size,
        start_y,
        project_start_to_heightmap,
        max_distance_from_center,
        use_expansion_hack,
    } = config;
    let max_depth = size.clamp(0, 20);
    let pool = TemplatePool::discover(start_pool)?;
    let rotation = Rotation::from_index(context.random.next_bounded_i32(4) as u8);
    let element = select_start_element(&pool, &mut context.random)?;
    let template = element.first_template()?;

    let position = BlockPos::new(
        context.chunk_x.checked_mul(16)?,
        start_y,
        context.chunk_z.checked_mul(16)?,
    );

    let mut box_ = rotated_box(position, template.size, rotation);
    let center_x = i32::midpoint(box_.max.x, box_.min.x);
    let center_z = i32::midpoint(box_.max.z, box_.min.z);

    let bottom_y = if project_start_to_heightmap {
        context
            .height_sampler
            .as_mut()
            .map_or(position.0.y, |sampler| {
                sampler.estimate_height(center_x, center_z)
            })
    } else {
        position.0.y
    };

    let ground_level_delta = 1;
    let y_offset = bottom_y - (box_.min.y + ground_level_delta);
    box_.move_pos(0, y_offset, 0);
    let mut piece_pos = position;
    piece_pos.0.y += y_offset;

    // Villages use ZERO dimension padding, so vanilla's
    // isStartTooCloseToWorldHeightLimits returns false without checking.

    let center_y = bottom_y;

    let mut jigsaw_blocks = Vec::new();
    for mut jigsaw in get_jigsaw_blocks(&template) {
        let rotated_pos = rotate_pos(jigsaw.pos.0, rotation);
        jigsaw.pos = BlockPos(rotated_pos).add(piece_pos.0.x, piece_pos.0.y, piece_pos.0.z);
        jigsaw.facing = rotate_direction(jigsaw.facing, rotation);
        jigsaw.up = rotate_direction(jigsaw.up, rotation);
        jigsaw_blocks.push(jigsaw);
    }

    let first_piece = PoolElementStructurePiece {
        piece: StructurePiece::new(StructurePieceType::Jigsaw, box_, 0),
        element: element.clone(),
        pos: piece_pos,
        rotation,
        mirror: Mirror::None,
        jigsaw_blocks,
        junctions: Vec::new(),
        ground_level_delta,
        liquid_settings: pumpkin_world::generation::structure::structures::jigsaw_placement::LiquidSettings::ApplyWaterlog,
        projection: element.projection,
    };

    let mut pieces = vec![first_piece];
    let mut piece_collision_boxes = vec![box_];
    let mut piece_projections = vec![element.projection];
    let mut collision_spaces = vec![initial_collision_space(
        center_x,
        center_y,
        center_z,
        max_distance_from_center,
        context.min_y,
        box_,
    )];

    if max_depth > 0 {
        let mut placing = PriorityQueue::new();
        let mut states: Vec<PieceState> = Vec::new();
        states.push(PieceState {
            piece_idx: 0,
            depth: 0,
            collision_space: 0,
        });
        placing.add(0, 0);

        while let Some(state_idx) = placing.pop() {
            let state = states[state_idx];
            let depth = state.depth;

            let source_piece_idx = state.piece_idx;
            let mut source_jigsaws = std::mem::take(&mut pieces[source_piece_idx].jigsaw_blocks);

            for i in (1..source_jigsaws.len()).rev() {
                let j = context.random.next_bounded_i32(i as i32 + 1) as usize;
                source_jigsaws.swap(i, j);
            }
            source_jigsaws.sort_by_key(|j| std::cmp::Reverse(j.selection_priority));

            let source_box = pieces[source_piece_idx].piece.bounding_box;
            let source_collision_box = piece_collision_boxes[source_piece_idx];
            let source_projection = piece_projections[source_piece_idx];
            let source_rigid = source_projection == JigsawProjection::Rigid;
            let source = SourceAttachment {
                piece_idx: source_piece_idx,
                depth,
                collision_space: state.collision_space,
                box_: source_box,
                collision_box: source_collision_box,
                projection: source_projection,
                rigid: source_rigid,
            };

            let mut interior_collision_space = None;

            'jigsaw_loop: for source_jigsaw in &source_jigsaws {
                let raw_pool_id = &source_jigsaw.pool;
                if raw_pool_id == "minecraft:empty" || raw_pool_id.is_empty() {
                    continue;
                }

                let Some(target_pool) = TemplatePool::discover(raw_pool_id) else {
                    continue;
                };

                let target_elements =
                    attachment_candidates(&target_pool, depth, max_depth, &mut context.random);

                for element in target_elements {
                    if element.is_empty() {
                        break;
                    }

                    let mut rotations = [
                        Rotation::None,
                        Rotation::Clockwise90,
                        Rotation::Rotate180,
                        Rotation::CounterClockwise90,
                    ];
                    for i in (1..4).rev() {
                        let j = context.random.next_bounded_i32(i as i32 + 1) as usize;
                        rotations.swap(i, j);
                    }

                    let Some(target_size) = get_element_size(&element) else {
                        continue;
                    };

                    for target_rotation in rotations {
                        let target_jigsaws = get_element_jigsaw_blocks(&element);

                        let mut target_jigsaws_shuffled = target_jigsaws.clone();
                        for i in (1..target_jigsaws_shuffled.len()).rev() {
                            let j = context.random.next_bounded_i32(i as i32 + 1) as usize;
                            target_jigsaws_shuffled.swap(i, j);
                        }
                        target_jigsaws_shuffled
                            .sort_by_key(|jigsaw| std::cmp::Reverse(jigsaw.selection_priority));

                        for target_jigsaw in target_jigsaws_shuffled {
                            let Some(attachment) = try_attach_piece(
                                source,
                                source_jigsaw,
                                TargetAttachment {
                                    element: &element,
                                    size: target_size,
                                    rotation: target_rotation,
                                    jigsaw: &target_jigsaw,
                                    all_jigsaws: &target_jigsaws,
                                },
                                AttachmentContext {
                                    interior_collision_space: &mut interior_collision_space,
                                    collision_spaces: &mut collision_spaces,
                                    generator: context,
                                    use_expansion_hack,
                                },
                            ) else {
                                continue;
                            };

                            let source_jigsaw_pos = source_jigsaw.pos;
                            let source_facing = source_jigsaw.facing;
                            let target_jigsaw_pos = source_jigsaw_pos.add(
                                source_facing.to_vector().x,
                                source_facing.to_vector().y,
                                source_facing.to_vector().z,
                            );
                            let mut child_jigsaw_blocks = Vec::new();
                            for mut child_jigsaw in get_element_jigsaw_blocks(&element) {
                                let rotated_pos = rotate_pos(child_jigsaw.pos.0, target_rotation);
                                child_jigsaw.pos = BlockPos(rotated_pos).add(
                                    attachment.target_pos.0.x,
                                    attachment.target_pos.0.y,
                                    attachment.target_pos.0.z,
                                );
                                child_jigsaw.facing =
                                    rotate_direction(child_jigsaw.facing, target_rotation);
                                child_jigsaw.up =
                                    rotate_direction(child_jigsaw.up, target_rotation);
                                child_jigsaw_blocks.push(child_jigsaw);
                            }

                            let source_ground_level_delta =
                                pieces[source.piece_idx].ground_level_delta;
                            let target_ground_level_delta = if attachment.target_rigid {
                                source_ground_level_delta - attachment.delta_y
                            } else {
                                1
                            };
                            let target_piece = PoolElementStructurePiece {
                                piece: StructurePiece::new(
                                    StructurePieceType::Jigsaw,
                                    attachment.target_box,
                                    source.depth as u32 + 1,
                                ),
                                element: element.clone(),
                                pos: attachment.target_pos,
                                rotation: target_rotation,
                                mirror: Mirror::None,
                                jigsaw_blocks: child_jigsaw_blocks,
                                junctions: Vec::new(),
                                ground_level_delta: target_ground_level_delta,
                                liquid_settings:
                                    pumpkin_world::generation::structure::structures::jigsaw_placement::LiquidSettings::ApplyWaterlog,
                                projection: attachment.target_projection,
                            };

                            let target_piece_idx = pieces.len();
                            pieces.push(target_piece);
                            piece_collision_boxes.push(attachment.target_collision_box);
                            piece_projections.push(attachment.target_projection);

                            let junction_y = if source.rigid {
                                source.box_.min.y + attachment.delta_y
                            } else if attachment.target_rigid {
                                attachment.target_box_y + attachment.target_jigsaw_local_y
                            } else {
                                attachment.source_jigsaw_base_height + attachment.delta_y / 2
                            };
                            pieces[source.piece_idx].add_junction(JigsawJunction {
                                source_x: target_jigsaw_pos.0.x,
                                source_ground_y: junction_y - attachment.source_jigsaw_local_y
                                    + source_ground_level_delta,
                                source_z: target_jigsaw_pos.0.z,
                                delta_y: attachment.delta_y,
                                projection: attachment.target_projection,
                            });
                            pieces[target_piece_idx].add_junction(JigsawJunction {
                                source_x: source_jigsaw_pos.0.x,
                                source_ground_y: junction_y - attachment.target_jigsaw_local_y
                                    + target_ground_level_delta,
                                source_z: source_jigsaw_pos.0.z,
                                delta_y: -attachment.delta_y,
                                projection: source.projection,
                            });

                            if source.depth < max_depth {
                                let child_state_idx = states.len();
                                states.push(PieceState {
                                    piece_idx: target_piece_idx,
                                    depth: source.depth + 1,
                                    collision_space: attachment.collision_space,
                                });
                                placing.add(source_jigsaw.placement_priority, child_state_idx);
                            }

                            continue 'jigsaw_loop;
                        }
                    }
                }
            }

            pieces[source_piece_idx].jigsaw_blocks = source_jigsaws;
        }
    }

    let mut collector = StructurePiecesCollector::new();
    for piece in pieces {
        collector.add_piece(Box::new(piece));
    }

    Some(StructurePosition {
        start_pos: BlockPos::new(center_x, center_y, center_z),
        collector: Arc::new(Mutex::new(collector)),
    })
}

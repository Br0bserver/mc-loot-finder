//! Vanilla 26.1.2 jigsaw placement for villages, implemented in this crate.
//!
//! The fork's `JigsawPlacement` cannot place village houses: the
//! `generate_structure_position` dispatch never enables the expansion hack for
//! villages (only pillager outposts), and the interior collision space is
//! bounded by the *unexpanded* piece box, so a house that sits fully inside a
//! street's expansion box is rejected. This module ports the vanilla algorithm
//! exactly:
//!
//! - pool shuffling uses vanilla `WeightedPicker.shuffle` semantics: one entry
//!   per element, weights NOT expanded (the fork expands by weight, which
//!   consumes a different random stream);
//! - the expansion hack (`use_expansion_hack` is true for villages) grows the
//!   candidate collision box by `max(childPoolMaxY, childFallbackMaxY) + 1`;
//! - the interior collision space is bounded by the *expanded* source
//!   collision box, so a child fully contained in the source's expansion box
//!   is accepted (vanilla `ONLY_SECOND` on the source shape);
//! - same-priority pieces are processed FIFO (vanilla `SequencedPriorityIterator`);
//! - start-piece jigsaw blocks are ordered by (y, x, z) like vanilla's
//!   block-entity ordering.

use std::collections::HashMap;
use std::collections::VecDeque;
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
    StructureGeneratorContext, StructurePiece, StructurePieceBase, StructurePiecesCollector,
    StructurePosition,
};
use pumpkin_world::generation::structure::template::get_template;

/// Maximum build height above `min_y` used for world-limit checks.
const WORLD_HEIGHT: i32 = 320;

#[derive(Clone, Copy)]
struct PieceState {
    piece_idx: usize,
    depth: i32,
    priority: i32,
    collision_space: usize,
}

/// Vanilla `SequencedPriorityIterator`: highest priority first, FIFO within a
/// priority level.
struct PriorityQueue {
    queues: HashMap<i32, VecDeque<usize>>,
    highest: i32,
}

impl PriorityQueue {
    fn new() -> Self {
        Self {
            queues: HashMap::new(),
            highest: i32::MIN,
        }
    }

    fn add(&mut self, priority: i32, piece_idx: usize) {
        if priority == self.highest
            && self
                .queues
                .get(&priority)
                .is_some_and(|queue| !queue.is_empty())
        {
            self.queues
                .get_mut(&priority)
                .expect("queue exists")
                .push_back(piece_idx);
            return;
        }
        let queue = self.queues.entry(priority).or_default();
        queue.push_back(piece_idx);
        if priority >= self.highest {
            self.highest = priority;
        }
    }

    fn pop(&mut self) -> Option<usize> {
        loop {
            let queue = self.queues.get_mut(&self.highest)?;
            if let Some(piece_idx) = queue.pop_front() {
                if queue.is_empty() {
                    self.highest = self
                        .queues
                        .iter()
                        .filter(|(_, q)| !q.is_empty())
                        .map(|(priority, _)| *priority)
                        .max()
                        .unwrap_or(i32::MIN);
                }
                return Some(piece_idx);
            }
            self.highest = self
                .queues
                .iter()
                .filter(|(_, q)| !q.is_empty())
                .map(|(priority, _)| *priority)
                .max()
                .unwrap_or(i32::MIN);
        }
    }

    fn is_empty(&self) -> bool {
        self.queues.values().all(VecDeque::is_empty)
    }
}

struct CollisionSpace {
    bounds: BlockBox,
    occupied: Vec<BlockBox>,
}

/// Vanilla `WeightedPicker.shuffle`: Fisher-Yates over the element list
/// (one entry per element, weights untouched).
fn shuffled_templates(pool: &TemplatePool, random: &mut impl RandomImpl) -> Vec<PoolElement> {
    let mut elements = pool.elements.clone();
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

/// Generates a village with the vanilla 26.1.2 jigsaw algorithm and collects
/// the resulting pieces into a `StructurePosition`.
#[allow(clippy::too_many_arguments)]
pub fn generate_village_position(
    start_pool: &str,
    size: i32,
    start_y: i32,
    project_start_to_heightmap: bool,
    max_distance_from_center: i32,
    context: &mut StructureGeneratorContext<'_>,
) -> Option<StructurePosition> {
    let max_depth = size.clamp(0, 20);
    let pool = TemplatePool::discover(start_pool)?;
    let rotation = Rotation::from_index(context.random.next_bounded_i32(4) as u8);
    let element = pool.get_random_element(&mut context.random).clone();
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

    if box_.min.y < context.min_y || box_.max.y > context.min_y + WORLD_HEIGHT {
        return None;
    }

    let center_y = bottom_y;
    let global_bounding_box = BlockBox::new(
        center_x - max_distance_from_center,
        (center_y - 384).max(context.min_y),
        center_z - max_distance_from_center,
        center_x + max_distance_from_center,
        (center_y + 384 + 1).min(context.min_y + WORLD_HEIGHT),
        center_z + max_distance_from_center,
    );

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
    let mut collision_spaces = vec![CollisionSpace {
        bounds: global_bounding_box,
        occupied: vec![box_],
    }];

    if max_depth > 0 {
        let mut placing = PriorityQueue::new();
        let mut states: Vec<PieceState> = Vec::new();
        placing.add(0, 0);
        states.push(PieceState {
            piece_idx: 0,
            depth: 0,
            priority: 0,
            collision_space: 0,
        });

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
            let mut interior_collision_space = None;

            'jigsaw_loop: for source_jigsaw in &source_jigsaws {
                let raw_pool_id = &source_jigsaw.pool;
                if raw_pool_id == "minecraft:empty" || raw_pool_id.is_empty() {
                    continue;
                }

                let Some(target_pool) = TemplatePool::discover(raw_pool_id) else {
                    continue;
                };

                let mut target_elements = Vec::new();
                if depth < max_depth {
                    target_elements.extend(shuffled_templates(&target_pool, &mut context.random));
                }

                let fallback_pool_id = target_pool.fallback.clone();
                if let Some(fallback_pool) = TemplatePool::discover(&fallback_pool_id) {
                    target_elements.extend(shuffled_templates(&fallback_pool, &mut context.random));
                }

                for element in target_elements {
                    if element.is_empty() {
                        break;
                    }

                    let target_projection = element.projection;
                    let target_rigid = target_projection == JigsawProjection::Rigid;

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
                            if !can_attach(source_jigsaw, &target_jigsaw, target_rotation) {
                                continue;
                            }

                            let source_facing = source_jigsaw.facing;
                            let source_jigsaw_pos = source_jigsaw.pos;
                            let target_jigsaw_pos = source_jigsaw_pos.add(
                                source_facing.to_vector().x,
                                source_facing.to_vector().y,
                                source_facing.to_vector().z,
                            );

                            let source_jigsaw_local_y = source_jigsaw_pos.0.y - source_box.min.y;
                            let target_jigsaw_local_pos =
                                rotate_pos(target_jigsaw.pos.0, target_rotation);
                            let target_jigsaw_local_y = target_jigsaw_local_pos.y;

                            let delta_y = source_jigsaw_local_y - target_jigsaw_local_y
                                + source_facing.to_vector().y;

                            let mut source_jigsaw_base_height = i32::MIN;

                            let target_box_y = if source_rigid && target_rigid {
                                source_box.min.y + delta_y
                            } else {
                                if source_jigsaw_base_height == i32::MIN {
                                    source_jigsaw_base_height = context
                                        .height_sampler
                                        .as_mut()
                                        .map_or(source_jigsaw_pos.0.y, |sampler| {
                                            sampler.estimate_height(
                                                source_jigsaw_pos.0.x,
                                                source_jigsaw_pos.0.z,
                                            )
                                        });
                                }
                                source_jigsaw_base_height - target_jigsaw_local_y
                            };

                            let raw_target_pos = BlockPos::new(
                                target_jigsaw_pos.0.x - target_jigsaw_local_pos.x,
                                target_jigsaw_pos.0.y - target_jigsaw_local_pos.y,
                                target_jigsaw_pos.0.z - target_jigsaw_local_pos.z,
                            );
                            let y_offset = target_box_y - raw_target_pos.0.y;
                            let mut target_pos = raw_target_pos;
                            target_pos.0.y += y_offset;

                            let target_box = rotated_box(target_pos, target_size, target_rotation);
                            let mut target_collision_box = target_box;

                            let mut expand_to = 0;
                            if (target_box.max.y - target_box.min.y + 1) <= 16 {
                                for tj in &target_jigsaws {
                                    let tj_facing = rotate_direction(tj.facing, target_rotation);
                                    let rotated_tj_pos = rotate_pos(tj.pos.0, target_rotation);
                                    let rotated_tj_target_pos =
                                        rotated_tj_pos.add(&tj_facing.to_vector());

                                    let hack_box = rotated_box(
                                        BlockPos::new(0, 0, 0),
                                        target_size,
                                        target_rotation,
                                    );
                                    if hack_box.contains(
                                        rotated_tj_target_pos.x,
                                        rotated_tj_target_pos.y,
                                        rotated_tj_target_pos.z,
                                    ) {
                                        let child_pool_id = &tj.pool;
                                        let child_pool_max_y = get_pool_max_y_size(child_pool_id);
                                        let child_fallback_max_y =
                                            TemplatePool::discover(child_pool_id)
                                                .map_or(0, |cp| get_pool_max_y_size(&cp.fallback));
                                        expand_to = expand_to
                                            .max(child_pool_max_y)
                                            .max(child_fallback_max_y);
                                    }
                                }
                            }

                            if expand_to > 0 {
                                let max_y_offset = (expand_to + 1)
                                    .max(target_collision_box.max.y - target_collision_box.min.y);
                                target_collision_box.max.y =
                                    target_collision_box.min.y + max_y_offset;
                            }

                            let collision_space = if source_box.contains(
                                target_jigsaw_pos.0.x,
                                target_jigsaw_pos.0.y,
                                target_jigsaw_pos.0.z,
                            ) {
                                *interior_collision_space.get_or_insert_with(|| {
                                    collision_spaces.push(CollisionSpace {
                                        bounds: source_collision_box,
                                        occupied: Vec::new(),
                                    });
                                    collision_spaces.len() - 1
                                })
                            } else {
                                state.collision_space
                            };
                            let space = &collision_spaces[collision_space];
                            let can_place = is_box_inside(&space.bounds, &target_collision_box)
                                && !space
                                    .occupied
                                    .iter()
                                    .any(|box_| boxes_intersect(box_, &target_collision_box));

                            if can_place {
                                collision_spaces[collision_space]
                                    .occupied
                                    .push(target_collision_box);
                                let mut child_jigsaw_blocks = Vec::new();
                                for mut cj in get_element_jigsaw_blocks(&element) {
                                    let rotated_pos = rotate_pos(cj.pos.0, target_rotation);
                                    cj.pos = BlockPos(rotated_pos).add(
                                        target_pos.0.x,
                                        target_pos.0.y,
                                        target_pos.0.z,
                                    );
                                    cj.facing = rotate_direction(cj.facing, target_rotation);
                                    cj.up = rotate_direction(cj.up, target_rotation);
                                    child_jigsaw_blocks.push(cj);
                                }

                                let source_ground_level_delta =
                                    pieces[source_piece_idx].ground_level_delta;
                                let target_ground_level_delta = if target_rigid {
                                    source_ground_level_delta - delta_y
                                } else {
                                    1
                                };

                                let target_piece = PoolElementStructurePiece {
                                    piece: StructurePiece::new(
                                        StructurePieceType::Jigsaw,
                                        target_box,
                                        depth as u32 + 1,
                                    ),
                                    element: element.clone(),
                                    pos: target_pos,
                                    rotation: target_rotation,
                                    mirror: Mirror::None,
                                    jigsaw_blocks: child_jigsaw_blocks,
                                    junctions: Vec::new(),
                                    ground_level_delta: target_ground_level_delta,
                                    liquid_settings:
                                        pumpkin_world::generation::structure::structures::jigsaw_placement::LiquidSettings::ApplyWaterlog,
                                    projection: target_projection,
                                };

                                let target_piece_idx = pieces.len();
                                pieces.push(target_piece);
                                piece_collision_boxes.push(target_collision_box);
                                piece_projections.push(target_projection);

                                let junction_y = if source_rigid {
                                    source_box.min.y + delta_y
                                } else if target_rigid {
                                    target_box_y + target_jigsaw_local_y
                                } else {
                                    if source_jigsaw_base_height == i32::MIN {
                                        source_jigsaw_base_height = context
                                            .height_sampler
                                            .as_mut()
                                            .map_or(source_jigsaw_pos.0.y, |sampler| {
                                                sampler.estimate_height(
                                                    source_jigsaw_pos.0.x,
                                                    source_jigsaw_pos.0.z,
                                                )
                                            });
                                    }
                                    source_jigsaw_base_height + delta_y / 2
                                };

                                pieces[source_piece_idx].add_junction(JigsawJunction {
                                    source_x: target_jigsaw_pos.0.x,
                                    source_ground_y: junction_y - source_jigsaw_local_y
                                        + source_ground_level_delta,
                                    source_z: target_jigsaw_pos.0.z,
                                    delta_y,
                                    projection: target_projection,
                                });
                                pieces[target_piece_idx].add_junction(JigsawJunction {
                                    source_x: source_jigsaw_pos.0.x,
                                    source_ground_y: junction_y - target_jigsaw_local_y
                                        + target_ground_level_delta,
                                    source_z: source_jigsaw_pos.0.z,
                                    delta_y: -delta_y,
                                    projection: source_projection,
                                });

                                if depth < max_depth {
                                    placing.add(source_jigsaw.placement_priority, target_piece_idx);
                                    states.push(PieceState {
                                        piece_idx: target_piece_idx,
                                        depth: depth + 1,
                                        priority: source_jigsaw.placement_priority,
                                        collision_space,
                                    });
                                }

                                continue 'jigsaw_loop;
                            }
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
        start_pos: piece_pos,
        collector: Arc::new(Mutex::new(collector)),
    })
}

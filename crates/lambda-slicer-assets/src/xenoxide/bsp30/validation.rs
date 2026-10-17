use std::ops::Range;

use super::{
    LUMP_COUNT, LumpKind, ParseError, SourceLocation, read_i16, read_i32, read_u16, record_stride,
};

pub(super) fn validate_topology(
    bytes: &[u8],
    lumps: &[Range<usize>; LUMP_COUNT],
    texture_count: usize,
) -> Result<(), ParseError> {
    let mut counts = std::array::from_fn(|index| {
        let lump = LumpKind::ALL[index];
        record_stride(lump)
            .map(|stride| lumps[index].len() / stride)
            .unwrap_or(0)
    });
    counts[LumpKind::Textures.index()] = texture_count;

    validate_planes(bytes, lumps)?;
    validate_texture_info(bytes, lumps, &counts)?;
    validate_edges(bytes, lumps, &counts)?;
    validate_surface_edges(bytes, lumps, &counts)?;
    validate_mark_surfaces(bytes, lumps, &counts)?;
    validate_faces(bytes, lumps, &counts)?;
    validate_nodes(bytes, lumps, &counts)?;
    validate_clip_nodes(bytes, lumps, &counts)?;
    validate_leaves(bytes, lumps, &counts)?;
    validate_models(bytes, lumps, &counts)?;
    validate_visibility(bytes, lumps, counts[LumpKind::Leaves.index()])?;
    validate_cycles(bytes, lumps, LumpKind::Nodes, 24, 4)?;
    validate_cycles(bytes, lumps, LumpKind::ClipNodes, 8, 4)?;
    Ok(())
}

fn validate_planes(bytes: &[u8], lumps: &[Range<usize>; LUMP_COUNT]) -> Result<(), ParseError> {
    for (element, record) in records(bytes, lumps, LumpKind::Planes, 20).enumerate() {
        let plane_type = read_i32(record, 16) as i64;
        if !(0..=5).contains(&plane_type) {
            return Err(ParseError::InvalidValue {
                source: location(LumpKind::Planes, element),
                field: "plane type",
                value: plane_type,
            });
        }
    }
    Ok(())
}

fn validate_edges(
    bytes: &[u8],
    lumps: &[Range<usize>; LUMP_COUNT],
    counts: &[usize; LUMP_COUNT],
) -> Result<(), ParseError> {
    for (element, record) in records(bytes, lumps, LumpKind::Edges, 4).enumerate() {
        check_index(
            LumpKind::Edges,
            element,
            "vertex 0",
            read_u16(record, 0) as i64,
            LumpKind::Vertices,
            counts,
        )?;
        check_index(
            LumpKind::Edges,
            element,
            "vertex 1",
            read_u16(record, 2) as i64,
            LumpKind::Vertices,
            counts,
        )?;
    }
    Ok(())
}

fn validate_surface_edges(
    bytes: &[u8],
    lumps: &[Range<usize>; LUMP_COUNT],
    counts: &[usize; LUMP_COUNT],
) -> Result<(), ParseError> {
    for (element, record) in records(bytes, lumps, LumpKind::SurfaceEdges, 4).enumerate() {
        let value = read_i32(record, 0);
        let edge = value.checked_abs().map(i64::from).unwrap_or(i64::MAX);
        check_index(
            LumpKind::SurfaceEdges,
            element,
            "edge",
            edge,
            LumpKind::Edges,
            counts,
        )?;
    }
    Ok(())
}

fn validate_mark_surfaces(
    bytes: &[u8],
    lumps: &[Range<usize>; LUMP_COUNT],
    counts: &[usize; LUMP_COUNT],
) -> Result<(), ParseError> {
    for (element, record) in records(bytes, lumps, LumpKind::MarkSurfaces, 2).enumerate() {
        check_index(
            LumpKind::MarkSurfaces,
            element,
            "face",
            read_u16(record, 0) as i64,
            LumpKind::Faces,
            counts,
        )?;
    }
    Ok(())
}

fn validate_faces(
    bytes: &[u8],
    lumps: &[Range<usize>; LUMP_COUNT],
    counts: &[usize; LUMP_COUNT],
) -> Result<(), ParseError> {
    for (element, record) in records(bytes, lumps, LumpKind::Faces, 20).enumerate() {
        check_index(
            LumpKind::Faces,
            element,
            "plane",
            read_u16(record, 0) as i64,
            LumpKind::Planes,
            counts,
        )?;
        let side = read_i16(record, 2) as i64;
        if !(0..=1).contains(&side) {
            return Err(ParseError::InvalidValue {
                source: location(LumpKind::Faces, element),
                field: "plane side",
                value: side,
            });
        }
        check_index(
            LumpKind::Faces,
            element,
            "texture info",
            read_i16(record, 10) as i64,
            LumpKind::TextureInfo,
            counts,
        )?;
        check_range(
            LumpKind::Faces,
            element,
            "surface edge",
            read_i32(record, 4) as i64,
            read_i16(record, 8) as i64,
            counts[LumpKind::SurfaceEdges.index()],
        )?;
        let lighting = read_i32(record, 16) as i64;
        let lighting_length = lumps[LumpKind::Lighting.index()].len();
        if lighting != -1
            && (lighting < 0 || lighting as usize >= lighting_length || lighting % 3 != 0)
        {
            return Err(ParseError::InvalidRecordRange {
                source: location(LumpKind::Faces, element),
                field: "lighting",
                first: lighting,
                count: 1,
                available: lighting_length,
            });
        }
    }
    Ok(())
}

fn validate_nodes(
    bytes: &[u8],
    lumps: &[Range<usize>; LUMP_COUNT],
    counts: &[usize; LUMP_COUNT],
) -> Result<(), ParseError> {
    for (element, record) in records(bytes, lumps, LumpKind::Nodes, 24).enumerate() {
        check_index(
            LumpKind::Nodes,
            element,
            "plane",
            read_i32(record, 0) as i64,
            LumpKind::Planes,
            counts,
        )?;
        for (field, offset) in [("child 0", 4), ("child 1", 6)] {
            let child = read_i16(record, offset) as i64;
            if child >= 0 {
                check_index(
                    LumpKind::Nodes,
                    element,
                    field,
                    child,
                    LumpKind::Nodes,
                    counts,
                )?;
            } else {
                check_index(
                    LumpKind::Nodes,
                    element,
                    field,
                    !child,
                    LumpKind::Leaves,
                    counts,
                )?;
            }
        }
        check_range(
            LumpKind::Nodes,
            element,
            "face",
            read_u16(record, 20) as i64,
            read_u16(record, 22) as i64,
            counts[LumpKind::Faces.index()],
        )?;
    }
    Ok(())
}

fn validate_clip_nodes(
    bytes: &[u8],
    lumps: &[Range<usize>; LUMP_COUNT],
    counts: &[usize; LUMP_COUNT],
) -> Result<(), ParseError> {
    for (element, record) in records(bytes, lumps, LumpKind::ClipNodes, 8).enumerate() {
        check_index(
            LumpKind::ClipNodes,
            element,
            "plane",
            read_i32(record, 0) as i64,
            LumpKind::Planes,
            counts,
        )?;
        for (field, offset) in [("child 0", 4), ("child 1", 6)] {
            let child = read_i16(record, offset) as i64;
            if child >= 0 {
                check_index(
                    LumpKind::ClipNodes,
                    element,
                    field,
                    child,
                    LumpKind::ClipNodes,
                    counts,
                )?;
            }
        }
    }
    Ok(())
}

fn validate_leaves(
    bytes: &[u8],
    lumps: &[Range<usize>; LUMP_COUNT],
    counts: &[usize; LUMP_COUNT],
) -> Result<(), ParseError> {
    let visibility_length = lumps[LumpKind::Visibility.index()].len();
    for (element, record) in records(bytes, lumps, LumpKind::Leaves, 28).enumerate() {
        let visibility = read_i32(record, 4) as i64;
        if visibility != -1 && (visibility < 0 || visibility as usize >= visibility_length) {
            return Err(ParseError::InvalidRecordRange {
                source: location(LumpKind::Leaves, element),
                field: "visibility",
                first: visibility,
                count: 1,
                available: visibility_length,
            });
        }
        check_range(
            LumpKind::Leaves,
            element,
            "mark surface",
            read_u16(record, 20) as i64,
            read_u16(record, 22) as i64,
            counts[LumpKind::MarkSurfaces.index()],
        )?;
    }
    Ok(())
}

fn validate_visibility(
    bytes: &[u8],
    lumps: &[Range<usize>; LUMP_COUNT],
    leaf_count: usize,
) -> Result<(), ParseError> {
    let visibility = &bytes[lumps[LumpKind::Visibility.index()].clone()];
    let visibility_leaf_count = records(bytes, lumps, LumpKind::Models, 64)
        .next()
        .map(|model| read_i32(model, 52) as usize)
        .unwrap_or_else(|| leaf_count.saturating_sub(1));
    let row_length = visibility_leaf_count.saturating_add(7) >> 3;
    for (leaf, record) in records(bytes, lumps, LumpKind::Leaves, 28).enumerate() {
        let raw_offset = read_i32(record, 4);
        if raw_offset == -1 {
            continue;
        }
        let offset = raw_offset as usize;
        let mut position = offset;
        let mut written = 0_usize;
        while written < row_length {
            let Some(&value) = visibility.get(position) else {
                return Err(ParseError::InvalidVisibility { leaf, offset });
            };
            position += 1;
            if value != 0 {
                written += 1;
                continue;
            }
            let Some(&run) = visibility.get(position) else {
                return Err(ParseError::InvalidVisibility { leaf, offset });
            };
            position += 1;
            let run_end = written.saturating_add(run as usize);
            if run == 0 {
                return Err(ParseError::InvalidVisibility { leaf, offset });
            }
            written = run_end;
        }
    }
    Ok(())
}

fn validate_models(
    bytes: &[u8],
    lumps: &[Range<usize>; LUMP_COUNT],
    counts: &[usize; LUMP_COUNT],
) -> Result<(), ParseError> {
    for (element, record) in records(bytes, lumps, LumpKind::Models, 64).enumerate() {
        let visibility_leaves = read_i32(record, 52) as i64;
        let available = counts[LumpKind::Leaves.index()].saturating_sub(1);
        if visibility_leaves < 0 || visibility_leaves as usize > available {
            return Err(ParseError::InvalidRecordRange {
                source: location(LumpKind::Models, element),
                field: "visibility leaf count",
                first: 0,
                count: visibility_leaves,
                available,
            });
        }
        check_index(
            LumpKind::Models,
            element,
            "render head node",
            read_i32(record, 36) as i64,
            LumpKind::Nodes,
            counts,
        )?;
        for (hull, offset) in [40, 44, 48].into_iter().enumerate() {
            let head = read_i32(record, offset) as i64;
            if head >= 0 {
                check_index(
                    LumpKind::Models,
                    element,
                    match hull {
                        0 => "hull 1 head node",
                        1 => "hull 2 head node",
                        _ => "hull 3 head node",
                    },
                    head,
                    LumpKind::ClipNodes,
                    counts,
                )?;
            }
        }
        check_range(
            LumpKind::Models,
            element,
            "face",
            read_i32(record, 56) as i64,
            read_i32(record, 60) as i64,
            counts[LumpKind::Faces.index()],
        )?;
    }
    Ok(())
}

fn validate_cycles(
    bytes: &[u8],
    lumps: &[Range<usize>; LUMP_COUNT],
    lump: LumpKind,
    stride: usize,
    children_offset: usize,
) -> Result<(), ParseError> {
    let records: Vec<_> = records(bytes, lumps, lump, stride).collect();
    let mut state = vec![0_u8; records.len()];
    for root in 0..records.len() {
        let mut pending = vec![(root, false)];
        while let Some((element, leaving)) = pending.pop() {
            if leaving {
                state[element] = 2;
                continue;
            }
            match state[element] {
                1 => return Err(ParseError::TopologyCycle { lump, element }),
                2 => continue,
                _ => {}
            }
            state[element] = 1;
            pending.push((element, true));
            for offset in [children_offset, children_offset + 2] {
                let child = read_i16(records[element], offset);
                if child >= 0 {
                    pending.push((child as usize, false));
                }
            }
        }
    }
    Ok(())
}

fn records<'a>(
    bytes: &'a [u8],
    lumps: &[Range<usize>; LUMP_COUNT],
    lump: LumpKind,
    stride: usize,
) -> impl Iterator<Item = &'a [u8]> {
    bytes[lumps[lump.index()].clone()].chunks_exact(stride)
}

fn check_index(
    source_lump: LumpKind,
    element: usize,
    field: &'static str,
    value: i64,
    target: LumpKind,
    counts: &[usize; LUMP_COUNT],
) -> Result<(), ParseError> {
    let target_count = counts[target.index()];
    if value < 0 || value as usize >= target_count {
        return Err(ParseError::InvalidReference {
            source: location(source_lump, element),
            field,
            value,
            target,
            target_count,
        });
    }
    Ok(())
}

fn check_range(
    source_lump: LumpKind,
    element: usize,
    field: &'static str,
    first: i64,
    count: i64,
    available: usize,
) -> Result<(), ParseError> {
    let valid = first >= 0
        && count >= 0
        && (first as usize)
            .checked_add(count as usize)
            .map(|end| end <= available)
            .unwrap_or(false);
    if !valid {
        return Err(ParseError::InvalidRecordRange {
            source: location(source_lump, element),
            field,
            first,
            count,
            available,
        });
    }
    Ok(())
}

fn location(lump: LumpKind, element: usize) -> SourceLocation {
    SourceLocation { lump, element }
}

fn validate_texture_info(
    bytes: &[u8],
    lumps: &[Range<usize>; LUMP_COUNT],
    counts: &[usize; LUMP_COUNT],
) -> Result<(), ParseError> {
    for (element, record) in records(bytes, lumps, LumpKind::TextureInfo, 40).enumerate() {
        check_index(
            LumpKind::TextureInfo,
            element,
            "mip texture",
            read_i32(record, 32) as i64,
            LumpKind::Textures,
            counts,
        )?;
    }
    Ok(())
}

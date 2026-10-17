//! Classic Half-Life studio models (MDL version 10), read into validated, owned data.
//!
//! Format reference, pinned: Valve's Half-Life SDK at commit
//! `b1b5cf5892918535619b2937bb927e46cb097ba1`: `engine/studio.h` for the record layouts
//! and limits, `utils/studiomdl/studiomdl.h` for the version and the `IDST`/`IDSQ`
//! identifiers, and `cl_dll/StudioModelRenderer.cpp` for how animation tracks are read.
//! No SDK code is included; the layouts are restated here and in
//! `STUDIO_MODEL_FORMAT.md`.
//!
//! This reads what poses and hit queries need: bones, bone controllers, hitboxes and
//! their hit groups, sequence groups, sequences with their events, and every sequence's
//! animation tracks, decoded to a value per frame with the run boundaries the renderer's
//! translation and rotation readings depend on. Meshes, textures, skins, body parts,
//! attachments and the transition table are not read yet.
//!
//! A sequence whose animation lives in an external sequence-group file (`IDSQ`) cannot
//! be completed from the main file alone: [`StudioModel::parse`] lists the files it
//! needs and [`UnresolvedStudioModel::resolve`] requires exactly those. A missing,
//! unexpected or malformed dependency is an error; nothing is ever substituted.

use std::collections::BTreeMap;
use std::fmt;

use sha2::{Digest, Sha256};

/// `STUDIO_VERSION`
pub const STUDIO_VERSION: i32 = 10;
/// Larger than any stock model by an order of magnitude; ours
pub const MAX_STUDIO_MODEL_BYTES: usize = 32 * 1024 * 1024;
/// `MAXSTUDIOBONES`
pub const MAX_STUDIO_BONES: usize = 128;
/// `MAXSTUDIOCONTROLLERS`
pub const MAX_STUDIO_BONE_CONTROLLERS: usize = 8;
/// No SDK limit; ours
pub const MAX_STUDIO_HITBOXES: usize = 512;
/// `MAXSTUDIOSEQUENCES`
pub const MAX_STUDIO_SEQUENCES: usize = 2048;
/// `MAXSTUDIOGROUPS`
pub const MAX_STUDIO_SEQUENCE_GROUPS: usize = 16;
/// `MAXSTUDIOEVENTS`, per sequence
pub const MAX_STUDIO_EVENTS: usize = 1024;
/// No SDK limit; ours, per sequence
pub const MAX_STUDIO_FRAMES: usize = 8192;
/// Decoded animation across a model and its sequence groups: three bytes per frame of
/// a track, its value and whether it ends its run's explicit values; ours
pub const MAX_STUDIO_ANIMATION_BYTES: usize = 32 * 1024 * 1024;
/// What one decoded frame of a track costs against `MAX_STUDIO_ANIMATION_BYTES`
const ANIMATION_BYTES_PER_FRAME: usize = 3;

const STUDIO_HEADER_ID: &[u8; 4] = b"IDST";
const SEQUENCE_GROUP_HEADER_ID: &[u8; 4] = b"IDSQ";
const HEADER_SIZE: usize = 244;
const SEQUENCE_GROUP_HEADER_SIZE: usize = 76;
const BONE_SIZE: usize = 112;
const BONE_CONTROLLER_SIZE: usize = 24;
const HITBOX_SIZE: usize = 32;
const SEQUENCE_GROUP_SIZE: usize = 104;
const SEQUENCE_SIZE: usize = 176;
const EVENT_SIZE: usize = 76;
const ANIMATION_OFFSETS_SIZE: usize = 12;

/// Motion and controller axis flags (`STUDIO_X` .. `STUDIO_ZR`) and `STUDIO_RLOOP`
const STUDIO_AXES: [i32; 6] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20];
const STUDIO_RLOOP: i32 = 0x8000;
/// The motion flags under which the renderer indexes the motion bone
/// (`StudioCalcRotations`): `STUDIO_X`, `STUDIO_Y`, `STUDIO_Z`, `STUDIO_LX`,
/// `STUDIO_LY`, `STUDIO_LZ`
const MOTION_BONE_FLAGS: i32 = 0x01 | 0x02 | 0x04 | 0x40 | 0x80 | 0x100;
/// Bone controller indices 0-3 are user controllers, 4 is the mouth
const MAX_CONTROLLER_INDEX: i32 = 4;

/// SHA-256 of an asset file's bytes, or of an identity built from them
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StudioDigest(pub [u8; 32]);

impl StudioDigest {
    fn of(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }
}

impl fmt::Display for StudioDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
            .iter()
            .try_for_each(|byte| write!(formatter, "{byte:02x}"))
    }
}

/// What a model's poses and hit queries depend on: the main file and every external
/// sequence group its sequences use, each by digest, combined into one identity. A
/// texture companion (`<name>T.mdl`) affects only rendering and is not part of it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StudioModelIdentity {
    pub main: StudioDigest,
    /// `(sequence group index, file name as the model names it, digest)`, by index
    pub sequence_groups: Vec<(usize, String, StudioDigest)>,
    /// SHA-256 over a fixed tag, the main digest and each sequence group's index, name
    /// and digest, in index order
    pub combined: StudioDigest,
}

impl StudioModelIdentity {
    fn new(main: StudioDigest, sequence_groups: Vec<(usize, String, StudioDigest)>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"xenoxide studio model identity v1\0");
        hasher.update(main.0);
        for (index, name, digest) in &sequence_groups {
            hasher.update((*index as u32).to_le_bytes());
            hasher.update((name.len() as u32).to_le_bytes());
            hasher.update(name.as_bytes());
            hasher.update(digest.0);
        }
        Self {
            main,
            sequence_groups,
            combined: StudioDigest(hasher.finalize().into()),
        }
    }
}

/// `mstudiobone_t`
#[derive(Clone, Debug, PartialEq)]
pub struct StudioBone {
    pub name: String,
    /// Always an earlier bone: transforms are built in bone order
    pub parent: Option<usize>,
    pub flags: i32,
    /// Per degree of freedom (X, Y, Z, XR, YR, ZR), an index into the bone controllers
    pub controllers: [Option<usize>; 6],
    /// Default value of each degree of freedom
    pub value: [f32; 6],
    /// Scale of the animation values of each degree of freedom
    pub scale: [f32; 6],
}

/// `mstudiobonecontroller_t`
#[derive(Clone, Debug, PartialEq)]
pub struct StudioBoneController {
    /// The bone it drives; `-1` in the file means bone 0
    pub bone: usize,
    /// One of the six axis flags (`STUDIO_X` .. `STUDIO_ZR`)
    pub axis: i32,
    /// `STUDIO_RLOOP`: wraps by the shortest distance
    pub wraps: bool,
    pub start: f32,
    pub end: f32,
    pub rest: i32,
    /// 0-3 user controllers, 4 the mouth
    pub index: u8,
}

/// `mstudiobbox_t`: a box attached to a bone, in its space
#[derive(Clone, Debug, PartialEq)]
pub struct StudioHitbox {
    pub bone: usize,
    /// The hit group, as authored
    pub group: i32,
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// `mstudioseqgroup_t`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StudioSequenceGroup {
    pub label: String,
    /// For groups after the first, the external file holding their animation
    pub name: String,
}

/// `mstudioevent_t`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StudioEvent {
    pub frame: i32,
    pub event: i32,
    pub kind: i32,
    pub options: String,
}

/// One degree of freedom of one bone in one blend, decoded from its runs: the raw value
/// of every frame (the pose is the bone's `value + raw * scale`), and which frames hold
/// the last explicit value of their run.
///
/// The renderer reads translations and rotations differently at such a frame. From the
/// last explicit value of a run, a translation is not interpolated
/// (`StudioCalcBonePosition`), while a rotation interpolates towards the next frame's
/// value, the next run's first when the run ends there (`StudioCalcBoneQuaterion`).
/// Everywhere else both interpolate towards the next frame's value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StudioTrack {
    values: Vec<i16>,
    last_explicit: Vec<bool>,
}

impl StudioTrack {
    pub fn frames(&self) -> usize {
        self.values.len()
    }

    /// The raw value of every frame
    pub fn values(&self) -> &[i16] {
        &self.values
    }

    /// Whether `frame` holds the last explicit value of its run
    pub fn ends_explicit_values(&self, frame: usize) -> bool {
        self.last_explicit[frame]
    }

    /// For a translation (X, Y, Z) at `frame`: its raw value and the one it interpolates
    /// towards. The renderer never takes a frame past the last; there its look-ahead is
    /// multiplied by zero, so the frame's own value is returned.
    pub fn translation(&self, frame: usize) -> (i16, i16) {
        let value = self.values[frame];
        if self.last_explicit[frame] || frame + 1 == self.values.len() {
            (value, value)
        } else {
            (value, self.values[frame + 1])
        }
    }

    /// For a rotation (XR, YR, ZR) at `frame`: its raw value and the one it interpolates
    /// towards, the frame's own at the last frame
    pub fn rotation(&self, frame: usize) -> (i16, i16) {
        let value = self.values[frame];
        (value, *self.values.get(frame + 1).unwrap_or(&value))
    }
}

/// The six tracks (X, Y, Z, XR, YR, ZR) of one bone in one blend; `None` where the
/// file has no track and the bone's default value holds
pub type StudioBoneTracks = [Option<StudioTrack>; 6];

/// `mstudioseqdesc_t`, with its events and decoded animation
#[derive(Clone, Debug, PartialEq)]
pub struct StudioSequence {
    pub label: String,
    pub fps: f32,
    pub flags: i32,
    pub activity: i32,
    pub activity_weight: i32,
    pub events: Vec<StudioEvent>,
    pub frames: usize,
    pub motion_type: i32,
    /// An existing bone whenever the motion type has a flag under which the renderer
    /// indexes it (X, Y, Z, LX, LY, LZ). Otherwise as stored: stock files hold leftover
    /// values here when no such flag is set (player.mdl: 1128127587 among others).
    pub motion_bone: i32,
    pub linear_movement: [f32; 3],
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub blend_type: [i32; 2],
    pub blend_start: [f32; 2],
    pub blend_end: [f32; 2],
    pub blend_parent: i32,
    pub group: usize,
    pub entry_node: i32,
    pub exit_node: i32,
    pub node_flags: i32,
    pub next_sequence: i32,
    /// `[blend][bone]`; 1, 2 or 4 blends
    pub animation: Vec<Vec<StudioBoneTracks>>,
}

impl StudioSequence {
    /// The motion bone, when the motion type makes the renderer use it
    pub fn active_motion_bone(&self) -> Option<usize> {
        (self.motion_type & MOTION_BONE_FLAGS != 0).then_some(self.motion_bone as usize)
    }
}

/// A main model file read and validated, waiting for the external sequence groups its
/// sequences need
#[derive(Debug)]
pub struct UnresolvedStudioModel {
    model: StudioModel,
    /// Sequences whose animation is in an external group, with what decoding needs
    pending: Vec<PendingAnimation>,
    dependencies: Vec<SequenceGroupDependency>,
    budget: usize,
}

/// An external sequence-group file a model needs
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SequenceGroupDependency {
    pub index: usize,
    /// As the model names it, for example `models/scientist01.mdl`
    pub name: String,
}

#[derive(Debug)]
struct PendingAnimation {
    sequence: usize,
    group: usize,
    animation_offset: i32,
    blends: usize,
}

/// A complete, validated studio model
#[derive(Clone, Debug, PartialEq)]
pub struct StudioModel {
    name: Vec<u8>,
    eye_position: [f32; 3],
    hull_min: [f32; 3],
    hull_max: [f32; 3],
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
    flags: i32,
    bones: Vec<StudioBone>,
    bone_controllers: Vec<StudioBoneController>,
    hitboxes: Vec<StudioHitbox>,
    sequence_groups: Vec<StudioSequenceGroup>,
    sequences: Vec<StudioSequence>,
    texture_companion: bool,
    identity: StudioModelIdentity,
}

impl StudioModel {
    /// Reads a main model file (`IDST`, version 10)
    pub fn parse(bytes: &[u8]) -> Result<UnresolvedStudioModel, StudioError> {
        parse_main(bytes, MAX_STUDIO_ANIMATION_BYTES)
    }

    /// The header's informational name, as stored up to its first NUL
    pub fn name(&self) -> &[u8] {
        &self.name
    }

    pub fn eye_position(&self) -> [f32; 3] {
        self.eye_position
    }

    /// The ideal movement hull; movement collision is a separate contract
    pub fn hull(&self) -> ([f32; 3], [f32; 3]) {
        (self.hull_min, self.hull_max)
    }

    pub fn bounds(&self) -> ([f32; 3], [f32; 3]) {
        (self.bounds_min, self.bounds_max)
    }

    pub fn flags(&self) -> i32 {
        self.flags
    }

    pub fn bones(&self) -> &[StudioBone] {
        &self.bones
    }

    pub fn bone_controllers(&self) -> &[StudioBoneController] {
        &self.bone_controllers
    }

    pub fn hitboxes(&self) -> &[StudioHitbox] {
        &self.hitboxes
    }

    pub fn sequence_groups(&self) -> &[StudioSequenceGroup] {
        &self.sequence_groups
    }

    pub fn sequences(&self) -> &[StudioSequence] {
        &self.sequences
    }

    /// The model keeps its textures in a companion file (`<name>T.mdl`), needed only
    /// for rendering
    pub fn texture_companion(&self) -> bool {
        self.texture_companion
    }

    pub fn identity(&self) -> &StudioModelIdentity {
        &self.identity
    }
}

impl UnresolvedStudioModel {
    /// The external sequence groups the sequences use, by index; groups no sequence
    /// uses are not dependencies
    pub fn dependencies(&self) -> &[SequenceGroupDependency] {
        &self.dependencies
    }

    /// Completes the model with the bytes of each dependency, by group index. Every
    /// dependency must be supplied, and nothing else.
    pub fn resolve(self, files: &[(usize, &[u8])]) -> Result<StudioModel, StudioError> {
        let Self {
            mut model,
            pending,
            dependencies,
            mut budget,
        } = self;
        let mut supplied = BTreeMap::new();
        for &(index, bytes) in files {
            if !dependencies
                .iter()
                .any(|dependency| dependency.index == index)
            {
                return Err(StudioError::UnexpectedDependency { group: index });
            }
            if supplied.insert(index, bytes).is_some() {
                return Err(StudioError::DuplicateDependency { group: index });
            }
        }
        let mut digests = Vec::new();
        for dependency in &dependencies {
            let Some(bytes) = supplied.get(&dependency.index) else {
                return Err(StudioError::MissingDependency {
                    group: dependency.index,
                    name: dependency.name.clone(),
                });
            };
            check_sequence_group_header(bytes)?;
            digests.push((
                dependency.index,
                dependency.name.clone(),
                StudioDigest::of(bytes),
            ));
        }
        let bones = model.bones.len();
        for animation in pending {
            let bytes = supplied[&animation.group];
            let sequence = &mut model.sequences[animation.sequence];
            sequence.animation = decode_animation(
                bytes,
                animation.animation_offset,
                animation.blends,
                bones,
                sequence.frames,
                animation.sequence,
                &mut budget,
            )?;
        }
        model.identity = StudioModelIdentity::new(model.identity.main, digests);
        Ok(model)
    }
}

/// Why a studio model or sequence group was refused
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StudioError {
    TooLarge {
        bytes: usize,
    },
    Truncated {
        what: &'static str,
    },
    InvalidMagic {
        what: &'static str,
    },
    UnsupportedVersion {
        what: &'static str,
        version: i32,
    },
    LengthMismatch {
        what: &'static str,
        declared: i32,
        actual: usize,
    },
    InvalidCount {
        what: &'static str,
        count: i32,
    },
    CountOverLimit {
        what: &'static str,
        count: usize,
        limit: usize,
    },
    OutOfBounds {
        what: &'static str,
        index: usize,
    },
    InvalidName {
        what: &'static str,
        index: usize,
    },
    NonFinite {
        what: &'static str,
        index: usize,
    },
    InvalidBoneParent {
        bone: usize,
        parent: i32,
    },
    InvalidBoneController {
        bone: usize,
        channel: usize,
        controller: i32,
    },
    InvalidController {
        index: usize,
    },
    InvalidHitbox {
        index: usize,
    },
    InvalidSequenceGroup {
        index: usize,
    },
    InvalidSequence {
        index: usize,
        reason: &'static str,
    },
    UnsupportedBlends {
        sequence: usize,
        blends: i32,
    },
    MalformedAnimation {
        sequence: usize,
        blend: usize,
        bone: usize,
        channel: usize,
        reason: &'static str,
    },
    AnimationBudgetExceeded {
        limit: usize,
    },
    /// The rendering data of the whole asset needs more of `what` than its budget
    RenderBudgetExceeded {
        what: &'static str,
        limit: usize,
    },
    MissingDependency {
        group: usize,
        name: String,
    },
    UnexpectedDependency {
        group: usize,
    },
    DuplicateDependency {
        group: usize,
    },
    /// Rendering data read from a file other than the one the model was read from
    RenderModelMismatch,
    /// The main file has no textures, and no texture companion was supplied
    MissingTextureCompanion,
    /// A texture companion supplied for a main file that has its own textures
    UnexpectedTextureCompanion,
    InvalidTexture {
        index: usize,
        reason: &'static str,
    },
    InvalidSkin {
        family: usize,
        reference: usize,
    },
    InvalidBodyPart {
        index: usize,
        reason: &'static str,
    },
    InvalidSubmodel {
        part: usize,
        model: usize,
        reason: &'static str,
    },
    InvalidMesh {
        part: usize,
        model: usize,
        mesh: usize,
        reason: &'static str,
    },
}

impl fmt::Display for StudioError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge { bytes } => write!(
                formatter,
                "studio model of {bytes} bytes exceeds {MAX_STUDIO_MODEL_BYTES}"
            ),
            Self::Truncated { what } => write!(formatter, "studio model {what} is truncated"),
            Self::InvalidMagic { what } => write!(formatter, "{what} has the wrong identifier"),
            Self::UnsupportedVersion { what, version } => {
                write!(
                    formatter,
                    "{what} version {version} is not {STUDIO_VERSION}"
                )
            }
            Self::LengthMismatch {
                what,
                declared,
                actual,
            } => write!(
                formatter,
                "{what} declares {declared} bytes but has {actual}"
            ),
            Self::InvalidCount { what, count } => {
                write!(formatter, "studio model {what} count {count} is invalid")
            }
            Self::CountOverLimit { what, count, limit } => write!(
                formatter,
                "studio model {what} count {count} exceeds {limit}"
            ),
            Self::OutOfBounds { what, index } => {
                write!(
                    formatter,
                    "studio model {what} {index} lies outside the file"
                )
            }
            Self::InvalidName { what, index } => {
                write!(formatter, "studio model {what} {index} has an invalid name")
            }
            Self::NonFinite { what, index } => {
                write!(
                    formatter,
                    "studio model {what} {index} has a nonfinite value"
                )
            }
            Self::InvalidBoneParent { bone, parent } => {
                write!(formatter, "studio bone {bone} has invalid parent {parent}")
            }
            Self::InvalidBoneController {
                bone,
                channel,
                controller,
            } => write!(
                formatter,
                "studio bone {bone} channel {channel} names invalid controller {controller}"
            ),
            Self::InvalidController { index } => {
                write!(formatter, "studio bone controller {index} is invalid")
            }
            Self::InvalidHitbox { index } => write!(formatter, "studio hitbox {index} is invalid"),
            Self::InvalidSequenceGroup { index } => {
                write!(formatter, "studio sequence group {index} is invalid")
            }
            Self::InvalidSequence { index, reason } => {
                write!(formatter, "studio sequence {index} is invalid: {reason}")
            }
            Self::UnsupportedBlends { sequence, blends } => write!(
                formatter,
                "studio sequence {sequence} has {blends} blends; only 1, 2 and 4 are supported"
            ),
            Self::MalformedAnimation {
                sequence,
                blend,
                bone,
                channel,
                reason,
            } => write!(
                formatter,
                "studio sequence {sequence} blend {blend} bone {bone} channel {channel}: {reason}"
            ),
            Self::AnimationBudgetExceeded { limit } => {
                write!(formatter, "studio animation exceeds {limit} decoded bytes")
            }
            Self::RenderBudgetExceeded { what, limit } => {
                write!(formatter, "studio rendering data exceeds {limit} {what}")
            }
            Self::MissingDependency { group, name } => {
                write!(
                    formatter,
                    "studio sequence group {group} ({name}) is missing"
                )
            }
            Self::UnexpectedDependency { group } => {
                write!(
                    formatter,
                    "studio sequence group {group} is not a dependency"
                )
            }
            Self::DuplicateDependency { group } => {
                write!(
                    formatter,
                    "studio sequence group {group} was supplied twice"
                )
            }
            Self::RenderModelMismatch => {
                formatter.write_str("studio rendering data belongs to another model file")
            }
            Self::MissingTextureCompanion => {
                formatter.write_str("studio model needs its texture companion")
            }
            Self::UnexpectedTextureCompanion => {
                formatter.write_str("studio model has its own textures; no companion is used")
            }
            Self::InvalidTexture { index, reason } => {
                write!(formatter, "studio texture {index} is invalid: {reason}")
            }
            Self::InvalidSkin { family, reference } => write!(
                formatter,
                "studio skin family {family} reference {reference} names no texture"
            ),
            Self::InvalidBodyPart { index, reason } => {
                write!(formatter, "studio body part {index} is invalid: {reason}")
            }
            Self::InvalidSubmodel {
                part,
                model,
                reason,
            } => write!(
                formatter,
                "studio body part {part} model {model} is invalid: {reason}"
            ),
            Self::InvalidMesh {
                part,
                model,
                mesh,
                reason,
            } => write!(
                formatter,
                "studio body part {part} model {model} mesh {mesh} is invalid: {reason}"
            ),
        }
    }
}

impl std::error::Error for StudioError {}

/// Little-endian reads at positions whose ranges were already checked
fn i32_at(bytes: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(bytes[at..at + 4].try_into().expect("four bytes"))
}

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(bytes[at..at + 2].try_into().expect("two bytes"))
}

fn finite(bytes: &[u8], at: usize, what: &'static str, index: usize) -> Result<f32, StudioError> {
    let value = f32::from_le_bytes(bytes[at..at + 4].try_into().expect("four bytes"));
    if value.is_finite() {
        Ok(value)
    } else {
        Err(StudioError::NonFinite { what, index })
    }
}

fn finite_array<const N: usize>(
    bytes: &[u8],
    at: usize,
    what: &'static str,
    index: usize,
) -> Result<[f32; N], StudioError> {
    let mut values = [0.0; N];
    for (slot, value) in values.iter_mut().enumerate() {
        *value = finite(bytes, at + slot * 4, what, index)?;
    }
    Ok(values)
}

/// A NUL-terminated name of printable ASCII within a fixed field
fn name_at(
    bytes: &[u8],
    at: usize,
    length: usize,
    what: &'static str,
    index: usize,
) -> Result<String, StudioError> {
    let field = &bytes[at..at + length];
    let end = field
        .iter()
        .position(|byte| *byte == 0)
        .ok_or(StudioError::InvalidName { what, index })?;
    let name = &field[..end];
    if name.iter().all(|byte| (0x20..=0x7e).contains(byte)) {
        Ok(String::from_utf8(name.to_vec()).expect("printable ASCII"))
    } else {
        Err(StudioError::InvalidName { what, index })
    }
}

/// The byte offset of a table of `count` records of `size` bytes at `offset`, checked
/// against the file and the limit. An empty table's offset is not used.
fn table(
    bytes: &[u8],
    what: &'static str,
    count: i32,
    offset: i32,
    size: usize,
    limit: usize,
) -> Result<(usize, usize), StudioError> {
    let count = usize::try_from(count).map_err(|_| StudioError::InvalidCount { what, count })?;
    if count > limit {
        return Err(StudioError::CountOverLimit { what, count, limit });
    }
    if count == 0 {
        return Ok((0, 0));
    }
    let start = usize::try_from(offset).map_err(|_| StudioError::OutOfBounds { what, index: 0 })?;
    let end = count
        .checked_mul(size)
        .and_then(|length| start.checked_add(length))
        .ok_or(StudioError::OutOfBounds { what, index: 0 })?;
    if end > bytes.len() {
        return Err(StudioError::OutOfBounds { what, index: 0 });
    }
    Ok((count, start))
}

/// The file's identifier, version and declared length, for a header of `size` bytes
fn check_file(
    bytes: &[u8],
    what: &'static str,
    id: &[u8; 4],
    size: usize,
) -> Result<(), StudioError> {
    if bytes.len() > MAX_STUDIO_MODEL_BYTES {
        return Err(StudioError::TooLarge { bytes: bytes.len() });
    }
    if bytes.len() < size {
        return Err(StudioError::Truncated { what });
    }
    if &bytes[0..4] != id {
        return Err(StudioError::InvalidMagic { what });
    }
    let version = i32_at(bytes, 4);
    if version != STUDIO_VERSION {
        return Err(StudioError::UnsupportedVersion { what, version });
    }
    let declared = i32_at(bytes, 72);
    if usize::try_from(declared).ok() != Some(bytes.len()) {
        return Err(StudioError::LengthMismatch {
            what,
            declared,
            actual: bytes.len(),
        });
    }
    Ok(())
}

/// The informational name in a file header: its bytes up to the first NUL, or the whole
/// field. Stock files hold anything here (lada5.mdl: its author's full path, in a
/// Cyrillic code page, filling all 64 bytes without a terminator).
fn header_name(bytes: &[u8]) -> Vec<u8> {
    let field = &bytes[8..72];
    let end = field
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(field.len());
    field[..end].to_vec()
}

fn check_sequence_group_header(bytes: &[u8]) -> Result<(), StudioError> {
    check_file(
        bytes,
        "sequence group file",
        SEQUENCE_GROUP_HEADER_ID,
        SEQUENCE_GROUP_HEADER_SIZE,
    )
}

pub(crate) fn parse_main(
    bytes: &[u8],
    budget: usize,
) -> Result<UnresolvedStudioModel, StudioError> {
    check_file(bytes, "header", STUDIO_HEADER_ID, HEADER_SIZE)?;
    let name = header_name(bytes);
    let eye_position = finite_array(bytes, 76, "header", 0)?;
    let hull_min = finite_array(bytes, 88, "header", 0)?;
    let hull_max = finite_array(bytes, 100, "header", 0)?;
    let bounds_min = finite_array(bytes, 112, "header", 0)?;
    let bounds_max = finite_array(bytes, 124, "header", 0)?;
    let flags = i32_at(bytes, 136);
    let field = |index: usize| i32_at(bytes, 140 + index * 4);

    let bones = parse_bones(bytes, field(0), field(1))?;
    let bone_controllers = parse_controllers(bytes, field(2), field(3), bones.len())?;
    for (index, bone) in bones.iter().enumerate() {
        for (channel, controller) in bone.controllers.iter().enumerate() {
            if controller.is_some_and(|controller| controller >= bone_controllers.len()) {
                return Err(StudioError::InvalidBoneController {
                    bone: index,
                    channel,
                    controller: controller.expect("checked") as i32,
                });
            }
        }
    }
    let hitboxes = parse_hitboxes(bytes, field(4), field(5), bones.len())?;
    let (sequence_count, sequence_offset) = table(
        bytes,
        "sequence",
        field(6),
        field(7),
        SEQUENCE_SIZE,
        MAX_STUDIO_SEQUENCES,
    )?;
    let sequence_groups = parse_sequence_groups(bytes, field(8), field(9))?;
    if sequence_count > 0 && sequence_groups.is_empty() {
        return Err(StudioError::InvalidSequenceGroup { index: 0 });
    }
    let texture_companion = field(10) == 0;

    let mut budget = budget;
    let mut sequences = Vec::with_capacity(sequence_count);
    let mut pending = Vec::new();
    for index in 0..sequence_count {
        let at = sequence_offset + index * SEQUENCE_SIZE;
        let (mut sequence, animation_offset, blends) = parse_sequence(
            bytes,
            at,
            index,
            bones.len(),
            sequence_count,
            sequence_groups.len(),
        )?;
        if sequence.group == 0 {
            sequence.animation = decode_animation(
                bytes,
                animation_offset,
                blends,
                bones.len(),
                sequence.frames,
                index,
                &mut budget,
            )?;
        } else {
            pending.push(PendingAnimation {
                sequence: index,
                group: sequence.group,
                animation_offset,
                blends,
            });
        }
        sequences.push(sequence);
    }
    let mut dependencies: Vec<SequenceGroupDependency> = Vec::new();
    for animation in &pending {
        if !dependencies
            .iter()
            .any(|dependency| dependency.index == animation.group)
        {
            dependencies.push(SequenceGroupDependency {
                index: animation.group,
                name: sequence_groups[animation.group].name.clone(),
            });
        }
    }
    dependencies.sort_by_key(|dependency| dependency.index);

    Ok(UnresolvedStudioModel {
        model: StudioModel {
            name,
            eye_position,
            hull_min,
            hull_max,
            bounds_min,
            bounds_max,
            flags,
            bones,
            bone_controllers,
            hitboxes,
            sequence_groups,
            sequences,
            texture_companion,
            identity: StudioModelIdentity::new(StudioDigest::of(bytes), Vec::new()),
        },
        pending,
        dependencies,
        budget,
    })
}

fn parse_bones(bytes: &[u8], count: i32, offset: i32) -> Result<Vec<StudioBone>, StudioError> {
    let (count, start) = table(bytes, "bone", count, offset, BONE_SIZE, MAX_STUDIO_BONES)?;
    let mut bones = Vec::with_capacity(count);
    for index in 0..count {
        let at = start + index * BONE_SIZE;
        let name = name_at(bytes, at, 32, "bone", index)?;
        let parent = i32_at(bytes, at + 32);
        // Parents precede their children, which also rules out cycles
        let parent = match parent {
            -1 => None,
            parent if (0..index as i32).contains(&parent) => Some(parent as usize),
            parent => {
                return Err(StudioError::InvalidBoneParent {
                    bone: index,
                    parent,
                });
            }
        };
        let mut controllers = [None; 6];
        for (channel, slot) in controllers.iter_mut().enumerate() {
            *slot = match i32_at(bytes, at + 40 + channel * 4) {
                -1 => None,
                controller if controller >= 0 => Some(controller as usize),
                controller => {
                    return Err(StudioError::InvalidBoneController {
                        bone: index,
                        channel,
                        controller,
                    });
                }
            };
        }
        bones.push(StudioBone {
            name,
            parent,
            flags: i32_at(bytes, at + 36),
            controllers,
            value: finite_array(bytes, at + 64, "bone", index)?,
            scale: finite_array(bytes, at + 88, "bone", index)?,
        });
    }
    Ok(bones)
}

fn parse_controllers(
    bytes: &[u8],
    count: i32,
    offset: i32,
    bones: usize,
) -> Result<Vec<StudioBoneController>, StudioError> {
    let (count, start) = table(
        bytes,
        "bone controller",
        count,
        offset,
        BONE_CONTROLLER_SIZE,
        MAX_STUDIO_BONE_CONTROLLERS,
    )?;
    let mut controllers = Vec::with_capacity(count);
    for index in 0..count {
        let at = start + index * BONE_CONTROLLER_SIZE;
        let invalid = StudioError::InvalidController { index };
        let bone = match i32_at(bytes, at) {
            -1 => 0,
            bone if bone >= 0 && (bone as usize) < bones => bone as usize,
            _ => return Err(invalid),
        };
        if bones == 0 {
            return Err(invalid);
        }
        let kind = i32_at(bytes, at + 4);
        let axis = kind & !STUDIO_RLOOP;
        if !STUDIO_AXES.contains(&axis) {
            return Err(invalid);
        }
        let controller_index = i32_at(bytes, at + 20);
        if !(0..=MAX_CONTROLLER_INDEX).contains(&controller_index) {
            return Err(invalid);
        }
        controllers.push(StudioBoneController {
            bone,
            axis,
            wraps: kind & STUDIO_RLOOP != 0,
            start: finite(bytes, at + 8, "bone controller", index)?,
            end: finite(bytes, at + 12, "bone controller", index)?,
            rest: i32_at(bytes, at + 16),
            index: controller_index as u8,
        });
    }
    Ok(controllers)
}

fn parse_hitboxes(
    bytes: &[u8],
    count: i32,
    offset: i32,
    bones: usize,
) -> Result<Vec<StudioHitbox>, StudioError> {
    let (count, start) = table(
        bytes,
        "hitbox",
        count,
        offset,
        HITBOX_SIZE,
        MAX_STUDIO_HITBOXES,
    )?;
    let mut hitboxes = Vec::with_capacity(count);
    for index in 0..count {
        let at = start + index * HITBOX_SIZE;
        let bone = i32_at(bytes, at);
        if !(0..bones as i32).contains(&bone) {
            return Err(StudioError::InvalidHitbox { index });
        }
        let min: [f32; 3] = finite_array(bytes, at + 8, "hitbox", index)?;
        let max: [f32; 3] = finite_array(bytes, at + 20, "hitbox", index)?;
        if (0..3).any(|axis| min[axis] > max[axis]) {
            return Err(StudioError::InvalidHitbox { index });
        }
        hitboxes.push(StudioHitbox {
            bone: bone as usize,
            group: i32_at(bytes, at + 4),
            min,
            max,
        });
    }
    Ok(hitboxes)
}

fn parse_sequence_groups(
    bytes: &[u8],
    count: i32,
    offset: i32,
) -> Result<Vec<StudioSequenceGroup>, StudioError> {
    let (count, start) = table(
        bytes,
        "sequence group",
        count,
        offset,
        SEQUENCE_GROUP_SIZE,
        MAX_STUDIO_SEQUENCE_GROUPS,
    )?;
    (0..count)
        .map(|index| {
            let at = start + index * SEQUENCE_GROUP_SIZE;
            Ok(StudioSequenceGroup {
                label: name_at(bytes, at, 32, "sequence group", index)?,
                name: name_at(bytes, at + 32, 64, "sequence group", index)?,
            })
        })
        .collect()
}

/// A sequence descriptor and its events; the animation is decoded separately. Returns
/// the animation offset (into the file of its sequence group) and the blend count.
fn parse_sequence(
    bytes: &[u8],
    at: usize,
    index: usize,
    bones: usize,
    sequences: usize,
    groups: usize,
) -> Result<(StudioSequence, i32, usize), StudioError> {
    let invalid = |reason| StudioError::InvalidSequence { index, reason };
    let label = name_at(bytes, at, 32, "sequence", index)?;
    let fps = finite(bytes, at + 32, "sequence", index)?;
    if fps < 0.0 {
        return Err(invalid("negative frame rate"));
    }
    let frames = i32_at(bytes, at + 56);
    let frames = usize::try_from(frames)
        .ok()
        .filter(|frames| (1..=MAX_STUDIO_FRAMES).contains(frames))
        .ok_or(invalid("frame count"))?;
    let blends = i32_at(bytes, at + 120);
    if ![1, 2, 4].contains(&blends) {
        return Err(StudioError::UnsupportedBlends {
            sequence: index,
            blends,
        });
    }
    let group = i32_at(bytes, at + 156);
    let group = usize::try_from(group)
        .ok()
        .filter(|group| *group < groups)
        .ok_or(invalid("sequence group"))?;
    let next_sequence = i32_at(bytes, at + 172);
    if !(0..sequences as i32).contains(&next_sequence) {
        return Err(invalid("next sequence"));
    }
    let motion_type = i32_at(bytes, at + 72);
    let motion_bone = i32_at(bytes, at + 76);
    if motion_type & MOTION_BONE_FLAGS != 0 && !(0..bones as i32).contains(&motion_bone) {
        return Err(invalid("motion bone"));
    }
    let (event_count, event_offset) = table(
        bytes,
        "event",
        i32_at(bytes, at + 48),
        i32_at(bytes, at + 52),
        EVENT_SIZE,
        MAX_STUDIO_EVENTS,
    )?;
    let events = (0..event_count)
        .map(|event| {
            let at = event_offset + event * EVENT_SIZE;
            Ok(StudioEvent {
                frame: i32_at(bytes, at),
                event: i32_at(bytes, at + 4),
                kind: i32_at(bytes, at + 8),
                options: name_at(bytes, at + 12, 64, "event", event)?,
            })
        })
        .collect::<Result<_, StudioError>>()?;
    let sequence = StudioSequence {
        label,
        fps,
        flags: i32_at(bytes, at + 36),
        activity: i32_at(bytes, at + 40),
        activity_weight: i32_at(bytes, at + 44),
        events,
        frames,
        motion_type,
        motion_bone,
        linear_movement: finite_array(bytes, at + 80, "sequence", index)?,
        min: finite_array(bytes, at + 100, "sequence", index)?,
        max: finite_array(bytes, at + 112, "sequence", index)?,
        blend_type: [i32_at(bytes, at + 128), i32_at(bytes, at + 132)],
        blend_start: finite_array(bytes, at + 136, "sequence", index)?,
        blend_end: finite_array(bytes, at + 144, "sequence", index)?,
        blend_parent: i32_at(bytes, at + 152),
        group,
        entry_node: i32_at(bytes, at + 160),
        exit_node: i32_at(bytes, at + 164),
        node_flags: i32_at(bytes, at + 168),
        next_sequence,
        animation: Vec::new(),
    };
    Ok((sequence, i32_at(bytes, at + 124), blends as usize))
}

/// Decodes a sequence's animation: `[blend][bone]` blocks of six 16-bit offsets at
/// `offset` in `bytes` (the main file for group 0, else the group's file), each
/// relative to its own block, to a track of run-length values or 0 for none
fn decode_animation(
    bytes: &[u8],
    offset: i32,
    blends: usize,
    bones: usize,
    frames: usize,
    sequence: usize,
    budget: &mut usize,
) -> Result<Vec<Vec<StudioBoneTracks>>, StudioError> {
    if bones == 0 {
        return Ok(vec![Vec::new(); blends]);
    }
    let count = i32::try_from(blends * bones).expect("bounded by the limits");
    let (_, start) = table(
        bytes,
        "animation",
        count,
        offset,
        ANIMATION_OFFSETS_SIZE,
        MAX_STUDIO_BONES * 4,
    )
    .map_err(|_| StudioError::InvalidSequence {
        index: sequence,
        reason: "animation offsets lie outside the file",
    })?;
    let mut animation = Vec::with_capacity(blends);
    for blend in 0..blends {
        let mut tracks = Vec::with_capacity(bones);
        for bone in 0..bones {
            let block = start + (blend * bones + bone) * ANIMATION_OFFSETS_SIZE;
            let mut channels: StudioBoneTracks = Default::default();
            for (channel, slot) in channels.iter_mut().enumerate() {
                let relative = u16_at(bytes, block + channel * 2);
                if relative == 0 {
                    continue;
                }
                let malformed = |reason| StudioError::MalformedAnimation {
                    sequence,
                    blend,
                    bone,
                    channel,
                    reason,
                };
                *budget = budget
                    .checked_sub(frames * ANIMATION_BYTES_PER_FRAME)
                    .ok_or(StudioError::AnimationBudgetExceeded {
                        limit: MAX_STUDIO_ANIMATION_BYTES,
                    })?;
                *slot = Some(
                    decode_track(bytes, block + relative as usize, frames).map_err(malformed)?,
                );
            }
            tracks.push(channels);
        }
        animation.push(tracks);
    }
    Ok(animation)
}

/// One track at every frame. Runs are a header (`valid` and `total` bytes) followed by
/// `valid` values; a run covers `total` frames, the last value repeating after the
/// explicit ones. The frame of each run's last explicit value is kept, which with the
/// values is everything the renderer's two readings use for frames up to the last. A
/// run with no frames, no values or more values than frames is malformed, as is any run
/// that reaches past the end of the file.
fn decode_track(bytes: &[u8], start: usize, frames: usize) -> Result<StudioTrack, &'static str> {
    let mut values = Vec::with_capacity(frames);
    let mut last_explicit = Vec::with_capacity(frames);
    let mut at = start;
    while values.len() < frames {
        let header = bytes.get(at..at + 2).ok_or("run lies outside the file")?;
        let (valid, total) = (header[0] as usize, header[1] as usize);
        if total == 0 {
            return Err("run covers no frames");
        }
        if valid == 0 {
            return Err("run has no values");
        }
        if valid > total {
            return Err("run has more values than frames");
        }
        let explicit = bytes
            .get(at + 2..at + 2 + valid * 2)
            .ok_or("run lies outside the file")?;
        let value = |slot: usize| i16::from_le_bytes([explicit[slot * 2], explicit[slot * 2 + 1]]);
        for frame in 0..total.min(frames - values.len()) {
            values.push(value(frame.min(valid - 1)));
            last_explicit.push(frame == valid - 1);
        }
        at += 2 + valid * 2;
    }
    Ok(StudioTrack {
        values,
        last_explicit,
    })
}

mod render;

pub use render::{
    MAX_STUDIO_ASSET_POINTS, MAX_STUDIO_ASSET_TEXTURE_PIXELS, MAX_STUDIO_ASSET_TRIANGLES,
    MAX_STUDIO_BODY_PARTS, MAX_STUDIO_MESHES, MAX_STUDIO_MODELS, MAX_STUDIO_SKIN_FAMILIES,
    MAX_STUDIO_TEXTURE_DIMENSION, MAX_STUDIO_TEXTURES, MAX_STUDIO_TRIANGLES, MAX_STUDIO_VERTICES,
    StudioBodyPart, StudioCorner, StudioMesh, StudioPoint, StudioRenderAsset, StudioRenderIdentity,
    StudioSubmodel, StudioTexture,
};

#[cfg(any(test, feature = "fixtures"))]
pub mod fixture;

#[cfg(test)]
mod tests;

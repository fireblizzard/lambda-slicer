//! The shared CPU pose of a studio model, and hit queries on its bone-attached boxes.
//!
//! Evaluation restates the pinned renderer (Valve's Half-Life SDK at commit
//! `b1b5cf5892918535619b2937bb927e46cb097ba1`): `StudioSetupBones` in
//! `cl_dll/StudioModelRenderer.cpp` and the routines it calls, with the math of
//! `cl_dll/studio_util.cpp`. Every input is explicit: no clock, frame counter or
//! renderer state is read, and what the renderer derives from them is available as pure
//! helpers ([`estimate_frame`], [`player_blend`], [`interpolate_controller`]). The
//! arithmetic is `f64`, IEEE-754 operations in the order written here, with sines,
//! cosines and arc cosines from the deterministic kernels of `xenoxide-physics`.
//!
//! One [`EvaluatedPose`] supplies both rendering and hit queries: the same bone
//! transforms, bound to the model's pose and hitbox identity and this evaluator's
//! revision. The movement hull is a separate contract.

use std::f64::consts::PI;

use super::studio::{StudioDigest, StudioModel, StudioSequence};

// Lambda Slicer: the deterministic kernels of `xenoxide-physics` are its calls into `libm`
fn sin_cos(radians: f64) -> (f64, f64) {
    libm::sincos(radians)
}

fn acos(value: f64) -> f64 {
    libm::acos(value)
}

/// Changes whenever evaluation could produce different transforms for the same inputs
pub const POSE_EVALUATOR_REVISION: u32 = 1;

const STUDIO_X: i32 = 0x01;
const STUDIO_Y: i32 = 0x02;
const STUDIO_Z: i32 = 0x04;
const STUDIO_XR: i32 = 0x08;
const STUDIO_YR: i32 = 0x10;
const STUDIO_ZR: i32 = 0x20;
const STUDIO_LOOPING: i32 = 0x01;

/// One sequence's state: which, where in it, and its blend controls
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SequenceState {
    pub sequence: usize,
    /// The frame, as the renderer estimates it ([`estimate_frame`])
    pub frame: f64,
    /// The two blend controls in byte units (0 to 255), after any interpolation
    /// between updates; divided by 255 and clamped to [0, 1] when applied
    pub blend: [f64; 2],
}

/// The previous sequence during a transition and its weight: the renderer's
/// `1 - (time - sequencetime) / 0.2`, clamped to [0, 1] when applied. The caller decides
/// whether a transition is under way; the renderer blends for 0.2 s after a change.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    pub from: SequenceState,
    pub weight: f64,
}

/// A player's gait: the sequence and frame whose pose replaces the legs and the bones
/// up to the pelvis. The renderer's gait sequence 0 means none, which is `None` here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaitState {
    pub sequence: usize,
    pub frame: f64,
}

/// Everything a pose depends on besides the model and its placement
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PoseInputs {
    pub current: SequenceState,
    /// User bone controllers 0 to 3 in byte units, after any interpolation between
    /// updates ([`interpolate_controller`]). A wrapping controller's value may leave
    /// 0 to 255; it maps linearly.
    pub controllers: [f64; 4],
    /// The mouth controller's opening, 0 or more (the renderer's byte)
    pub mouth: f64,
    pub transition: Option<Transition>,
    pub gait: Option<GaitState>,
}

/// Where the model is: its origin and its angles (pitch, yaw, roll, in degrees) as the
/// renderer takes them from the entity. As `StudioSetUpTransform` does, the pitch is
/// negated for studio models.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub origin: [f64; 3],
    pub angles: [f64; 3],
}

/// A bone's world transform: three rows of a rotation and a translation column
pub type BoneTransform = [[f64; 4]; 3];

/// One evaluated pose, immutable: the world transform of every bone, and what it was
/// evaluated from
#[derive(Clone, Debug, PartialEq)]
pub struct EvaluatedPose {
    model: StudioDigest,
    evaluator: u32,
    inputs: PoseInputs,
    placement: Placement,
    bones: Vec<BoneTransform>,
}

impl EvaluatedPose {
    /// The model's pose and hitbox identity (`StudioModelIdentity::combined`)
    pub fn model(&self) -> StudioDigest {
        self.model
    }

    pub fn evaluator_revision(&self) -> u32 {
        self.evaluator
    }

    pub fn inputs(&self) -> &PoseInputs {
        &self.inputs
    }

    pub fn placement(&self) -> &Placement {
        &self.placement
    }

    /// Every bone's world transform, in bone order
    pub fn bones(&self) -> &[BoneTransform] {
        &self.bones
    }
}

/// Why a pose could not be evaluated or queried
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoseError {
    /// A sequence index past the model's sequences. The renderer resets it to 0; here
    /// it is refused.
    SequenceOutOfRange { sequence: usize },
    /// An input that is not finite, or a mouth opening below zero
    InvalidInput { input: &'static str },
    /// A hit query against a pose evaluated for another model
    ModelMismatch,
    /// Finite inputs whose arithmetic overflowed: in the controller adjustments
    /// (`"controllers"`), the pose's transforms (`"pose"`) or a hit query's segment
    /// (`"segment"`). No pose or hit is made from it.
    Overflow { stage: &'static str },
    /// A bone transform a hit query cannot invert: its determinant is not a normal
    /// nonzero number, or its inverse or a face normal is not finite
    UnusableTransform { bone: usize },
}

impl std::fmt::Display for PoseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SequenceOutOfRange { sequence } => {
                write!(formatter, "sequence {sequence} is not in the model")
            }
            Self::InvalidInput { input } => write!(formatter, "pose input {input} is invalid"),
            Self::ModelMismatch => formatter.write_str("the pose belongs to another model"),
            Self::Overflow { stage } => write!(formatter, "pose arithmetic overflowed in {stage}"),
            Self::UnusableTransform { bone } => {
                write!(formatter, "the transform of bone {bone} cannot be inverted")
            }
        }
    }
}

impl std::error::Error for PoseError {}

type Quaternion = [f64; 4];
type Vector = [f64; 3];

/// `StudioEstimateFrame` without clocks: the frame from the entity's cycle (0 to 256,
/// the renderer's `curstate.frame`) and the frames advanced since (`advance`, its
/// `(time - animtime) * framerate * fps`). A looping sequence wraps below its last
/// frame; any other is clamped to `frames - 1.001` and 0. A single-frame sequence
/// starts at 0.
pub fn estimate_frame(sequence: &StudioSequence, cycle: f64, advance: f64) -> f64 {
    let last = sequence.frames as f64 - 1.0;
    let mut frame = if sequence.frames <= 1 {
        0.0
    } else {
        cycle * last / 256.0
    };
    frame += advance;
    if sequence.flags & STUDIO_LOOPING != 0 {
        if sequence.frames > 1 {
            frame -= (frame / last).trunc() * last;
        }
        if frame < 0.0 {
            frame += last;
        }
    } else {
        if frame >= sequence.frames as f64 - 1.001 {
            frame = sequence.frames as f64 - 1.001;
        }
        if frame < 0.0 {
            frame = 0.0;
        }
    }
    frame
}

/// `StudioPlayerBlend`: a player's first blend control from the view pitch, which is
/// three times the pitch within the sequence's first blend range, mapped to 0 to 255
/// and truncated, as the renderer's integer does; and the pitch left over for the
/// model's own angles, beyond the range (0 within it)
pub fn player_blend(sequence: &StudioSequence, pitch: f64) -> (f64, f64) {
    let start = sequence.blend_start[0] as f64;
    let end = sequence.blend_end[0] as f64;
    let mut blend = (pitch * 3.0).trunc();
    let mut pitch = pitch;
    if blend < start {
        pitch -= start / 3.0;
        blend = 0.0;
    } else if blend > end {
        pitch -= end / 3.0;
        blend = 255.0;
    } else {
        blend = if end - start < 0.1 {
            127.0
        } else {
            (255.0 * (blend - start) / (end - start)).trunc()
        };
        pitch = 0.0;
    }
    (blend, pitch)
}

/// `StudioCalcBoneAdj`'s interpolation of one user controller between two updates, in
/// byte units: `fraction` of the way from `previous` to `current`. A wrapping
/// controller whose values are more than 128 apart goes the short way round, through
/// 255 and 0, and may leave 0 to 255.
///
/// The renderer reads that wrapped case from the controller arrays at the controller's
/// position in the model instead of its user index (`pcontroller1[j]` against
/// `pcontroller1[i]`), so with controllers out of index order it would interpolate
/// another controller's values. This helper takes the controller's own two values.
pub fn interpolate_controller(previous: u8, current: u8, fraction: f64, wraps: bool) -> f64 {
    let (previous, current) = (previous as i32, current as i32);
    if wraps && (current - previous).abs() > 128 {
        let a = ((current + 128) % 256) as f64;
        let b = ((previous + 128) % 256) as f64;
        a * fraction + b * (1.0 - fraction) - 128.0
    } else {
        current as f64 * fraction + previous as f64 * (1.0 - fraction)
    }
}

/// Evaluates the pose of `model` from `inputs`, placed at `placement`
pub fn evaluate_pose(
    model: &StudioModel,
    inputs: &PoseInputs,
    placement: &Placement,
) -> Result<EvaluatedPose, PoseError> {
    validate(model, inputs, placement)?;
    let adjustments = controller_adjustments(model, inputs);
    if !adjustments.iter().all(|adjustment| adjustment.is_finite()) {
        return Err(PoseError::Overflow {
            stage: "controllers",
        });
    }
    let (mut rotations, mut positions) = sequence_pose(model, &inputs.current, &adjustments);
    if let Some(transition) = inputs.transition {
        let (from_rotations, from_positions) = sequence_pose(model, &transition.from, &adjustments);
        slerp_bones(
            &mut rotations,
            &mut positions,
            &from_rotations,
            &from_positions,
            transition.weight,
        );
    }
    if let Some(gait) = inputs.gait {
        let sequence = &model.sequences()[gait.sequence];
        let (gait_rotations, gait_positions) =
            calc_rotations(model, sequence, 0, gait.frame, &adjustments);
        // From the root down to "Bip01 Spine", and from each child of "Bip01 Pelvis"
        // on, the gait's pose replaces the bone's. The renderer also reads the root's
        // parent, which does not exist; the root keeps the current state.
        let bones = model.bones();
        let mut copy = true;
        for (index, bone) in bones.iter().enumerate() {
            if bone.name == "Bip01 Spine" {
                copy = false;
            } else if bone
                .parent
                .is_some_and(|parent| bones[parent].name == "Bip01 Pelvis")
            {
                copy = true;
            }
            if copy {
                rotations[index] = gait_rotations[index];
                positions[index] = gait_positions[index];
            }
        }
    }

    let placement_matrix = placement_matrix(placement);
    let mut bones: Vec<BoneTransform> = Vec::with_capacity(model.bones().len());
    for (index, bone) in model.bones().iter().enumerate() {
        let mut matrix = quaternion_matrix(rotations[index]);
        for (row, position) in positions[index].iter().enumerate() {
            matrix[row][3] = *position;
        }
        let parent = match bone.parent {
            None => &placement_matrix,
            Some(parent) => &bones[parent],
        };
        let transform = concat_transforms(parent, &matrix);
        bones.push(transform);
    }
    // Finite inputs can still overflow, through the placement's origin or the bone
    // chain: no pose is made of what does not fit
    if !bones
        .iter()
        .flatten()
        .flatten()
        .all(|value| value.is_finite())
    {
        return Err(PoseError::Overflow { stage: "pose" });
    }
    Ok(EvaluatedPose {
        model: model.identity().combined,
        evaluator: POSE_EVALUATOR_REVISION,
        inputs: *inputs,
        placement: *placement,
        bones,
    })
}

fn validate(
    model: &StudioModel,
    inputs: &PoseInputs,
    placement: &Placement,
) -> Result<(), PoseError> {
    let sequences = model.sequences().len();
    let mut states = vec![(inputs.current.sequence, "current")];
    if let Some(transition) = inputs.transition {
        states.push((transition.from.sequence, "transition"));
    }
    if let Some(gait) = inputs.gait {
        states.push((gait.sequence, "gait"));
    }
    for (sequence, _) in states {
        if sequence >= sequences {
            return Err(PoseError::SequenceOutOfRange { sequence });
        }
    }
    let finite = |values: &[f64], input| {
        if values.iter().all(|value| value.is_finite()) {
            Ok(())
        } else {
            Err(PoseError::InvalidInput { input })
        }
    };
    let state = |state: &SequenceState, input| {
        finite(&[state.frame, state.blend[0], state.blend[1]], input)
    };
    state(&inputs.current, "current")?;
    finite(&inputs.controllers, "controllers")?;
    finite(&[inputs.mouth], "mouth")?;
    if inputs.mouth < 0.0 {
        return Err(PoseError::InvalidInput { input: "mouth" });
    }
    if let Some(transition) = inputs.transition {
        state(&transition.from, "transition")?;
        finite(&[transition.weight], "transition")?;
    }
    if let Some(gait) = inputs.gait {
        finite(&[gait.frame], "gait")?;
    }
    finite(&placement.origin, "placement")?;
    finite(&placement.angles, "placement")?;
    Ok(())
}

/// `StudioCalcBoneAdj` with the controllers already interpolated: each bone
/// controller's adjustment, in radians for rotations, in model units for translations
fn controller_adjustments(model: &StudioModel, inputs: &PoseInputs) -> Vec<f64> {
    model
        .bone_controllers()
        .iter()
        .map(|controller| {
            let (start, end) = (controller.start as f64, controller.end as f64);
            let value = if controller.index <= 3 {
                let value = inputs.controllers[controller.index as usize];
                if controller.wraps {
                    value * (360.0 / 256.0) + start
                } else {
                    let value = (value / 255.0).clamp(0.0, 1.0);
                    (1.0 - value) * start + value * end
                }
            } else {
                let value = (inputs.mouth / 64.0).min(1.0);
                (1.0 - value) * start + value * end
            };
            match controller.axis {
                STUDIO_XR | STUDIO_YR | STUDIO_ZR => value * (PI / 180.0),
                _ => value,
            }
        })
        .collect()
}

/// A sequence's pose with its blends: two blends slerped by the first control; four
/// as the renderer does, the third and fourth by the first control, then the result of
/// the first two with that by the second
fn sequence_pose(
    model: &StudioModel,
    state: &SequenceState,
    adjustments: &[f64],
) -> (Vec<Quaternion>, Vec<Vector>) {
    let sequence = &model.sequences()[state.sequence];
    let rotations_of = |blend| calc_rotations(model, sequence, blend, state.frame, adjustments);
    let (mut rotations, mut positions) = rotations_of(0);
    if sequence.animation.len() > 1 {
        let (second_rotations, second_positions) = rotations_of(1);
        slerp_bones(
            &mut rotations,
            &mut positions,
            &second_rotations,
            &second_positions,
            state.blend[0] / 255.0,
        );
        if sequence.animation.len() == 4 {
            let (mut third_rotations, mut third_positions) = rotations_of(2);
            let (fourth_rotations, fourth_positions) = rotations_of(3);
            slerp_bones(
                &mut third_rotations,
                &mut third_positions,
                &fourth_rotations,
                &fourth_positions,
                state.blend[0] / 255.0,
            );
            slerp_bones(
                &mut rotations,
                &mut positions,
                &third_rotations,
                &third_positions,
                state.blend[1] / 255.0,
            );
        }
    }
    (rotations, positions)
}

/// `StudioCalcRotations` for one blend of a sequence at `frame`: a frame past the
/// last restarts at 0, one below -0.01 is held at -0.01; the whole frame is `frame`
/// truncated, the fraction what remains (negative just below 0). Then the motion flags
/// zero the motion bone's translation on their axes. The linear motion flags add the
/// sequence's movement scaled by zero in the renderer, so they change nothing.
fn calc_rotations(
    model: &StudioModel,
    sequence: &StudioSequence,
    blend: usize,
    frame: f64,
    adjustments: &[f64],
) -> (Vec<Quaternion>, Vec<Vector>) {
    let mut frame = frame;
    if frame > sequence.frames as f64 - 1.0 {
        frame = 0.0;
    } else if frame < -0.01 {
        frame = -0.01;
    }
    let whole = frame.trunc();
    let fraction = frame - whole;
    let whole = whole as usize;
    let tracks = &sequence.animation[blend];
    let mut rotations = Vec::with_capacity(model.bones().len());
    let mut positions = Vec::with_capacity(model.bones().len());
    for (index, bone) in model.bones().iter().enumerate() {
        // StudioCalcBoneQuaterion
        let mut first = [0.0; 3];
        let mut second = [0.0; 3];
        for axis in 0..3 {
            let channel = axis + 3;
            let value = bone.value[channel] as f64;
            match &tracks[index][channel] {
                None => {
                    first[axis] = value;
                    second[axis] = value;
                }
                Some(track) => {
                    let (from, towards) = track.rotation(whole);
                    let scale = bone.scale[channel] as f64;
                    first[axis] = value + from as f64 * scale;
                    second[axis] = value + towards as f64 * scale;
                }
            }
            if let Some(controller) = bone.controllers[channel] {
                first[axis] += adjustments[controller];
                second[axis] += adjustments[controller];
            }
        }
        rotations.push(if first != second {
            quaternion_slerp(angle_quaternion(first), angle_quaternion(second), fraction)
        } else {
            angle_quaternion(first)
        });
        // StudioCalcBonePosition
        let mut position = [0.0; 3];
        for (axis, slot) in position.iter_mut().enumerate() {
            *slot = bone.value[axis] as f64;
            if let Some(track) = &tracks[index][axis] {
                let (from, towards) = track.translation(whole);
                *slot += (from as f64 * (1.0 - fraction) + fraction * towards as f64)
                    * bone.scale[axis] as f64;
            }
            if let Some(controller) = bone.controllers[axis] {
                *slot += adjustments[controller];
            }
        }
        positions.push(position);
    }
    if let Some(motion_bone) = sequence.active_motion_bone() {
        for (axis, flag) in [STUDIO_X, STUDIO_Y, STUDIO_Z].into_iter().enumerate() {
            if sequence.motion_type & flag != 0 {
                positions[motion_bone][axis] = 0.0;
            }
        }
    }
    (rotations, positions)
}

/// `StudioSlerpBones`: `weight` (clamped to [0, 1]) of the way from the first pose to
/// the second, rotations by slerp, translations linearly
fn slerp_bones(
    rotations: &mut [Quaternion],
    positions: &mut [Vector],
    other_rotations: &[Quaternion],
    other_positions: &[Vector],
    weight: f64,
) {
    let weight = weight.clamp(0.0, 1.0);
    let complement = 1.0 - weight;
    for index in 0..rotations.len() {
        rotations[index] = quaternion_slerp(rotations[index], other_rotations[index], weight);
        for axis in 0..3 {
            positions[index][axis] =
                positions[index][axis] * complement + other_positions[index][axis] * weight;
        }
    }
}

fn sin(radians: f64) -> f64 {
    sin_cos(radians).0
}

/// `AngleQuaternion`: angles about X, Y and Z, in radians
fn angle_quaternion(angles: Vector) -> Quaternion {
    let (sy, cy) = sin_cos(angles[2] * 0.5);
    let (sp, cp) = sin_cos(angles[1] * 0.5);
    let (sr, cr) = sin_cos(angles[0] * 0.5);
    [
        sr * cp * cy - cr * sp * sy,
        cr * sp * cy + sr * cp * sy,
        cr * cp * sy - sr * sp * cy,
        cr * cp * cy + sr * sp * sy,
    ]
}

/// `QuaternionSlerp`, including its choice of the nearer of `q` and `-q`, its linear
/// interpolation for nearly equal rotations and its handling of opposite ones
fn quaternion_slerp(p: Quaternion, q: Quaternion, t: f64) -> Quaternion {
    let mut q = q;
    let mut a = 0.0;
    let mut b = 0.0;
    for i in 0..4 {
        a += (p[i] - q[i]) * (p[i] - q[i]);
        b += (p[i] + q[i]) * (p[i] + q[i]);
    }
    if a > b {
        q = q.map(|component| -component);
    }
    let cosom = p[0] * q[0] + p[1] * q[1] + p[2] * q[2] + p[3] * q[3];
    let mut result = [0.0; 4];
    if 1.0 + cosom > 0.000001 {
        let (sclp, sclq) = if 1.0 - cosom > 0.000001 {
            let omega = acos(cosom);
            let sinom = sin(omega);
            (sin((1.0 - t) * omega) / sinom, sin(t * omega) / sinom)
        } else {
            (1.0 - t, t)
        };
        for i in 0..4 {
            result[i] = sclp * p[i] + sclq * q[i];
        }
    } else {
        result = [-q[1], q[0], -q[3], q[2]];
        let sclp = sin((1.0 - t) * (0.5 * PI));
        let sclq = sin(t * (0.5 * PI));
        for i in 0..3 {
            result[i] = sclp * p[i] + sclq * result[i];
        }
    }
    result
}

/// `QuaternionMatrix`, with a zero translation
fn quaternion_matrix(q: Quaternion) -> BoneTransform {
    [
        [
            1.0 - 2.0 * q[1] * q[1] - 2.0 * q[2] * q[2],
            2.0 * q[0] * q[1] - 2.0 * q[3] * q[2],
            2.0 * q[0] * q[2] + 2.0 * q[3] * q[1],
            0.0,
        ],
        [
            2.0 * q[0] * q[1] + 2.0 * q[3] * q[2],
            1.0 - 2.0 * q[0] * q[0] - 2.0 * q[2] * q[2],
            2.0 * q[1] * q[2] - 2.0 * q[3] * q[0],
            0.0,
        ],
        [
            2.0 * q[0] * q[2] - 2.0 * q[3] * q[1],
            2.0 * q[1] * q[2] + 2.0 * q[3] * q[0],
            1.0 - 2.0 * q[0] * q[0] - 2.0 * q[1] * q[1],
            0.0,
        ],
    ]
}

/// `StudioSetUpTransform`'s rotation (`AngleMatrix` of the angles, pitch negated) and
/// the origin as translation
fn placement_matrix(placement: &Placement) -> BoneTransform {
    let angles = [
        -placement.angles[0],
        placement.angles[1],
        placement.angles[2],
    ];
    let mut matrix = angle_matrix(angles);
    for (row, origin) in placement.origin.iter().enumerate() {
        matrix[row][3] = *origin;
    }
    matrix
}

/// `AngleMatrix`: pitch, yaw and roll in degrees, the rotation `(yaw * pitch) * roll`
fn angle_matrix(angles: Vector) -> BoneTransform {
    let (sy, cy) = sin_cos(angles[1] * (PI * 2.0 / 360.0));
    let (sp, cp) = sin_cos(angles[0] * (PI * 2.0 / 360.0));
    let (sr, cr) = sin_cos(angles[2] * (PI * 2.0 / 360.0));
    [
        [
            cp * cy,
            sr * sp * cy + cr * -sy,
            cr * sp * cy + -sr * -sy,
            0.0,
        ],
        [
            cp * sy,
            sr * sp * sy + cr * cy,
            cr * sp * sy + -sr * cy,
            0.0,
        ],
        [-sp, sr * cp, cr * cp, 0.0],
    ]
}

/// `ConcatTransforms`: `first` applied after `second`
fn concat_transforms(first: &BoneTransform, second: &BoneTransform) -> BoneTransform {
    let mut out = [[0.0; 4]; 3];
    for row in 0..3 {
        for column in 0..3 {
            out[row][column] = first[row][0] * second[0][column]
                + first[row][1] * second[1][column]
                + first[row][2] * second[2][column];
        }
        out[row][3] = first[row][0] * second[0][3]
            + first[row][1] * second[1][3]
            + first[row][2] * second[2][3]
            + first[row][3];
    }
    out
}

/// The nearest hitbox a segment meets
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitboxHit {
    pub hitbox: usize,
    pub bone: usize,
    /// The hitbox's hit group, as authored
    pub group: i32,
    /// How far along the segment, 0 to 1
    pub fraction: f64,
    /// Where, in world coordinates
    pub point: [f64; 3],
    /// The world normal of the face entered; zero when the segment starts inside
    pub normal: [f64; 3],
    pub starts_inside: bool,
}

/// The first of `model`'s hitboxes, posed by `pose`, that the segment from `start` to
/// `end` meets. Each box lies in its bone's space; the segment is carried there by the
/// inverse of the bone's stored transform ([`inverse`]), and met by the slab method. The
/// transform need not be orthonormal: the renderer's slerp interpolates nearly equal
/// rotations without normalizing them. The smallest fraction wins, and among equal
/// fractions the lowest hitbox index. A segment starting inside a box meets it at
/// fraction 0. A face's world normal is its local normal carried by the inverse
/// transpose, normalized. Arithmetic that overflows, or a transform that cannot be
/// inverted, is an error: no hit or miss is made from it.
pub fn trace_hitboxes(
    model: &StudioModel,
    pose: &EvaluatedPose,
    start: [f64; 3],
    end: [f64; 3],
) -> Result<Option<HitboxHit>, PoseError> {
    if pose.model != model.identity().combined || pose.bones.len() != model.bones().len() {
        return Err(PoseError::ModelMismatch);
    }
    if !start.iter().chain(&end).all(|value| value.is_finite()) {
        return Err(PoseError::InvalidInput { input: "segment" });
    }
    let overflow = PoseError::Overflow { stage: "segment" };
    let movement = [0, 1, 2].map(|axis| end[axis] - start[axis]);
    if !movement.iter().all(|value| value.is_finite()) {
        return Err(overflow);
    }
    let mut nearest: Option<(HitboxHit, Option<(usize, f64)>, [[f64; 3]; 3])> = None;
    for (index, hitbox) in model.hitboxes().iter().enumerate() {
        let transform = &pose.bones[hitbox.bone];
        let inverse =
            inverse(transform).ok_or(PoseError::UnusableTransform { bone: hitbox.bone })?;
        let local = |point: [f64; 3]| {
            let offset = [
                point[0] - transform[0][3],
                point[1] - transform[1][3],
                point[2] - transform[2][3],
            ];
            [0, 1, 2].map(|row| {
                inverse[row][0] * offset[0]
                    + inverse[row][1] * offset[1]
                    + inverse[row][2] * offset[2]
            })
        };
        let (from, to) = (local(start), local(end));
        let direction = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        if !from
            .iter()
            .chain(&to)
            .chain(&direction)
            .all(|value| value.is_finite())
        {
            return Err(overflow);
        }
        let min = hitbox.min.map(f64::from);
        let max = hitbox.max.map(f64::from);
        let Some((fraction, face)) = slab(from, direction, min, max) else {
            continue;
        };
        if nearest.is_some_and(|(nearest, _, _)| nearest.fraction <= fraction) {
            continue;
        }
        let point = [0, 1, 2].map(|axis| start[axis] + movement[axis] * fraction);
        nearest = Some((
            HitboxHit {
                hitbox: index,
                bone: hitbox.bone,
                group: hitbox.group,
                fraction,
                point,
                normal: [0.0; 3],
                starts_inside: face.is_none(),
            },
            face,
            inverse,
        ));
    }
    let Some((mut hit, face, inverse)) = nearest else {
        return Ok(None);
    };
    if !hit.point.iter().all(|value| value.is_finite()) {
        return Err(overflow);
    }
    if let Some((axis, sign)) = face {
        // The inverse transpose carries the local normal (the unit vector along `axis`,
        // with `sign`): row `axis` of the inverse
        let normal = inverse[axis].map(|component| component * sign);
        let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        if !length.is_normal() {
            return Err(PoseError::UnusableTransform { bone: hit.bone });
        }
        hit.normal = normal.map(|component| component / length);
    }
    Ok(Some(hit))
}

/// How a segment crosses one hitbox, as a box hull trace sees it: starting strictly inside
/// it (`starts_inside`), lying wholly inside it (`all_inside`), or entering it from outside
/// at `entry`, a fraction from 0 to below 1 with the world normal of the face entered.
/// A box's surface counts as outside.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitboxCrossing {
    pub hitbox: usize,
    pub bone: usize,
    pub group: i32,
    pub starts_inside: bool,
    pub all_inside: bool,
    pub entry: Option<(f64, [f64; 3])>,
}

/// How the segment from `start` to `end` crosses each of `model`'s hitboxes, posed by
/// `pose`, in hitbox order, carried into each box's bone space as [`trace_hitboxes`]
/// carries it. A box the segment neither starts in nor enters is left out. Arithmetic that
/// overflows, or a transform that cannot be inverted, is an error.
pub fn hitbox_crossings(
    model: &StudioModel,
    pose: &EvaluatedPose,
    start: [f64; 3],
    end: [f64; 3],
) -> Result<Vec<HitboxCrossing>, PoseError> {
    if pose.model != model.identity().combined || pose.bones.len() != model.bones().len() {
        return Err(PoseError::ModelMismatch);
    }
    if !start.iter().chain(&end).all(|value| value.is_finite()) {
        return Err(PoseError::InvalidInput { input: "segment" });
    }
    let overflow = PoseError::Overflow { stage: "segment" };
    let mut crossings = Vec::new();
    for (index, hitbox) in model.hitboxes().iter().enumerate() {
        let transform = &pose.bones[hitbox.bone];
        let inverse =
            inverse(transform).ok_or(PoseError::UnusableTransform { bone: hitbox.bone })?;
        let local = |point: [f64; 3]| {
            let offset = [
                point[0] - transform[0][3],
                point[1] - transform[1][3],
                point[2] - transform[2][3],
            ];
            [0, 1, 2].map(|row| {
                inverse[row][0] * offset[0]
                    + inverse[row][1] * offset[1]
                    + inverse[row][2] * offset[2]
            })
        };
        let (from, to) = (local(start), local(end));
        let direction = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        if !from
            .iter()
            .chain(&to)
            .chain(&direction)
            .all(|value| value.is_finite())
        {
            return Err(overflow);
        }
        let min = hitbox.min.map(f64::from);
        let max = hitbox.max.map(f64::from);
        let inside =
            |point: [f64; 3]| (0..3).all(|axis| min[axis] < point[axis] && point[axis] < max[axis]);
        let starts_inside = inside(from);
        let mut crossing = HitboxCrossing {
            hitbox: index,
            bone: hitbox.bone,
            group: hitbox.group,
            starts_inside,
            all_inside: starts_inside && inside(to),
            entry: None,
        };
        if !starts_inside {
            if let Some((fraction, Some((axis, sign)))) = slab(from, direction, min, max) {
                if fraction < 1.0 {
                    let normal = inverse[axis].map(|component| component * sign);
                    let length =
                        (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2])
                            .sqrt();
                    if !length.is_normal() {
                        return Err(PoseError::UnusableTransform { bone: hitbox.bone });
                    }
                    crossing.entry = Some((fraction, normal.map(|component| component / length)));
                }
            }
        }
        if crossing.starts_inside || crossing.entry.is_some() {
            crossings.push(crossing);
        }
    }
    Ok(crossings)
}

/// The inverse of a transform's 3 by 3 part, from its cofactors divided by its
/// determinant, `m = [[a, b, c], [d, e, f], [g, h, i]]`:
/// `det = a(ei - fh) - b(di - fg) + c(dh - eg)`, each entry `(cofactor) / det` in the
/// order written below. `None` when the determinant is not a normal nonzero number or
/// an entry is not finite.
fn inverse(transform: &BoneTransform) -> Option<[[f64; 3]; 3]> {
    let [a, b, c] = [transform[0][0], transform[0][1], transform[0][2]];
    let [d, e, f] = [transform[1][0], transform[1][1], transform[1][2]];
    let [g, h, i] = [transform[2][0], transform[2][1], transform[2][2]];
    let determinant = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    if !determinant.is_normal() {
        return None;
    }
    let inverse = [
        [
            (e * i - f * h) / determinant,
            (c * h - b * i) / determinant,
            (b * f - c * e) / determinant,
        ],
        [
            (f * g - d * i) / determinant,
            (a * i - c * g) / determinant,
            (c * d - a * f) / determinant,
        ],
        [
            (d * h - e * g) / determinant,
            (b * g - a * h) / determinant,
            (a * e - b * d) / determinant,
        ],
    ];
    inverse
        .iter()
        .flatten()
        .all(|value| value.is_finite())
        .then_some(inverse)
}

/// Where along `direction` from `from` (0 to 1) a box is entered, and through which
/// face (axis and outward sign), or `None` for the face when `from` is inside. An axis
/// along which the segment does not move (exactly) is met only from within its slab.
/// Among faces entered at once, the lowest axis.
fn slab(
    from: Vector,
    direction: Vector,
    min: Vector,
    max: Vector,
) -> Option<(f64, Option<(usize, f64)>)> {
    let mut enter = f64::NEG_INFINITY;
    let mut exit = f64::INFINITY;
    let mut face = None;
    for axis in 0..3 {
        if direction[axis] == 0.0 {
            if from[axis] < min[axis] || from[axis] > max[axis] {
                return None;
            }
            continue;
        }
        let near = (min[axis] - from[axis]) / direction[axis];
        let far = (max[axis] - from[axis]) / direction[axis];
        let (near, far, sign) = if near <= far {
            (near, far, -1.0)
        } else {
            (far, near, 1.0)
        };
        if near > enter {
            enter = near;
            face = Some((axis, sign));
        }
        if far < exit {
            exit = far;
        }
    }
    if enter > exit || exit < 0.0 || enter > 1.0 {
        return None;
    }
    if enter <= 0.0 {
        // Inside along every moving axis at the start, and within every still slab
        return Some((0.0, None));
    }
    Some((enter, face))
}

#[cfg(test)]
mod tests;

//! Authored studio model fixtures: small `IDST` and `IDSQ` files written byte by byte to
//! the pinned layouts, with the offsets of their records for corrupting them. Built for
//! this crate's tests and, with the `fixtures` feature, for other crates' tests.

/// One run of a track: explicit values, then the frames it covers
#[derive(Clone)]
pub struct Run {
    pub values: Vec<i16>,
    pub total: u8,
}

pub fn run(values: &[i16], total: u8) -> Run {
    Run {
        values: values.to_vec(),
        total,
    }
}

/// The runs of each track, `[blend][bone][channel]`, `None` for no track
pub type Tracks = Vec<Vec<[Option<Vec<Run>>; 6]>>;

#[derive(Clone)]
pub struct Bone {
    pub name: &'static str,
    pub parent: i32,
    pub controllers: [i32; 6],
    pub value: [f32; 6],
    pub scale: [f32; 6],
}

#[derive(Clone)]
pub struct Controller {
    pub bone: i32,
    pub kind: i32,
    pub start: f32,
    pub end: f32,
    pub rest: i32,
    pub index: i32,
}

#[derive(Clone)]
pub struct Hitbox {
    pub bone: i32,
    pub group: i32,
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Clone)]
pub struct Sequence {
    pub label: &'static str,
    pub fps: f32,
    pub frames: i32,
    pub group: i32,
    pub next: i32,
    /// `STUDIO_LOOPING` and the like
    pub flags: i32,
    pub motion_type: i32,
    pub motion_bone: i32,
    pub blend_start: [f32; 2],
    pub blend_end: [f32; 2],
    pub events: Vec<(i32, i32, i32, &'static str)>,
    pub tracks: Tracks,
}

impl Sequence {
    /// A sequence of `frames` frames at `fps` in the model itself, with no flags,
    /// motion, blend ranges or events
    pub fn new(label: &'static str, fps: f32, frames: i32, tracks: Tracks) -> Self {
        Self {
            label,
            fps,
            frames,
            group: 0,
            next: 0,
            flags: 0,
            motion_type: 0,
            motion_bone: 0,
            blend_start: [0.0; 2],
            blend_end: [0.0; 2],
            events: Vec::new(),
            tracks,
        }
    }
}

#[derive(Clone)]
pub struct Fixture {
    pub bones: Vec<Bone>,
    pub controllers: Vec<Controller>,
    pub hitboxes: Vec<Hitbox>,
    /// `(label, file name)`
    pub groups: Vec<(&'static str, &'static str)>,
    pub sequences: Vec<Sequence>,
    /// The header's texture count when no texture data is written
    pub textures: i32,
    pub body_parts: Vec<BodyPart>,
    pub texture_data: Vec<Texture>,
    /// `[family][skin reference]` texture indices
    pub skins: Vec<Vec<i16>>,
    /// Write the textures and skins to a companion file instead of the main one
    pub textures_in_companion: bool,
}

/// A texture's pixels and palette
#[derive(Clone)]
pub struct Texture {
    pub name: &'static str,
    pub flags: i32,
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<u8>,
    pub palette: Vec<[u8; 3]>,
}

impl Texture {
    /// `width` by `height` texels of palette index `index`, whose colour is `colour`;
    /// every other palette entry black
    pub fn solid(name: &'static str, width: i32, height: i32, index: u8, colour: [u8; 3]) -> Self {
        let mut palette = vec![[0; 3]; 256];
        palette[index as usize] = colour;
        Self {
            name,
            flags: 0,
            width,
            height,
            pixels: vec![index; (width * height) as usize],
            palette,
        }
    }
}

/// A mesh's triangle commands: a signed corner count (positive for a strip, negative
/// for a fan) and its corners `[vertex, normal, s, t]`. The declared triangle count is
/// the commands' unless overridden.
#[derive(Clone)]
pub struct Mesh {
    pub skin_reference: i32,
    pub commands: Vec<(i16, Vec<[i16; 4]>)>,
    pub declared: Option<i32>,
}

/// One model of a body part: its vertices and normals with their bones, and meshes
#[derive(Clone)]
pub struct Submodel {
    pub name: &'static str,
    pub vertices: Vec<([f32; 3], u8)>,
    pub normals: Vec<([f32; 3], u8)>,
    pub meshes: Vec<Mesh>,
}

#[derive(Clone)]
pub struct BodyPart {
    pub name: &'static str,
    pub base: i32,
    pub models: Vec<Submodel>,
}

/// Where the records of a built file are
pub struct Built {
    pub bytes: Vec<u8>,
    pub bones: usize,
    pub controllers: usize,
    pub hitboxes: usize,
    pub groups: usize,
    pub sequences: usize,
    /// Per sequence in this file, its animation offset blocks
    pub animation: Vec<Option<usize>>,
    /// Per sequence in this file, `[blend][bone][channel]` start of each written track
    pub track_starts: Vec<Vec<Vec<[Option<usize>; 6]>>>,
    /// Where the rendering records are, when written: the texture table, the skin
    /// table, the body part table, `[part][model]` model records, `[part][model][mesh]`
    /// mesh records and command starts, and the first byte of rendering data
    pub texture_table: usize,
    pub skin_table: usize,
    pub body_part_table: usize,
    pub models: Vec<Vec<usize>>,
    pub meshes: Vec<Vec<Vec<usize>>>,
    pub commands: Vec<Vec<Vec<usize>>>,
    pub rendering_start: usize,
}

impl Built {
    pub fn bone(&self, index: usize) -> usize {
        self.bones + index * 112
    }

    pub fn controller(&self, index: usize) -> usize {
        self.controllers + index * 24
    }

    pub fn hitbox(&self, index: usize) -> usize {
        self.hitboxes + index * 32
    }

    pub fn group(&self, index: usize) -> usize {
        self.groups + index * 104
    }

    pub fn sequence(&self, index: usize) -> usize {
        self.sequences + index * 176
    }

    pub fn put_i32(&mut self, at: usize, value: i32) {
        self.bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    pub fn put_f32(&mut self, at: usize, value: f32) {
        self.bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    pub fn put_u16(&mut self, at: usize, value: u16) {
        self.bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }

    /// Cuts the file at `length`, keeping its declared length consistent
    pub fn truncated(&self, length: usize) -> Vec<u8> {
        let mut bytes = self.bytes[..length].to_vec();
        bytes[72..76].copy_from_slice(&(length as i32).to_le_bytes());
        bytes
    }
}

#[derive(Default)]
struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn f32(&mut self, value: f32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn floats(&mut self, values: &[f32]) {
        values.iter().for_each(|value| self.f32(*value));
    }

    fn name(&mut self, name: &str, length: usize) {
        assert!(name.len() < length);
        let mut field = vec![0; length];
        field[..name.len()].copy_from_slice(name.as_bytes());
        self.bytes.extend_from_slice(&field);
    }

    fn patch(&mut self, at: usize, value: i32) {
        self.bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn len(&self) -> usize {
        self.bytes.len()
    }

    fn i16(&mut self, value: i16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    /// The texture table, each texture's pixels and palette, and the skin table; their
    /// header fields patched. The texture and skin table offsets.
    fn textures(&mut self, textures: &[Texture], skins: &[Vec<i16>]) -> (usize, usize) {
        let counts = 140;
        let table = self.len();
        self.bytes.resize(table + textures.len() * 80, 0);
        let data = self.len();
        for (index, texture) in textures.iter().enumerate() {
            let at = table + index * 80;
            let pixels = self.len();
            self.bytes.extend_from_slice(&texture.pixels);
            for colour in &texture.palette {
                self.bytes.extend_from_slice(colour);
            }
            self.bytes[at..at + texture.name.len()].copy_from_slice(texture.name.as_bytes());
            self.patch(at + 64, texture.flags);
            self.patch(at + 68, texture.width);
            self.patch(at + 72, texture.height);
            self.patch(at + 76, pixels as i32);
        }
        let skin_table = self.len();
        for family in skins {
            family.iter().for_each(|texture| self.i16(*texture));
        }
        self.patch(counts + 40, textures.len() as i32);
        self.patch(counts + 44, table as i32);
        self.patch(counts + 48, data as i32);
        self.patch(counts + 52, skins.first().map_or(0, Vec::len) as i32);
        self.patch(counts + 56, skins.len() as i32);
        self.patch(counts + 60, skin_table as i32);
        (table, skin_table)
    }

    /// The body part table and everything below it; header fields patched when there
    /// are body parts. The table offset and `[part][model]`, `[part][model][mesh]` record
    /// and command offsets.
    #[allow(clippy::type_complexity)]
    fn body_parts(
        &mut self,
        parts: &[BodyPart],
    ) -> (
        usize,
        Vec<Vec<usize>>,
        Vec<Vec<Vec<usize>>>,
        Vec<Vec<Vec<usize>>>,
    ) {
        if parts.is_empty() {
            return (0, Vec::new(), Vec::new(), Vec::new());
        }
        let counts = 140;
        let table = self.len();
        self.bytes.resize(table + parts.len() * 76, 0);
        self.patch(counts + 64, parts.len() as i32);
        self.patch(counts + 68, table as i32);
        let (mut models, mut meshes, mut commands) = (Vec::new(), Vec::new(), Vec::new());
        for (index, part) in parts.iter().enumerate() {
            let at = table + index * 76;
            self.bytes[at..at + part.name.len()].copy_from_slice(part.name.as_bytes());
            self.patch(at + 64, part.models.len() as i32);
            self.patch(at + 68, part.base);
            let model_table = self.len();
            self.patch(at + 72, model_table as i32);
            self.bytes.resize(model_table + part.models.len() * 112, 0);
            let (mut part_models, mut part_meshes, mut part_commands) =
                (Vec::new(), Vec::new(), Vec::new());
            for (model_index, model) in part.models.iter().enumerate() {
                let record = model_table + model_index * 112;
                part_models.push(record);
                self.bytes[record..record + model.name.len()]
                    .copy_from_slice(model.name.as_bytes());
                let points = |w: &mut Self, points: &[([f32; 3], u8)], count_at: usize| {
                    let info = w.len();
                    points.iter().for_each(|(_, bone)| w.bytes.push(*bone));
                    let data = w.len();
                    points.iter().for_each(|(position, _)| w.floats(position));
                    w.patch(record + count_at, points.len() as i32);
                    w.patch(record + count_at + 4, info as i32);
                    w.patch(record + count_at + 8, data as i32);
                };
                points(self, &model.vertices, 80);
                points(self, &model.normals, 92);
                let mesh_table = self.len();
                self.bytes.resize(mesh_table + model.meshes.len() * 20, 0);
                self.patch(record + 72, model.meshes.len() as i32);
                self.patch(record + 76, mesh_table as i32);
                let (mut model_meshes, mut model_commands) = (Vec::new(), Vec::new());
                for (mesh_index, mesh) in model.meshes.iter().enumerate() {
                    let mesh_at = mesh_table + mesh_index * 20;
                    model_meshes.push(mesh_at);
                    let start = self.len();
                    model_commands.push(start);
                    let mut triangles = 0;
                    for (count, corners) in &mesh.commands {
                        self.i16(*count);
                        for corner in corners {
                            corner.iter().for_each(|value| self.i16(*value));
                        }
                        triangles += corners.len().saturating_sub(2) as i32;
                    }
                    self.i16(0);
                    self.patch(mesh_at, mesh.declared.unwrap_or(triangles));
                    self.patch(mesh_at + 4, start as i32);
                    self.patch(mesh_at + 8, mesh.skin_reference);
                }
                part_meshes.push(model_meshes);
                part_commands.push(model_commands);
            }
            models.push(part_models);
            meshes.push(part_meshes);
            commands.push(part_commands);
        }
        (table, models, meshes, commands)
    }

    /// Animation offset blocks for `tracks`, then every written track; the block
    /// offsets and track starts
    fn animation(
        &mut self,
        tracks: &Tracks,
        bones: usize,
    ) -> (usize, Vec<Vec<[Option<usize>; 6]>>) {
        let start = self.len();
        self.bytes.resize(start + tracks.len() * bones * 12, 0);
        let mut starts = Vec::new();
        for (blend, bone_tracks) in tracks.iter().enumerate() {
            let mut blend_starts = Vec::new();
            for (bone, channels) in bone_tracks.iter().enumerate() {
                let block = start + (blend * bones + bone) * 12;
                let mut bone_starts = [None; 6];
                for (channel, runs) in channels.iter().enumerate() {
                    let Some(runs) = runs else { continue };
                    let at = self.len();
                    let relative = u16::try_from(at - block).expect("fixture fits");
                    self.bytes[block + channel * 2..block + channel * 2 + 2]
                        .copy_from_slice(&relative.to_le_bytes());
                    for run in runs {
                        self.bytes.push(run.values.len() as u8);
                        self.bytes.push(run.total);
                        for value in &run.values {
                            self.bytes.extend_from_slice(&value.to_le_bytes());
                        }
                    }
                    bone_starts[channel] = Some(at);
                }
                blend_starts.push(bone_starts);
            }
            starts.push(blend_starts);
        }
        (start, starts)
    }
}

impl Fixture {
    /// Three bones in a chain, a wrapping yaw controller on the second, a head box on
    /// the third and a chest box on the second, and two sequences in the model itself:
    /// `idle`, one blend of four frames, and `aim`, two blends of two
    pub fn standard() -> Self {
        let bone = |name, parent, controllers| Bone {
            name,
            parent,
            controllers,
            value: [0.0, 0.0, 10.0, 0.0, 0.0, 0.0],
            scale: [0.5, 0.5, 0.5, 0.001, 0.001, 0.001],
        };
        let none = [-1; 6];
        let mut yaw = none;
        yaw[4] = 0;
        let idle = vec![vec![
            [
                Some(vec![run(&[10, 20], 3), run(&[30], 1)]),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                None,
                None,
                None,
                None,
                Some(vec![run(&[5, 6, 7, 8], 4)]),
                None,
            ],
            // Runs ending at their last explicit value, for a translation and a
            // rotation, beside one run of the same values
            [
                Some(vec![run(&[5], 1), run(&[6], 1), run(&[7, 8], 2)]),
                Some(vec![run(&[5, 6, 7, 8], 4)]),
                None,
                None,
                None,
                Some(vec![run(&[5], 1), run(&[6], 1), run(&[7, 8], 2)]),
            ],
        ]];
        let pitch = |values: &[i16]| {
            vec![
                [None, None, None, Some(vec![run(values, 2)]), None, None],
                [None, None, None, None, None, None],
                [None, None, None, None, None, None],
            ]
        };
        Self {
            bones: vec![
                bone("Bip01", -1, none),
                bone("Bip01 Spine", 0, yaw),
                bone("Bip01 Head", 1, none),
            ],
            controllers: vec![Controller {
                bone: 1,
                kind: 0x10 | 0x8000,
                start: -90.0,
                end: 90.0,
                rest: 0,
                index: 0,
            }],
            hitboxes: vec![
                Hitbox {
                    bone: 2,
                    group: 1,
                    min: [-4.0, -4.0, -2.0],
                    max: [4.0, 4.0, 6.0],
                },
                Hitbox {
                    bone: 1,
                    group: 2,
                    min: [-8.0, -6.0, -4.0],
                    max: [8.0, 6.0, 12.0],
                },
            ],
            groups: vec![("default", "")],
            sequences: vec![
                Sequence {
                    label: "idle",
                    fps: 15.0,
                    frames: 4,
                    group: 0,
                    next: 0,
                    flags: 0,
                    motion_type: 0,
                    motion_bone: 0,
                    blend_start: [0.0; 2],
                    blend_end: [0.0; 2],
                    events: vec![(1, 5001, 0, "30")],
                    tracks: idle,
                },
                Sequence {
                    label: "aim",
                    fps: 30.0,
                    frames: 2,
                    group: 0,
                    next: 1,
                    flags: 0,
                    motion_type: 0,
                    motion_bone: 0,
                    blend_start: [0.0; 2],
                    blend_end: [0.0; 2],
                    events: Vec::new(),
                    tracks: vec![pitch(&[-100, -200]), pitch(&[100, 200])],
                },
            ],
            textures: 0,
            body_parts: Vec::new(),
            texture_data: Vec::new(),
            skins: Vec::new(),
            textures_in_companion: false,
        }
    }

    /// The standard fixture with a third sequence, `walk`, whose animation is in an
    /// external sequence group, `models\fixture01.mdl`
    pub fn with_external_group() -> Self {
        let mut fixture = Self::standard();
        fixture.groups.push(("walk", "models\\fixture01.mdl"));
        fixture.sequences.push(Sequence {
            label: "walk",
            fps: 24.0,
            frames: 3,
            group: 1,
            next: 2,
            flags: 0,
            motion_type: 0,
            motion_bone: 0,
            blend_start: [0.0; 2],
            blend_end: [0.0; 2],
            events: Vec::new(),
            tracks: vec![vec![
                [None, Some(vec![run(&[1, 2, 3], 3)]), None, None, None, None],
                [None, None, None, None, None, None],
                [None, None, None, None, None, None],
            ]],
        });
        fixture
    }

    /// The main file. Records come in header order and the animation of the model's own
    /// sequences last, so cutting the file anywhere removes something it refers to.
    pub fn build(&self) -> Built {
        let mut w = Writer::default();
        let bones = self.bones.len();
        w.bytes.extend_from_slice(b"IDST");
        w.i32(10);
        w.name("fixture.mdl", 64);
        w.i32(0);
        w.floats(&[0.0, 0.0, 28.0]);
        w.floats(&[-16.0, -16.0, -36.0, 16.0, 16.0, 36.0]);
        w.floats(&[-20.0, -20.0, -40.0, 20.0, 20.0, 40.0]);
        w.i32(0);
        let counts = 140;
        w.bytes.resize(244, 0);
        w.patch(counts, bones as i32);
        w.patch(counts + 8, self.controllers.len() as i32);
        w.patch(counts + 16, self.hitboxes.len() as i32);
        w.patch(counts + 24, self.sequences.len() as i32);
        w.patch(counts + 32, self.groups.len() as i32);
        w.patch(counts + 40, self.textures);

        let bone_offset = w.len();
        w.patch(counts + 4, bone_offset as i32);
        for bone in &self.bones {
            w.name(bone.name, 32);
            w.i32(bone.parent);
            w.i32(0);
            bone.controllers
                .iter()
                .for_each(|controller| w.i32(*controller));
            w.floats(&bone.value);
            w.floats(&bone.scale);
        }
        let controller_offset = w.len();
        w.patch(counts + 12, controller_offset as i32);
        for controller in &self.controllers {
            w.i32(controller.bone);
            w.i32(controller.kind);
            w.f32(controller.start);
            w.f32(controller.end);
            w.i32(controller.rest);
            w.i32(controller.index);
        }
        let hitbox_offset = w.len();
        w.patch(counts + 20, hitbox_offset as i32);
        for hitbox in &self.hitboxes {
            w.i32(hitbox.bone);
            w.i32(hitbox.group);
            w.floats(&hitbox.min);
            w.floats(&hitbox.max);
        }
        let group_offset = w.len();
        w.patch(counts + 36, group_offset as i32);
        for (label, name) in &self.groups {
            w.name(label, 32);
            w.name(name, 64);
            w.i32(0);
            w.i32(0);
        }
        let sequence_offset = w.len();
        w.patch(counts + 28, sequence_offset as i32);
        w.bytes
            .resize(sequence_offset + self.sequences.len() * 176, 0);
        let mut animation = Vec::new();
        let mut track_starts = Vec::new();
        for (index, sequence) in self.sequences.iter().enumerate() {
            let at = sequence_offset + index * 176;
            let event_offset = w.len();
            for (frame, event, kind, options) in &sequence.events {
                w.i32(*frame);
                w.i32(*event);
                w.i32(*kind);
                w.name(options, 64);
            }
            w.bytes[at..at + 32].fill(0);
            w.bytes[at..at + sequence.label.len()].copy_from_slice(sequence.label.as_bytes());
            w.bytes[at + 32..at + 36].copy_from_slice(&sequence.fps.to_le_bytes());
            w.patch(at + 48, sequence.events.len() as i32);
            w.patch(at + 52, event_offset as i32);
            w.patch(at + 56, sequence.frames);
            w.patch(at + 120, sequence.tracks.len() as i32);
            w.patch(at + 156, sequence.group);
            w.patch(at + 172, sequence.next);
            w.patch(at + 36, sequence.flags);
            w.patch(at + 72, sequence.motion_type);
            w.patch(at + 76, sequence.motion_bone);
            for (slot, value) in sequence.blend_start.iter().enumerate() {
                w.bytes[at + 136 + slot * 4..at + 140 + slot * 4]
                    .copy_from_slice(&value.to_le_bytes());
            }
            for (slot, value) in sequence.blend_end.iter().enumerate() {
                w.bytes[at + 144 + slot * 4..at + 148 + slot * 4]
                    .copy_from_slice(&value.to_le_bytes());
            }
        }
        for (index, sequence) in self.sequences.iter().enumerate() {
            if sequence.group == 0 {
                let (start, starts) = w.animation(&sequence.tracks, bones);
                w.patch(sequence_offset + index * 176 + 124, start as i32);
                animation.push(Some(start));
                track_starts.push(starts);
            } else {
                animation.push(None);
                track_starts.push(Vec::new());
            }
        }
        let rendering_start = w.len();
        let (texture_table, skin_table) =
            if !self.texture_data.is_empty() && !self.textures_in_companion {
                w.textures(&self.texture_data, &self.skins)
            } else {
                (0, 0)
            };
        let (body_part_table, models, meshes, commands) = w.body_parts(&self.body_parts);
        let length = w.len() as i32;
        w.patch(72, length);
        let mut built = Built {
            bytes: w.bytes,
            bones: bone_offset,
            controllers: controller_offset,
            hitboxes: hitbox_offset,
            groups: group_offset,
            sequences: sequence_offset,
            animation,
            track_starts,
            texture_table,
            skin_table,
            body_part_table,
            models,
            meshes,
            commands,
            rendering_start,
        };
        // The external sequences' animation offsets are into their group files
        for (index, sequence) in self.sequences.iter().enumerate() {
            if sequence.group != 0 {
                let offset = self.group_layout(sequence.group, index);
                built.put_i32(sequence_offset + index * 176 + 124, offset as i32);
            }
        }
        built
    }

    /// Where sequence `target`'s animation lands in the file of `group`
    fn group_layout(&self, group: i32, target: usize) -> usize {
        let mut at = 76;
        for (index, sequence) in self.sequences.iter().enumerate() {
            if sequence.group != group {
                continue;
            }
            if index == target {
                return at;
            }
            let mut w = Writer::default();
            w.animation(&sequence.tracks, self.bones.len());
            at += w.len();
        }
        unreachable!("the sequence is in the group")
    }

    /// The external sequence-group file (`IDSQ`) of `group`
    /// The texture companion (`<name>T.mdl`): a studio header with only the textures
    /// and skins
    pub fn build_texture_companion(&self) -> Built {
        let mut w = Writer::default();
        w.bytes.extend_from_slice(b"IDST");
        w.i32(10);
        w.name("fixturet.mdl", 64);
        w.i32(0);
        w.bytes.resize(244, 0);
        let rendering_start = w.len();
        let (texture_table, skin_table) = w.textures(&self.texture_data, &self.skins);
        let length = w.len() as i32;
        w.patch(72, length);
        Built {
            bytes: w.bytes,
            bones: 0,
            controllers: 0,
            hitboxes: 0,
            groups: 0,
            sequences: 0,
            animation: Vec::new(),
            track_starts: Vec::new(),
            texture_table,
            skin_table,
            body_part_table: 0,
            models: Vec::new(),
            meshes: Vec::new(),
            commands: Vec::new(),
            rendering_start,
        }
    }

    pub fn build_group(&self, group: i32) -> Built {
        let mut w = Writer::default();
        w.bytes.extend_from_slice(b"IDSQ");
        w.i32(10);
        w.name("fixture01.mdl", 64);
        w.i32(0);
        let mut animation = Vec::new();
        let mut track_starts = Vec::new();
        for sequence in &self.sequences {
            if sequence.group == group {
                let (start, starts) = w.animation(&sequence.tracks, self.bones.len());
                animation.push(Some(start));
                track_starts.push(starts);
            }
        }
        let length = w.len() as i32;
        w.patch(72, length);
        Built {
            bytes: w.bytes,
            bones: 0,
            controllers: 0,
            hitboxes: 0,
            groups: 0,
            sequences: 0,
            animation,
            track_starts,
            texture_table: 0,
            skin_table: 0,
            body_part_table: 0,
            models: Vec::new(),
            meshes: Vec::new(),
            commands: Vec::new(),
            rendering_start: 0,
        }
    }
}

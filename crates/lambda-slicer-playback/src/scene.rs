//! What the recording client could show at a moment, built record by record
//!
//! Entities come from the decoder's newest stored snapshot, so a packet the
//! client rejected never reaches the scene. An entity's identity is its
//! session, its number and its life. A life ends when the entity leaves the
//! packet or its model changes, so a reused slot is a different entity and
//! is never interpolated from the one before. A new session starts an empty
//! scene.
//!
//! Entities are presented as the client draws them, [`INTERPOLATION`]
//! seconds behind its clock, each moved along its own samples from the last
//! snapshots. A new life or a move longer than [`TELEPORT`] is not
//! interpolated through.
//!
//! A player's legs follow how far it moved, as the game's client estimates
//! it once a frame from the drawn position (`StudioEstimateGait` and
//! `StudioProcessGait`). The scene steps that once a record group, each
//! group being one frame the recording client drew, so the state is the
//! same however a moment is reached. What depends on the model, the gait
//! frame and the aim blend, is left to the renderer, which has the model.
//!
//! The camera is the view the client rendered when it wrote each network
//! record, from the record's `ref_params_t`, with the field of view of the
//! last client data record. Its angles carry the kick of shots and hits
//! and its height the recorder's bob, as the client added them
//! (`V_CalcNormalRefdef`), so the weapon in view can be placed as the
//! client placed it, from the aim without the kick and with the same bob

use std::collections::VecDeque;

use std::collections::BTreeMap;
use std::sync::Arc;

use lambda_slicer_core::container::{ClientDataRecord, FrameBody, NetworkFrame};
use lambda_slicer_core::protocol::delta::{DeltaSchema, convert};
use lambda_slicer_core::protocol::messages::{Message, ParsedMessage, resource_type};
use lambda_slicer_core::state::{Decoder, EntityState, Snapshot};

use crate::Table;
use crate::animation::IN_EYE;
use crate::time::{ServerTime, SourceTime};
use crate::trace::{Hull, SOLID};

/// The client's `ex_interp` in seconds, how far behind its latest snapshot
/// it draws entities
pub const INTERPOLATION: f64 = 0.1;

/// A move longer than this between two snapshots is drawn as a jump
pub const TELEPORT: f32 = 128.;

/// Snapshots kept for interpolation, more than 0.1 s of any server's
const RECENT: usize = 32;

/// A player's legs, as the game's client keeps them, without the model
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Gait {
    /// The way the legs face, in degrees
    pub yaw: f32,
    /// The torso controller value, 0 to 255, turning the upper body from
    /// the legs to the view
    pub torso: f32,
    /// Units moved since the player's life began, counted back while
    /// walking backwards
    pub distance: f64,
    /// Seconds since the player's life began
    pub elapsed: f64,
}

#[derive(Clone, Copy, Debug)]
struct GaitTrack {
    life: u32,
    gait: Gait,
    last_origin: [f32; 3],
}

/// `x - (int)(x / 360) * 360`, the game's wrap of an angle
fn wrap(x: f32) -> f32 {
    x - (x / 360.).trunc() * 360.
}

/// An angle wrapped into -180 to 180
fn half_turn(x: f32) -> f32 {
    let x = wrap(x);
    if x > 180. {
        x - 360.
    } else if x < -180. {
        x + 360.
    } else {
        x
    }
}

impl GaitTrack {
    /// One client frame of `dt` seconds with the player drawn at `origin`
    /// facing `yaw`
    fn step(&mut self, origin: [f32; 3], yaw: f32, dt: f32) {
        let gait = &mut self.gait;

        // StudioEstimateGait
        let mut movement = 0.;
        if dt > 0. {
            let mut velocity = [0, 1, 2].map(|k| origin[k] - self.last_origin[k]);
            self.last_origin = origin;
            movement = (velocity[0].powi(2) + velocity[1].powi(2) + velocity[2].powi(2)).sqrt();
            if movement / dt < 5. {
                movement = 0.;
                velocity[0] = 0.;
                velocity[1] = 0.;
            }
            if velocity[0] == 0. && velocity[1] == 0. {
                let difference = half_turn(yaw - gait.yaw);
                let scale = if dt < 0.25 { dt * 4. } else { dt };
                gait.yaw = wrap(gait.yaw + difference * scale);
                movement = 0.;
            } else {
                gait.yaw = velocity[1]
                    .atan2(velocity[0])
                    .to_degrees()
                    .clamp(-180., 180.);
            }
        }

        // StudioProcessGait, turning the legs round when the body faces
        // more than 120 degrees from them
        let mut turn = half_turn(yaw - gait.yaw);
        if turn > 120. {
            gait.yaw -= 180.;
            movement = -movement;
            turn -= 180.;
        } else if turn < -120. {
            gait.yaw += 180.;
            movement = -movement;
            turn += 180.;
        }
        gait.torso = (turn / 4. + 30.) / (60. / 255.);
        gait.distance += f64::from(movement);
        gait.elapsed += f64::from(dt);
    }
}

/// Which entity, across sessions and slot reuse
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId {
    pub segment: u32,
    pub number: u16,
    pub life: u32,
}

/// One entity as the newest snapshot holding it left it. Fields a session's
/// schema does not send are 0
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Entity {
    pub id: EntityId,
    /// A beam the server draws (a custom entity, `env_beam` and the like),
    /// running from its origin to the point its angles hold, its render
    /// mode its beam type, its sequence and skin naming the entities it
    /// may start and end at, its body its noise and its scale its width
    pub beam: bool,
    pub model_index: i64,
    /// The precached model's name, none when the resource list lacks it
    pub model: Option<Arc<str>>,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub sequence: i64,
    pub gait_sequence: i64,
    pub frame: f32,
    pub framerate: f32,
    pub animtime: f32,
    pub body: i64,
    pub skin: i64,
    pub controllers: [f32; 4],
    pub blending: [f32; 2],
    pub effects: i64,
    pub render_mode: i64,
    pub render_amount: i64,
    pub render_fx: i64,
    pub render_color: [u8; 3],
    pub scale: f32,
    pub move_type: i64,
    pub solid: i64,
    /// A player's observer mode and the player it observes
    pub iuser1: i64,
    pub iuser2: i64,
    /// A player's weapon model index and name
    pub weapon_model: i64,
    pub weapon: Option<Arc<str>>,
    /// A player's hull, 1 while they crouch
    pub use_hull: i64,
    /// A player's legs, in presented entities whose gait sequence is set
    pub gait: Option<Gait>,
    /// The sequence it last changed from, while it keeps its life
    pub change: Option<SequenceChange>,
    /// Server time of the snapshot it was last taken from
    pub updated: ServerTime,
}

/// The sequence an entity left for its current one, which the game's
/// renderer fades from for a moment, as the client latches it
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SequenceChange {
    pub sequence: i64,
    pub frame: f32,
    pub framerate: f32,
    pub animtime: f32,
    pub blending: [f32; 2],
    /// The client's time when the change arrived, the last the old
    /// sequence was drawn at, where its frame then stays
    pub held_at: f64,
    /// The new sequence's `animtime`, when the fade begins
    pub at: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Camera {
    pub origin: [f32; 3],
    /// The recorder's own origin, the middle of their box standing or
    /// crouched, as the frame recorded it
    pub player_origin: [f32; 3],
    pub angles: [f32; 3],
    /// The recorder's own aim, without the kick `angles` has
    pub aim: [f32; 3],
    /// How far the recorder's bob raised the view, 0 for a recorder whose
    /// `cl_bob` is 0, from the view's height over their eyes'
    pub bob: f32,
    /// From client data records, which some recordings lack
    pub fov: Option<f32>,
    pub view_entity: Option<u16>,
    pub spectating: bool,
    pub intermission: bool,
    /// The recorder's observer mode and the player it observes, from its
    /// client data as the game's client takes them. 0 when not observing
    pub observer_mode: i64,
    pub observer_target: i64,
    /// The recorder's first-person weapon model and health, from its client
    /// data
    pub viewmodel: Option<Arc<str>>,
    pub health: i64,
    /// The weapons the recorder owns, a bit for each weapon's id and the
    /// suit's, from its client data
    pub weapons: u32,
    /// The recorder's velocity and whether they stand on the ground, as the
    /// frame recorded them
    pub velocity: [f32; 3],
    pub on_ground: bool,
    /// The recorder's velocity as they last left the ground going up, as a
    /// jump does, none before the first of the session
    pub last_jump: Option<[f32; 3]>,
    /// The recorder's frames a second, their frame times averaged over
    /// each [`FPS_SPAN`] of them, none before their first
    pub fps: Option<f32>,
    /// Whether the recorder holds the key that shows the scoreboard, as the
    /// commands the demo recorded say
    pub scores_held: bool,
}

/// The lowest and highest the game's bob goes (`V_CalcBob`)
pub const LOWEST_BOB: f32 = -7.;
pub const HIGHEST_BOB: f32 = 4.;

/// How much of the recorder's frame time each fps reading averages, so it
/// holds still long enough to read, as the game's own counter does
pub const FPS_SPAN: f32 = 0.5;

/// A frame time this long or longer is a pause or a load, not a frame
const LONGEST_FRAME: f32 = 1.;

/// A player of the session, with what its userinfo names for drawing it
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerView {
    pub slot: u8,
    /// Its entity number, one more than the slot
    pub entity: u16,
    pub name: String,
    /// The `model` key, such as `gordon`, none when unset
    pub model: Option<String>,
    pub top_color: Option<u8>,
    pub bottom_color: Option<u8>,
}

/// The value of `key` in a `\key\value` userinfo string
fn userinfo<'a>(info: &'a [u8], key: &[u8]) -> Option<&'a [u8]> {
    let info = info.strip_prefix(b"\\").unwrap_or(info);
    let mut parts = info.split(|&b| b == b'\\');
    while let (Some(k), Some(v)) = (parts.next(), parts.next()) {
        if k == key {
            return Some(v);
        }
    }
    None
}

/// The session's players as the decoder knows them now
pub fn players(decoder: &Decoder) -> Vec<PlayerView> {
    let text = |v: &[u8]| String::from_utf8_lossy(v).into_owned();
    let color = |v: &[u8]| std::str::from_utf8(v).ok()?.trim().parse::<u8>().ok();
    decoder
        .players
        .values()
        .map(|p| PlayerView {
            slot: p.slot,
            entity: u16::from(p.slot) + 1,
            name: p.name.clone(),
            model: userinfo(&p.info, b"model")
                .filter(|m| !m.is_empty())
                .map(text),
            top_color: userinfo(&p.info, b"topcolor").and_then(color),
            bottom_color: userinfo(&p.info, b"bottomcolor").and_then(color),
        })
        .collect()
}

/// The first-person weapon's animation, from the last weapon animation
/// record or message
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewmodel {
    pub sequence: i32,
    pub body: i32,
    pub started: SourceTime,
}

#[derive(Clone, Debug, Default)]
pub struct Scene {
    pub segment: u32,
    /// From the session's server info, `maps/<name>.bsp`
    pub map: Option<String>,
    /// Ordered by number
    pub entities: Arc<Vec<Entity>>,
    pub camera: Camera,
    pub viewmodel: Option<Viewmodel>,
    /// The weapon of the player a spectator watches through the eyes of
    watched_weapon: Option<Arc<str>>,
    /// The recorder's frame times since the last fps reading, added up, and
    /// how many
    frame_times: (f32, u32),
    pub light_styles: BTreeMap<u8, Vec<u8>>,
    /// The sky the server names in its movement variables (`sv_skyname`),
    /// which the game draws whatever the map's worldspawn names
    pub sky: Option<Arc<str>>,
    /// Grows whenever anything here changes, so a renderer can skip work
    pub revision: u64,
    /// The snapshot the entities were taken from
    snapshot: Option<Arc<Snapshot>>,
    /// Each slot's current or last life, by entity number
    lives: Vec<Option<Slot>>,
    /// Model names by precache index, and the resource count they were
    /// read from
    models: Arc<Table<u16, Arc<str>>>,
    models_from: usize,
    /// Field positions per schema
    tables: Vec<FieldTable>,
    /// The state each of `entities` was converted from, in the same order,
    /// which snapshots share while the entity is unchanged
    sources: Vec<Arc<EntityState>>,
    /// The latest snapshots' entities by server time, oldest first
    recent: VecDeque<(f64, Arc<Vec<Entity>>)>,
    /// When the latest snapshot arrived, and its server time
    snapshot_at: SourceTime,
    server_time: f64,
    max_clients: u8,
    gaits: Table<u16, GaitTrack>,
    /// The last client frame the legs were stepped for
    last_frame: Option<SourceTime>,
    /// Each player's movement as the snapshots show it, by entity number
    pub movement: BTreeMap<u16, Movement>,
}

/// A player's movement worked out from how far they moved between the last
/// two snapshots, as the demo holds no speed but the recorder's
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Movement {
    /// Units a second, none while the last two snapshots did not both
    /// have them in the same life, or they moved too far to have run it
    pub velocity: Option<[f32; 3]>,
    /// Their velocity as they last rose from level as a jump does, none
    /// before the first of the session
    pub last_jump: Option<[f32; 3]>,
    /// They were level, or have only begun to rise since, as a speed
    /// measured over a span rises over a few snapshots
    from_level: bool,
}

/// Fastest a player is taken to move between snapshots, past which they
/// were moved, as by a respawn or a teleporter
const MOST_SPEED: f32 = 3500.;

/// The shortest span of server time a player's speed is measured over
const MEASURED_OVER: f64 = 0.1;

/// A rise from level counts as a jump when it is within these, in units a
/// second up. A jump starts at 268 and slows by gravity, and a stair's
/// step rises faster
const LEVEL: f32 = 60.;
const JUMP_RISE: (f32, f32) = (150., 300.);

/// A slot's current or last life
#[derive(Clone, Copy, Debug, Default)]
struct Slot {
    life: u32,
    model: i64,
}

/// The entity fields the scene reads, in this order
const FIELDS: [&str; 34] = [
    "modelindex",
    "origin[0]",
    "origin[1]",
    "origin[2]",
    "angles[0]",
    "angles[1]",
    "angles[2]",
    "sequence",
    "gaitsequence",
    "frame",
    "framerate",
    "animtime",
    "body",
    "skin",
    "controller[0]",
    "controller[1]",
    "controller[2]",
    "controller[3]",
    "blending[0]",
    "blending[1]",
    "effects",
    "rendermode",
    "renderamt",
    "renderfx",
    "scale",
    "movetype",
    "solid",
    "iuser1",
    "iuser2",
    "rendercolor.r",
    "rendercolor.g",
    "rendercolor.b",
    "weaponmodel",
    "usehull",
];

/// Where each of [`FIELDS`] sits in one schema, looked up once
#[derive(Clone, Debug)]
struct FieldTable {
    schema: Arc<DeltaSchema>,
    /// The offset of each field the schema sends and its place in
    /// [`FIELDS`], ordered by offset as a state's values are
    at: Vec<(u32, usize)>,
}

impl FieldTable {
    fn new(schema: Arc<DeltaSchema>) -> Self {
        let mut at = Vec::new();
        for (slot, name) in FIELDS.iter().enumerate() {
            if let Some(index) = schema.field_index(name) {
                at.push((schema.fields[index].offset, slot));
            }
        }
        at.sort_unstable();

        Self { schema, at }
    }

    /// Every field's value, 0 when the schema does not send it or the state
    /// has none, read in one pass over the state's values
    fn read(&self, e: &EntityState) -> [f64; FIELDS.len()] {
        let mut values = [0.; FIELDS.len()];

        // Both lists are ordered by offset, so one walk finds every match
        let mut next = 0;
        for (field, wire) in e.fields.iter() {
            while next < self.at.len() && self.at[next].0 < field.offset {
                next += 1;
            }
            let Some(&(offset, slot)) = self.at.get(next) else {
                break;
            };
            if offset == field.offset {
                values[slot] = convert(field, wire).as_f64().unwrap_or(0.);
                next += 1;
            }
        }

        values
    }
}

/// Each weapon's model in a player's hand and in view
const WEAPON_MODELS: [(&str, &str); 15] = [
    ("models/p_crossbow.mdl", "models/v_crossbow.mdl"),
    ("models/p_crowbar.mdl", "models/v_crowbar.mdl"),
    ("models/p_egon.mdl", "models/v_egon.mdl"),
    ("models/p_gauss.mdl", "models/v_gauss.mdl"),
    ("models/p_9mmhandgun.mdl", "models/v_9mmhandgun.mdl"),
    ("models/p_grenade.mdl", "models/v_grenade.mdl"),
    ("models/p_hgun.mdl", "models/v_hgun.mdl"),
    ("models/p_9mmAR.mdl", "models/v_9mmAR.mdl"),
    ("models/p_357.mdl", "models/v_357.mdl"),
    ("models/p_rpg.mdl", "models/v_rpg.mdl"),
    ("models/p_shotgun.mdl", "models/v_shotgun.mdl"),
    ("models/p_squeak.mdl", "models/v_squeak.mdl"),
    ("models/p_tripmine.mdl", "models/v_tripmine.mdl"),
    ("models/p_satchel_radio.mdl", "models/v_satchel_radio.mdl"),
    ("models/p_satchel.mdl", "models/v_satchel.mdl"),
];

/// The viewmodel of a player's weapon model, as the client finds it for
/// a spectator watching through the player's eyes
/// (`V_FindViewModelByWeaponModel`)
pub fn view_model_for(weapon: &str) -> Option<&'static str> {
    WEAPON_MODELS
        .iter()
        .find(|(p, _)| p.eq_ignore_ascii_case(weapon))
        .map(|(_, v)| *v)
}

/// The model in a player's hand of a viewmodel, the other way round
pub fn weapon_model_for(viewmodel: &str) -> Option<&'static str> {
    WEAPON_MODELS
        .iter()
        .find(|(_, v)| v.eq_ignore_ascii_case(viewmodel))
        .map(|(p, _)| *p)
}

/// The sky's name in an `svc_newmovevars` body, a string after 16 floats,
/// the footsteps byte and 8 more floats. None when it names none
fn sky_name(body: &[u8]) -> Option<Arc<str>> {
    let name = body.get(97..)?;
    let end = name.iter().position(|b| *b == 0).unwrap_or(name.len());
    let name = String::from_utf8_lossy(&name[..end]);
    let name = name.trim();
    (!name.is_empty()).then(|| Arc::from(name))
}

impl Scene {
    fn changed(&mut self) {
        self.revision += 1;
    }

    /// A message the decoder has just applied, at `time`
    pub(crate) fn on_message(&mut self, decoder: &Decoder, m: &ParsedMessage, time: SourceTime) {
        match &m.message {
            Message::ServerInfo(info) => {
                *self = Self {
                    segment: decoder.segment,
                    map: Some(String::from_utf8_lossy(&info.map_name).into_owned()),
                    revision: self.revision + 1,
                    ..Self::default()
                };
            }
            Message::LightStyle { index, style } => {
                self.light_styles.insert(*index, style.clone());
                self.changed();
            }
            Message::NewMoveVars(body) => {
                let sky = sky_name(body);
                if sky != self.sky {
                    self.sky = sky;
                    self.changed();
                }
            }
            Message::WeaponAnim { sequence, body } => {
                self.viewmodel = Some(Viewmodel {
                    sequence: i32::from(*sequence),
                    body: i32::from(*body),
                    started: time,
                });
                self.changed();
            }
            _ => {}
        }
    }

    /// After a network record at `time`, its camera and any new snapshot
    pub(crate) fn after_network(
        &mut self,
        decoder: &Decoder,
        frame: &NetworkFrame,
        time: SourceTime,
    ) {
        let client = |field: &str| {
            let schema = decoder.schema("clientdata_t")?;
            let data = decoder.latest_snapshot()?.clientdata.clone()?;
            data.value(&schema, field)?.as_i64()
        };
        let mut viewmodel = client("viewmodel").and_then(|i| self.model_name(decoder, i));
        let observer_mode = client("iuser1").unwrap_or(0);
        let observer_target = client("iuser2").unwrap_or(0);
        if viewmodel.is_none() && observer_mode == IN_EYE {
            // Through a player's eyes the client draws the watched player's
            // weapon, which the spectator's own client data does not name
            let weapon = self
                .entities
                .iter()
                .find(|e| i64::from(e.id.number) == observer_target)
                .and_then(|e| e.weapon.clone());
            if weapon != self.watched_weapon {
                // A new weapon starts on its first animation
                self.watched_weapon = weapon.clone();
                self.viewmodel = Some(Viewmodel {
                    sequence: 0,
                    body: 0,
                    started: time,
                });
            }
            viewmodel = weapon.as_deref().and_then(view_model_for).map(Arc::from);
        }
        // Leaving the ground going up is a jump, its speed kept until the
        // next one
        let velocity = frame.info_simvel();
        let on_ground = frame.info_onground() != 0;
        let jumped = self.camera.on_ground && !on_ground && velocity[2] > 0.;
        let last_jump = if jumped {
            Some(velocity)
        } else {
            self.camera.last_jump
        };

        // The first frame time read gives a reading at once, and later ones
        // a new reading every half second of them
        let frametime = frame.info_frametime();
        let mut fps = self.camera.fps;
        if frametime > 0. && frametime < LONGEST_FRAME {
            let (sum, count) = &mut self.frame_times;
            *sum += frametime;
            *count += 1;
            if fps.is_none() || *sum >= FPS_SPAN {
                fps = Some(*count as f32 / *sum);
                self.frame_times = (0., 0);
            }
        }

        // The bob is what is left of the view's height over the eyes', as
        // far as the game's bob goes
        let (origin, player_origin) = (frame.info_vieworg(), frame.info_simorg());
        let raised = origin[2] - player_origin[2] - frame.info_viewheight()[2];
        let camera = Camera {
            origin,
            player_origin,
            angles: frame.info_viewangles(),
            aim: frame.info_cl_viewangles(),
            bob: raised.clamp(LOWEST_BOB, HIGHEST_BOB),
            fov: self.camera.fov,
            view_entity: decoder.view_entity,
            spectating: frame.info_spectator() != 0,
            intermission: frame.info_intermission() != 0,
            observer_mode,
            observer_target,
            viewmodel,
            health: client("health").unwrap_or(0),
            weapons: client("weapons").unwrap_or(0) as u32,
            velocity,
            on_ground,
            last_jump,
            fps,
            scores_held: self.camera.scores_held,
        };
        if camera != self.camera {
            self.camera = camera;
            self.changed();
        }

        let Some(snapshot) = decoder.latest_snapshot() else {
            return;
        };
        if self
            .snapshot
            .as_ref()
            .is_some_and(|s| Arc::ptr_eq(s, snapshot))
        {
            return;
        }
        self.snapshot = Some(snapshot.clone());
        self.snapshot_at = time;
        self.server_time = f64::from(f32::from_bits(snapshot.server_time_bits));
        self.max_clients = decoder.max_clients;
        if let Some(entities) = &snapshot.entities {
            let updated = ServerTime(f32::from_bits(snapshot.server_time_bits));
            self.take_entities(decoder, entities, updated);
            self.recent
                .push_back((self.server_time, self.entities.clone()));
            if self.recent.len() > RECENT {
                self.recent.pop_front();
            }
            self.estimate_movement();
            self.changed();
        }
    }

    /// A record other than a network one
    pub(crate) fn on_record(&mut self, body: &FrameBody, time: SourceTime) {
        match body {
            FrameBody::ClientData(data) => {
                let fov = Some(ClientDataRecord(data).fov());
                if fov != self.camera.fov {
                    self.camera.fov = fov;
                    self.changed();
                }
            }
            FrameBody::WeaponAnim { sequence, body } => {
                self.viewmodel = Some(Viewmodel {
                    sequence: *sequence,
                    body: *body,
                    started: time,
                });
                self.changed();
            }
            _ => {}
        }
    }

    /// A command the recorder's game ran, which the game runs again in
    /// playback. The scoreboard shows while `+showscores` holds, as the
    /// buttons a frame records can say it is held all the time
    pub(crate) fn on_command(&mut self, command: &str) {
        let held = match command.split_whitespace().next() {
            Some("+showscores") => true,
            Some("-showscores") => false,
            _ => return,
        };
        if held != self.camera.scores_held {
            self.camera.scores_held = held;
            self.changed();
        }
    }

    /// Start a weapon animation on the viewmodel at `time`
    pub(crate) fn animate_viewmodel(&mut self, sequence: i32, body: i32, time: SourceTime) {
        self.viewmodel = Some(Viewmodel {
            sequence,
            body,
            started: time,
        });
        self.changed();
    }

    fn model_name(&mut self, decoder: &Decoder, index: i64) -> Option<Arc<str>> {
        let resources = decoder.resources();
        if resources.len() != self.models_from {
            self.models = resources
                .iter()
                .filter(|r| r.kind == resource_type::MODEL)
                .map(|r| {
                    (
                        r.index,
                        Arc::from(String::from_utf8_lossy(&r.name).as_ref()),
                    )
                })
                .collect::<Table<_, _>>()
                .into();
            self.models_from = resources.len();
        }
        self.models.get(&u16::try_from(index).ok()?).cloned()
    }

    fn table(&mut self, decoder: &Decoder, e: &EntityState) -> Option<usize> {
        let schema = decoder.entity_schema(e.number, e.custom)?;
        if let Some(i) = self
            .tables
            .iter()
            .position(|t| Arc::ptr_eq(&t.schema, &schema))
        {
            return Some(i);
        }
        self.tables.push(FieldTable::new(schema));
        Some(self.tables.len() - 1)
    }

    fn take_entities(
        &mut self,
        decoder: &Decoder,
        entities: &[Arc<EntityState>],
        updated: ServerTime,
    ) {
        // The packet before's entities and the states they came from,
        // ordered by number as these are. One that left the packet is not
        // among them, so its slot's next occupant starts a new life
        let before = self.entities.clone();
        let before_sources = std::mem::take(&mut self.sources);
        let mut next_before = 0;

        let mut out = Vec::with_capacity(entities.len());
        let mut sources = Vec::with_capacity(entities.len());
        for e in entities {
            // The world is drawn from the map, not as an entity
            if e.number == 0 {
                continue;
            }

            // The entity with this number in the packet before, if any
            while next_before < before.len() && before[next_before].id.number < e.number {
                next_before += 1;
            }
            let previous = match before.get(next_before) {
                Some(entity) if entity.id.number == e.number => {
                    Some((entity, &before_sources[next_before]))
                }
                _ => None,
            };

            // An unchanged state converts to the same entity
            if let Some((entity, source)) = previous
                && Arc::ptr_eq(source, e)
            {
                out.push(Entity {
                    updated,
                    ..entity.clone()
                });
                sources.push(e.clone());
                continue;
            }

            let Some(table) = self.table(decoder, e) else {
                continue;
            };
            let v = self.tables[table].read(e);
            let model_index = v[0] as i64;

            // A slot keeps its life while it stays in the packet with the
            // same model
            let n = usize::from(e.number);
            if self.lives.len() <= n {
                self.lives.resize(n + 1, None);
            }
            let life = match self.lives[n] {
                Some(slot) if previous.is_some() && slot.model == model_index => slot.life,
                Some(slot) => slot.life + 1,
                None => 0,
            };
            self.lives[n] = Some(Slot {
                life,
                model: model_index,
            });

            // The sequence before is kept from a change on, as the client
            // keeps it to fade from
            let sequence = v[7] as i64;
            let change = match previous {
                Some((old, _)) if old.id.life == life && old.sequence != sequence => {
                    Some(SequenceChange {
                        sequence: old.sequence,
                        frame: old.frame,
                        framerate: old.framerate,
                        animtime: old.animtime,
                        blending: old.blending,
                        held_at: f64::from(updated.0),
                        at: v[11] as f32,
                    })
                }
                Some((old, _)) if old.id.life == life => old.change,
                _ => None,
            };

            out.push(Entity {
                beam: e.custom,
                id: EntityId {
                    segment: self.segment,
                    number: e.number,
                    life,
                },
                model_index,
                model: self.model_name(decoder, model_index),
                origin: [v[1] as f32, v[2] as f32, v[3] as f32],
                angles: [v[4] as f32, v[5] as f32, v[6] as f32],
                sequence,
                gait_sequence: v[8] as i64,
                frame: v[9] as f32,
                framerate: v[10] as f32,
                animtime: v[11] as f32,
                body: v[12] as i64,
                skin: v[13] as i64,
                controllers: [v[14] as f32, v[15] as f32, v[16] as f32, v[17] as f32],
                blending: [v[18] as f32, v[19] as f32],
                effects: v[20] as i64,
                render_mode: v[21] as i64,
                render_amount: v[22] as i64,
                render_fx: v[23] as i64,
                scale: v[24] as f32,
                move_type: v[25] as i64,
                solid: v[26] as i64,
                iuser1: v[27] as i64,
                iuser2: v[28] as i64,
                render_color: [v[29] as u8, v[30] as u8, v[31] as u8],
                weapon_model: v[32] as i64,
                weapon: self.model_name(decoder, v[32] as i64),
                use_hull: v[33] as i64,
                gait: None,
                change,
                updated,
            });
            sources.push(e.clone());
        }

        self.entities = Arc::new(out);
        self.sources = sources;
    }

    /// The client's clock at source time `at`, the latest snapshot's server
    /// time plus the source time since it arrived
    pub fn client_time(&self, at: SourceTime) -> f64 {
        self.server_time + (at.0 - self.snapshot_at.0)
    }

    /// The entities of the snapshot before the latest, and how much server
    /// time before it it was
    pub fn previous_snapshot(&self) -> Option<(f64, &[Entity])> {
        let latest = self.recent.back()?.0;
        let at = self.recent.len().checked_sub(2)?;
        let (time, entities) = &self.recent[at];
        Some((latest - time, entities.as_slice()))
    }

    /// Each player's velocity from how far they moved since the latest
    /// snapshot at least [`MEASURED_OVER`] before the latest, as positions
    /// sent an eighth of a unit apart make a short span's speed jump about,
    /// and their last jump, a rise from level as fast as a jump's. A player
    /// missing from either snapshot has no velocity and keeps their last
    /// jump
    fn estimate_movement(&mut self) {
        let Some((t1, now)) = self.recent.back() else {
            return;
        };
        let earlier = self
            .recent
            .iter()
            .rev()
            .find(|(t, _)| *t <= t1 - MEASURED_OVER);
        let Some((t0, before)) = earlier else {
            return;
        };
        let elapsed = (t1 - t0) as f32;

        for e in now.iter() {
            let number = e.id.number;
            if !self.is_player(number) {
                continue;
            }
            let previous = self.movement.get(&number).copied().unwrap_or_default();
            let velocity = before
                .iter()
                .find(|old| old.id == e.id)
                .map(|old| [0, 1, 2].map(|k| (e.origin[k] - old.origin[k]) / elapsed))
                .filter(|v| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt() <= MOST_SPEED);

            let up = velocity.map(|v| v[2]);
            let level = up.is_some_and(|z| z.abs() < LEVEL);
            let beginning = up.is_some_and(|z| (0. ..JUMP_RISE.0).contains(&z));
            let rising = up.is_some_and(|z| (JUMP_RISE.0..=JUMP_RISE.1).contains(&z));
            let jumped = previous.from_level && rising;
            let last_jump = if jumped { velocity } else { previous.last_jump };
            self.movement.insert(
                number,
                Movement {
                    velocity,
                    last_jump,
                    from_level: level || (previous.from_level && beginning),
                },
            );
        }
    }

    fn is_player(&self, number: u16) -> bool {
        number >= 1 && number <= u16::from(self.max_clients)
    }

    /// Where `id` is at server time `time`, between the snapshots around it,
    /// or none to draw it as the latest snapshot has it
    fn position_at(&self, id: EntityId, time: f64) -> Option<([f32; 3], [f32; 3])> {
        let sample = |list: &Arc<Vec<Entity>>| {
            let at = list
                .binary_search_by_key(&id.number, |e| e.id.number)
                .ok()?;
            let e = &list[at];
            (e.id == id).then_some((e.origin, e.angles))
        };
        let later = self.recent.iter().position(|(t, _)| *t > time)?;
        let (t1, after) = &self.recent[later];
        let after = sample(after)?;
        let Some((t0, before)) = later.checked_sub(1).map(|i| &self.recent[i]) else {
            return Some(after);
        };
        let Some(before) = sample(before) else {
            return Some(after);
        };

        let step = [0, 1, 2].map(|k| after.0[k] - before.0[k]);
        if (step[0].powi(2) + step[1].powi(2) + step[2].powi(2)).sqrt() > TELEPORT || t1 <= t0 {
            return Some(after);
        }
        let f = ((time - t0) / (t1 - t0)).clamp(0., 1.) as f32;
        let origin = [0, 1, 2].map(|k| before.0[k] + step[k] * f);
        let angles = [0, 1, 2].map(|k| before.1[k] + half_turn(after.1[k] - before.1[k]) * f);
        Some((origin, angles))
    }

    /// The entities as the client draws them at its time `client_time`,
    /// moved between snapshots and with players' legs
    pub fn presented(&self, client_time: f64) -> Vec<Entity> {
        let render = client_time - INTERPOLATION;
        self.entities
            .iter()
            .map(|e| {
                let mut e = e.clone();
                // A beam's angles are a point, not turned the short way round
                if !e.beam
                    && let Some((origin, angles)) = self.position_at(e.id, render)
                {
                    e.origin = origin;
                    e.angles = angles;
                }
                e.gait = self
                    .gaits
                    .get(&e.id.number)
                    .filter(|t| t.life == e.id.life && e.gait_sequence != 0)
                    .map(|t| t.gait);
                e
            })
            .collect()
    }

    /// Step every player's legs for the client frame of the group at `at`
    pub(crate) fn step_gaits(&mut self, at: SourceTime) {
        let dt = self
            .last_frame
            .map_or(0., |last| (at.0 - last.0).clamp(0., 1.)) as f32;
        self.last_frame = Some(at);
        let render = self.client_time(at) - INTERPOLATION;

        let mut gaits = Table::with_capacity(self.gaits.len());
        for e in self.entities.iter() {
            if !self.is_player(e.id.number) || e.gait_sequence == 0 {
                continue;
            }
            let (origin, angles) = self
                .position_at(e.id, render)
                .unwrap_or((e.origin, e.angles));
            let mut track = match self.gaits.get(&e.id.number) {
                Some(t) if t.life == e.id.life => *t,
                _ => GaitTrack {
                    life: e.id.life,
                    gait: Gait {
                        yaw: angles[1],
                        ..Gait::default()
                    },
                    last_origin: origin,
                },
            };
            track.step(origin, angles[1], dt);
            gaits.insert(e.id.number, track);
        }
        self.gaits = gaits;
    }
}

/// How far around an entity's origin the world is looked into for open
/// space, a small box standing in for the model's own
const OPEN_REACH: f32 = 8.;

/// The entities the game draws, leaving out a model or sprite wholly inside
/// the world's solid, as a spare corpse is parked outside the map. The game
/// draws an entity whose box reaches an open part of the map, which its
/// origin and a point a little way out along each axis stand in for. Brush
/// entities and beams keep places of their own and are always kept, and so
/// is the recorder's own entity, `own`, which the server leaves at nought,
/// often inside the solid, for the viewer to place where the recorder is
pub(crate) fn in_the_open(entities: Vec<Entity>, world: &Hull, own: u16) -> Vec<Entity> {
    entities
        .into_iter()
        .filter(|e| {
            let brush = e.model.as_deref().is_some_and(|m| m.starts_with('*'));
            if brush || e.beam || e.id.number == own {
                return true;
            }

            let o = e.origin;
            let mut points = vec![o];
            for axis in 0..3 {
                for side in [-OPEN_REACH, OPEN_REACH] {
                    let mut p = o;
                    p[axis] += side;
                    points.push(p);
                }
            }
            points.into_iter().any(|p| world.contents(p) != SOLID)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_recorders_fps_is_read_at_once_then_every_half_second() {
        let frame = |frametime: f32| {
            let mut demo_info = [0u8; 436];
            demo_info[64..68].copy_from_slice(&frametime.to_le_bytes());
            NetworkFrame {
                demo_info: Box::new(demo_info),
                sequence: [0; 7],
                message: Vec::new(),
            }
        };
        let decoder = Decoder::new();
        let mut scene = Scene::default();
        let fps = |scene: &Scene| scene.camera.fps.map(|f| f.round());

        // The first frame gives a reading at once
        scene.after_network(&decoder, &frame(0.01), SourceTime(0.));
        assert_eq!(fps(&scene), Some(100.));

        // Faster frames change it only once half a second of them is in
        for _ in 0..90 {
            scene.after_network(&decoder, &frame(0.005), SourceTime(0.));
        }
        assert_eq!(fps(&scene), Some(100.));
        for _ in 0..20 {
            scene.after_network(&decoder, &frame(0.005), SourceTime(0.));
        }
        assert_eq!(fps(&scene), Some(200.));

        // A pause or a load is no frame
        let before = scene.frame_times;
        scene.after_network(&decoder, &frame(5.), SourceTime(0.));
        scene.after_network(&decoder, &frame(0.), SourceTime(0.));
        assert_eq!(scene.frame_times, before);
        assert_eq!(fps(&scene), Some(200.));
    }

    #[test]
    fn a_model_parked_outside_the_map_is_not_drawn() {
        use crate::trace::Child;

        // Open above the floor at z = 0, solid below
        let world = Hull {
            planes: vec![([0., 0., 1.], 0.)],
            nodes: vec![(0, [Child::Contents(-1), Child::Contents(SOLID)])],
            models: vec![Child::Node(0)],
            node_faces: Vec::new(),
            faces: Vec::new(),
        };
        let at = |number: u16, model: &str, z: f32| Entity {
            id: EntityId {
                number,
                ..EntityId::default()
            },
            model: Some(Arc::from(model)),
            origin: [0., 0., z],
            ..Entity::default()
        };

        // A body in the solid, one just sunk into the floor, a brush model
        // whose origin is the map's, a player in the open, and the
        // recorder's own entity left in the solid at nought
        let kept = in_the_open(
            vec![
                at(1, "models/player.mdl", -500.),
                at(2, "models/w_battery.mdl", -4.),
                at(3, "*12", -500.),
                at(4, "models/player.mdl", 36.),
                at(5, "models/player.mdl", -500.),
            ],
            &world,
            5,
        );
        let numbers: Vec<u16> = kept.iter().map(|e| e.id.number).collect();
        assert_eq!(numbers, [2, 3, 4, 5]);
    }

    fn track(yaw: f32) -> GaitTrack {
        GaitTrack {
            life: 0,
            gait: Gait {
                yaw,
                ..Gait::default()
            },
            last_origin: [0.; 3],
        }
    }

    #[test]
    fn a_watched_players_weapon_gives_its_viewmodel() {
        assert_eq!(
            view_model_for("models/p_gauss.mdl"),
            Some("models/v_gauss.mdl")
        );
        assert_eq!(
            view_model_for("MODELS/P_9MMAR.MDL"),
            Some("models/v_9mmAR.mdl")
        );
        assert_eq!(view_model_for("models/p_flag.mdl"), None);
    }

    #[test]
    fn the_sky_is_named_after_the_movement_variables_numbers() {
        let mut body = vec![0u8; 97];
        body.extend_from_slice(b"dusk\0");
        assert_eq!(sky_name(&body).as_deref(), Some("dusk"));
        body.truncate(97);
        body.push(0);
        assert_eq!(sky_name(&body), None);
        assert_eq!(sky_name(&[0; 40]), None);
    }

    #[test]
    fn legs_follow_movement_and_turn_round_walking_backwards() {
        // Forward along x at 100 units a second
        let mut t = track(0.);
        for i in 1..=10 {
            t.step([i as f32, 0., 0.], 0., 0.01);
        }
        assert_eq!(t.gait.yaw, 0.);
        assert!((t.gait.distance - 10.).abs() < 1e-4);
        assert!((t.gait.torso - 127.5).abs() < 1e-4);

        // The body faces back along -x, so the legs turn and count backwards
        let mut t = track(0.);
        t.step([1., 0., 0.], 180., 0.01);
        assert_eq!(t.gait.yaw, -180.);
        assert!((t.gait.distance + 1.).abs() < 1e-4);

        // Standing still the legs turn towards the body, four times the
        // frame's share of the difference while frames are short
        let mut t = track(0.);
        t.step([0.; 3], 90., 0.1);
        assert!((t.gait.yaw - 36.).abs() < 1e-4);
        assert_eq!(t.gait.distance, 0.);
        assert!((t.gait.elapsed - 0.1).abs() < 1e-6);
    }

    fn entity(number: u16, life: u32, x: f32, yaw: f32) -> Entity {
        Entity {
            beam: false,
            id: EntityId {
                segment: 1,
                number,
                life,
            },
            model_index: 0,
            model: None,
            origin: [x, 0., 0.],
            angles: [0., yaw, 0.],
            sequence: 0,
            gait_sequence: 0,
            frame: 0.,
            framerate: 0.,
            animtime: 0.,
            body: 0,
            skin: 0,
            controllers: [0.; 4],
            blending: [0.; 2],
            effects: 0,
            render_mode: 0,
            render_amount: 0,
            render_fx: 0,
            render_color: [0; 3],
            scale: 0.,
            move_type: 0,
            solid: 0,
            iuser1: 0,
            iuser2: 0,
            weapon_model: 0,
            weapon: None,
            use_hull: 0,
            gait: None,
            change: None,
            updated: ServerTime(0.),
        }
    }

    #[test]
    fn entities_are_drawn_between_snapshots_but_not_across_jumps_or_lives() {
        let mut scene = Scene::default();
        let at = |t: f64, list: Vec<Entity>| (t, Arc::new(list));
        scene.recent.push_back(at(
            1.0,
            vec![
                entity(1, 0, 0., 350.),
                entity(2, 0, 0., 0.),
                entity(3, 0, 0., 0.),
                Entity {
                    beam: true,
                    ..entity(4, 0, 0., 300.)
                },
            ],
        ));
        scene.recent.push_back(at(
            1.1,
            vec![
                entity(1, 0, 10., 10.),
                entity(2, 0, 500., 0.),
                entity(3, 1, 20., 0.),
                Entity {
                    beam: true,
                    ..entity(4, 0, 10., 20.)
                },
            ],
        ));
        scene.entities = scene.recent.back().unwrap().1.clone();
        scene.server_time = 1.1;
        scene.snapshot_at = SourceTime(5.);

        // Half way between the snapshots, 0.1 s behind the client's clock
        let drawn = scene.presented(scene.client_time(SourceTime(5.05)) + 0.);
        assert!((drawn[0].origin[0] - 5.).abs() < 1e-4);
        // The short way round from 350 to 10 degrees
        assert!((drawn[0].angles[1] - 360.).abs() < 1e-3);
        // A jump and a new life are drawn where the latest snapshot has them
        assert_eq!(drawn[1].origin[0], 500.);
        assert_eq!(drawn[2].origin[0], 20.);
        // A beam's angles are its end point, drawn as the latest snapshot
        // has it, never turned
        assert_eq!((drawn[3].origin[0], drawn[3].angles[1]), (10., 20.));

        // Past the latest snapshot nothing is extrapolated
        let late = scene.presented(5.);
        assert_eq!(late[0].origin[0], 10.);
    }
}

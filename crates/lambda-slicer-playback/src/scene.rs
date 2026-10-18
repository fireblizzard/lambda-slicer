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
//! last client data record. Effects and HUD state come with M10

use std::collections::VecDeque;

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use lambda_slicer_core::container::{ClientDataRecord, FrameBody, NetworkFrame};
use lambda_slicer_core::protocol::delta::DeltaSchema;
use lambda_slicer_core::protocol::messages::{Message, ParsedMessage, resource_type};
use lambda_slicer_core::state::{Decoder, EntityState, Snapshot};

use crate::time::{ServerTime, SourceTime};

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId {
    pub segment: u32,
    pub number: u16,
    pub life: u32,
}

/// One entity as the newest snapshot holding it left it. Fields a session's
/// schema does not send are 0
#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    pub id: EntityId,
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
    /// A player's legs, in presented entities whose gait sequence is set
    pub gait: Option<Gait>,
    /// Server time of the snapshot it was last taken from
    pub updated: ServerTime,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Camera {
    pub origin: [f32; 3],
    pub angles: [f32; 3],
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
}

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
    pub light_styles: BTreeMap<u8, Vec<u8>>,
    /// Grows whenever anything here changes, so a renderer can skip work
    pub revision: u64,
    /// The snapshot the entities were taken from
    snapshot: Option<Arc<Snapshot>>,
    lives: HashMap<u16, Slot>,
    /// Model names by precache index, and the resource count they were
    /// read from
    models: Arc<HashMap<u16, Arc<str>>>,
    models_from: usize,
    /// Field positions per schema, and each entity as last converted with
    /// the state it came from, which snapshots share while it is unchanged
    tables: Vec<FieldTable>,
    converted: HashMap<u16, (Arc<EntityState>, Entity)>,
    /// The latest snapshots' entities by server time, oldest first
    recent: VecDeque<(f64, Arc<Vec<Entity>>)>,
    /// When the latest snapshot arrived, and its server time
    snapshot_at: SourceTime,
    server_time: f64,
    max_clients: u8,
    gaits: HashMap<u16, GaitTrack>,
    /// The last client frame the legs were stepped for
    last_frame: Option<SourceTime>,
}

/// A slot's current or last life
#[derive(Clone, Copy, Debug, Default)]
struct Slot {
    life: u32,
    model: i64,
    present: bool,
}

/// The entity fields the scene reads, in this order
const FIELDS: [&str; 33] = [
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
];

/// Where each of [`FIELDS`] sits in one schema, looked up once
#[derive(Clone, Debug)]
struct FieldTable {
    schema: Arc<DeltaSchema>,
    at: [Option<usize>; FIELDS.len()],
}

impl FieldTable {
    fn new(schema: Arc<DeltaSchema>) -> Self {
        let at = FIELDS.map(|name| schema.field_index(name));
        Self { schema, at }
    }

    /// Every field's value, 0 when the schema does not send it
    fn read(&self, e: &EntityState) -> [f64; FIELDS.len()] {
        let mut values = [0.; FIELDS.len()];
        for (value, at) in values.iter_mut().zip(self.at) {
            *value = at
                .and_then(|i| e.fields.value_at(&self.schema, i))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.);
        }
        values
    }
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
        let viewmodel = client("viewmodel").and_then(|i| self.model_name(decoder, i));
        let camera = Camera {
            origin: frame.info_vieworg(),
            angles: frame.info_viewangles(),
            fov: self.camera.fov,
            view_entity: decoder.view_entity,
            spectating: frame.info_spectator() != 0,
            intermission: frame.info_intermission() != 0,
            observer_mode: client("iuser1").unwrap_or(0),
            observer_target: client("iuser2").unwrap_or(0),
            viewmodel,
            health: client("health").unwrap_or(0),
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
                .collect::<HashMap<_, _>>()
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
        let mut lives = HashMap::with_capacity(entities.len());
        let mut converted = HashMap::with_capacity(entities.len());
        let mut out = Vec::with_capacity(entities.len());
        for e in entities {
            // The world is drawn from the map, not as an entity
            if e.number == 0 {
                continue;
            }

            // An unchanged state converts to the same entity
            if let Some((state, entity)) = self.converted.remove(&e.number)
                && Arc::ptr_eq(&state, e)
                && self.lives.get(&e.number).is_some_and(|s| s.present)
            {
                let entity = Entity { updated, ..entity };
                lives.insert(e.number, self.lives[&e.number]);
                converted.insert(e.number, (state, entity.clone()));
                out.push(entity);
                continue;
            }

            let Some(table) = self.table(decoder, e) else {
                continue;
            };
            let v = self.tables[table].read(e);
            let model_index = v[0] as i64;
            let life = match self.lives.get(&e.number) {
                Some(slot) if slot.present && slot.model == model_index => slot.life,
                Some(slot) => slot.life + 1,
                None => 0,
            };
            lives.insert(
                e.number,
                Slot {
                    life,
                    model: model_index,
                    present: true,
                },
            );

            let entity = Entity {
                id: EntityId {
                    segment: self.segment,
                    number: e.number,
                    life,
                },
                model_index,
                model: self.model_name(decoder, model_index),
                origin: [v[1] as f32, v[2] as f32, v[3] as f32],
                angles: [v[4] as f32, v[5] as f32, v[6] as f32],
                sequence: v[7] as i64,
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
                gait: None,
                updated,
            };
            converted.insert(e.number, (e.clone(), entity.clone()));
            out.push(entity);
        }

        // A slot that left the packet keeps its count, so its next
        // occupant starts a new life
        for (number, slot) in &self.lives {
            lives.entry(*number).or_insert(Slot {
                present: false,
                ..*slot
            });
        }
        self.lives = lives;
        self.converted = converted;
        self.entities = Arc::new(out);
    }

    /// The client's clock at source time `at`, the latest snapshot's server
    /// time plus the source time since it arrived
    pub fn client_time(&self, at: SourceTime) -> f64 {
        self.server_time + (at.0 - self.snapshot_at.0)
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
                if let Some((origin, angles)) = self.position_at(e.id, render) {
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

        let mut gaits = HashMap::with_capacity(self.gaits.len());
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

#[cfg(test)]
mod tests {
    use super::*;

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
            gait: None,
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
            ],
        ));
        scene.recent.push_back(at(
            1.1,
            vec![
                entity(1, 0, 10., 10.),
                entity(2, 0, 500., 0.),
                entity(3, 1, 20., 0.),
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

        // Past the latest snapshot nothing is extrapolated
        let late = scene.presented(5.);
        assert_eq!(late[0].origin[0], 10.);
    }
}

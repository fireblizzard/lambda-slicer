//! What the recording client could show at a moment, built record by record
//!
//! Entities come from the decoder's newest stored snapshot, so a packet the
//! client rejected never reaches the scene. An entity's identity is its
//! session, its number and its life. A life ends when the entity leaves the
//! packet or its model changes, so a reused slot is a different entity and
//! is never interpolated from the one before. A new session starts an empty
//! scene.
//!
//! The camera is the view the client rendered when it wrote each network
//! record, from the record's `ref_params_t`, with the field of view of the
//! last client data record. Interpolation between samples, effects and HUD
//! state come with M9 and M10

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use lambda_slicer_core::container::{ClientDataRecord, FrameBody, NetworkFrame};
use lambda_slicer_core::protocol::delta::DeltaSchema;
use lambda_slicer_core::protocol::messages::{Message, ParsedMessage, resource_type};
use lambda_slicer_core::state::{Decoder, EntityState, Snapshot};

use crate::time::{ServerTime, SourceTime};

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
    pub scale: f32,
    pub move_type: i64,
    pub solid: i64,
    /// Server time of the snapshot it was last taken from
    pub updated: ServerTime,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Camera {
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    /// From client data records, which some recordings lack
    pub fov: Option<f32>,
    pub view_entity: Option<u16>,
    pub spectating: bool,
    pub intermission: bool,
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
    models: HashMap<u16, Arc<str>>,
    models_from: usize,
    /// Field positions per schema, and each entity as last converted with
    /// the state it came from, which snapshots share while it is unchanged
    tables: Vec<FieldTable>,
    converted: HashMap<u16, (Arc<EntityState>, Entity)>,
}

/// A slot's current or last life
#[derive(Clone, Copy, Debug, Default)]
struct Slot {
    life: u32,
    model: i64,
    present: bool,
}

/// The entity fields the scene reads, in this order
const FIELDS: [&str; 27] = [
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

    /// After a network record, its camera and any new snapshot
    pub(crate) fn after_network(&mut self, decoder: &Decoder, frame: &NetworkFrame) {
        let camera = Camera {
            origin: frame.info_vieworg(),
            angles: frame.info_viewangles(),
            fov: self.camera.fov,
            view_entity: decoder.view_entity,
            spectating: frame.info_spectator() != 0,
            intermission: frame.info_intermission() != 0,
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
        if let Some(entities) = &snapshot.entities {
            let updated = ServerTime(f32::from_bits(snapshot.server_time_bits));
            self.take_entities(decoder, entities, updated);
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
                .collect();
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
}

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use serde::Serialize;

use crate::container::{EventRecord, NetworkFrame};
use crate::protocol::bit::BitReader;
use crate::protocol::delta::{DeltaSchema, StructState, Value, meta_schema};
use crate::protocol::messages::{
    ClientDataMessage, EntityPacket, Message, MessageError, ParseContext, ParsedMessage, Resource,
    UserMessageDef, parse_message, resource_type, svc,
};
use crate::protocol::profile;

/// Where a record came from in the source file
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct FrameLocation {
    pub entry: u32,
    pub ordinal: u64,
    pub offset: u64,
    pub time_bits: u32,
    pub frame_number: i32,
}

impl FrameLocation {
    /// The record's own time, on whichever clock its entry uses
    pub fn time(&self) -> f32 {
        f32::from_bits(self.time_bits)
    }

    /// The time on the playback clock. The LOADING entry comes before
    /// playback and is timed by the recording client's own clock, often
    /// minutes ahead, so its records count as playback's start, the state
    /// it begins with
    pub fn playback_time(&self) -> f32 {
        if self.entry == 0 { 0. } else { self.time() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityState {
    pub number: u16,
    pub custom: bool,
    pub fields: StructState,
}

/// How the base of a delta was chosen
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BaseResolution {
    /// The frame whose recorded sequence has the reference's low byte, within
    /// the server's frame window. That is the frame the server deltas from,
    /// and for entity packets its rebuild also reproduces the declared count
    Strict,
    /// Matched under a sequence offset inferred earlier in the recording
    Offset,
    /// The nearest recent frame whose rebuild reproduces the declared entity
    /// count. Equal counts do not prove the frame is the right one
    CountSearch,
    /// No frame was available, so the base is empty
    Empty,
}

impl BaseResolution {
    /// Whether state built on this base is a guess rather than proven
    pub fn inferred(self) -> bool {
        self != BaseResolution::Strict
    }
}

/// Complete decoded state carried by one received packet
#[derive(Clone, Debug)]
pub struct Snapshot {
    /// Netchan incoming sequence of the packet
    pub sequence: u32,
    pub server_time_bits: u32,
    /// Some part of this state was built on an inferred delta base, directly
    /// or through an earlier inferred snapshot. Only a full update, rebuilt
    /// from baselines alone, clears it
    pub inferred: bool,
    /// Ordered by entity number, `None` if the packet had no entity update
    pub entities: Option<Arc<Vec<Arc<EntityState>>>>,
    pub clientdata: Option<Arc<StructState>>,
    /// 64 weapon slots
    pub weapons: Option<Arc<Vec<Option<Arc<StructState>>>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlayerInfo {
    pub slot: u8,
    pub user_id: u32,
    /// Raw userinfo string
    #[serde(skip)]
    pub info: Vec<u8>,
    pub name: String,
    /// Increments when the slot's user id changes or after the slot was
    /// vacated, so a reused slot is a different player
    pub incarnation: u32,
}

/// Who a player was at one moment. Name is the name at that moment, so
/// later renames and slot reuse do not rewrite it
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlayerIdentity {
    pub segment: u32,
    pub slot: u8,
    pub user_id: u32,
    pub incarnation: u32,
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NameChangeKind {
    /// A new player in the slot, or the first userinfo seen for it
    Joined,
    /// Same player with a different name
    Renamed,
    /// Userinfo cleared, which the server sends when a client drops
    Left,
}

/// One entry of the name history
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NameChange {
    pub location: FrameLocation,
    pub kind: NameChangeKind,
    pub player: PlayerIdentity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum EventSource {
    Demo,
    Svc,
    SvcReliable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum ActorResolution {
    /// Resolved through this packet's entity array
    PacketEntity {
        packet_index: u16,
        entity: u16,
    },
    /// Explicit `entindex` in the event arguments
    Explicit {
        entity: i32,
    },
    /// Packet index past the array that is not the documented fallback
    OutOfRange {
        packet_index: u16,
        entities: usize,
    },
    /// Packet index given but no entity update in this packet
    NoPacketContext {
        packet_index: u16,
    },
    Unknown,
}

#[derive(Clone, Debug)]
pub struct DecodedEvent {
    pub location: FrameLocation,
    /// Session the event belongs to, counted from 1 at each server info
    pub segment: u32,
    pub source: EventSource,
    pub message_ordinal: Option<u32>,
    pub index: u16,
    pub resource: Option<Vec<u8>>,
    /// Decoded `event_args_t` for network events
    pub args: Option<StructState>,
    /// Standalone demo event record
    pub demo: Option<EventRecord>,
    pub actor: ActorResolution,
    /// The player behind the actor entity when the event was decoded, if the
    /// actor is a player entity with a known occupant
    pub player: Option<PlayerIdentity>,
    /// For network events, the fire time or reliable delay, ×100. For demo
    /// events, the delay f32 bits
    pub delay_raw: Option<u32>,
    pub server_time_bits: u32,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DecodeErrorKind {
    #[error(transparent)]
    Message(#[from] MessageError),
    #[error("instanced baseline {index} of {available}")]
    InstancedBaselineOutOfRange { index: u8, available: usize },
    #[error("baseline offset {offset} at packet position {position}")]
    OffsetOutOfRange { offset: u8, position: usize },
    #[error("entity packet declares {declared} entities, reconstruction has {actual}")]
    CountMismatch { declared: u16, actual: usize },
    #[error("entity numbers out of order: {previous} then {next}")]
    Unordered { previous: u16, next: u16 },
    #[error("svc_serverinfo announces network protocol {0}, only 48 is supported")]
    UnsupportedProtocol(i32),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error(
    "entry {}, frame {} at byte {} (time {}), message {} {} at payload byte {}: {kind}",
    location.entry, location.ordinal, location.offset, location.time(),
    message_ordinal, svc::name(*opcode), payload_offset
)]
pub struct DecodeError {
    pub location: FrameLocation,
    pub message_ordinal: u32,
    pub opcode: u8,
    pub payload_offset: u32,
    pub kind: DecodeErrorKind,
}

impl crate::message::Coded for DecodeError {
    /// One code for every network decoding failure, which only a developer
    /// can act on, except an unsupported protocol announced mid-file
    fn code(&self) -> &'static str {
        match self.kind {
            DecodeErrorKind::UnsupportedProtocol(_) => "protocol-unsupported-network",
            _ => "decode-failed",
        }
    }

    fn args(&self) -> Vec<(&'static str, crate::message::Arg)> {
        let mut args = vec![
            ("entry", self.location.entry.into()),
            ("frame", self.location.ordinal.into()),
            ("offset", self.location.offset.into()),
            ("time", self.location.time().to_string().into()),
            ("detail", self.kind.to_string().into()),
        ];
        if let DecodeErrorKind::UnsupportedProtocol(n) = self.kind {
            args.push(("network", n.into()));
        }
        args
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct DecodeStats {
    pub network_frames: u64,
    pub messages: u64,
    pub by_opcode: BTreeMap<String, u64>,
    pub user_messages: BTreeMap<String, u64>,
    pub segments: u32,
    pub full_packets: u64,
    pub delta_packets: u64,
    pub max_entities: usize,
    pub snapshots: u64,
    pub duplicate_sequences: u64,
    /// Largest sequence distance between a packet and a strict delta base
    pub max_delta_distance: u32,
    /// Delta bases by how they were chosen, see [`BaseResolution`]
    pub bases_strict: u64,
    pub bases_offset: u64,
    pub bases_count_search: u64,
    pub bases_empty: u64,
    /// Snapshots whose state is inferred rather than proven
    pub inferred_snapshots: u64,
    /// Where the first inferred delta base was needed
    pub first_inferred: Option<FrameLocation>,
    /// Delta entity packets the client drops because their base is too old.
    /// Their entity and client state are not kept
    pub discarded_packets: u64,
    /// Delta bases from before the first recorded packet of the numbering
    /// run, which the recording never received. They are guessed by count
    /// without adopting an offset
    pub unrecorded_bases: u64,
    /// Removals of entities absent from the delta base (ignored, as the
    /// engine does)
    pub removed_absent_entities: u64,
    /// Times the recorded-to-server sequence offset was (re)inferred. Some
    /// recordings store an `incoming_sequence` shifted from the numbering the
    /// server uses in delta references
    pub sequence_offset_changes: u64,
    /// Last inferred offset (recorded minus server sequence, low byte)
    pub sequence_offset: u8,
    pub entities_without_baseline: u64,
    pub events_by_resource: BTreeMap<String, u64>,
    pub unresolved_event_actors: u64,
    pub max_payload: usize,
}

/// Frame ring sizes. Server and client delta from `frames[delta & (backup - 1)]`
/// with 8 frames when `max_clients == 1`, otherwise 64 (ReHLDS
/// `SV_UPDATE_BACKUP`). A referenced frame is still in its slot while it is at
/// most `backup` packets old
const SINGLEPLAYER_BACKUP: usize = 8;
const MULTIPLAYER_BACKUP: usize = 64;

/// A delta reference this many packets back or more is too old for the
/// client, which drops the packet. This is the client's frame mask, 64 - 1
const CLIENT_REJECTS_BACK: u8 = 63;

/// Whether the client would delta from `from` for a packet with this
/// sequence in its own numbering
fn client_accepts(from: u8, sequence: u32) -> bool {
    let back = (sequence as u8).wrapping_sub(from);

    (1..CLIENT_REJECTS_BACK).contains(&back)
}

/// The last delta entity packet of a session, to follow runs the client
/// rejects
#[derive(Clone, Copy, Debug)]
struct LastDelta {
    sequence: u32,
    /// Packets between it and its base in the recorded numbering
    back: u8,
    discarded: bool,
}

/// How many recent frames a count search may try
const COUNT_SEARCH_DEPTH: usize = 64;

/// A decoder's whole state, so a clone goes on exactly as the original
/// would. Tables that change only when a session sets them up are shared
/// between clones until one of them changes
#[derive(Clone)]
pub struct Decoder {
    meta: DeltaSchema,
    schemas: HashMap<Vec<u8>, Arc<DeltaSchema>>,
    user_messages: Arc<[Option<UserMessageDef>; 256]>,
    pub max_clients: u8,
    pub player_num: u8,
    pub hltv: bool,
    pub time_bits: u32,
    pub view_entity: Option<u16>,
    /// Body of the last `svc_newmovevars`
    pub movevars: Option<Vec<u8>>,
    baselines: Arc<HashMap<u16, Arc<EntityState>>>,
    instanced: Arc<Vec<Arc<StructState>>>,
    resources: Arc<Vec<Resource>>,
    event_names: Arc<HashMap<u16, Vec<u8>>>,
    /// Frame ring size, either 8 or 64
    backup: usize,
    /// Recently stored snapshots, newest last
    history: std::collections::VecDeque<Arc<Snapshot>>,
    /// Recorded minus server sequence, modulo 256
    sequence_offset: u8,
    /// This session's recorded sequences were shown not to be the server's
    /// own numbering, so no base in it can be strict any more. A same-byte
    /// match would name the wrong frame
    numbering_suspect: bool,
    last_delta: Option<LastDelta>,
    /// Recorded sequence of the last network frame
    last_recorded: Option<u32>,
    /// First recorded sequence of the current numbering run. Playback
    /// recordings continue the netchan numbering of a connection whose
    /// earlier packets were never recorded, so it jumps from the loading
    /// segment, for example from 134 to 83458
    run_start: u32,

    /// Current occupant of each player slot in this session
    pub players: BTreeMap<u8, PlayerInfo>,
    /// Joins, renames and departures in recording order. Each session starts
    /// with no players, so its occupants join again
    pub name_history: Vec<NameChange>,
    /// Last incarnation per slot in this session, kept after the slot is
    /// vacated so the next occupant gets a new one
    incarnations: BTreeMap<u8, u32>,
    pub segment: u32,
    pub stats: DecodeStats,
    pub events: Vec<DecodedEvent>,
    /// Keep decoded events in `events`
    pub collect_events: bool,
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot under construction for the packet being decoded
struct Building {
    sequence: u32,
    /// Delta clientdata waiting for the entity packet of the same frame to
    /// select the delta base, stored as (message, ordinal, payload offset)
    pending_clientdata: Option<(ClientDataMessage, u32, u32)>,
    /// Base chosen by this frame's delta entity packet, reused by its
    /// clientdata, which carries the same reference byte
    entity_base: Option<(u8, Arc<Snapshot>, BaseResolution)>,
    /// Some part of this packet's state rests on an inferred base
    inferred: bool,
    /// The client rejects this packet's delta entity update as too old, so
    /// the packet's entity and client state never reach the client
    discarded: bool,
    entities: Option<Arc<Vec<Arc<EntityState>>>>,
    clientdata: Option<Arc<StructState>>,
    weapons: Option<Arc<Vec<Option<Arc<StructState>>>>>,
}

fn userinfo_value(info: &[u8], key: &[u8]) -> Option<Vec<u8>> {
    // Infostrings are "\key\value\key\value"
    let mut parts = info.split(|&b| b == b'\\');
    if info.first() == Some(&b'\\') {
        parts.next();
    }

    while let (Some(k), Some(v)) = (parts.next(), parts.next()) {
        if k == key {
            return Some(v.to_vec());
        }
    }

    None
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            meta: meta_schema(),
            schemas: HashMap::new(),
            user_messages: Arc::new([const { None }; 256]),
            max_clients: 0,
            player_num: 0,
            hltv: false,
            time_bits: 0,
            view_entity: None,
            movevars: None,
            baselines: Arc::default(),
            instanced: Arc::default(),
            resources: Arc::default(),
            event_names: Arc::default(),
            backup: MULTIPLAYER_BACKUP,
            history: std::collections::VecDeque::new(),
            sequence_offset: 0,
            numbering_suspect: false,
            last_delta: None,
            last_recorded: None,
            run_start: 0,

            players: BTreeMap::new(),
            name_history: Vec::new(),
            incarnations: BTreeMap::new(),
            segment: 0,
            stats: DecodeStats::default(),
            events: Vec::new(),
            collect_events: true,
        }
    }

    pub fn schema(&self, name: &str) -> Option<Arc<DeltaSchema>> {
        self.schemas.get(name.as_bytes()).cloned()
    }

    pub fn resources(&self) -> &[Resource] {
        &self.resources
    }

    pub fn event_name(&self, index: u16) -> Option<&[u8]> {
        self.event_names.get(&index).map(Vec::as_slice)
    }

    pub fn user_message(&self, id: u8) -> Option<&UserMessageDef> {
        self.user_messages[usize::from(id)].as_ref()
    }

    /// Spawn baseline of an entity number in this session
    pub fn baseline(&self, number: u16) -> Option<&Arc<EntityState>> {
        self.baselines.get(&number)
    }

    /// Number of instanced baselines in this session
    pub fn instanced_baselines(&self) -> usize {
        self.instanced.len()
    }

    /// The schema an entity is sent with, like the engine's choice between
    /// custom, player and ordinary entities
    pub fn entity_schema(&self, number: u16, custom: bool) -> Option<Arc<DeltaSchema>> {
        let name = if custom {
            "custom_entity_state_t"
        } else if number >= 1 && number <= u16::from(self.max_clients) {
            "entity_state_player_t"
        } else {
            "entity_state_t"
        };

        self.schema(name)
    }

    /// Server frame window in use (8 or 64)
    pub fn backup(&self) -> usize {
        self.backup
    }

    fn server_sequence(&self, recorded: u32) -> u32 {
        recorded.wrapping_sub(self.sequence_offset.into())
    }

    /// About how many bytes the stored snapshots and the name history hold,
    /// for budgeting copies of this decoder. A state several snapshots share
    /// counts once, and the tables a session sets up are left out, as
    /// copies share them
    pub fn retained_bytes(&self) -> usize {
        let mut seen = std::collections::HashSet::new();
        let mut seen_states = std::collections::HashSet::new();
        let mut bytes = 0;
        let mut state = |s: &Arc<StructState>, size: usize| {
            if seen_states.insert(Arc::as_ptr(s) as usize) {
                size + s.heap_bytes()
            } else {
                0
            }
        };
        for snapshot in &self.history {
            bytes += size_of::<Snapshot>();
            if let Some(entities) = &snapshot.entities {
                bytes += entities.capacity() * size_of::<Arc<EntityState>>();
                for e in entities.iter() {
                    if seen.insert(Arc::as_ptr(e) as usize) {
                        bytes += size_of::<EntityState>() + e.fields.heap_bytes();
                    }
                }
            }
            if let Some(c) = &snapshot.clientdata {
                bytes += state(c, size_of::<StructState>());
            }
            if let Some(weapons) = &snapshot.weapons {
                bytes += weapons.capacity() * size_of::<Option<Arc<StructState>>>();
                for w in weapons.iter().flatten() {
                    bytes += state(w, size_of::<StructState>());
                }
            }
        }
        bytes + self.name_history.capacity() * size_of::<NameChange>()
    }

    /// The most recently stored snapshot
    pub fn latest_snapshot(&self) -> Option<&Arc<Snapshot>> {
        self.history.back()
    }

    /// The newest stored snapshot with exactly this recorded sequence
    pub fn snapshot(&self, sequence: u32) -> Option<&Arc<Snapshot>> {
        self.history.iter().rev().find(|s| s.sequence == sequence)
    }

    /// The newest stored snapshot whose sequence, under the current inferred
    /// offset, has this low byte
    pub fn snapshot_for_byte(&self, byte: u8) -> Option<&Arc<Snapshot>> {
        self.history
            .iter()
            .rev()
            .find(|s| self.server_sequence(s.sequence) as u8 == byte)
    }

    /// The frame the client resolves reference `from` to, when the recorded
    /// sequences are the client's own. Like the engine, that is the newest
    /// frame in ring slot `from & (backup - 1)`. It must be at most `backup`
    /// packets old, because an older occupant means the slot's real content
    /// was never recorded, like a loading-segment frame from far earlier.
    ///
    /// A reference more than `backup` packets back names a slot the server
    /// has since overwritten, so the newer occupant is right. Exactly
    /// `backup` back happens when the slot is the one being written, and the
    /// delta then carries no updates
    fn strict_base(&self, from: u8, sequence: u32) -> Option<Arc<Snapshot>> {
        if self.numbering_suspect || !client_accepts(from, sequence) {
            return None;
        }

        let mask = self.backup as u32 - 1;
        let occupant = self
            .history
            .iter()
            .rev()
            .find(|s| s.sequence & mask == u32::from(from) & mask)?;

        let distance = sequence.wrapping_sub(occupant.sequence);
        if distance == 0 || distance > self.backup as u32 {
            return None;
        }

        Some(occupant.clone())
    }

    /// Whether the client drops a delta entity packet as too old. It counts
    /// back from the packet's own sequence in its numbering and rejects a
    /// reference to the packet itself or 63 or more packets back, then
    /// treats the frame as invalid and asks for a full update. The server
    /// keeps referencing the last frame the client acknowledged however old
    /// it is, so a lagging client receives runs of such packets.
    ///
    /// A run starts only where the distance grows across the limit, from an
    /// accepted packet at most two sequences earlier. It lasts until a
    /// reference names a frame the client kept, because the client only
    /// acknowledges those. A run longer than 255 packets wraps the byte back
    /// into range, but it then names a dropped frame. A far reference that
    /// appears suddenly points to shifted numbering instead and goes to the
    /// count search. Once the numbering is suspect the recorded sequences
    /// are not the client's, so the rule stays off and the inferred state is
    /// reported
    fn client_rejects(&self, from: u8, sequence: u32) -> bool {
        if self.numbering_suspect {
            return false;
        }

        let Some(previous) = self.last_delta else {
            return false;
        };
        let accepts = client_accepts(from, sequence);
        if previous.discarded {
            let back = (sequence as u8).wrapping_sub(from);
            let named = sequence.wrapping_sub(u32::from(back));
            let kept = self.history.iter().any(|s| s.sequence == named);

            return !accepts || !kept;
        }

        if accepts {
            return false;
        }

        let gap = sequence.wrapping_sub(previous.sequence);

        (1..=2).contains(&gap) && u32::from(previous.back) + gap >= u32::from(CLIENT_REJECTS_BACK)
    }

    /// Follow the recorded numbering. A step back or 63 or more packets
    /// forward starts a new run, because no reference the client accepts
    /// can reach across it. A lag burst can leave such a gap too, and a run
    /// of dropped packets goes on across it
    fn note_recorded(&mut self, sequence: u32) {
        let continues = self
            .last_recorded
            .is_some_and(|last| sequence.wrapping_sub(last) < u32::from(CLIENT_REJECTS_BACK));
        if !continues {
            self.run_start = sequence;
        }

        self.last_recorded = Some(sequence);
    }

    /// Whether `from` names a frame received before the first recorded
    /// packet of this numbering run. Recording can begin while the client
    /// still acknowledges such a frame, so the base exists but was never
    /// recorded. That says nothing about the numbering, and later references
    /// stay strict
    fn unrecorded_base(&self, from: u8, sequence: u32) -> bool {
        if self.numbering_suspect || !client_accepts(from, sequence) {
            return false;
        }

        let back = (sequence as u8).wrapping_sub(from);
        let into_run = sequence.wrapping_sub(self.run_start);

        u32::from(back) > into_run
    }

    /// A frame matching `from` under an offset inferred earlier in this
    /// session
    fn offset_base(&self, from: u8) -> Option<Arc<Snapshot>> {
        if !self.numbering_suspect {
            return None;
        }

        self.snapshot_for_byte(from).cloned()
    }

    fn count_resolution(&mut self, resolution: BaseResolution) {
        let counter = match resolution {
            BaseResolution::Strict => &mut self.stats.bases_strict,
            BaseResolution::Offset => &mut self.stats.bases_offset,
            BaseResolution::CountSearch => &mut self.stats.bases_count_search,
            BaseResolution::Empty => &mut self.stats.bases_empty,
        };
        *counter += 1;
    }

    fn empty_snapshot(sequence: u32) -> Arc<Snapshot> {
        Arc::new(Snapshot {
            sequence,
            server_time_bits: 0,
            inferred: true,
            entities: None,
            clientdata: None,
            weapons: None,
        })
    }

    /// Decode one network record. `on_message` sees each message after it
    /// has been applied to the state
    pub fn network_frame(
        &mut self,
        location: FrameLocation,
        frame: &NetworkFrame,
        mut on_message: impl FnMut(&Decoder, &ParsedMessage),
    ) -> Result<(), DecodeError> {
        self.stats.network_frames += 1;
        self.stats.max_payload = self.stats.max_payload.max(frame.message.len());

        let sequence = frame.sequence[0] as u32;
        self.note_recorded(sequence);

        let mut building = Building {
            sequence,
            pending_clientdata: None,
            entity_base: None,
            inferred: false,
            discarded: false,
            entities: None,
            clientdata: None,
            weapons: None,
        };

        let mut r = BitReader::new(&frame.message);
        let mut ordinal = 0u32;
        while !r.is_empty() {
            let at = r.byte_pos() as u32;
            let opcode = frame.message[at as usize];
            let fail = |kind: DecodeErrorKind| DecodeError {
                location,
                message_ordinal: ordinal,
                opcode,
                payload_offset: at,
                kind,
            };

            let parsed = {
                let schemas = &self.schemas;
                let lookup = |name: &'static str| schemas.get(name.as_bytes()).cloned();
                let ctx = ParseContext {
                    meta: &self.meta,
                    schema: &lookup,
                    user_messages: &self.user_messages,
                    max_clients: self.max_clients,
                    instanced_baselines: self.instanced.len() as u8,
                    hltv: self.hltv,
                    time_bits: self.time_bits,
                };
                parse_message(&mut r, &ctx).map_err(|e| fail(e.into()))?
            };
            self.apply(&parsed, &mut building, location, ordinal)
                .map_err(fail)?;

            self.stats.messages += 1;
            *self
                .stats
                .by_opcode
                .entry(svc::name(parsed.opcode).to_owned())
                .or_default() += 1;
            if let Message::User { id, .. } = &parsed.message {
                let name = self.user_messages[usize::from(*id)]
                    .as_ref()
                    .map_or_else(|| format!("#{id}"), UserMessageDef::name_lossy);
                *self.stats.user_messages.entry(name).or_default() += 1;
            }

            on_message(self, &parsed);
            ordinal += 1;
        }

        if let Some((cd, ordinal, at)) = building.pending_clientdata.take() {
            self.apply_clientdata(&cd, &mut building)
                .map_err(|kind| DecodeError {
                    location,
                    message_ordinal: ordinal,
                    opcode: svc::CLIENTDATA,
                    payload_offset: at,
                    kind,
                })?;
        }

        self.finish_packet(building, location);
        Ok(())
    }

    fn finish_packet(&mut self, b: Building, location: FrameLocation) {
        if b.discarded || (b.entities.is_none() && b.clientdata.is_none()) {
            return;
        }

        if b.inferred {
            self.stats.inferred_snapshots += 1;
            self.stats.first_inferred.get_or_insert(location);
        }

        if self
            .history
            .back()
            .is_some_and(|s| s.sequence == b.sequence)
        {
            self.stats.duplicate_sequences += 1;
        }

        if self.history.len() == 256 {
            self.history.pop_front();
        }
        self.history.push_back(Arc::new(Snapshot {
            sequence: b.sequence,
            server_time_bits: self.time_bits,
            inferred: b.inferred,
            entities: b.entities,
            clientdata: b.clientdata,
            weapons: b.weapons,
        }));
        self.stats.snapshots += 1;
    }

    /// Base for a clientdata delta with no entity packet to settle it. Only
    /// the strict and earlier-offset rules apply because there is no count to
    /// check against
    fn clientdata_base(&mut self, from: u8, sequence: u32) -> (Arc<Snapshot>, BaseResolution) {
        if let Some(snap) = self.strict_base(from, sequence) {
            return (snap, BaseResolution::Strict);
        }

        if let Some(snap) = self.offset_base(from) {
            return (snap, BaseResolution::Offset);
        }

        (Self::empty_snapshot(sequence), BaseResolution::Empty)
    }

    fn new_segment(&mut self) {
        self.segment += 1;
        self.stats.segments += 1;
        self.baselines = Arc::default();
        self.instanced = Arc::default();
        self.resources = Arc::default();
        self.event_names = Arc::default();
        self.history.clear();
        self.sequence_offset = 0;
        self.numbering_suspect = false;
        self.last_delta = None;

        // A new server info starts a new client state, and the server sends
        // every player's userinfo again during sign-on. Until then a slot has
        // no known occupant, so no identity outlives its session
        self.players.clear();
        self.incarnations.clear();

        self.hltv = false;
    }

    fn apply(
        &mut self,
        parsed: &ParsedMessage,
        building: &mut Building,
        location: FrameLocation,
        ordinal: u32,
    ) -> Result<(), DecodeErrorKind> {
        match &parsed.message {
            Message::Time(bits) => self.time_bits = *bits,
            Message::ServerInfo(info) => {
                if info.protocol != profile::NETWORK_PROTOCOL {
                    return Err(DecodeErrorKind::UnsupportedProtocol(info.protocol));
                }

                self.new_segment();

                self.max_clients = info.max_clients;
                let backup = if info.max_clients == 1 {
                    SINGLEPLAYER_BACKUP
                } else {
                    MULTIPLAYER_BACKUP
                };
                self.backup = backup;
                self.player_num = info.player_num;
            }
            Message::DeltaDescription(schema) => {
                self.schemas.insert(schema.name.clone(), schema.clone());
            }
            Message::NewUserMessage { id, def } => {
                Arc::make_mut(&mut self.user_messages)[usize::from(*id)] = Some(def.clone());
            }
            Message::Hltv(0) => self.hltv = true,
            Message::SetView(e) => self.view_entity = Some(*e),
            Message::NewMoveVars(b) => self.movevars = Some(b.clone()),
            Message::ResourceList { resources, .. } => {
                self.resources = Arc::new(resources.clone());
                self.event_names = Arc::new(
                    resources
                        .iter()
                        .filter(|r| r.kind == resource_type::EVENT_SCRIPT)
                        .map(|r| (r.index, r.name.clone()))
                        .collect(),
                );
            }
            Message::UpdateUserInfo {
                slot,
                user_id,
                info,
                ..
            } => self.update_player(*slot, *user_id, info, location),
            Message::SpawnBaseline(b) => {
                let mut baselines = HashMap::new();
                for e in &b.entities {
                    let fields = StructState::apply(&StructState::default(), &e.schema, &e.delta);
                    baselines.insert(
                        e.number,
                        Arc::new(EntityState {
                            number: e.number,
                            custom: e.entity_type & 1 == 0,
                            fields,
                        }),
                    );
                }
                self.baselines = Arc::new(baselines);

                let schema = self.schemas.get(&b"entity_state_t"[..]).cloned();
                self.instanced = Arc::new(match schema {
                    Some(s) => b
                        .instanced
                        .iter()
                        .map(|d| Arc::new(StructState::apply(&StructState::default(), &s, d)))
                        .collect(),
                    None => Vec::new(),
                });
            }
            Message::ClientData(_) if building.discarded => {}
            Message::ClientData(cd) => match (cd.delta_from, &cd.client) {
                (Some(_), Some(_)) => {
                    building.pending_clientdata = Some((cd.clone(), ordinal, parsed.start));
                }
                _ => self.apply_clientdata(cd, building)?,
            },
            Message::PacketEntities(p) => {
                self.stats.full_packets += 1;
                self.last_delta = None;
                let ents = self.full_entities(p)?;
                self.stats.max_entities = self.stats.max_entities.max(ents.len());
                building.entities = Some(Arc::new(ents));
            }
            Message::DeltaPacketEntities(p) => {
                self.stats.delta_packets += 1;
                let from = p.delta_from.unwrap_or(0);
                let discarded = self.client_rejects(from, building.sequence);
                self.last_delta = Some(LastDelta {
                    sequence: building.sequence,
                    back: (building.sequence as u8).wrapping_sub(from),
                    discarded,
                });
                if discarded {
                    self.stats.discarded_packets += 1;
                    building.discarded = true;
                    building.pending_clientdata = None;
                    return Ok(());
                }

                let ents = self.delta_entities(p, building)?;
                if let Some((cd, _, _)) = building.pending_clientdata.take() {
                    self.apply_clientdata(&cd, building)?;
                }

                self.stats.max_entities = self.stats.max_entities.max(ents.len());
                building.entities = Some(Arc::new(ents));
            }
            Message::Event(entries) => {
                let schema = self.schemas.get(&b"event_t"[..]).cloned();
                for e in entries {
                    let args = match (&e.args, &schema) {
                        (Some(d), Some(s)) => StructState::apply(&StructState::default(), s, d),
                        _ => StructState::default(),
                    };
                    let explicit = schema
                        .as_ref()
                        .and_then(|s| args.value(s, "entindex"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32;

                    let actor = match (e.packet_index, &building.entities) {
                        (None, _) => ActorResolution::Unknown,
                        (Some(pi), None) => ActorResolution::NoPacketContext { packet_index: pi },
                        (Some(pi), Some(ents)) => match ents.get(usize::from(pi)) {
                            Some(ent) => ActorResolution::PacketEntity {
                                packet_index: pi,
                                entity: ent.number,
                            },
                            None if usize::from(pi) == ents.len() && explicit > 0 => {
                                ActorResolution::Explicit { entity: explicit }
                            }
                            None => ActorResolution::OutOfRange {
                                packet_index: pi,
                                entities: ents.len(),
                            },
                        },
                    };

                    self.record_event(DecodedEvent {
                        location,
                        segment: self.segment,
                        source: EventSource::Svc,
                        message_ordinal: Some(ordinal),
                        index: e.index,
                        resource: self.event_names.get(&e.index).cloned(),
                        args: Some(args),
                        demo: None,
                        actor,
                        player: None,
                        delay_raw: e.fire_time.map(u32::from),
                        server_time_bits: self.time_bits,
                    });
                }
            }
            Message::EventReliable(e) => {
                let schema = self.schemas.get(&b"event_t"[..]).cloned();
                let args = match &schema {
                    Some(s) => StructState::apply(&StructState::default(), s, &e.args),
                    None => StructState::default(),
                };
                let explicit = schema
                    .as_ref()
                    .and_then(|s| args.value(s, "entindex"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;

                self.record_event(DecodedEvent {
                    location,
                    segment: self.segment,
                    source: EventSource::SvcReliable,
                    message_ordinal: Some(ordinal),
                    index: e.index,
                    resource: self.event_names.get(&e.index).cloned(),
                    args: Some(args),
                    demo: None,
                    actor: if explicit > 0 {
                        ActorResolution::Explicit { entity: explicit }
                    } else {
                        ActorResolution::Unknown
                    },
                    player: None,
                    delay_raw: e.delay.map(u32::from),
                    server_time_bits: self.time_bits,
                });
            }
            _ => {}
        }

        Ok(())
    }

    fn update_player(&mut self, slot: u8, user_id: u32, info: &[u8], location: FrameLocation) {
        let name = userinfo_value(info, b"name").map(|n| String::from_utf8_lossy(&n).into_owned());

        // The server clears a dropped client's userinfo
        let Some(name) = name else {
            if let Some(gone) = self.players.remove(&slot) {
                self.incarnations.insert(slot, gone.incarnation);
                let player = self.identity(&gone);
                self.name_history.push(NameChange {
                    location,
                    kind: NameChangeKind::Left,
                    player,
                });
            }
            return;
        };

        let previous = self.players.get(&slot);
        let same_player = previous.is_some_and(|p| p.user_id == user_id);
        let kind = match previous {
            Some(p) if same_player && p.name == name => None,
            Some(_) if same_player => Some(NameChangeKind::Renamed),
            _ => Some(NameChangeKind::Joined),
        };

        let incarnation = match (previous, self.incarnations.get(&slot)) {
            (Some(p), _) if same_player => p.incarnation,
            (Some(p), _) => p.incarnation + 1,
            (None, Some(&last)) => last + 1,
            (None, None) => 0,
        };
        self.incarnations.insert(slot, incarnation);

        let info = PlayerInfo {
            slot,
            user_id,
            info: info.to_vec(),
            name,
            incarnation,
        };
        if let Some(kind) = kind {
            let player = self.identity(&info);
            self.name_history.push(NameChange {
                location,
                kind,
                player,
            });
        }

        self.players.insert(slot, info);
    }

    fn identity(&self, p: &PlayerInfo) -> PlayerIdentity {
        PlayerIdentity {
            segment: self.segment,
            slot: p.slot,
            user_id: p.user_id,
            incarnation: p.incarnation,
            name: p.name.clone(),
        }
    }

    /// The player occupying an entity's slot right now, when the entity is a
    /// player entity (1 to `max_clients`)
    pub fn player_for_entity(&self, entity: i64) -> Option<PlayerIdentity> {
        if entity < 1 || entity > i64::from(self.max_clients) {
            return None;
        }

        let slot = u8::try_from(entity - 1).ok()?;
        self.players.get(&slot).map(|p| self.identity(p))
    }

    fn record_event(&mut self, mut ev: DecodedEvent) {
        let entity = match ev.actor {
            ActorResolution::PacketEntity { entity, .. } => Some(i64::from(entity)),
            ActorResolution::Explicit { entity } => Some(i64::from(entity)),
            _ => None,
        };
        ev.player = entity.and_then(|e| self.player_for_entity(e));

        let key = ev.resource.as_ref().map_or_else(
            || format!("#{}", ev.index),
            |r| String::from_utf8_lossy(r).into_owned(),
        );
        *self.stats.events_by_resource.entry(key).or_default() += 1;

        if !matches!(
            ev.actor,
            ActorResolution::PacketEntity { .. } | ActorResolution::Explicit { .. }
        ) {
            self.stats.unresolved_event_actors += 1;
        }

        if self.collect_events {
            self.events.push(ev);
        }
    }

    /// Standalone demo `Event` record (outer tag 6)
    pub fn demo_event(&mut self, location: FrameLocation, record: &EventRecord) {
        let entity = record.entity_index();
        let index = u16::try_from(record.index).unwrap_or(u16::MAX);

        self.record_event(DecodedEvent {
            location,
            segment: self.segment,
            source: EventSource::Demo,
            message_ordinal: None,
            index,
            resource: self.event_names.get(&index).cloned(),
            args: None,
            demo: Some(record.clone()),
            actor: if entity > 0 {
                ActorResolution::Explicit { entity }
            } else {
                ActorResolution::Unknown
            },
            player: None,
            delay_raw: Some(record.delay_bits),
            server_time_bits: self.time_bits,
        });
    }

    fn apply_clientdata(
        &mut self,
        cd: &ClientDataMessage,
        building: &mut Building,
    ) -> Result<(), DecodeErrorKind> {
        let Some(update) = &cd.client else {
            return Ok(());
        };

        let empty = || (Arc::new(StructState::default()), Arc::new(vec![None; 64]));
        let (base_client, base_weapons) = match cd.delta_from {
            Some(from) => {
                // The entity packet of the same frame carries the same
                // reference and has already chosen, with a count to check
                let (snap, resolution) = match &building.entity_base {
                    Some((f, snap, resolution)) if *f == from => (snap.clone(), *resolution),
                    _ => {
                        let (snap, resolution) = self.clientdata_base(from, building.sequence);
                        self.count_resolution(resolution);
                        (snap, resolution)
                    }
                };
                building.inferred |= resolution.inferred() || snap.inferred;

                match (snap.clientdata.clone(), snap.weapons.clone()) {
                    (Some(c), Some(w)) => (c, w),
                    // A base without clientdata leaves nothing to delta from
                    _ => {
                        building.inferred = true;
                        empty()
                    }
                }
            }
            None => empty(),
        };

        let cschema = self.schema("clientdata_t").ok_or(DecodeErrorKind::Message(
            MessageError::MissingSchema("clientdata_t"),
        ))?;
        let client = StructState::apply(&base_client, &cschema, update);

        let mut weapons = (*base_weapons).clone();
        if !cd.weapons.is_empty() {
            let wschema = self
                .schema("weapon_data_t")
                .ok_or(DecodeErrorKind::Message(MessageError::MissingSchema(
                    "weapon_data_t",
                )))?;

            let empty = StructState::default();
            for (slot, d) in &cd.weapons {
                let slot = usize::from(*slot);
                let base = weapons[slot].as_deref().unwrap_or(&empty);
                weapons[slot] = Some(Arc::new(StructState::apply(base, &wschema, d)));
            }
        }

        building.clientdata = Some(Arc::new(client));
        building.weapons = Some(Arc::new(weapons));
        Ok(())
    }

    fn baseline_for(&mut self, number: u16) -> StructState {
        match self.baselines.get(&number) {
            Some(b) => b.fields.clone(),
            None => {
                self.stats.entities_without_baseline += 1;
                StructState::default()
            }
        }
    }

    fn instanced_baseline(&self, index: u8) -> Result<StructState, DecodeErrorKind> {
        self.instanced
            .get(usize::from(index))
            .map(|s| (**s).clone())
            .ok_or(DecodeErrorKind::InstancedBaselineOutOfRange {
                index,
                available: self.instanced.len(),
            })
    }

    fn full_entities(
        &mut self,
        p: &EntityPacket,
    ) -> Result<Vec<Arc<EntityState>>, DecodeErrorKind> {
        let mut out: Vec<Arc<EntityState>> = Vec::with_capacity(p.updates.len());
        for u in &p.updates {
            let base = if let Some(i) = u.new_baseline {
                self.instanced_baseline(i)?
            } else if u.offset > 0 {
                let pos = out.len().checked_sub(usize::from(u.offset)).ok_or(
                    DecodeErrorKind::OffsetOutOfRange {
                        offset: u.offset,
                        position: out.len(),
                    },
                )?;
                out[pos].fields.clone()
            } else {
                self.baseline_for(u.number)
            };

            push_ordered(&out, u.number)?;
            let (Some(schema), Some(delta)) = (&u.schema, &u.delta) else {
                return Err(DecodeErrorKind::Other(
                    "full packet entity without delta".into(),
                ));
            };
            out.push(Arc::new(EntityState {
                number: u.number,
                custom: u.custom,
                fields: StructState::apply(&base, schema, delta),
            }));
        }

        if usize::from(p.declared_count) != out.len() {
            return Err(DecodeErrorKind::CountMismatch {
                declared: p.declared_count,
                actual: out.len(),
            });
        }

        Ok(out)
    }

    /// Rebuild a delta entity packet, choosing its base strictly when the
    /// recording allows and by inference otherwise. The choice is recorded in
    /// `building` so the frame's clientdata uses the same base
    fn delta_entities(
        &mut self,
        p: &EntityPacket,
        building: &mut Building,
    ) -> Result<Vec<Arc<EntityState>>, DecodeErrorKind> {
        let from = p.delta_from.unwrap_or(0);
        let sequence = building.sequence;
        let entities_of = |s: &Arc<Snapshot>| s.entities.clone().unwrap_or_default();

        // Strict and offset bases must also reproduce the declared count, or
        // they are not the frame the server used
        let mut first_error = None;
        let named = [
            (self.strict_base(from, sequence), BaseResolution::Strict),
            (self.offset_base(from), BaseResolution::Offset),
        ];
        for (candidate, resolution) in named {
            let Some(snap) = candidate else {
                continue;
            };

            match self.rebuild(p, &entities_of(&snap)) {
                Ok(r) => {
                    if resolution == BaseResolution::Strict {
                        let distance = sequence.wrapping_sub(snap.sequence);
                        self.stats.max_delta_distance = self.stats.max_delta_distance.max(distance);
                    }

                    return Ok(self.accept(r, from, snap, resolution, building));
                }
                Err(e) => {
                    first_error.get_or_insert(e);
                }
            }
        }

        // No named frame works. Some recordings store a sequence counter that
        // drifts from the server's numbering, so try recent frames, nearest
        // first, and keep the first whose rebuild has the declared count. A
        // base from before the recording is guessed the same way, but its
        // absence is no sign of drift
        let unrecorded = self.unrecorded_base(from, sequence);
        let depth = self.history.len().min(COUNT_SEARCH_DEPTH);
        for back in 0..depth {
            let snap = self.history[self.history.len() - 1 - back].clone();
            let Ok(r) = self.rebuild(p, &entities_of(&snap)) else {
                continue;
            };

            if unrecorded {
                self.stats.unrecorded_bases += 1;
            } else {
                self.numbering_suspect = true;
                self.sequence_offset = (snap.sequence as u8).wrapping_sub(from);
                self.stats.sequence_offset = self.sequence_offset;
                self.stats.sequence_offset_changes += 1;
            }

            return Ok(self.accept(r, from, snap, BaseResolution::CountSearch, building));
        }

        if let Some(e) = first_error {
            return Err(e);
        }

        // Nothing received yet, so rebuild from an empty base
        let snap = Self::empty_snapshot(sequence);
        let r = self.rebuild(p, &[])?;
        Ok(self.accept(r, from, snap, BaseResolution::Empty, building))
    }

    fn accept(
        &mut self,
        r: Rebuilt,
        from: u8,
        base: Arc<Snapshot>,
        resolution: BaseResolution,
        building: &mut Building,
    ) -> Vec<Arc<EntityState>> {
        self.stats.removed_absent_entities += r.removed_absent;
        self.stats.entities_without_baseline += r.without_baseline;

        self.count_resolution(resolution);
        building.inferred |= resolution.inferred() || base.inferred;
        building.entity_base = Some((from, base, resolution));

        r.entities
    }

    /// Apply a delta entity packet to `old` without touching decoder state
    fn rebuild(
        &self,
        p: &EntityPacket,
        old: &[Arc<EntityState>],
    ) -> Result<Rebuilt, DecodeErrorKind> {
        let mut r = Rebuilt {
            entities: Vec::with_capacity(old.len() + 8),
            removed_absent: 0,
            without_baseline: 0,
        };
        let out = &mut r.entities;
        let mut oi = 0;
        for u in &p.updates {
            while oi < old.len() && old[oi].number < u.number {
                push_ordered(out, old[oi].number)?;
                out.push(old[oi].clone());
                oi += 1;
            }

            let same = oi < old.len() && old[oi].number == u.number;
            if u.remove {
                if same {
                    oi += 1;
                } else {
                    r.removed_absent += 1;
                }
                continue;
            }

            let (Some(schema), Some(delta)) = (&u.schema, &u.delta) else {
                return Err(DecodeErrorKind::Other(
                    "delta packet entity without delta".into(),
                ));
            };

            let base = if same {
                let b = old[oi].fields.clone();
                oi += 1;
                b
            } else if let Some(i) = u.new_baseline {
                self.instanced_baseline(i)?
            } else {
                match self.baselines.get(&u.number) {
                    Some(b) => b.fields.clone(),
                    None => {
                        r.without_baseline += 1;
                        StructState::default()
                    }
                }
            };

            push_ordered(out, u.number)?;
            out.push(Arc::new(EntityState {
                number: u.number,
                custom: u.custom,
                fields: StructState::apply(&base, schema, delta),
            }));
        }
        out.extend(old[oi..].iter().cloned());

        if usize::from(p.declared_count) != out.len() {
            return Err(DecodeErrorKind::CountMismatch {
                declared: p.declared_count,
                actual: out.len(),
            });
        }

        Ok(r)
    }

    /// Converted value of a named entity field, using the schema that applies
    /// to that entity
    pub fn entity_value(&self, e: &EntityState, field: &str) -> Option<Value> {
        let schema = if e.custom {
            self.schema("custom_entity_state_t")
        } else if e.number >= 1 && e.number <= u16::from(self.max_clients) {
            self.schema("entity_state_player_t")
        } else {
            self.schema("entity_state_t")
        }?;
        e.fields.value(&schema, field)
    }
}

struct Rebuilt {
    entities: Vec<Arc<EntityState>>,
    removed_absent: u64,
    without_baseline: u64,
}

fn push_ordered(out: &[Arc<EntityState>], next: u16) -> Result<(), DecodeErrorKind> {
    match out.last() {
        Some(prev) if prev.number >= next => Err(DecodeErrorKind::Unordered {
            previous: prev.number,
            next,
        }),
        _ => Ok(()),
    }
}

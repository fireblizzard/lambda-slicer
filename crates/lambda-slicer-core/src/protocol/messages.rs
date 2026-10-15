//! Server-to-client message grammar for network protocol 48.
//!
//! Layouts follow the engine writers (ReHLDS `sv_main.cpp`, `pr_cmds.cpp`,
//! `delta.cpp`) and the Half-Life SDK `const.h`; see
//! `docs/protocol-profile-5-48.md` for sources and disagreements between
//! references. An opcode without a known layout stops decoding: messages are
//! not length-prefixed, so nothing after it can be located.

use std::sync::Arc;

use super::bit::{BitError, BitReader};
use super::delta::{DeltaError, DeltaSchema, DeltaUpdate, parse_delta, parse_description};

pub mod svc {
    pub const BAD: u8 = 0;
    pub const NOP: u8 = 1;
    pub const DISCONNECT: u8 = 2;
    pub const EVENT: u8 = 3;
    pub const VERSION: u8 = 4;
    pub const SETVIEW: u8 = 5;
    pub const SOUND: u8 = 6;
    pub const TIME: u8 = 7;
    pub const PRINT: u8 = 8;
    pub const STUFFTEXT: u8 = 9;
    pub const SETANGLE: u8 = 10;
    pub const SERVERINFO: u8 = 11;
    pub const LIGHTSTYLE: u8 = 12;
    pub const UPDATEUSERINFO: u8 = 13;
    pub const DELTADESCRIPTION: u8 = 14;
    pub const CLIENTDATA: u8 = 15;
    pub const STOPSOUND: u8 = 16;
    pub const PINGS: u8 = 17;
    pub const PARTICLE: u8 = 18;
    pub const DAMAGE: u8 = 19;
    pub const SPAWNSTATIC: u8 = 20;
    pub const EVENT_RELIABLE: u8 = 21;
    pub const SPAWNBASELINE: u8 = 22;
    pub const TEMPENTITY: u8 = 23;
    pub const SETPAUSE: u8 = 24;
    pub const SIGNONNUM: u8 = 25;
    pub const CENTERPRINT: u8 = 26;
    pub const KILLEDMONSTER: u8 = 27;
    pub const FOUNDSECRET: u8 = 28;
    pub const SPAWNSTATICSOUND: u8 = 29;
    pub const INTERMISSION: u8 = 30;
    pub const FINALE: u8 = 31;
    pub const CDTRACK: u8 = 32;
    pub const RESTORE: u8 = 33;
    pub const CUTSCENE: u8 = 34;
    pub const WEAPONANIM: u8 = 35;
    pub const DECALNAME: u8 = 36;
    pub const ROOMTYPE: u8 = 37;
    pub const ADDANGLE: u8 = 38;
    pub const NEWUSERMSG: u8 = 39;
    pub const PACKETENTITIES: u8 = 40;
    pub const DELTAPACKETENTITIES: u8 = 41;
    pub const CHOKE: u8 = 42;
    pub const RESOURCELIST: u8 = 43;
    pub const NEWMOVEVARS: u8 = 44;
    pub const RESOURCEREQUEST: u8 = 45;
    pub const CUSTOMIZATION: u8 = 46;
    pub const CROSSHAIRANGLE: u8 = 47;
    pub const SOUNDFADE: u8 = 48;
    pub const FILETXFERFAILED: u8 = 49;
    pub const HLTV: u8 = 50;
    pub const DIRECTOR: u8 = 51;
    pub const VOICEINIT: u8 = 52;
    pub const VOICEDATA: u8 = 53;
    pub const SENDEXTRAINFO: u8 = 54;
    pub const TIMESCALE: u8 = 55;
    pub const RESOURCELOCATION: u8 = 56;
    pub const SENDCVARVALUE: u8 = 57;
    pub const SENDCVARVALUE2: u8 = 58;
    /// First registered user message id.
    pub const FIRST_USER_MESSAGE: u8 = 64;

    const NAMES: [&str; 59] = [
        "svc_bad",
        "svc_nop",
        "svc_disconnect",
        "svc_event",
        "svc_version",
        "svc_setview",
        "svc_sound",
        "svc_time",
        "svc_print",
        "svc_stufftext",
        "svc_setangle",
        "svc_serverinfo",
        "svc_lightstyle",
        "svc_updateuserinfo",
        "svc_deltadescription",
        "svc_clientdata",
        "svc_stopsound",
        "svc_pings",
        "svc_particle",
        "svc_damage",
        "svc_spawnstatic",
        "svc_event_reliable",
        "svc_spawnbaseline",
        "svc_temp_entity",
        "svc_setpause",
        "svc_signonnum",
        "svc_centerprint",
        "svc_killedmonster",
        "svc_foundsecret",
        "svc_spawnstaticsound",
        "svc_intermission",
        "svc_finale",
        "svc_cdtrack",
        "svc_restore",
        "svc_cutscene",
        "svc_weaponanim",
        "svc_decalname",
        "svc_roomtype",
        "svc_addangle",
        "svc_newusermsg",
        "svc_packetentities",
        "svc_deltapacketentities",
        "svc_choke",
        "svc_resourcelist",
        "svc_newmovevars",
        "svc_resourcerequest",
        "svc_customization",
        "svc_crosshairangle",
        "svc_soundfade",
        "svc_filetxferfailed",
        "svc_hltv",
        "svc_director",
        "svc_voiceinit",
        "svc_voicedata",
        "svc_sendextrainfo",
        "svc_timescale",
        "svc_resourcelocation",
        "svc_sendcvarvalue",
        "svc_sendcvarvalue2",
    ];

    pub fn name(op: u8) -> &'static str {
        NAMES
            .get(usize::from(op))
            .copied()
            .unwrap_or(if op >= FIRST_USER_MESSAGE {
                "user_message"
            } else {
                "svc_unknown"
            })
    }
}

/// Byte length of each temp-entity body, `None` for unknown types and
/// `Some(-1)` for the two variable-length types. Agrees with ReHLDS's HLTV
/// table except TE_PLAYERSPRITES (121), which follows the SDK layout (5).
const TE_LENGTH: [i8; 128] = {
    let mut t = [-2i8; 128];
    let known: [(usize, i8); 52] = [
        (0, 24),
        (1, 20),
        (2, 6),
        (3, 11),
        (4, 6),
        (5, 10),
        (6, 12),
        (7, 17),
        (8, 16),
        (9, 6),
        (10, 6),
        (11, 6),
        (12, 8),
        (13, -1),
        (14, 9),
        (15, 19),
        (17, 10),
        (18, 16),
        (19, 24),
        (20, 24),
        (21, 24),
        (22, 10),
        (23, 11),
        (24, 16),
        (25, 19),
        (27, 12),
        (28, 16),
        (29, -1),
        (30, 17),
        (31, 17),
        (99, 2),
        (100, 10),
        (101, 14),
        (102, 12),
        (103, 14),
        (104, 9),
        (105, 5),
        (106, 17),
        (107, 13),
        (108, 24),
        (109, 9),
        (110, 17),
        (111, 7),
        (112, 10),
        (113, 19),
        (114, 19),
        (115, 12),
        (116, 7),
        (117, 7),
        (118, 9),
        (119, 16),
        (120, 18),
    ];
    let mut i = 0;
    while i < known.len() {
        t[known[i].0] = known[i].1;
        i += 1;
    }
    t[121] = 5;
    t[122] = 10;
    t[123] = 13;
    t[124] = 7;
    t[125] = 1;
    t[126] = 18;
    t[127] = 15;
    t
};

const TE_BSPDECAL: u8 = 13;
const TE_TEXTMESSAGE: u8 = 29;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum MessageError {
    #[error(transparent)]
    Bits(#[from] BitError),
    #[error(transparent)]
    Delta(#[from] DeltaError),
    #[error("opcode {0} has no known layout")]
    UnknownOpcode(u8),
    #[error("user message {0} was never registered")]
    UnknownUserMessage(u8),
    #[error("missing delta schema {0:?}")]
    MissingSchema(&'static str),
    #[error("temp entity type {0} has no known layout")]
    UnknownTempEntity(u8),
    #[error("{0}")]
    Unsupported(String),
    #[error("{what} count {count} exceeds limit {limit}")]
    TooMany {
        what: &'static str,
        count: usize,
        limit: usize,
    },
    #[error("bad terminator in {0}")]
    BadTerminator(&'static str),
}

/// Engine limit on entities per packet.
pub const MAX_PACKET_ENTITIES: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserMessageDef {
    pub name: [u8; 16],
    /// Fixed body size, or `None` for a byte-length-prefixed body.
    pub size: Option<u8>,
}

impl UserMessageDef {
    pub fn name_lossy(&self) -> String {
        crate::container::c_string_lossy(&self.name)
    }
}

/// The parts of decoder state that change how later bytes are parsed.
pub struct ParseContext<'a> {
    pub meta: &'a DeltaSchema,
    pub schema: &'a dyn Fn(&'static str) -> Option<Arc<DeltaSchema>>,
    pub user_messages: &'a [Option<UserMessageDef>; 256],
    pub max_clients: u8,
    pub instanced_baselines: u8,
    pub hltv: bool,
    pub time_bits: u32,
}

impl ParseContext<'_> {
    fn require(&self, name: &'static str) -> Result<Arc<DeltaSchema>, MessageError> {
        (self.schema)(name).ok_or(MessageError::MissingSchema(name))
    }

    fn entity_schema(&self, number: u16, custom: bool) -> Result<Arc<DeltaSchema>, MessageError> {
        if custom {
            self.require("custom_entity_state_t")
        } else if number >= 1 && number <= u16::from(self.max_clients) {
            self.require("entity_state_player_t")
        } else {
            self.require("entity_state_t")
        }
    }
}

/// Bit-coordinate as sent (`MSG_ReadBitCoord`): optional 12-bit integer and
/// 3-bit eighths parts with a sign.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitCoord {
    pub negative: bool,
    pub int: Option<u16>,
    pub frac: Option<u8>,
}

impl BitCoord {
    pub fn value(self) -> f32 {
        let v = f32::from(self.int.unwrap_or(0)) + f32::from(self.frac.unwrap_or(0)) / 8.0;
        if self.negative { -v } else { v }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoundMessage {
    pub flags: u16,
    pub volume: Option<u8>,
    pub attenuation: Option<u8>,
    pub channel: u8,
    pub entity: u16,
    pub sound_index: u16,
    pub origin: [Option<BitCoord>; 3],
    pub pitch: Option<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServerInfo {
    pub protocol: i32,
    pub spawn_count: i32,
    pub map_crc: i32,
    pub client_dll_md5: [u8; 16],
    pub max_clients: u8,
    pub player_num: u8,
    pub game_type: u8,
    pub game_dir: Vec<u8>,
    pub host_name: Vec<u8>,
    pub map_name: Vec<u8>,
    pub map_cycle: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Resource {
    pub kind: u8,
    pub name: Vec<u8>,
    pub index: u16,
    pub download_size: u32,
    pub flags: u8,
    pub md5: Option<[u8; 16]>,
    pub reserved: Option<[u8; 32]>,
}

pub mod resource_type {
    pub const SOUND: u8 = 0;
    pub const SKIN: u8 = 1;
    pub const MODEL: u8 = 2;
    pub const DECAL: u8 = 3;
    pub const GENERIC: u8 = 4;
    pub const EVENT_SCRIPT: u8 = 5;
    pub const WORLD: u8 = 6;
}

/// One entry of `svc_event`.
#[derive(Clone, Debug, PartialEq)]
pub struct EventEntry {
    pub index: u16,
    /// Position in this packet's entity array (not an entity number).
    pub packet_index: Option<u16>,
    pub args: Option<DeltaUpdate>,
    /// `fire_time * 100`.
    pub fire_time: Option<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReliableEvent {
    pub index: u16,
    pub args: DeltaUpdate,
    /// `delay * 100`.
    pub delay: Option<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntityUpdate {
    pub number: u16,
    pub remove: bool,
    pub custom: bool,
    /// Instanced baseline index.
    pub new_baseline: Option<u8>,
    /// Full packets: delta from the entity `offset` positions earlier in
    /// this packet.
    pub offset: u8,
    pub schema: Option<Arc<DeltaSchema>>,
    pub delta: Option<DeltaUpdate>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntityPacket {
    pub declared_count: u16,
    /// Low byte of the source sequence this delta packet is based on.
    pub delta_from: Option<u8>,
    pub updates: Vec<EntityUpdate>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BaselineEntry {
    pub number: u16,
    pub entity_type: u8,
    pub schema: Arc<DeltaSchema>,
    pub delta: DeltaUpdate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnBaseline {
    pub entities: Vec<BaselineEntry>,
    pub instanced: Vec<DeltaUpdate>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientDataMessage {
    pub delta_from: Option<u8>,
    /// `None` for HLTV recordings, whose clientdata body is empty.
    pub client: Option<DeltaUpdate>,
    pub weapons: Vec<(u8, DeltaUpdate)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Nop,
    Disconnect(Vec<u8>),
    Event(Vec<EventEntry>),
    Version(u32),
    SetView(u16),
    Sound(SoundMessage),
    /// f32 bits.
    Time(u32),
    Print(Vec<u8>),
    StuffText(Vec<u8>),
    SetAngle([u16; 3]),
    ServerInfo(Box<ServerInfo>),
    LightStyle {
        index: u8,
        style: Vec<u8>,
    },
    UpdateUserInfo {
        slot: u8,
        user_id: u32,
        info: Vec<u8>,
        hash: [u8; 16],
    },
    DeltaDescription(Arc<DeltaSchema>),
    ClientData(ClientDataMessage),
    StopSound(u16),
    /// Raw 24-bit records: slot (5), ping (12), loss (7).
    Pings(Vec<u32>),
    Particle(Vec<u8>),
    SpawnStatic(Vec<u8>),
    EventReliable(ReliableEvent),
    SpawnBaseline(SpawnBaseline),
    TempEntity {
        kind: u8,
        body: Vec<u8>,
    },
    SetPause(u8),
    SignOnNum(u8),
    CenterPrint(Vec<u8>),
    KilledMonster,
    FoundSecret,
    SpawnStaticSound(Vec<u8>),
    Intermission,
    Finale(Vec<u8>),
    CdTrack(u8, u8),
    Restore {
        save: Vec<u8>,
        maps: Vec<Vec<u8>>,
    },
    Cutscene(Vec<u8>),
    WeaponAnim {
        sequence: u8,
        body: u8,
    },
    DecalName {
        index: u8,
        name: Vec<u8>,
    },
    RoomType(u16),
    AddAngle(u16),
    NewUserMessage {
        id: u8,
        def: UserMessageDef,
    },
    PacketEntities(EntityPacket),
    DeltaPacketEntities(EntityPacket),
    Choke,
    ResourceList {
        resources: Vec<Resource>,
        /// Consistency list entries: (short form, value).
        consistency: Option<Vec<(bool, u16)>>,
    },
    NewMoveVars(Vec<u8>),
    ResourceRequest(u32, u32),
    Customization(Vec<u8>),
    CrosshairAngle(u8, u8),
    SoundFade([u8; 4]),
    FileTxferFailed(Vec<u8>),
    Hltv(u8),
    Director(Vec<u8>),
    VoiceInit {
        codec: Vec<u8>,
        quality: u8,
    },
    VoiceData {
        slot: u8,
        data: Vec<u8>,
    },
    SendExtraInfo {
        fallback_dir: Vec<u8>,
        can_cheat: u8,
    },
    /// f32 bits.
    TimeScale(u32),
    ResourceLocation(Vec<u8>),
    SendCvarValue(Vec<u8>),
    SendCvarValue2 {
        request: u32,
        name: Vec<u8>,
    },
    User {
        id: u8,
        body: Vec<u8>,
    },
}

/// A parsed message and its byte span within the network payload.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedMessage {
    pub opcode: u8,
    pub start: u32,
    pub end: u32,
    pub message: Message,
}

/// Parse the message starting at the reader's (byte-aligned) position.
pub fn parse_message(
    r: &mut BitReader<'_>,
    ctx: &ParseContext<'_>,
) -> Result<ParsedMessage, MessageError> {
    let start = r.byte_pos();
    let opcode = r.u8()?;
    let message = parse_body(r, ctx, opcode)?;
    r.align();
    Ok(ParsedMessage {
        opcode,
        start: start as u32,
        end: r.byte_pos() as u32,
        message,
    })
}

fn fixed(r: &mut BitReader<'_>, n: usize) -> Result<Vec<u8>, MessageError> {
    Ok(r.bytes(n)?.to_vec())
}

fn array<const N: usize>(r: &mut BitReader<'_>) -> Result<[u8; N], MessageError> {
    Ok(r.bytes(N)?.try_into().unwrap())
}

fn bit_bytes<const N: usize>(r: &mut BitReader<'_>) -> Result<[u8; N], MessageError> {
    let mut out = [0u8; N];
    for b in &mut out {
        *b = r.bits(8)? as u8;
    }
    Ok(out)
}

fn bit_coord(r: &mut BitReader<'_>) -> Result<BitCoord, MessageError> {
    let has_int = r.bit()?;
    let has_frac = r.bit()?;
    if !has_int && !has_frac {
        return Ok(BitCoord {
            negative: false,
            int: None,
            frac: None,
        });
    }
    let negative = r.bit()?;
    let int = if has_int {
        Some(r.bits(12)? as u16)
    } else {
        None
    };
    let frac = if has_frac {
        Some(r.bits(3)? as u8)
    } else {
        None
    };
    Ok(BitCoord {
        negative,
        int,
        frac,
    })
}

fn parse_body(
    r: &mut BitReader<'_>,
    ctx: &ParseContext<'_>,
    op: u8,
) -> Result<Message, MessageError> {
    use Message as M;
    Ok(match op {
        svc::NOP => M::Nop,
        svc::DISCONNECT => M::Disconnect(r.string()?),
        svc::EVENT => M::Event(parse_events(r, ctx)?),
        svc::VERSION => M::Version(r.u32()?),
        svc::SETVIEW => M::SetView(r.u16()?),
        svc::SOUND => M::Sound(parse_sound(r)?),
        svc::TIME => M::Time(r.u32()?),
        svc::PRINT => M::Print(r.string()?),
        svc::STUFFTEXT => M::StuffText(r.string()?),
        svc::SETANGLE => M::SetAngle([r.u16()?, r.u16()?, r.u16()?]),
        svc::SERVERINFO => M::ServerInfo(Box::new(parse_serverinfo(r)?)),
        svc::LIGHTSTYLE => M::LightStyle {
            index: r.u8()?,
            style: r.string()?,
        },
        svc::UPDATEUSERINFO => M::UpdateUserInfo {
            slot: r.u8()?,
            user_id: r.u32()?,
            info: r.string()?,
            hash: array(r)?,
        },
        svc::DELTADESCRIPTION => {
            let name = r.string()?;
            M::DeltaDescription(Arc::new(parse_description(r, name, ctx.meta)?))
        }
        svc::CLIENTDATA => M::ClientData(parse_clientdata(r, ctx)?),
        svc::STOPSOUND => M::StopSound(r.u16()?),
        svc::PINGS => {
            let mut v = Vec::new();
            while r.bit()? {
                v.push(r.bits(24)?);
                if v.len() > 64 {
                    return Err(MessageError::TooMany {
                        what: "ping",
                        count: v.len(),
                        limit: 64,
                    });
                }
            }
            M::Pings(v)
        }
        svc::PARTICLE => M::Particle(fixed(r, 11)?),
        svc::SPAWNSTATIC => {
            let start = r.byte_pos();
            r.bytes(2 + 1 + 1 + 2 + 1 + 9)?;
            if r.u8()? != 0 {
                r.bytes(5)?;
            }
            M::SpawnStatic(r.slice(start, r.byte_pos()).to_vec())
        }
        svc::EVENT_RELIABLE => {
            let index = r.bits(10)? as u16;
            let schema = ctx.require("event_t")?;
            let args = parse_delta(r, &schema, ctx.time_bits)?;
            let delay = if r.bit()? {
                Some(r.bits(16)? as u16)
            } else {
                None
            };
            M::EventReliable(ReliableEvent { index, args, delay })
        }
        svc::SPAWNBASELINE => M::SpawnBaseline(parse_baseline(r, ctx)?),
        svc::TEMPENTITY => parse_temp_entity(r)?,
        svc::SETPAUSE => M::SetPause(r.u8()?),
        svc::SIGNONNUM => M::SignOnNum(r.u8()?),
        svc::CENTERPRINT => M::CenterPrint(r.string()?),
        svc::KILLEDMONSTER => M::KilledMonster,
        svc::FOUNDSECRET => M::FoundSecret,
        svc::SPAWNSTATICSOUND => M::SpawnStaticSound(fixed(r, 14)?),
        svc::INTERMISSION => M::Intermission,
        svc::FINALE => M::Finale(r.string()?),
        svc::CDTRACK => M::CdTrack(r.u8()?, r.u8()?),
        svc::RESTORE => {
            let save = r.string()?;
            let n = r.u8()?;
            let mut maps = Vec::with_capacity(n.into());
            for _ in 0..n {
                maps.push(r.string()?);
            }
            M::Restore { save, maps }
        }
        svc::CUTSCENE => M::Cutscene(r.string()?),
        svc::WEAPONANIM => M::WeaponAnim {
            sequence: r.u8()?,
            body: r.u8()?,
        },
        svc::DECALNAME => M::DecalName {
            index: r.u8()?,
            name: r.string()?,
        },
        svc::ROOMTYPE => M::RoomType(r.u16()?),
        svc::ADDANGLE => M::AddAngle(r.u16()?),
        svc::NEWUSERMSG => {
            let id = r.u8()?;
            let size = r.u8()?;
            M::NewUserMessage {
                id,
                def: UserMessageDef {
                    name: array(r)?,
                    size: (size != 255).then_some(size),
                },
            }
        }
        svc::PACKETENTITIES => M::PacketEntities(parse_entities(r, ctx, false)?),
        svc::DELTAPACKETENTITIES => M::DeltaPacketEntities(parse_entities(r, ctx, true)?),
        svc::CHOKE => M::Choke,
        svc::RESOURCELIST => parse_resources(r)?,
        svc::NEWMOVEVARS => {
            let start = r.byte_pos();
            r.bytes(16 * 4 + 1 + 2 * 4 + 6 * 4)?;
            r.string()?;
            M::NewMoveVars(r.slice(start, r.byte_pos()).to_vec())
        }
        svc::RESOURCEREQUEST => M::ResourceRequest(r.u32()?, r.u32()?),
        svc::CUSTOMIZATION => {
            let start = r.byte_pos();
            r.bytes(2)?;
            r.string()?;
            r.bytes(2 + 4)?;
            if r.u8()? & 4 != 0 {
                r.bytes(16)?;
            }
            M::Customization(r.slice(start, r.byte_pos()).to_vec())
        }
        svc::CROSSHAIRANGLE => M::CrosshairAngle(r.u8()?, r.u8()?),
        svc::SOUNDFADE => M::SoundFade(array(r)?),
        svc::FILETXFERFAILED => M::FileTxferFailed(r.string()?),
        svc::HLTV => {
            let cmd = r.u8()?;
            if cmd != 0 {
                return Err(MessageError::Unsupported(format!(
                    "svc_hltv command {cmd} (only HLTV_ACTIVE is known)"
                )));
            }
            M::Hltv(cmd)
        }
        svc::DIRECTOR => {
            let n = r.u8()?;
            M::Director(fixed(r, n.into())?)
        }
        svc::VOICEINIT => M::VoiceInit {
            codec: r.string()?,
            quality: r.u8()?,
        },
        svc::VOICEDATA => {
            let slot = r.u8()?;
            let n = r.u16()?;
            M::VoiceData {
                slot,
                data: fixed(r, n.into())?,
            }
        }
        svc::SENDEXTRAINFO => M::SendExtraInfo {
            fallback_dir: r.string()?,
            can_cheat: r.u8()?,
        },
        svc::TIMESCALE => M::TimeScale(r.u32()?),
        svc::RESOURCELOCATION => M::ResourceLocation(r.string()?),
        svc::SENDCVARVALUE => M::SendCvarValue(r.string()?),
        svc::SENDCVARVALUE2 => M::SendCvarValue2 {
            request: r.u32()?,
            name: r.string()?,
        },
        id if id >= svc::FIRST_USER_MESSAGE => {
            let def = ctx.user_messages[usize::from(id)]
                .as_ref()
                .ok_or(MessageError::UnknownUserMessage(id))?;
            let n = match def.size {
                Some(n) => n,
                None => r.u8()?,
            };
            M::User {
                id,
                body: fixed(r, n.into())?,
            }
        }
        other => return Err(MessageError::UnknownOpcode(other)),
    })
}

fn parse_serverinfo(r: &mut BitReader<'_>) -> Result<ServerInfo, MessageError> {
    let info = ServerInfo {
        protocol: r.u32()? as i32,
        spawn_count: r.u32()? as i32,
        map_crc: r.u32()? as i32,
        client_dll_md5: array(r)?,
        max_clients: r.u8()?,
        player_num: r.u8()?,
        game_type: r.u8()?,
        game_dir: r.string()?,
        host_name: r.string()?,
        map_name: r.string()?,
        map_cycle: r.string()?,
    };
    let vac2 = r.u8()?;
    if vac2 != 0 {
        return Err(MessageError::Unsupported(format!(
            "svc_serverinfo VAC2 byte {vac2} (only 0 is known)"
        )));
    }
    Ok(info)
}

fn parse_sound(r: &mut BitReader<'_>) -> Result<SoundMessage, MessageError> {
    let flags = r.bits(9)? as u16;
    let volume = if flags & 1 != 0 {
        Some(r.bits(8)? as u8)
    } else {
        None
    };
    let attenuation = if flags & 2 != 0 {
        Some(r.bits(8)? as u8)
    } else {
        None
    };
    let channel = r.bits(3)? as u8;
    let entity = r.bits(11)? as u16;
    let sound_index = r.bits(if flags & 4 != 0 { 16 } else { 8 })? as u16;
    let present = [r.bit()?, r.bit()?, r.bit()?];
    let mut origin = [None; 3];
    for (o, p) in origin.iter_mut().zip(present) {
        if p {
            *o = Some(bit_coord(r)?);
        }
    }
    let pitch = if flags & 8 != 0 {
        Some(r.bits(8)? as u8)
    } else {
        None
    };
    Ok(SoundMessage {
        flags,
        volume,
        attenuation,
        channel,
        entity,
        sound_index,
        origin,
        pitch,
    })
}

fn parse_events(
    r: &mut BitReader<'_>,
    ctx: &ParseContext<'_>,
) -> Result<Vec<EventEntry>, MessageError> {
    let count = r.bits(5)?;
    let mut out = Vec::with_capacity(count as usize);
    let mut schema = None;
    for _ in 0..count {
        let index = r.bits(10)? as u16;
        let mut packet_index = None;
        let mut args = None;
        if r.bit()? {
            packet_index = Some(r.bits(11)? as u16);
            if r.bit()? {
                let s = match &schema {
                    Some(s) => s,
                    None => schema.insert(ctx.require("event_t")?),
                };
                args = Some(parse_delta(r, s, ctx.time_bits)?);
            }
        }
        let fire_time = if r.bit()? {
            Some(r.bits(16)? as u16)
        } else {
            None
        };
        out.push(EventEntry {
            index,
            packet_index,
            args,
            fire_time,
        });
    }
    Ok(out)
}

fn parse_clientdata(
    r: &mut BitReader<'_>,
    ctx: &ParseContext<'_>,
) -> Result<ClientDataMessage, MessageError> {
    if ctx.hltv {
        return Ok(ClientDataMessage {
            delta_from: None,
            client: None,
            weapons: Vec::new(),
        });
    }
    let delta_from = if r.bit()? {
        Some(r.bits(8)? as u8)
    } else {
        None
    };
    let schema = ctx.require("clientdata_t")?;
    let client = parse_delta(r, &schema, ctx.time_bits)?;
    let mut weapons = Vec::new();
    let mut weapon_schema = None;
    while r.bit()? {
        let slot = r.bits(6)? as u8;
        let s = match &weapon_schema {
            Some(s) => s,
            None => weapon_schema.insert(ctx.require("weapon_data_t")?),
        };
        weapons.push((slot, parse_delta(r, s, ctx.time_bits)?));
        if weapons.len() > 64 {
            return Err(MessageError::TooMany {
                what: "weapon data",
                count: weapons.len(),
                limit: 64,
            });
        }
    }
    Ok(ClientDataMessage {
        delta_from,
        client: Some(client),
        weapons,
    })
}

fn parse_entities(
    r: &mut BitReader<'_>,
    ctx: &ParseContext<'_>,
    delta: bool,
) -> Result<EntityPacket, MessageError> {
    let declared_count = r.u16()?;
    let delta_from = if delta { Some(r.u8()?) } else { None };
    let full = !delta;
    let mut updates = Vec::new();
    let mut numbase: u32 = 0;
    while r.peek_bits(16)? != 0 {
        let (is_delta, remove) = if full {
            (r.bit()?, false)
        } else {
            (false, r.bit()?)
        };
        let number = if is_delta {
            numbase + 1
        } else if r.bit()? {
            r.bits(11)?
        } else {
            numbase + r.bits(6)?
        };
        numbase = number;
        if number >= 2048 {
            return Err(MessageError::TooMany {
                what: "entity number",
                count: number as usize,
                limit: 2047,
            });
        }
        let number = number as u16;
        let mut update = EntityUpdate {
            number,
            remove,
            custom: false,
            new_baseline: None,
            offset: 0,
            schema: None,
            delta: None,
        };
        if !remove {
            update.custom = r.bit()?;
            if ctx.instanced_baselines > 0 && r.bit()? {
                update.new_baseline = Some(r.bits(6)? as u8);
            }
            if full && update.new_baseline.is_none() && r.bit()? {
                update.offset = r.bits(6)? as u8;
            }
            let schema = ctx.entity_schema(number, update.custom)?;
            update.delta = Some(parse_delta(r, &schema, ctx.time_bits)?);
            update.schema = Some(schema);
        }
        updates.push(update);
        if updates.len() > 2 * MAX_PACKET_ENTITIES {
            return Err(MessageError::TooMany {
                what: "entity update",
                count: updates.len(),
                limit: 2 * MAX_PACKET_ENTITIES,
            });
        }
    }
    r.bits(16)?;
    Ok(EntityPacket {
        declared_count,
        delta_from,
        updates,
    })
}

fn parse_baseline(
    r: &mut BitReader<'_>,
    ctx: &ParseContext<'_>,
) -> Result<SpawnBaseline, MessageError> {
    let mut entities = Vec::new();
    while r.peek_bits(16)? != 0xFFFF {
        let number = r.bits(11)? as u16;
        let entity_type = r.bits(2)? as u8;
        // ENTITY_NORMAL = 1, ENTITY_BEAM = 2; custom when not normal.
        let custom = entity_type & 1 == 0;
        let schema = ctx.entity_schema(number, custom)?;
        let delta = parse_delta(r, &schema, ctx.time_bits)?;
        entities.push(BaselineEntry {
            number,
            entity_type,
            schema,
            delta,
        });
        if entities.len() > 2048 {
            return Err(MessageError::TooMany {
                what: "baseline",
                count: entities.len(),
                limit: 2048,
            });
        }
    }
    r.bits(16)?;
    let n = r.bits(6)?;
    let schema = ctx.require("entity_state_t")?;
    let mut instanced = Vec::with_capacity(n as usize);
    for _ in 0..n {
        instanced.push(parse_delta(r, &schema, ctx.time_bits)?);
    }
    Ok(SpawnBaseline {
        entities,
        instanced,
    })
}

fn parse_temp_entity(r: &mut BitReader<'_>) -> Result<Message, MessageError> {
    let kind = r.u8()?;
    let start = r.byte_pos();
    match TE_LENGTH.get(usize::from(kind)).copied().unwrap_or(-2) {
        -2 => return Err(MessageError::UnknownTempEntity(kind)),
        -1 if kind == TE_BSPDECAL => {
            r.bytes(8)?;
            if r.u16()? != 0 {
                r.bytes(2)?;
            }
        }
        -1 if kind == TE_TEXTMESSAGE => {
            r.bytes(5)?;
            let effect = r.u8()?;
            r.bytes(14)?;
            if effect == 2 {
                r.bytes(2)?;
            }
            r.string()?;
        }
        -1 => unreachable!(),
        n => {
            r.bytes(n as usize)?;
        }
    }
    Ok(Message::TempEntity {
        kind,
        body: r.slice(start, r.byte_pos()).to_vec(),
    })
}

fn parse_resources(r: &mut BitReader<'_>) -> Result<Message, MessageError> {
    let count = r.bits(12)?;
    let mut resources = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let kind = r.bits(4)? as u8;
        let name = r.bit_string()?;
        let index = r.bits(12)? as u16;
        let download_size = r.bits(24)?;
        let flags = r.bits(3)? as u8;
        let md5 = if flags & 4 != 0 {
            Some(bit_bytes::<16>(r)?)
        } else {
            None
        };
        let reserved = if r.bit()? {
            Some(bit_bytes::<32>(r)?)
        } else {
            None
        };
        resources.push(Resource {
            kind,
            name,
            index,
            download_size,
            flags,
            md5,
            reserved,
        });
    }
    let consistency = if r.bit()? {
        let mut v = Vec::new();
        while r.bit()? {
            let short = r.bit()?;
            v.push((short, r.bits(if short { 5 } else { 10 })? as u16));
            if v.len() > 4096 {
                return Err(MessageError::TooMany {
                    what: "consistency entry",
                    count: v.len(),
                    limit: 4096,
                });
            }
        }
        Some(v)
    } else {
        None
    };
    Ok(Message::ResourceList {
        resources,
        consistency,
    })
}

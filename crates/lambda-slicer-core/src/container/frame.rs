//! Outer frame records. Layouts follow the GoldSrc demo format as written by
//! the engine, see `docs/protocol-profile-5-48.md`

/// Outer frame tag values
pub mod tag {
    pub const NETWORK_START: u8 = 0;
    pub const NETWORK: u8 = 1;
    pub const DEMO_START: u8 = 2;
    pub const CONSOLE_COMMAND: u8 = 3;
    pub const CLIENT_DATA: u8 = 4;
    pub const NEXT_SECTION: u8 = 5;
    pub const EVENT: u8 = 6;
    pub const WEAPON_ANIM: u8 = 7;
    pub const SOUND: u8 = 8;
    pub const DEMO_BUFFER: u8 = 9;

    pub fn name(tag: u8) -> &'static str {
        match tag {
            NETWORK_START => "NetworkStart",
            NETWORK => "Network",
            DEMO_START => "DemoStart",
            CONSOLE_COMMAND => "ConsoleCommand",
            CLIENT_DATA => "ClientData",
            NEXT_SECTION => "NextSection",
            EVENT => "Event",
            WEAPON_ANIM => "WeaponAnim",
            SOUND => "Sound",
            DEMO_BUFFER => "DemoBuffer",
            _ => "Unknown",
        }
    }
}

/// Recorded demo info (timestamp, ref_params, usercmd, movevars, view,
/// viewmodel) preceding the sequence numbers of a network record
pub const DEMO_INFO_LEN: usize = 436;
pub const CONSOLE_COMMAND_LEN: usize = 64;
pub const CLIENT_DATA_LEN: usize = 32;
pub const EVENT_ARGS_LEN: usize = 72;

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    /// Original tag byte (0 and 1 are both network records)
    pub tag: u8,
    pub time_bits: u32,
    pub frame_number: i32,
    pub body: FrameBody,
}

impl Frame {
    pub fn time(&self) -> f32 {
        f32::from_bits(self.time_bits)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FrameBody {
    Network(Box<NetworkFrame>),
    DemoStart,
    ConsoleCommand(Box<[u8; CONSOLE_COMMAND_LEN]>),
    ClientData([u8; CLIENT_DATA_LEN]),
    NextSection,
    Event(EventRecord),
    WeaponAnim { sequence: i32, body: i32 },
    Sound(SoundRecord),
    DemoBuffer(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkFrame {
    pub demo_info: Box<[u8; DEMO_INFO_LEN]>,
    /// incoming_sequence, incoming_acknowledged,
    /// incoming_reliable_acknowledged, incoming_reliable_sequence,
    /// outgoing_sequence, reliable_sequence, last_reliable_sequence
    pub sequence: [i32; 7],
    pub message: Vec<u8>,
}

/// Standalone demo `Event` record, holding the outer dispatch fields plus
/// raw `event_args_t`
#[derive(Clone, Debug, PartialEq)]
pub struct EventRecord {
    pub flags: i32,
    pub index: i32,
    pub delay_bits: u32,
    pub args: [u8; EVENT_ARGS_LEN],
}

fn f32_at(b: &[u8], at: usize) -> f32 {
    f32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}

fn i32_at(b: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}

fn vec3_at(b: &[u8], at: usize) -> [f32; 3] {
    [f32_at(b, at), f32_at(b, at + 4), f32_at(b, at + 8)]
}

/// Offsets inside the 436-byte demo info, which holds the timestamp,
/// `ref_params_t` (232 bytes from 4), `usercmd_t` (52 from 236), `movevars_t`
/// (132 from 288), view (420) and viewmodel (432)
impl NetworkFrame {
    pub fn info_timestamp(&self) -> f32 {
        f32_at(&self.demo_info[..], 0)
    }
    pub fn info_vieworg(&self) -> [f32; 3] {
        vec3_at(&self.demo_info[..], 4)
    }
    pub fn info_viewangles(&self) -> [f32; 3] {
        vec3_at(&self.demo_info[..], 16)
    }
    /// How long the recorder's frame took, in seconds
    pub fn info_frametime(&self) -> f32 {
        f32_at(&self.demo_info[..], 64)
    }
    pub fn info_intermission(&self) -> i32 {
        i32_at(&self.demo_info[..], 72)
    }
    pub fn info_paused(&self) -> i32 {
        i32_at(&self.demo_info[..], 76)
    }
    pub fn info_spectator(&self) -> i32 {
        i32_at(&self.demo_info[..], 80)
    }
    pub fn info_onground(&self) -> i32 {
        i32_at(&self.demo_info[..], 84)
    }
    pub fn info_waterlevel(&self) -> i32 {
        i32_at(&self.demo_info[..], 88)
    }
    pub fn info_simvel(&self) -> [f32; 3] {
        vec3_at(&self.demo_info[..], 92)
    }
    pub fn info_simorg(&self) -> [f32; 3] {
        vec3_at(&self.demo_info[..], 104)
    }
    pub fn info_viewheight(&self) -> [f32; 3] {
        vec3_at(&self.demo_info[..], 116)
    }
    pub fn info_health(&self) -> i32 {
        i32_at(&self.demo_info[..], 144)
    }
    pub fn info_maxclients(&self) -> i32 {
        i32_at(&self.demo_info[..], 176)
    }
    pub fn info_viewentity(&self) -> i32 {
        i32_at(&self.demo_info[..], 180)
    }
    pub fn info_playernum(&self) -> i32 {
        i32_at(&self.demo_info[..], 184)
    }
    /// Raw `movevars_t`
    pub fn info_movevars(&self) -> &[u8] {
        &self.demo_info[288..420]
    }
}

/// Fields of a standalone `ClientData` record body
pub struct ClientDataRecord<'a>(pub &'a [u8; CLIENT_DATA_LEN]);

impl ClientDataRecord<'_> {
    pub fn origin(&self) -> [f32; 3] {
        vec3_at(self.0, 0)
    }
    pub fn viewangles(&self) -> [f32; 3] {
        vec3_at(self.0, 12)
    }
    pub fn weapon_bits(&self) -> i32 {
        i32_at(self.0, 24)
    }
    pub fn fov(&self) -> f32 {
        f32_at(self.0, 28)
    }
}

impl EventRecord {
    fn i32_at(&self, at: usize) -> i32 {
        i32::from_le_bytes(self.args[at..at + 4].try_into().unwrap())
    }

    fn f32_at(&self, at: usize) -> f32 {
        f32::from_le_bytes(self.args[at..at + 4].try_into().unwrap())
    }

    fn vec3_at(&self, at: usize) -> [f32; 3] {
        [self.f32_at(at), self.f32_at(at + 4), self.f32_at(at + 8)]
    }

    pub fn delay(&self) -> f32 {
        f32::from_bits(self.delay_bits)
    }
    pub fn args_flags(&self) -> i32 {
        self.i32_at(0)
    }
    pub fn entity_index(&self) -> i32 {
        self.i32_at(4)
    }
    pub fn origin(&self) -> [f32; 3] {
        self.vec3_at(8)
    }
    pub fn angles(&self) -> [f32; 3] {
        self.vec3_at(20)
    }
    pub fn velocity(&self) -> [f32; 3] {
        self.vec3_at(32)
    }
    pub fn ducking(&self) -> i32 {
        self.i32_at(44)
    }
    pub fn fparam1(&self) -> f32 {
        self.f32_at(48)
    }
    pub fn fparam2(&self) -> f32 {
        self.f32_at(52)
    }
    pub fn iparam1(&self) -> i32 {
        self.i32_at(56)
    }
    pub fn iparam2(&self) -> i32 {
        self.i32_at(60)
    }
    pub fn bparam1(&self) -> i32 {
        self.i32_at(64)
    }
    pub fn bparam2(&self) -> i32 {
        self.i32_at(68)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoundRecord {
    pub channel: i32,
    pub sample: Vec<u8>,
    pub attenuation_bits: u32,
    pub volume_bits: u32,
    pub flags: i32,
    pub pitch: i32,
}

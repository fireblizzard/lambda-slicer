//! Outer frame records. Layouts follow the GoldSrc demo format as written by
//! the engine; see `docs/protocol-profile-5-48.md`.

/// Outer frame tag values.
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
/// viewmodel) preceding the sequence numbers of a network record.
pub const DEMO_INFO_LEN: usize = 436;
pub const CONSOLE_COMMAND_LEN: usize = 64;
pub const CLIENT_DATA_LEN: usize = 32;
pub const EVENT_ARGS_LEN: usize = 72;

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    /// Original tag byte (0 and 1 are both network records).
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
    /// outgoing_sequence, reliable_sequence, last_reliable_sequence.
    pub sequence: [i32; 7],
    pub message: Vec<u8>,
}

/// Standalone demo `Event` record: outer dispatch fields plus raw
/// `event_args_t`.
#[derive(Clone, Debug, PartialEq)]
pub struct EventRecord {
    pub flags: i32,
    pub index: i32,
    pub delay_bits: u32,
    pub args: [u8; EVENT_ARGS_LEN],
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

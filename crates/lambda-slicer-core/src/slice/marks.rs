//! The marks on the walls at a clip's start
//!
//! Decals are lasting world state. The client keeps them until a new
//! session, so a clip that starts later would show clean walls where the
//! source showed blood, scorches and bullet holes. They are placed again
//! after the bootstrap group, oldest first, as `svc_temp_entity` decals.
//!
//! The server sends some of them itself (blood, explosions), and those
//! messages are sent again as they were. The game's client makes the rest
//! from events (bullets, the gauss, the crowbar), tracing through the map,
//! which needs the game's files. A caller that played the source to the
//! start, as the viewer does, can hand over every decal there instead
//! ([`Mark`]). Map decals come with the session's setup, so they are never
//! among them, and sprays are left out, as they need the player's own
//! logo

use crate::protocol::messages::{Resource, resource_type};

/// As many decals as the engine keeps (`MAX_DECALS`)
pub const MAX_DECALS: usize = 4096;

/// The most bytes of decals in one record. Recordings hold playback records
/// of up to 24 KB, which the game plays
const RECORD_BYTES: usize = 16_000;

/// `svc_temp_entity` and the decal kinds placed again
const SVC_TEMP_ENTITY: u8 = 23;
const TE_DECAL: u8 = 104;
const TE_WORLD_DECAL: u8 = 116;
const TE_WORLD_DECAL_HIGH: u8 = 117;
const TE_DECAL_HIGH: u8 = 118;

/// Which decal a mark is
#[derive(Clone, Debug, PartialEq)]
pub enum MarkTexture {
    /// The server's own number for it, from its message
    Number(u16),
    /// Its name in `decals.wad` and its place there, from the game's own
    /// code
    Named { name: String, place: u16 },
}

/// A decal on the map at a clip's start
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    pub texture: MarkTexture,
    /// The brush entity it is on, 0 for the world
    pub entity: u16,
    /// Where it was aimed, in the world as the entity stands at the start
    pub origin: [f32; 3],
}

/// The server's number for a mark's decal in a session with `resources`.
/// A named decal takes the number the session gives its name. The client
/// takes a number the session names no decal for as its own place in
/// `decals.wad`, so a name the session lacks keeps its place when that
/// number is free. None when it has no number the client would read as it
fn number(texture: &MarkTexture, resources: &[Resource]) -> Option<u16> {
    let (name, place) = match texture {
        MarkTexture::Number(n) => return Some(*n),
        MarkTexture::Named { name, place } => (name, *place),
    };

    let decals = || resources.iter().filter(|r| r.kind == resource_type::DECAL);
    let named = decals().find(|r| r.name.eq_ignore_ascii_case(name.as_bytes()));
    if let Some(r) = named {
        return Some(r.index);
    }
    let taken = decals().any(|r| r.index == place);
    if taken { None } else { Some(place) }
}

/// A coordinate as the protocol writes it, in eighths of a unit
fn coord(value: f32) -> [u8; 2] {
    let eighths = (value * 8.)
        .round()
        .clamp(f32::from(i16::MIN), f32::from(i16::MAX));
    (eighths as i16).to_le_bytes()
}

/// The `svc_temp_entity` that places `mark`, none when the session has no
/// number for its decal or the number is past what a message carries
pub fn mark_message(mark: &Mark, resources: &[Resource]) -> Option<Vec<u8>> {
    let number = number(&mark.texture, resources)?;
    let high = number >= 256;
    let byte = u8::try_from(number % 256).ok().filter(|_| number < 512)?;
    let kind = match (mark.entity, high) {
        (0, false) => TE_WORLD_DECAL,
        (0, true) => TE_WORLD_DECAL_HIGH,
        (_, false) => TE_DECAL,
        (_, true) => TE_DECAL_HIGH,
    };

    let mut message = vec![SVC_TEMP_ENTITY, kind];
    for value in mark.origin {
        message.extend_from_slice(&coord(value));
    }
    message.push(byte);
    if mark.entity != 0 {
        message.extend_from_slice(&mark.entity.to_le_bytes());
    }
    Some(message)
}

/// Decal messages gathered into records of at most [`RECORD_BYTES`] each,
/// in order
pub fn records(messages: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = Vec::new();
    for message in messages {
        let full = out
            .last()
            .is_none_or(|r| r.len() + message.len() > RECORD_BYTES);
        if full {
            out.push(Vec::new());
        }
        if let Some(record) = out.last_mut() {
            record.extend_from_slice(message);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decal(name: &str, index: u16) -> Resource {
        Resource {
            kind: resource_type::DECAL,
            name: name.as_bytes().to_vec(),
            index,
            download_size: 0,
            flags: 0,
            md5: None,
            reserved: None,
        }
    }

    fn mark(texture: MarkTexture, entity: u16) -> Mark {
        Mark {
            texture,
            entity,
            origin: [1., -2.5, 3.125],
        }
    }

    #[test]
    fn a_world_mark_is_a_world_decal_at_its_place() {
        let message = mark_message(&mark(MarkTexture::Number(27), 0), &[]).unwrap();
        assert_eq!(message, [23, 116, 8, 0, 236, 255, 25, 0, 27]);
    }

    #[test]
    fn a_mark_on_a_brush_names_it_and_high_numbers_take_their_own_kind() {
        let message = mark_message(&mark(MarkTexture::Number(300), 7), &[]).unwrap();
        assert_eq!(message[1], TE_DECAL_HIGH);
        assert_eq!(message[8], 44);
        assert_eq!(&message[9..], [7, 0]);
        assert_eq!(mark_message(&mark(MarkTexture::Number(512), 0), &[]), None);
    }

    #[test]
    fn a_named_mark_takes_the_sessions_number_for_its_name() {
        let resources = [decal("{shot1", 40), decal("{bigshot1", 12)];
        let named = |name: &str, place| MarkTexture::Named {
            name: name.into(),
            place,
        };

        // Named by the session, whatever the case
        let m = mark_message(&mark(named("{SHOT1", 3), 0), &resources).unwrap();
        assert_eq!(m[8], 40);

        // Not named, so its own place, unless the session gave that number
        // to another decal
        let m = mark_message(&mark(named("{scorch1", 5), 0), &resources).unwrap();
        assert_eq!(m[8], 5);
        assert_eq!(
            mark_message(&mark(named("{scorch1", 12), 0), &resources),
            None
        );
    }

    #[test]
    fn records_hold_whole_messages_up_to_their_size() {
        let message = vec![0; 9];
        let messages = vec![message; 4000];
        let made = records(&messages);
        assert_eq!(made.len(), 3);
        assert!(
            made.iter()
                .all(|r| r.len() <= RECORD_BYTES && r.len() % 9 == 0)
        );
        assert_eq!(made.iter().map(Vec::len).sum::<usize>(), 36_000);
        assert!(records(&[]).is_empty());
    }
}

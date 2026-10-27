//! Lines of chat as the library keeps them, who they went to and what was
//! said
//!
//! AG starts a player's line with who it went to, `(A)` for everyone and
//! `(T)` for the sender's team, and a spectator's with `(S)` for everyone
//! and `(ST)` for the other spectators. The game's own team chat starts
//! with `(TEAM)`, and its other lines with the name. The sender's name and
//! a colon follow

/// The codes a line of chat names its sender's place with, which the
/// client fills in
pub const PLACE_CODES: [&str; 4] = ["%l", "%L", "%d", "%D"];

/// What AG's server writes in a line for whether its sender has the long
/// jump module
const LONG_JUMP: [&str; 2] = ["got LJ", "no LJ"];

/// Whether a line names its sender's place, as players' binds for telling
/// their team where they are do
pub fn names_place(line: &str) -> bool {
    PLACE_CODES.iter().any(|code| line.contains(code))
}

/// Whether a line says if its sender has the long jump module, as players'
/// binds for telling their team do. Colour codes may come between the words
pub fn says_long_jump(line: &str) -> bool {
    let plain = without_colours(line);
    LONG_JUMP.iter().any(|words| plain.contains(words))
}

/// `text` without AG's colour codes, a `^` and a digit
fn without_colours(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '^' && chars.peek().is_some_and(char::is_ascii_digit) {
            chars.next();
            continue;
        }
        out.push(c);
    }
    out
}

/// Who a line of chat went to and who sent it
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Said {
    /// Only to the sender's team, or to the other spectators
    pub team: bool,
    /// Sent by a spectator
    pub spectator: bool,
}

/// The tags a line can start with, and what each says
const TAGS: [(&str, Said); 5] = [
    (
        "(A) ",
        Said {
            team: false,
            spectator: false,
        },
    ),
    (
        "(T) ",
        Said {
            team: true,
            spectator: false,
        },
    ),
    (
        "(S) ",
        Said {
            team: false,
            spectator: true,
        },
    ),
    (
        "(ST) ",
        Said {
            team: true,
            spectator: true,
        },
    ),
    (
        "(TEAM) ",
        Said {
            team: true,
            spectator: false,
        },
    ),
];

/// A player's line of chat as who it went to and what follows `sender`'s
/// name and its colon, colour codes kept. A line without the name where
/// it should be keeps everything after its tag
pub fn split(line: &str, sender: &str) -> (Said, String) {
    let mut said = Said::default();
    let mut rest = line;
    for (tag, tagged) in TAGS {
        if let Some(after) = line.strip_prefix(tag) {
            said = tagged;
            rest = after;
            break;
        }
    }

    let after_name = rest.strip_prefix(sender).and_then(|r| r.strip_prefix(':'));
    let message = match after_name {
        Some(after) => after.trim_start(),
        None => rest,
    };
    (said, message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_is_who_it_went_to_and_what_follows_the_name() {
        let (said, message) = split("(T) -<[ATK]>-naz: at ^3Bridge", "-<[ATK]>-naz");
        assert_eq!((said.team, said.spectator), (true, false));
        assert_eq!(message, "at ^3Bridge");

        // A name in colour, and the spaces after the colon
        let (said, message) = split("(A) ^1red^0:  hi", "^1red^0");
        assert_eq!(said, Said::default());
        assert_eq!(message, "hi");

        let (said, _) = split("(ST) spec: started?", "spec");
        assert_eq!((said.team, said.spectator), (true, true));
        let (said, _) = split("(S) spec: gg", "spec");
        assert_eq!((said.team, said.spectator), (false, true));
    }

    #[test]
    fn a_line_without_a_tag_or_its_name_keeps_the_rest() {
        assert_eq!(
            split("naz: hello", "naz"),
            (Said::default(), "hello".into())
        );
        assert_eq!(split("(A) hello", "naz").1, "hello");
    }
}

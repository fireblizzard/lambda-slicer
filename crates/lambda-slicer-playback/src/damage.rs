//! Where the recording player is hit from and what hurts them, as the game's
//! client shows it (HLSDK's `CHudHealth`)
//!
//! `Damage` is the armour and the health taken, the kinds of damage as
//! bits, and the point the damage came from. A hit that takes health or
//! armour marks the sides it came from against where the player looks, the
//! front, right, rear and left, each by how squarely, and all four at once
//! from 50 units or closer. A mark shows while it is over 0.4 and fades by
//! 2 a second. A point of nought clears them all.
//!
//! Damage that lasts, such as drowning or burning, shows its icon at the
//! left, the newest at the bottom and the rest stacked above it. An icon
//! stays two seconds after its last hit, then goes the next time its flash
//! dims, and those above it move down

use crate::events::angle_vectors;
use crate::time::SourceTime;

/// How many of the game's units on a side a coordinate is sent in
const COORD_STEP: f32 = 8.;

/// How close a hit marks every side, how squarely it must come from a side
/// to mark it, the least mark shown, and how fast marks fade a second
const CLOSE: f32 = 50.;
const SQUARELY: f32 = 0.3;
const LEAST_MARK: f32 = 0.4;
const MARK_FADE: f32 = 2.;

/// How long a kind's icon stays after its last hit, in seconds
/// (`DMG_IMAGE_LIFE`)
const KIND_SHOWN: f64 = 2.;

/// An icon's flash below which it can go, of 256
const DIM_FLASH: f64 = 40.;

/// The kinds of damage that show an icon, their bits and their icons in
/// `sprites/hud.txt` (`giDmgFlags`)
const KINDS: [(u32, &str); 8] = [
    (1 << 17, "dmg_poison"),
    (1 << 20, "dmg_chem"),
    ((1 << 4) | (1 << 22), "dmg_cold"),
    (1 << 14, "dmg_drown"),
    ((1 << 3) | (1 << 21), "dmg_heat"),
    (1 << 16, "dmg_gas"),
    (1 << 18, "dmg_rad"),
    (1 << 8, "dmg_shock"),
];

/// The sides a hit can mark, in the order the marks are kept
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Front,
    Right,
    Rear,
    Left,
}

const SIDES: [Side; 4] = [Side::Front, Side::Right, Side::Rear, Side::Left];

/// What shows at a moment
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DamageShown {
    /// Each side marked and how strongly, from 0.4 to 1
    pub marks: Vec<(Side, f32)>,
    /// Each kind's icon in `sprites/hud.txt` and its place counted up from
    /// the lowest
    pub kinds: Vec<(&'static str, usize)>,
    /// How bright every icon flashes, of 256
    pub flash: f32,
}

/// What the messages last said
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Damage {
    /// Each side's mark as of `marked`, in the order of `SIDES`
    marks: [f32; 4],
    marked: SourceTime,
    /// Each kind shown by its place in `KINDS`, newest first, and when its
    /// last hit's two seconds end
    kinds: Vec<(usize, SourceTime)>,
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// How bright the icons flash at `at`, of 256
fn flash(at: SourceTime) -> f64 {
    (at.0 * 2.).sin().abs() * 256.
}

/// When an icon whose two seconds end at `until` goes, the first moment
/// from then its `flash` is under `DIM_FLASH`
fn goes_at(until: SourceTime) -> SourceTime {
    let half_turn = std::f64::consts::FRAC_PI_2;
    let dim = (DIM_FLASH / 256.).asin() / 2.;
    let into = until.0.rem_euclid(half_turn);
    if into < dim || into > half_turn - dim {
        until
    } else {
        SourceTime(until.0 + half_turn - dim - into)
    }
}

impl Damage {
    /// A `Damage` message's `body` at `time`, the player at `origin` looking
    /// along `angles`
    pub(crate) fn message(
        &mut self,
        body: &[u8],
        time: SourceTime,
        origin: [f32; 3],
        angles: [f32; 3],
    ) {
        let Some(&[armour, health, b0, b1, b2, b3]) = body.get(..6) else {
            return;
        };
        let bits = u32::from_le_bytes([b0, b1, b2, b3]);
        let coord = |k: usize| {
            let bytes = body.get(6 + k * 2..8 + k * 2)?;
            Some(f32::from(i16::from_le_bytes([bytes[0], bytes[1]])) / COORD_STEP)
        };
        let (Some(x), Some(y), Some(z)) = (coord(0), coord(1), coord(2)) else {
            return;
        };

        self.kinds_hit(bits, time);
        if health > 0 || armour > 0 {
            self.mark([x, y, z], time, origin, angles);
        }
    }

    /// Each kind of damage in `bits` hit at `time`, a kind not shown yet
    /// going at the bottom
    fn kinds_hit(&mut self, bits: u32, time: SourceTime) {
        self.kinds.retain(|&(_, until)| goes_at(until) > time);
        let until = SourceTime(time.0 + KIND_SHOWN);
        for (kind, &(kind_bits, _)) in KINDS.iter().enumerate() {
            if bits & kind_bits == 0 {
                continue;
            }
            match self.kinds.iter_mut().find(|(k, _)| *k == kind) {
                Some((_, last)) => *last = until,
                None => self.kinds.insert(0, (kind, until)),
            }
        }
    }

    /// The marks faded to `at`, those no longer shown cleared
    fn marks_at(&self, at: SourceTime) -> [f32; 4] {
        let since = (at.0 - self.marked.0).max(0.) as f32;
        self.marks.map(|mark| {
            let faded = mark - since * MARK_FADE;
            if faded > LEAST_MARK { faded } else { 0. }
        })
    }

    /// A hit from `from` at `time` (`CalcDamageDirection`)
    fn mark(&mut self, from: [f32; 3], time: SourceTime, origin: [f32; 3], angles: [f32; 3]) {
        let mut marks = self.marks_at(time);
        self.marked = time;
        if from == [0.; 3] {
            self.marks = [0.; 4];
            return;
        }

        let to = [0, 1, 2].map(|k| from[k] - origin[k]);
        let distance = dot(to, to).sqrt();
        if distance <= CLOSE {
            self.marks = [1.; 4];
            return;
        }

        // How squarely the hit comes from ahead and from the right
        let to = to.map(|c| c / distance);
        let [forward, right, _] = angle_vectors(angles);
        let ahead = dot(to, forward);
        let rightward = dot(to, right);

        let side_marks = [ahead, rightward, -ahead, -rightward];
        for (mark, squarely) in marks.iter_mut().zip(side_marks) {
            if squarely > SQUARELY {
                *mark = mark.max(squarely);
            }
        }
        self.marks = marks;
    }

    /// What shows at `at`
    pub fn at(&self, at: SourceTime) -> DamageShown {
        let mut marks = Vec::new();
        for (side, mark) in SIDES.into_iter().zip(self.marks_at(at)) {
            if mark > 0. {
                marks.push((side, mark));
            }
        }

        let mut kinds = Vec::new();
        for &(kind, until) in &self.kinds {
            if goes_at(until) > at {
                kinds.push((KINDS[kind].1, kinds.len()));
            }
        }
        DamageShown {
            marks,
            kinds,
            flash: flash(at) as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `Damage` body taking `health`, of kinds `bits`, from `from`
    fn damage(health: u8, bits: u32, from: [f32; 3]) -> Vec<u8> {
        let mut body = vec![0, health];
        body.extend(bits.to_le_bytes());
        for c in from {
            body.extend(((c * COORD_STEP) as i16).to_le_bytes());
        }
        body
    }

    #[test]
    fn a_hit_marks_the_side_it_came_from_and_fades() {
        let mut d = Damage::default();
        // Looking along x, hit from the left, which is +y
        d.message(
            &damage(10, 0, [0., 500., 0.]),
            SourceTime(1.),
            [0.; 3],
            [0.; 3],
        );
        assert_eq!(d.at(SourceTime(1.)).marks, [(Side::Left, 1.)]);
        assert_eq!(d.at(SourceTime(1.25)).marks, [(Side::Left, 0.5)]);
        assert!(d.at(SourceTime(1.3)).marks.is_empty());

        // From close by, every side
        d.message(
            &damage(10, 0, [20., 0., 0.]),
            SourceTime(2.),
            [0.; 3],
            [0.; 3],
        );
        assert_eq!(d.at(SourceTime(2.)).marks.len(), 4);
    }

    #[test]
    fn lasting_damage_stacks_its_icons_newest_lowest() {
        let mut d = Damage::default();
        let drown = 1 << 14;
        let burn = 1 << 21;
        d.message(&damage(5, drown, [0.; 3]), SourceTime(0.), [0.; 3], [0.; 3]);
        d.message(&damage(5, burn, [0.; 3]), SourceTime(1.9), [0.; 3], [0.; 3]);
        assert_eq!(
            d.at(SourceTime(1.95)).kinds,
            [("dmg_heat", 0), ("dmg_drown", 1)]
        );

        // Drowning goes after its two seconds once its flash dims, and the
        // burn stays a flash longer
        let gone = goes_at(SourceTime(2.));
        assert!(gone >= SourceTime(2.) && gone.0 < 2. + std::f64::consts::FRAC_PI_2);
        assert_eq!(d.at(gone).kinds, [("dmg_heat", 0)]);
    }
}

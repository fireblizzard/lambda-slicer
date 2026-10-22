//! The recording player's health, armour and ammo, as the game's client
//! keeps them for its HUD (HLSDK's `CHudHealth`, `CHudBattery` and
//! `CHudAmmo`)
//!
//! `Health` is a byte and `Battery` a short. `WeaponList` names each weapon
//! with its ammo types, `CurWeapon` says which weapon is out and what its
//! magazine holds, and `AmmoX` how much of an ammo type is carried.
//! `HideWeapon` hides parts of the HUD by its flags. A reading remembers
//! when it last changed, as the client brightens a number that changes and
//! fades it back over the next seconds

use std::collections::BTreeMap;

use crate::time::SourceTime;

/// `HideWeapon`'s flags that hide the weapon's ammo, everything, and the
/// health and armour (`HIDEHUD_WEAPONS`, `HIDEHUD_ALL`, `HIDEHUD_HEALTH`)
const HIDE_WEAPONS: u8 = 1;
const HIDE_ALL: u8 = 4;
const HIDE_HEALTH: u8 = 8;

/// A magazine of this means the weapon has none
const NO_MAGAZINE: i8 = -1;

/// A number on the HUD, and when it last changed
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reading {
    pub value: i32,
    pub changed: SourceTime,
}

/// The weapon out and its ammo, and when the client was last told of it
#[derive(Clone, Debug, PartialEq)]
pub struct WeaponStatus {
    /// Such as `weapon_9mmAR`, whose `sprites/weapon_9mmAR.txt` has its
    /// ammo's icons
    pub name: String,
    /// What its magazine holds, none for a weapon without one
    pub magazine: Option<i32>,
    /// The ammo carried for it, and for a second fire mode that takes its
    /// own, such as the MP5's grenades, while there is any
    pub ammo: Option<i32>,
    pub ammo2: Option<i32>,
    pub changed: SourceTime,
}

/// What the HUD shows at a moment
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shown {
    pub health: Option<Reading>,
    pub armour: Option<Reading>,
    pub weapon: Option<WeaponStatus>,
}

/// A weapon as `WeaponList` gave it
#[derive(Clone, Debug, PartialEq)]
struct WeaponInfo {
    name: String,
    /// Its ammo types, negative for none
    ammo: i8,
    ammo2: i8,
}

/// What the messages last said
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Status {
    health: Option<Reading>,
    armour: Option<Reading>,
    hide: u8,
    weapons: BTreeMap<i8, WeaponInfo>,
    /// The weapon out and its magazine, and when `CurWeapon` last came
    current: Option<(i8, i8)>,
    weapon_changed: SourceTime,
    /// How much of each ammo type is carried
    ammo: BTreeMap<u8, u8>,
}

/// A string ending at its first nought, and what follows it
fn c_string(bytes: &[u8]) -> (String, &[u8]) {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let text = String::from_utf8_lossy(&bytes[..end]).into_owned();
    (text, bytes.get(end + 1..).unwrap_or_default())
}

/// `reading` with `value`, its time of change kept when the value is the
/// same
fn read(reading: Option<Reading>, value: i32, time: SourceTime) -> Option<Reading> {
    match reading {
        Some(r) if r.value == value => Some(r),
        _ => Some(Reading {
            value,
            changed: time,
        }),
    }
}

impl Status {
    /// The user message `name` with `body` at `time`. Returns whether it
    /// was one of these
    pub(crate) fn message(&mut self, name: &str, body: &[u8], time: SourceTime) -> bool {
        match name {
            "Health" => {
                if let Some(&health) = body.first() {
                    self.health = read(self.health, i32::from(health), time);
                }
            }
            "Battery" => {
                if let Some(&[low, high]) = body.get(..2) {
                    let armour = i32::from(i16::from_le_bytes([low, high]));
                    self.armour = read(self.armour, armour, time);
                }
            }
            "HideWeapon" => self.hide = body.first().copied().unwrap_or(0),
            "WeaponList" => self.weapon_list(body),
            "CurWeapon" => self.current_weapon(body, time),
            "AmmoX" => {
                if let Some(&[index, count]) = body.get(..2) {
                    self.ammo.insert(index, count);
                }
            }
            _ => return false,
        }
        true
    }

    /// A `WeaponList`, a weapon's name, its ammo type and most ammo, its
    /// second ammo type and most of it, and then its slot, its place in
    /// the slot and its id
    fn weapon_list(&mut self, body: &[u8]) {
        let (name, rest) = c_string(body);
        let Some(&[ammo, _, ammo2, _, _, _, id]) = rest.get(..7) else {
            return;
        };
        let info = WeaponInfo {
            name,
            ammo: ammo as i8,
            ammo2: ammo2 as i8,
        };
        self.weapons.insert(id as i8, info);
    }

    /// A `CurWeapon`, whether the weapon is out, its id and its magazine
    fn current_weapon(&mut self, body: &[u8], time: SourceTime) {
        let Some(&[state, id, magazine]) = body.get(..3) else {
            return;
        };
        let (id, magazine) = (id as i8, magazine as i8);

        // No weapon out, as when dead
        if id < 1 {
            self.current = None;
            return;
        }

        // A weapon carried but not out, which leaves the one out as it is
        if state == 0 {
            return;
        }

        // A magazine under -1 is sent as its negative
        let magazine = if magazine < NO_MAGAZINE {
            magazine.saturating_abs()
        } else {
            magazine
        };
        self.current = Some((id, magazine));
        self.weapon_changed = time;
    }

    /// How much of ammo type `kind` is carried, none for a weapon without
    /// that ammo
    fn carried(&self, kind: i8) -> Option<i32> {
        let kind = u8::try_from(kind).ok().filter(|&k| k > 0)?;
        Some(i32::from(self.ammo.get(&kind).copied().unwrap_or(0)))
    }

    /// What the HUD shows, less what `HideWeapon` hides
    pub fn shown(&self) -> Shown {
        if self.hide & HIDE_ALL != 0 {
            return Shown::default();
        }

        let hide_health = self.hide & HIDE_HEALTH != 0;
        let hide_weapon = self.hide & HIDE_WEAPONS != 0;
        let weapon = self
            .current
            .filter(|_| !hide_weapon)
            .and_then(|(id, magazine)| self.weapon_status(id, magazine));
        Shown {
            health: self.health.filter(|_| !hide_health),
            armour: self.armour.filter(|_| !hide_health),
            weapon,
        }
    }

    /// The weapon `id` with `magazine`, none for one without ammo, such as
    /// the crowbar
    fn weapon_status(&self, id: i8, magazine: i8) -> Option<WeaponStatus> {
        let info = self.weapons.get(&id)?;
        if info.ammo < 0 && info.ammo2 < 0 {
            return None;
        }

        let second = self.carried(info.ammo2).filter(|&n| n > 0);
        Some(WeaponStatus {
            name: info.name.clone(),
            magazine: (magazine != NO_MAGAZINE).then_some(i32::from(magazine)),
            ammo: self.carried(info.ammo),
            ammo2: second,
            changed: self.weapon_changed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `WeaponList` body for `name` with ammo types `ammo` and `ammo2`
    /// and `id`
    fn weapon(name: &str, ammo: i8, ammo2: i8, id: u8) -> Vec<u8> {
        let mut body = name.as_bytes().to_vec();
        body.push(0);
        body.extend([ammo as u8, 250, ammo2 as u8, 10, 2, 0, id, 0]);
        body
    }

    #[test]
    fn health_and_armour_remember_when_they_changed() {
        let mut s = Status::default();
        s.message("Health", &[100], SourceTime(1.));
        s.message("Health", &[100], SourceTime(2.));
        s.message("Battery", &[30, 0], SourceTime(3.));

        let shown = s.shown();
        assert_eq!(shown.health.unwrap().changed, SourceTime(1.));
        assert_eq!(shown.armour.unwrap().value, 30);

        // Hidden by `HideWeapon`
        s.message("HideWeapon", &[HIDE_HEALTH], SourceTime(4.));
        assert_eq!(s.shown().health, None);
    }

    #[test]
    fn the_weapon_out_shows_its_magazine_and_ammo() {
        let mut s = Status::default();
        s.message(
            "WeaponList",
            &weapon("weapon_9mmAR", 1, 2, 4),
            SourceTime(0.),
        );
        s.message(
            "WeaponList",
            &weapon("weapon_crowbar", -1, -1, 1),
            SourceTime(0.),
        );
        s.message("AmmoX", &[1, 150], SourceTime(0.));
        s.message("AmmoX", &[2, 2], SourceTime(0.));
        s.message("CurWeapon", &[1, 4, 37], SourceTime(5.));

        let weapon = s.shown().weapon.unwrap();
        assert_eq!(weapon.name, "weapon_9mmAR");
        assert_eq!(
            (weapon.magazine, weapon.ammo, weapon.ammo2),
            (Some(37), Some(150), Some(2))
        );
        assert_eq!(weapon.changed, SourceTime(5.));

        // A weapon carried but not out leaves the one out
        s.message("CurWeapon", &[0, 1, 255], SourceTime(6.));
        assert_eq!(s.shown().weapon.unwrap().name, "weapon_9mmAR");

        // The crowbar has no ammo to show
        s.message("CurWeapon", &[1, 1, 255], SourceTime(7.));
        assert_eq!(s.shown().weapon, None);
    }
}

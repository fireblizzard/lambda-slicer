//! The recording player's health, armour and ammo, and what they picked
//! up, as the game's client keeps them for its HUD (HLSDK's `CHudHealth`,
//! `CHudBattery`, `CHudAmmo` and `HistoryResource`)
//!
//! `Health` is a byte and `Battery` a short. `WeaponList` names each weapon
//! with its ammo types, `CurWeapon` says which weapon is out and what its
//! magazine holds, and `AmmoX` how much of an ammo type is carried.
//! `HideWeapon` hides parts of the HUD by its flags. A reading remembers
//! when it last changed, as the client brightens a number that changes and
//! fades it back over the next seconds
//!
//! `AmmoPickup` is an ammo type and how much, `WeapPickup` a weapon's id and
//! `ItemPickup` an item's name in `sprites/hud.txt`. Each shows for five
//! seconds in the next of six places up from the bottom, as many as fit on
//! the game's 480 row screen a weapon's height apart, the places starting
//! from the bottom again once all are empty or none is left

use std::collections::BTreeMap;

use crate::time::SourceTime;

/// `HideWeapon`'s flags that hide the weapon's ammo, everything, and the
/// health and armour (`HIDEHUD_WEAPONS`, `HIDEHUD_ALL`, `HIDEHUD_HEALTH`)
const HIDE_WEAPONS: u8 = 1;
const HIDE_ALL: u8 = 4;
const HIDE_HEALTH: u8 = 8;

/// A magazine of this means the weapon has none
const NO_MAGAZINE: i8 = -1;

/// The weapon menu's columns and the places in each, and the room kept
/// for both (`MAX_WEAPON_SLOTS`, `MAX_WEAPON_POSITIONS`)
pub(crate) const MENU_SLOTS: usize = 5;
pub(crate) const MENU_PLACES: usize = 5;
const MENU_ROOM: usize = 6;

/// The weapon in each column and place of the menu, by id
pub(crate) type MenuSlots = [[Option<i8>; MENU_ROOM]; MENU_ROOM];

/// The most ammo that means a weapon takes none, and `WeaponList`'s flag
/// for a weapon that can be taken out empty (`WEAPON_FLAGS_SELECTONEMPTY`)
const NO_MOST_AMMO: u8 = 255;
const SELECT_ON_EMPTY: u8 = 1;

/// How long a pickup shows, in seconds, and how many places fit, those
/// whose top stays under `AMMO_PICKUP_HEIGHT_MAX` (`hud_drawhistory_time`)
const PICKUP_SHOWN: f64 = 5.;
const PICKUP_PLACES: usize = 6;

/// How long after a weapon is picked up its ammo can still be on the way,
/// in seconds. The server may send the ammo a few frames after the weapon,
/// and the game shows the weapon as empty meanwhile, so a weapon only
/// shows empty once this has passed
const AMMO_ON_THE_WAY: f64 = 0.5;

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

/// What was picked up, as its message gave it
#[derive(Clone, Debug, PartialEq)]
enum Picked {
    /// An ammo type and how much
    Ammo(u8, i32),
    /// A weapon's id
    Weapon(i8),
    /// An item's name in `sprites/hud.txt`
    Item(String),
}

/// What a pickup shows
#[derive(Clone, Debug, PartialEq)]
pub enum PickupShown {
    /// The icon `icon` of the weapon `weapon`'s list, `ammo` or `ammo2`,
    /// and how much was picked up
    Ammo {
        weapon: String,
        icon: &'static str,
        count: i32,
    },
    /// A weapon's name, and whether it has no ammo to fire
    Weapon { name: String, empty: bool },
    /// An item's name in `sprites/hud.txt`
    Item(String),
}

/// A pickup in its place, counted up from the bottom, and when it goes
#[derive(Clone, Debug, PartialEq)]
pub struct Pickup {
    pub shown: PickupShown,
    pub place: usize,
    pub until: SourceTime,
}

/// A weapon as `WeaponList` gave it
#[derive(Clone, Debug, PartialEq)]
struct WeaponInfo {
    name: String,
    /// Its ammo types, negative for none
    ammo: i8,
    ammo2: i8,
    /// The most of its first and second ammo it holds, `NO_MOST_AMMO` for
    /// none
    most_ammo: u8,
    most_ammo2: u8,
    /// Its column in the weapon menu and its place in the column
    slot: u8,
    position: u8,
    flags: u8,
    /// What its magazine holds, as `CurWeapon` last said, out or not
    magazine: i8,
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
    /// What was picked up by place, when it goes, and the next place
    pickups: BTreeMap<usize, (Picked, SourceTime)>,
    next_place: usize,
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
            "AmmoPickup" => {
                // No amount adds nothing
                if let Some(&[kind, count]) = body.get(..2)
                    && count != 0
                {
                    let count = i32::from(count as i8).abs();
                    self.picked(Picked::Ammo(kind, count), time);
                }
            }
            "WeapPickup" => {
                if let Some(&id) = body.first() {
                    self.picked(Picked::Weapon(id as i8), time);
                }
            }
            "ItemPickup" => self.picked(Picked::Item(c_string(body).0), time),
            _ => return false,
        }
        true
    }

    /// A `WeaponList`, a weapon's name, its ammo type and most ammo, its
    /// second ammo type and most of it, and then its slot, its place in
    /// the slot and its id
    fn weapon_list(&mut self, body: &[u8]) {
        let (name, rest) = c_string(body);
        let Some(&[ammo, most_ammo, ammo2, most_ammo2, slot, position, id]) = rest.get(..7) else {
            return;
        };
        let info = WeaponInfo {
            name,
            ammo: ammo as i8,
            ammo2: ammo2 as i8,
            most_ammo,
            most_ammo2,
            slot,
            position,
            flags: rest.get(7).copied().unwrap_or(0),
            magazine: 0,
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

        // A magazine under -1 is sent as its negative. Each weapon keeps its
        // own, out or not
        let magazine = if magazine < NO_MAGAZINE {
            magazine.saturating_abs()
        } else {
            magazine
        };
        if let Some(info) = self.weapons.get_mut(&id) {
            info.magazine = magazine;
        }

        // A weapon carried but not out, which leaves the one out as it is
        if state == 0 {
            return;
        }
        self.current = Some((id, magazine));
        self.weapon_changed = time;
    }

    /// `picked` at `time`, in the next place
    fn picked(&mut self, picked: Picked, time: SourceTime) {
        self.pickups.retain(|_, (_, until)| *until > time);
        if self.pickups.is_empty() || self.next_place >= PICKUP_PLACES {
            self.next_place = 0;
        }

        let until = SourceTime(time.0 + PICKUP_SHOWN);
        self.pickups.insert(self.next_place, (picked, until));
        self.next_place += 1;
    }

    /// What was picked up and still shows at `at`, less what `HideWeapon`
    /// hides
    pub fn pickups(&self, at: SourceTime) -> Vec<Pickup> {
        if self.hide & (HIDE_WEAPONS | HIDE_ALL) != 0 {
            return Vec::new();
        }

        let mut out = Vec::new();
        for (&place, (picked, until)) in &self.pickups {
            if *until <= at {
                continue;
            }
            let picked_at = until.0 - PICKUP_SHOWN;
            let settled = at.0 >= picked_at + AMMO_ON_THE_WAY;
            if let Some(shown) = self.pickup_shown(picked, settled) {
                let until = *until;
                out.push(Pickup {
                    shown,
                    place,
                    until,
                });
            }
        }
        out
    }

    /// How `picked` shows, none for a weapon or ammo the client does not
    /// know yet. A weapon shows empty only once its ammo has `settled`
    fn pickup_shown(&self, picked: &Picked, settled: bool) -> Option<PickupShown> {
        match picked {
            Picked::Ammo(kind, count) => {
                // The first weapon taking that ammo has its icon
                let kind = *kind as i8;
                let mut found = None;
                for info in self.weapons.values() {
                    if info.ammo == kind {
                        found = Some((info, "ammo"));
                    } else if info.ammo2 == kind {
                        found = Some((info, "ammo2"));
                    }
                    if found.is_some() {
                        break;
                    }
                }
                let (info, icon) = found?;
                Some(PickupShown::Ammo {
                    weapon: info.name.clone(),
                    icon,
                    count: *count,
                })
            }
            Picked::Weapon(id) => {
                let info = self.weapons.get(id)?;
                Some(PickupShown::Weapon {
                    name: info.name.clone(),
                    empty: settled && !self.has_ammo(info),
                })
            }
            Picked::Item(name) => Some(PickupShown::Item(name.clone())),
        }
    }

    /// Whether `info`'s weapon has ammo to fire, in its magazine or carried,
    /// takes none, or can be taken out empty (`HasAmmo`)
    fn has_ammo(&self, info: &WeaponInfo) -> bool {
        if info.most_ammo == NO_MOST_AMMO || info.ammo < 0 {
            return true;
        }
        if info.flags & SELECT_ON_EMPTY != 0 {
            return true;
        }
        let carried = |kind| self.carried(kind).is_some_and(|n| n > 0);
        info.magazine > 0 || carried(info.ammo) || carried(info.ammo2)
    }

    /// How much of ammo type `kind` is carried, none for a weapon without
    /// that ammo
    fn carried(&self, kind: i8) -> Option<i32> {
        let kind = u8::try_from(kind).ok().filter(|&k| k > 0)?;
        Some(i32::from(self.ammo.get(&kind).copied().unwrap_or(0)))
    }

    /// The weapons of `owned`, the client data's weapon bits, in their
    /// columns and places of the menu (`PickupWeapon`)
    pub(crate) fn menu_slots(&self, owned: u32) -> MenuSlots {
        let mut slots: MenuSlots = Default::default();
        for (&id, info) in self.weapons.iter().rev() {
            let owns = u32::try_from(id).is_ok_and(|bit| bit < 32 && owned & (1 << bit) != 0);
            let (slot, position) = (usize::from(info.slot), usize::from(info.position));
            if owns && slot < MENU_ROOM && position < MENU_ROOM {
                slots[slot][position] = Some(id);
            }
        }
        slots
    }

    /// Whether the weapon `id` has ammo to fire
    pub(crate) fn weapon_has_ammo(&self, id: i8) -> bool {
        self.weapons
            .get(&id)
            .is_some_and(|info| self.has_ammo(info))
    }

    /// The weapon out, by id
    pub(crate) fn current_id(&self) -> Option<i8> {
        self.current.map(|(id, _)| id)
    }

    /// The weapon `id`'s name, and how full its ammo is as shares of the
    /// most it holds, none without ammo carried (`DrawAmmoBar`)
    pub(crate) fn menu_weapon(&self, id: i8) -> Option<(String, Vec<f32>)> {
        let info = self.weapons.get(&id)?;
        // The client takes no most ammo as -1, so such a bar shows empty
        let share = |kind: i8, most: u8| {
            let carried = self.carried(kind).unwrap_or(0) as f32;
            let most = if most == NO_MOST_AMMO {
                -1.
            } else {
                f32::from(most)
            };
            (carried / most).clamp(0., 1.)
        };

        let mut bars = Vec::new();
        if info.ammo >= 0 && self.carried(info.ammo).unwrap_or(0) > 0 {
            bars.push(share(info.ammo, info.most_ammo));
            if info.ammo2 >= 0 {
                bars.push(share(info.ammo2, info.most_ammo2));
            }
        }

        Some((info.name.clone(), bars))
    }

    /// Whether `HideWeapon` hides the weapon's ammo, and with it the menu
    pub(crate) fn hides_weapons(&self) -> bool {
        self.hide & (HIDE_WEAPONS | HIDE_ALL) != 0
    }

    /// Whether `HideWeapon` hides the health, and with it the damage
    pub(crate) fn hides_health(&self) -> bool {
        self.hide & (HIDE_HEALTH | HIDE_ALL) != 0
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

    #[test]
    fn pickups_take_places_up_from_the_bottom_for_five_seconds() {
        let mut s = Status::default();
        s.message(
            "WeaponList",
            &weapon("weapon_9mmAR", 1, 2, 4),
            SourceTime(0.),
        );
        s.message("AmmoPickup", &[2, 1], SourceTime(0.));
        s.message("WeapPickup", &[4], SourceTime(1.));
        s.message("ItemPickup", b"item_battery\0", SourceTime(2.));
        s.message("AmmoPickup", &[1, 0], SourceTime(2.));

        let shown: Vec<(usize, PickupShown)> = s
            .pickups(SourceTime(3.))
            .into_iter()
            .map(|p| (p.place, p.shown))
            .collect();
        assert_eq!(
            shown,
            [
                (
                    0,
                    PickupShown::Ammo {
                        weapon: "weapon_9mmAR".into(),
                        icon: "ammo2",
                        count: 1,
                    }
                ),
                (
                    1,
                    PickupShown::Weapon {
                        name: "weapon_9mmAR".into(),
                        empty: true,
                    }
                ),
                (2, PickupShown::Item("item_battery".into())),
            ]
        );
        assert_eq!(s.pickups(SourceTime(5.)).len(), 2);

        // Once all are gone the places start from the bottom again
        s.message("ItemPickup", b"item_longjump\0", SourceTime(8.));
        assert_eq!(s.pickups(SourceTime(8.))[0].place, 0);
    }

    #[test]
    fn a_weapon_picked_up_loaded_but_not_out_has_ammo() {
        let mut s = Status::default();
        s.message(
            "WeaponList",
            &weapon("weapon_rpg", 3, -1, 8),
            SourceTime(0.),
        );
        s.message("WeapPickup", &[8], SourceTime(0.));
        let empty = |s: &Status, at: f64| match &s.pickups(SourceTime(at))[0].shown {
            PickupShown::Weapon { empty, .. } => *empty,
            _ => unreachable!(),
        };

        // Not empty while its ammo may still be on the way, then empty
        assert!(!empty(&s, 0.4));
        assert!(empty(&s, 0.5));

        // Its magazine comes for it while another weapon is out
        s.message("CurWeapon", &[0, 8, 1], SourceTime(0.1));
        assert!(!empty(&s, 0.5));
    }
}

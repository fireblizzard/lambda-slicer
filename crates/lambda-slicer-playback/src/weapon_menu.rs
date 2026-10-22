//! The weapon menu the recording player opens with their keys, as the
//! game's client keeps it (HLSDK's `CHudAmmo` and `WeaponsResource`)
//!
//! A recording keeps some of the player's commands. `slot1` to `slot5` open
//! the menu on that column's first weapon with ammo, and again move down
//! it, starting over at its top. A column with none shows the menu without
//! a weapon picked. `invnext` and `invprev` move to the next or the one
//! before with ammo across all the columns, from the weapon out when the
//! menu is closed. Firing takes the weapon picked, if any, and closes the
//! menu with a click, and nothing else closes it but `cancelselect`. The menu needs the suit and
//! a weapon, and `HideWeapon` hiding the weapons closes it. Opening and
//! moving down a column and taking a weapon play the client's sounds. The
//! client's quick switching, a column of one weapon taking it at once, is
//! taken as off, its default

use crate::status::{MENU_PLACES, MENU_SLOTS, MenuSlots, Status};

/// The weapon bit that means the suit (`WEAPON_SUIT`)
const SUIT: u32 = 1 << 31;

/// The sounds the client plays when the menu opens or moves down a column,
/// and when a weapon is taken
const OPENED: &str = "common/wpn_hudon.wav";
const MOVED: &str = "common/wpn_moveselect.wav";
const TAKEN: &str = "common/wpn_select.wav";

/// What the menu has picked
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Picked {
    /// The menu shows with no weapon picked
    Nothing,
    Weapon(i8),
}

/// One weapon in the menu
#[derive(Clone, Debug, PartialEq)]
pub struct MenuWeapon {
    /// Such as `weapon_crossbow`, whose `sprites/weapon_crossbow.txt` has
    /// its pictures
    pub name: String,
    pub picked: bool,
    pub has_ammo: bool,
    /// How full its ammo is, as shares of the most it holds, none without
    /// ammo carried
    pub bars: Vec<f32>,
}

/// The menu as it shows
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuShown {
    /// The column whose weapons show as pictures, none when no weapon is
    /// picked
    pub open_column: Option<usize>,
    /// Each column's weapons from the top
    pub columns: Vec<Vec<MenuWeapon>>,
}

/// What the menu has picked, none while closed
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeaponMenu {
    picked: Option<Picked>,
}

/// The first weapon with ammo in `slot`'s places from `from` on
/// (`GetFirstPos`, `GetNextActivePos`)
fn first_with_ammo(status: &Status, slots: &MenuSlots, slot: usize, from: usize) -> Option<i8> {
    let column = slots.get(slot)?;
    let places = column.get(from..MENU_PLACES).unwrap_or_default();
    for &weapon in places {
        if let Some(id) = weapon
            && status.weapon_has_ammo(id)
        {
            return Some(id);
        }
    }
    None
}

/// The column and place of `id` in `slots`
fn place_of(slots: &MenuSlots, id: i8) -> Option<(usize, usize)> {
    for (slot, column) in slots.iter().enumerate() {
        if let Some(place) = column.iter().position(|&w| w == Some(id)) {
            return Some((slot, place));
        }
    }
    None
}

impl WeaponMenu {
    /// The command `command` at a moment the player owns the weapons of
    /// `owned`, and the sound the client plays for it
    pub(crate) fn command(
        &mut self,
        command: &str,
        status: &Status,
        owned: u32,
    ) -> Option<&'static str> {
        if status.hides_weapons() {
            self.picked = None;
            return None;
        }

        let slots = status.menu_slots(owned);
        if let Some(number) = command.strip_prefix("slot") {
            let slot = number.parse::<usize>().ok()?.checked_sub(1)?;
            return self.slot(status, &slots, slot, owned);
        }

        match command {
            "invnext" | "invprev" => {
                self.step(status, &slots, command == "invnext");
                None
            }
            "+attack" => {
                // The client clicks as it closes, a weapon picked or not
                self.picked.take()?;
                Some(TAKEN)
            }
            "cancelselect" => {
                self.picked = None;
                None
            }
            _ => None,
        }
    }

    /// A `slot` key, counted from nought (`SelectSlot`)
    fn slot(
        &mut self,
        status: &Status,
        slots: &MenuSlots,
        slot: usize,
        owned: u32,
    ) -> Option<&'static str> {
        // Nothing opens past the last column, or without the suit and a
        // weapon besides it
        if slot > MENU_SLOTS || owned & SUIT == 0 || owned & !SUIT == 0 {
            return None;
        }

        let same_column = match self.picked {
            Some(Picked::Weapon(id)) => place_of(slots, id).filter(|&(s, _)| s == slot),
            _ => None,
        };
        let (weapon, sound) = match same_column {
            Some((_, place)) => {
                let next = first_with_ammo(status, slots, slot, place + 1);
                (
                    next.or_else(|| first_with_ammo(status, slots, slot, 0)),
                    MOVED,
                )
            }
            None => (first_with_ammo(status, slots, slot, 0), OPENED),
        };
        self.picked = Some(weapon.map_or(Picked::Nothing, Picked::Weapon));
        Some(sound)
    }

    /// `invnext` going `forward`, or `invprev`, through every column
    /// (`UserCmd_NextWeapon`, `UserCmd_PrevWeapon`)
    fn step(&mut self, status: &Status, slots: &MenuSlots, forward: bool) {
        let from = match self.picked {
            Some(Picked::Weapon(id)) => Some(id),
            _ => status.current_id(),
        };

        // Places count through the columns in turn
        let places = MENU_SLOTS * MENU_PLACES;
        let start = from
            .and_then(|id| place_of(slots, id))
            .map(|(slot, place)| slot * MENU_PLACES + place);

        // The places the client looks at, on from where it starts to the
        // end, then all of them again
        let mut order: Vec<usize> = Vec::with_capacity(places * 2);
        if forward {
            let first = start.map_or(0, |at| at + 1);
            order.extend(first..places);
            order.extend(0..places);
        } else {
            let first = start.unwrap_or(places);
            order.extend((0..first).rev());
            order.extend((0..places).rev());
        }

        self.picked = None;
        for at in order {
            let (slot, place) = (at / MENU_PLACES, at % MENU_PLACES);
            if let Some(id) = slots[slot][place]
                && status.weapon_has_ammo(id)
            {
                self.picked = Some(Picked::Weapon(id));
                return;
            }
        }
    }

    /// The menu as it shows with the weapons of `owned`, none while closed
    /// or hidden (`DrawWList`)
    pub fn shown(&self, status: &Status, owned: u32) -> Option<MenuShown> {
        let picked = self.picked?;
        if status.hides_weapons() {
            return None;
        }

        let slots = status.menu_slots(owned);

        // A column that has run out of ammo shows nothing picked, which the
        // client checks for every column but the first
        let mut open_column = match picked {
            Picked::Weapon(id) => place_of(&slots, id).map(|(slot, _)| slot),
            Picked::Nothing => None,
        };
        if let Some(slot) = open_column
            && slot > 0
            && first_with_ammo(status, &slots, slot, 0).is_none()
        {
            open_column = None;
        }

        let mut columns = Vec::with_capacity(MENU_SLOTS);
        for column in slots.iter().take(MENU_SLOTS) {
            let mut weapons = Vec::new();
            for &id in column.iter().take(MENU_PLACES).flatten() {
                let Some((name, bars)) = status.menu_weapon(id) else {
                    continue;
                };
                weapons.push(MenuWeapon {
                    name,
                    picked: open_column.is_some() && picked == Picked::Weapon(id),
                    has_ammo: status.weapon_has_ammo(id),
                    bars,
                });
            }
            columns.push(weapons);
        }

        Some(MenuShown {
            open_column,
            columns,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::SourceTime;

    /// A `WeaponList` body for `name` with ammo type `ammo` in `slot` at
    /// `position`, as `id`
    fn weapon(name: &str, ammo: i8, slot: u8, position: u8, id: u8) -> Vec<u8> {
        let mut body = name.as_bytes().to_vec();
        body.push(0);
        body.extend([ammo as u8, 50, 255, 255, slot, position, id, 0]);
        body
    }

    /// A status with the crowbar, the shotgun and the crossbow, the
    /// crossbow out of ammo, and the bits owning all three and the suit
    fn armed() -> (Status, u32) {
        let mut s = Status::default();
        let t = SourceTime(0.);
        s.message("WeaponList", &weapon("weapon_crowbar", -1, 0, 0, 1), t);
        s.message("WeaponList", &weapon("weapon_shotgun", 1, 2, 0, 7), t);
        s.message("WeaponList", &weapon("weapon_crossbow", 2, 2, 1, 6), t);
        s.message("AmmoX", &[1, 12], t);
        s.message("CurWeapon", &[1, 1, 255], t);
        (s, SUIT | (1 << 1) | (1 << 7) | (1 << 6))
    }

    fn picked_name(menu: &WeaponMenu, s: &Status, owned: u32) -> Option<String> {
        let shown = menu.shown(s, owned)?;
        let column = &shown.columns[shown.open_column?];
        column.iter().find(|w| w.picked).map(|w| w.name.clone())
    }

    #[test]
    fn a_slot_opens_its_column_and_skips_weapons_without_ammo() {
        let (s, owned) = armed();
        let mut menu = WeaponMenu::default();
        assert_eq!(menu.command("slot3", &s, owned), Some(OPENED));
        assert_eq!(
            picked_name(&menu, &s, owned).as_deref(),
            Some("weapon_shotgun")
        );

        // Again stays on the shotgun, the crossbow having no ammo
        assert_eq!(menu.command("slot3", &s, owned), Some(MOVED));
        assert_eq!(
            picked_name(&menu, &s, owned).as_deref(),
            Some("weapon_shotgun")
        );
        let shown = menu.shown(&s, owned).unwrap();
        assert!(!shown.columns[2][1].has_ammo);

        // Firing takes it and closes the menu
        assert_eq!(menu.command("+attack", &s, owned), Some(TAKEN));
        assert_eq!(menu.shown(&s, owned), None);
    }

    #[test]
    fn next_and_previous_go_across_the_columns_from_the_weapon_out() {
        let (s, owned) = armed();
        let mut menu = WeaponMenu::default();
        menu.command("invnext", &s, owned);
        assert_eq!(
            picked_name(&menu, &s, owned).as_deref(),
            Some("weapon_shotgun")
        );
        menu.command("invnext", &s, owned);
        assert_eq!(
            picked_name(&menu, &s, owned).as_deref(),
            Some("weapon_crowbar")
        );
        menu.command("invprev", &s, owned);
        assert_eq!(
            picked_name(&menu, &s, owned).as_deref(),
            Some("weapon_shotgun")
        );
    }

    #[test]
    fn a_column_without_weapons_shows_the_menu_with_nothing_picked() {
        let (s, owned) = armed();
        let mut menu = WeaponMenu::default();
        menu.command("slot5", &s, owned);
        let shown = menu.shown(&s, owned).unwrap();
        assert_eq!(shown.open_column, None);

        // Firing closes it with the click, and fires again once closed
        assert_eq!(menu.command("+attack", &s, owned), Some(TAKEN));
        assert_eq!(menu.shown(&s, owned), None);
        assert_eq!(menu.command("+attack", &s, owned), None);
    }
}

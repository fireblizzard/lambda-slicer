//! The keys and mouse buttons that run the app's commands, which the
//! settings can change
//!
//! Every action has up to two bindings. A binding is a key with Ctrl, Shift
//! and Alt as held, mouse 3, 4 or 5, or the wheel turned up or down, which
//! counts over the demo's picture. The free camera moves while its keys
//! are held, so its bindings are a key, a mouse button, or Ctrl, Shift or
//! Alt alone. A binding belongs to one action at a time, the free camera's
//! apart from the rest, as they only count while it flies. Escape is never
//! bound, as it cancels choosing a key and closes what is open
//!
//! A press runs the action whose binding has exactly the modifiers held, or
//! else one whose Shift and Alt are among those held, so Shift+Left skips
//! while Left seeks, and a + typed with Shift still plays faster

use std::collections::BTreeMap;

use eframe::egui::{self, Event, InputState, Key, Modifiers, PointerButton};
use fluent_bundle::FluentValue;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::i18n::Catalog;

/// What a shortcut does
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Action {
    OpenDemo,
    Settings,
    Back,
    Forward,
    PreviousEvent,
    NextEvent,
    ClipStart,
    ClipEnd,
    PlayPause,
    Rewind,
    Pause,
    SeekBack,
    SeekForward,
    SkipBack,
    SkipForward,
    FrameBack,
    FrameForward,
    Slower,
    Faster,
    Start,
    Fullscreen,
    Zoom,
    SeeThrough,
    Fullbright,
    Names,
    FreeCamera,
    Recorder,
    /// The map's best players, from the first
    Follow(u8),
    PreviousPlayer,
    NextPlayer,
    FlyForward,
    FlyBack,
    FlyLeft,
    FlyRight,
    FlyUp,
    FlyDown,
    FlyFast,
}

/// The settings' headings the actions stand under
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Anywhere,
    Demo,
    Playback,
    View,
    Players,
    FreeCamera,
}

impl Group {
    pub fn text_id(self) -> &'static str {
        match self {
            Group::Anywhere => "shortcuts-anywhere",
            Group::Demo => "shortcuts-demo",
            Group::Playback => "shortcuts-playback",
            Group::View => "shortcuts-view",
            Group::Players => "shortcuts-players",
            Group::FreeCamera => "shortcuts-free-camera",
        }
    }
}

/// How many of the map's best players have an action of their own
pub const FOLLOWED: u8 = 10;

impl Action {
    /// Every action, in the order the settings list them
    pub const ALL: [Action; 46] = [
        Action::OpenDemo,
        Action::Settings,
        Action::Back,
        Action::Forward,
        Action::PreviousEvent,
        Action::NextEvent,
        Action::ClipStart,
        Action::ClipEnd,
        Action::PlayPause,
        Action::Rewind,
        Action::Pause,
        Action::SeekBack,
        Action::SeekForward,
        Action::SkipBack,
        Action::SkipForward,
        Action::FrameBack,
        Action::FrameForward,
        Action::Slower,
        Action::Faster,
        Action::Start,
        Action::Fullscreen,
        Action::Zoom,
        Action::SeeThrough,
        Action::Fullbright,
        Action::Names,
        Action::FreeCamera,
        Action::Recorder,
        Action::Follow(0),
        Action::Follow(1),
        Action::Follow(2),
        Action::Follow(3),
        Action::Follow(4),
        Action::Follow(5),
        Action::Follow(6),
        Action::Follow(7),
        Action::Follow(8),
        Action::Follow(9),
        Action::PreviousPlayer,
        Action::NextPlayer,
        Action::FlyForward,
        Action::FlyBack,
        Action::FlyLeft,
        Action::FlyRight,
        Action::FlyUp,
        Action::FlyDown,
        Action::FlyFast,
    ];

    pub fn group(self) -> Group {
        match self {
            Action::OpenDemo | Action::Settings | Action::Back | Action::Forward => Group::Anywhere,
            Action::PreviousEvent | Action::NextEvent | Action::ClipStart | Action::ClipEnd => {
                Group::Demo
            }
            Action::PlayPause
            | Action::Rewind
            | Action::Pause
            | Action::SeekBack
            | Action::SeekForward
            | Action::SkipBack
            | Action::SkipForward
            | Action::FrameBack
            | Action::FrameForward
            | Action::Slower
            | Action::Faster
            | Action::Start => Group::Playback,
            Action::Fullscreen
            | Action::Zoom
            | Action::SeeThrough
            | Action::Fullbright
            | Action::Names
            | Action::FreeCamera => Group::View,
            Action::Recorder | Action::Follow(_) | Action::PreviousPlayer | Action::NextPlayer => {
                Group::Players
            }
            Action::FlyForward
            | Action::FlyBack
            | Action::FlyLeft
            | Action::FlyRight
            | Action::FlyUp
            | Action::FlyDown
            | Action::FlyFast => Group::FreeCamera,
        }
    }

    /// Whether the action moves the free camera while held
    pub fn flies(self) -> bool {
        self.group() == Group::FreeCamera
    }

    /// The action's name in the settings file
    pub fn name(self) -> String {
        let name = match self {
            Action::OpenDemo => "open-demo",
            Action::Settings => "settings",
            Action::Back => "back",
            Action::Forward => "forward",
            Action::PreviousEvent => "previous-event",
            Action::NextEvent => "next-event",
            Action::ClipStart => "clip-start",
            Action::ClipEnd => "clip-end",
            Action::PlayPause => "play-pause",
            Action::Rewind => "rewind",
            Action::Pause => "pause",
            Action::SeekBack => "seek-back",
            Action::SeekForward => "seek-forward",
            Action::SkipBack => "skip-back",
            Action::SkipForward => "skip-forward",
            Action::FrameBack => "frame-back",
            Action::FrameForward => "frame-forward",
            Action::Slower => "slower",
            Action::Faster => "faster",
            Action::Start => "start",
            Action::Fullscreen => "fullscreen",
            Action::Zoom => "zoom",
            Action::SeeThrough => "see-through",
            Action::Fullbright => "fullbright",
            Action::Names => "names",
            Action::FreeCamera => "free-camera",
            Action::Recorder => "recorder",
            Action::Follow(place) => return format!("follow-{}", place + 1),
            Action::PreviousPlayer => "previous-player",
            Action::NextPlayer => "next-player",
            Action::FlyForward => "fly-forward",
            Action::FlyBack => "fly-back",
            Action::FlyLeft => "fly-left",
            Action::FlyRight => "fly-right",
            Action::FlyUp => "fly-up",
            Action::FlyDown => "fly-down",
            Action::FlyFast => "fly-fast",
        };
        name.to_owned()
    }

    /// What the action does, in the catalog's words
    pub fn text(self, catalog: &Catalog) -> String {
        match self {
            Action::Follow(place) => {
                let place = FluentValue::from(i64::from(place) + 1);
                catalog.text_with("shortcut-follow", &[("place", place)])
            }
            _ => catalog.text(&format!("shortcut-{}", self.name())),
        }
    }

    /// The keys the action has until the settings change them
    fn defaults(self) -> [Option<Binding>; 2] {
        let one = |input| [Some(Binding::plain(input)), None];
        let key = |key| one(Input::Key(key));
        let ctrl = |key| [Some(Binding::ctrl(key)), None];
        let shift = |key| [Some(Binding::shift(key)), None];
        match self {
            Action::OpenDemo => ctrl(Key::O),
            Action::Settings => ctrl(Key::Comma),
            Action::Back => one(Input::Mouse4),
            Action::Forward => one(Input::Mouse5),
            Action::PreviousEvent => ctrl(Key::ArrowLeft),
            Action::NextEvent => ctrl(Key::ArrowRight),
            Action::ClipStart => key(Key::I),
            Action::ClipEnd => key(Key::O),
            Action::PlayPause => [
                Some(Binding::plain(Input::Key(Key::Space))),
                Some(Binding::plain(Input::Key(Key::L))),
            ],
            Action::Rewind => key(Key::J),
            Action::Pause => key(Key::K),
            Action::SeekBack => key(Key::ArrowLeft),
            Action::SeekForward => key(Key::ArrowRight),
            Action::SkipBack => shift(Key::ArrowLeft),
            Action::SkipForward => shift(Key::ArrowRight),
            Action::FrameBack => key(Key::Comma),
            Action::FrameForward => key(Key::Period),
            Action::Slower => key(Key::Minus),
            Action::Faster => [
                Some(Binding::plain(Input::Key(Key::Plus))),
                Some(Binding::plain(Input::Key(Key::Equals))),
            ],
            Action::Start => key(Key::Home),
            Action::Fullscreen => key(Key::F11),
            Action::Zoom => key(Key::Z),
            Action::SeeThrough => key(Key::X),
            Action::Fullbright => key(Key::F),
            Action::Names => key(Key::N),
            Action::FreeCamera => key(Key::C),
            Action::Recorder => key(Key::Backtick),
            Action::Follow(place) => key(NUMBER_KEYS[usize::from(place)]),
            Action::PreviousPlayer => [
                Some(Binding::plain(Input::Key(Key::PageUp))),
                Some(Binding::plain(Input::WheelUp)),
            ],
            Action::NextPlayer => [
                Some(Binding::plain(Input::Key(Key::PageDown))),
                Some(Binding::plain(Input::WheelDown)),
            ],
            Action::FlyForward => key(Key::W),
            Action::FlyBack => key(Key::S),
            Action::FlyLeft => key(Key::A),
            Action::FlyRight => key(Key::D),
            Action::FlyUp => key(Key::Space),
            Action::FlyDown => one(Input::Ctrl),
            Action::FlyFast => one(Input::Shift),
        }
    }
}

/// The keys that follow the map's best players, first to tenth
const NUMBER_KEYS: [Key; FOLLOWED as usize] = [
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
    Key::Num0,
];

/// A key, a mouse button, the wheel turned, or for the free camera a
/// modifier alone
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Input {
    Key(Key),
    Mouse3,
    Mouse4,
    Mouse5,
    WheelUp,
    WheelDown,
    Ctrl,
    Shift,
    Alt,
}

impl Input {
    /// The input's name in the settings file
    fn name(self) -> &'static str {
        match self {
            Input::Key(key) => key.name(),
            Input::Mouse3 => "Mouse3",
            Input::Mouse4 => "Mouse4",
            Input::Mouse5 => "Mouse5",
            Input::WheelUp => "WheelUp",
            Input::WheelDown => "WheelDown",
            Input::Ctrl => "Ctrl",
            Input::Shift => "Shift",
            Input::Alt => "Alt",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        let input = match name {
            "Mouse3" => Input::Mouse3,
            "Mouse4" => Input::Mouse4,
            "Mouse5" => Input::Mouse5,
            "WheelUp" => Input::WheelUp,
            "WheelDown" => Input::WheelDown,
            "Ctrl" => Input::Ctrl,
            "Shift" => Input::Shift,
            "Alt" => Input::Alt,
            _ => Input::Key(Key::from_name(name)?),
        };
        Some(input)
    }

    /// The input as the catalog names it, or as the key shows its symbol
    fn shown(self, catalog: &Catalog) -> String {
        let id = match self {
            Input::Key(Key::Space) => "key-space",
            Input::Key(Key::ArrowLeft) => "key-left",
            Input::Key(Key::ArrowRight) => "key-right",
            Input::Key(Key::ArrowUp) => "key-up",
            Input::Key(Key::ArrowDown) => "key-down",
            Input::Key(Key::Home) => "key-home",
            Input::Key(Key::End) => "key-end",
            Input::Key(Key::PageUp) => "key-page-up",
            Input::Key(Key::PageDown) => "key-page-down",
            Input::Key(Key::Enter) => "key-enter",
            Input::Key(Key::Backspace) => "key-backspace",
            Input::Key(Key::Delete) => "key-delete",
            Input::Key(Key::Insert) => "key-insert",
            Input::Key(Key::Tab) => "key-tab",
            Input::Key(key) => return key.symbol_or_name().to_owned(),
            Input::Mouse3 => "key-mouse-3",
            Input::Mouse4 => "key-mouse-4",
            Input::Mouse5 => "key-mouse-5",
            Input::WheelUp => "key-wheel-up",
            Input::WheelDown => "key-wheel-down",
            Input::Ctrl => "key-ctrl",
            Input::Shift => "key-shift",
            Input::Alt => "key-alt",
        };
        catalog.text(id)
    }

    /// Whether a focused control takes the key for itself, as a button
    /// takes Space and the time bar the arrows
    fn navigates(self) -> bool {
        matches!(
            self,
            Input::Key(
                Key::Space
                    | Key::Enter
                    | Key::ArrowLeft
                    | Key::ArrowRight
                    | Key::ArrowUp
                    | Key::ArrowDown
                    | Key::Home
                    | Key::End
            )
        )
    }
}

/// An input and the modifiers held with it
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Binding {
    pub input: Input,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Binding {
    pub fn plain(input: Input) -> Self {
        Binding {
            input,
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    fn ctrl(key: Key) -> Self {
        Binding {
            ctrl: true,
            ..Binding::plain(Input::Key(key))
        }
    }

    fn shift(key: Key) -> Self {
        Binding {
            shift: true,
            ..Binding::plain(Input::Key(key))
        }
    }

    /// `key` with the modifiers in `held`
    pub fn held(key: Key, held: Modifiers) -> Self {
        Binding {
            input: Input::Key(key),
            ctrl: held.command || held.ctrl,
            shift: held.shift,
            alt: held.alt,
        }
    }

    /// The modifiers' names before the input's, joined by +
    fn joined(self, input: String, ctrl: &str, shift: &str, alt: &str) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push(ctrl.to_owned());
        }
        if self.shift {
            parts.push(shift.to_owned());
        }
        if self.alt {
            parts.push(alt.to_owned());
        }
        parts.push(input);
        parts.join("+")
    }

    /// The binding as the settings file writes it, such as Ctrl+Shift+Left
    fn name(self) -> String {
        self.joined(self.input.name().to_owned(), "Ctrl", "Shift", "Alt")
    }

    fn from_name(name: &str) -> Option<Self> {
        let mut parts: Vec<&str> = name.split('+').collect();
        let input = Input::from_name(parts.pop()?)?;
        let mut binding = Binding::plain(input);
        for part in parts {
            match part {
                "Ctrl" => binding.ctrl = true,
                "Shift" => binding.shift = true,
                "Alt" => binding.alt = true,
                _ => return None,
            }
        }
        Some(binding)
    }

    /// The binding in the catalog's words
    pub fn shown(self, catalog: &Catalog) -> String {
        let (ctrl, shift, alt) = (
            catalog.text("key-ctrl"),
            catalog.text("key-shift"),
            catalog.text("key-alt"),
        );
        self.joined(self.input.shown(catalog), &ctrl, &shift, &alt)
    }

    /// Whether a press of `key` with `held` is this binding, exactly when
    /// `exact`, or else with its Shift and Alt among those held
    fn pressed_by(self, key: Key, held: Modifiers, exact: bool) -> bool {
        if self.input != Input::Key(key) || self.ctrl != (held.command || held.ctrl) {
            return false;
        }
        if exact {
            self.shift == held.shift && self.alt == held.alt
        } else {
            (!self.shift || held.shift) && (!self.alt || held.alt)
        }
    }
}

/// Every action's bindings
#[derive(Clone, Debug, PartialEq)]
pub struct Shortcuts {
    bindings: BTreeMap<Action, [Option<Binding>; 2]>,
}

impl Default for Shortcuts {
    fn default() -> Self {
        let bindings = Action::ALL.iter().map(|&a| (a, a.defaults())).collect();
        Shortcuts { bindings }
    }
}

impl Shortcuts {
    /// The action's bindings, the second none where it has one
    pub fn of(&self, action: Action) -> [Option<Binding>; 2] {
        self.bindings.get(&action).copied().unwrap_or_default()
    }

    /// The action's `slot` bound to `binding`, or to nothing. A binding
    /// another action had is taken from it, and that action is returned
    pub fn set(&mut self, action: Action, slot: usize, binding: Option<Binding>) -> Option<Action> {
        let mut taken_from = None;
        if let Some(binding) = binding {
            for (&other, slots) in &mut self.bindings {
                if other.flies() != action.flies() {
                    continue;
                }
                for held in slots.iter_mut() {
                    if *held == Some(binding) {
                        *held = None;
                        if other != action {
                            taken_from = Some(other);
                        }
                    }
                }
            }
        }

        let slots = self.bindings.entry(action).or_default();
        slots[slot] = binding;

        // The first slot holds the binding when there is only one
        if slots[0].is_none() {
            slots.swap(0, 1);
        }
        taken_from
    }

    /// The action's bindings in the catalog's words, none when it has none
    pub fn label(&self, action: Action, catalog: &Catalog) -> Option<String> {
        let shown: Vec<String> = self
            .of(action)
            .iter()
            .flatten()
            .map(|b| b.shown(catalog))
            .collect();
        if shown.is_empty() {
            None
        } else {
            Some(shown.join(", "))
        }
    }

    /// The action's bindings in the catalog's words, or its word for none
    pub fn keys(&self, action: Action, catalog: &Catalog) -> String {
        self.label(action, catalog)
            .unwrap_or_else(|| catalog.text("shortcut-none"))
    }

    /// `text` with a line under it naming the action's bindings, as the
    /// controls' tooltips have it
    pub fn with_keys(&self, text: &str, action: Action, catalog: &Catalog) -> String {
        match self.label(action, catalog) {
            Some(keys) => {
                let keys = FluentValue::from(keys);
                let line = catalog.text_with("viewer-shortcut", &[("key", keys)]);
                format!("{text}\n{line}")
            }
            None => text.to_owned(),
        }
    }

    /// The actions of `actions` pressed this frame, the keys and the wheel
    /// that ran them taken from the input. A binding `skip` turns down is
    /// left to whatever else wants its key
    pub fn take(
        &self,
        ctx: &egui::Context,
        actions: &[Action],
        skip: impl Fn(Binding) -> bool,
    ) -> Vec<Action> {
        let mut candidates = Vec::new();
        for &action in actions {
            for binding in self.of(action).into_iter().flatten() {
                if !skip(binding) {
                    candidates.push((action, binding));
                }
            }
        }

        let over_picture = over_picture(ctx);
        let mut pressed = Vec::new();
        ctx.input_mut(|i| {
            // The mouse's buttons belong to whichever action has them
            for (action, binding) in &candidates {
                let Some(button) = button(binding.input) else {
                    continue;
                };
                if i.pointer.button_pressed(button) {
                    pressed.push(*action);
                }
            }

            // The wheel runs its action once a frame however far it turns,
            // and then scrolls nothing else
            let turned = if over_picture { wheel(i) } else { None };
            let wheeled = candidates.iter().find(|(_, b)| Some(b.input) == turned);
            if let Some((action, _)) = wheeled {
                pressed.push(*action);
                i.events
                    .retain(|event| !matches!(event, Event::MouseWheel { .. }));
                i.smooth_scroll_delta = egui::Vec2::ZERO;
            }

            i.events.retain(|event| {
                let Event::Key {
                    key,
                    modifiers,
                    pressed: true,
                    ..
                } = event
                else {
                    return true;
                };
                let exact = candidates
                    .iter()
                    .find(|(_, b)| b.pressed_by(*key, *modifiers, true));
                let loose = || {
                    candidates
                        .iter()
                        .find(|(_, b)| b.pressed_by(*key, *modifiers, false))
                };
                match exact.or_else(loose) {
                    Some((action, _)) => {
                        pressed.push(*action);
                        false
                    }
                    None => true,
                }
            });
        });
        pressed
    }

    /// Whether any of the action's bindings is held in `input`
    pub fn held(&self, input: &InputState, action: Action) -> bool {
        self.of(action)
            .into_iter()
            .flatten()
            .any(|b| match b.input {
                Input::Key(key) => input.key_down(key),
                Input::Mouse3 | Input::Mouse4 | Input::Mouse5 => {
                    button(b.input).is_some_and(|button| input.pointer.button_down(button))
                }
                Input::WheelUp | Input::WheelDown => false,
                Input::Ctrl => input.modifiers.ctrl || input.modifiers.command,
                Input::Shift => input.modifiers.shift,
                Input::Alt => input.modifiers.alt,
            })
    }

    /// The inputs the free camera holds, which other actions leave to it
    /// while it flies
    pub fn flying_inputs(&self) -> Vec<Input> {
        let mut inputs = Vec::new();
        for (action, slots) in &self.bindings {
            if action.flies() {
                inputs.extend(slots.iter().flatten().map(|b| b.input));
            }
        }
        inputs
    }
}

/// Whether a focused control takes `binding`'s key for itself
pub fn navigates(binding: Binding) -> bool {
    binding.input.navigates()
}

/// The mouse button an input is, if it is one
fn button(input: Input) -> Option<PointerButton> {
    match input {
        Input::Mouse3 => Some(PointerButton::Middle),
        Input::Mouse4 => Some(PointerButton::Extra1),
        Input::Mouse5 => Some(PointerButton::Extra2),
        _ => None,
    }
}

/// Which way the wheel turned this frame, if it did
fn wheel(input: &InputState) -> Option<Input> {
    let mut turned = 0.;
    for event in &input.events {
        if let Event::MouseWheel { delta, .. } = event {
            turned += delta.y;
        }
    }
    if turned > 0. {
        Some(Input::WheelUp)
    } else if turned < 0. {
        Some(Input::WheelDown)
    } else {
        None
    }
}

/// Where the frame the demo's picture was last under the pointer is kept
fn picture_id() -> egui::Id {
    egui::Id::unique("shortcuts over picture")
}

/// Note that the demo's picture is under the pointer this frame, so the
/// wheel's bindings count over it the next, as shortcuts run before the
/// picture is drawn
pub fn under_pointer(ctx: &egui::Context) {
    let frame = ctx.cumulative_frame_nr();
    ctx.data_mut(|d| d.insert_temp(picture_id(), frame));
}

/// Whether the demo's picture was under the pointer last frame or this one
fn over_picture(ctx: &egui::Context) -> bool {
    let frame = ctx.cumulative_frame_nr();
    let last = ctx.data(|d| d.get_temp::<u64>(picture_id()));
    last.is_some_and(|last| last + 1 >= frame)
}

/// For the key of a modifier, the input it holds alone, none for the
/// Windows key, which binds nothing. None for any other key
fn modifier(key: Key) -> Option<Option<Input>> {
    let input = match key {
        Key::ControlLeft | Key::ControlRight => Input::Ctrl,
        Key::ShiftLeft | Key::ShiftRight => Input::Shift,
        Key::AltLeft | Key::AltRight => Input::Alt,
        Key::SuperLeft | Key::SuperRight => return Some(None),
        _ => return None,
    };
    Some(Some(input))
}

/// What choosing a binding took from this frame's input
pub enum Chosen {
    /// Escape, which gives up choosing
    Cancelled,
    Binding(Binding),
}

/// The binding pressed this frame for an action that `flies` or not, with
/// the modifiers `before` held last frame, every key taken from the input
/// so nothing else runs. A free camera's binding is a key without
/// modifiers, a mouse button or a modifier newly held, and any other's a
/// key with its modifiers, a mouse button, or the wheel turned while
/// `hovered`, the wheel then scrolling nothing else
pub fn choose(
    ctx: &egui::Context,
    flies: bool,
    before: Modifiers,
    hovered: bool,
) -> Option<Chosen> {
    ctx.input_mut(|i| {
        let mut chosen = None;
        i.events.retain(|event| {
            let Event::Key {
                key,
                modifiers,
                pressed,
                ..
            } = event
            else {
                return true;
            };
            if !*pressed || chosen.is_some() {
                return false;
            }

            // A modifier's own key waits for the key it is held with,
            // but is the free camera's to hold alone
            if let Some(held) = modifier(*key) {
                if flies {
                    chosen = held.map(|input| Chosen::Binding(Binding::plain(input)));
                }
            } else if *key == Key::Escape {
                chosen = Some(Chosen::Cancelled);
            } else if flies {
                chosen = Some(Chosen::Binding(Binding::plain(Input::Key(*key))));
            } else {
                chosen = Some(Chosen::Binding(Binding::held(*key, *modifiers)));
            }
            false
        });
        if chosen.is_some() {
            return chosen;
        }

        for input in [Input::Mouse3, Input::Mouse4, Input::Mouse5] {
            let pressed = button(input).is_some_and(|b| i.pointer.button_pressed(b));
            if pressed {
                return Some(Chosen::Binding(Binding::plain(input)));
            }
        }

        // The free camera's keys are held, which the wheel can't be
        if hovered
            && !flies
            && let Some(turned) = wheel(i)
        {
            i.events
                .retain(|event| !matches!(event, Event::MouseWheel { .. }));
            i.smooth_scroll_delta = egui::Vec2::ZERO;
            return Some(Chosen::Binding(Binding::plain(turned)));
        }

        // A modifier alone, as the free camera holds it
        let now = i.modifiers;
        if !flies {
            return None;
        }
        let ctrl = |m: Modifiers| m.ctrl || m.command;
        let input = if ctrl(now) && !ctrl(before) {
            Input::Ctrl
        } else if now.shift && !before.shift {
            Input::Shift
        } else if now.alt && !before.alt {
            Input::Alt
        } else {
            return None;
        };
        Some(Chosen::Binding(Binding::plain(input)))
    })
}

/// Written as each action's name and its bindings' names, an empty name
/// for a slot left empty
impl Serialize for Shortcuts {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = BTreeMap::new();
        for action in Action::ALL {
            let names: Vec<String> = self
                .of(action)
                .iter()
                .map(|b| b.map_or(String::new(), Binding::name))
                .collect();
            map.insert(action.name(), names);
        }
        map.serialize(serializer)
    }
}

/// Read over the defaults, so an action the file does not name keeps its
/// keys. An action or binding this version does not know is left out
impl<'de> Deserialize<'de> for Shortcuts {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let saved: BTreeMap<String, Vec<String>> =
            serde_json::from_value(value).unwrap_or_default();

        let mut shortcuts = Shortcuts::default();
        for action in Action::ALL {
            let Some(names) = saved.get(&action.name()) else {
                continue;
            };
            let mut slots = [None, None];
            for (slot, name) in slots.iter_mut().zip(names) {
                *slot = Binding::from_name(name);
            }
            shortcuts.bindings.insert(action, slots);
        }
        Ok(shortcuts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(key: Key, modifiers: Modifiers) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    /// The wheel turned by `y` points, up for more than none
    fn wheel(y: f32) -> Event {
        Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0., y),
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Unknown,
            modifiers: Modifiers::NONE,
        }
    }

    /// The middle button pressed in the middle of nowhere
    fn middle_click() -> Vec<Event> {
        let pos = egui::pos2(10., 10.);
        vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Middle,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ]
    }

    /// The actions `shortcuts` takes from a frame of `events`
    fn taken(shortcuts: &Shortcuts, events: Vec<Event>) -> Vec<Action> {
        taken_over(shortcuts, events, false)
    }

    /// The actions `shortcuts` takes from a frame of `events`, after a
    /// frame with the demo's picture under the pointer or not
    fn taken_over(shortcuts: &Shortcuts, events: Vec<Event>, picture: bool) -> Vec<Action> {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            if picture {
                under_pointer(ui.ctx());
            }
        });

        // Nothing is drawn, so the fonts egui made go unused
        output.textures_delta.clear();

        let input = egui::RawInput {
            events,
            ..Default::default()
        };
        let mut pressed = Vec::new();
        let mut output = ctx.run_ui(input, |ui| {
            pressed = shortcuts.take(ui.ctx(), &Action::ALL, |_| false);
        });
        output.textures_delta.clear();
        pressed
    }

    #[test]
    fn a_press_runs_the_binding_with_exactly_its_modifiers_first() {
        let s = Shortcuts::default();
        let left = taken(&s, vec![press(Key::ArrowLeft, Modifiers::NONE)]);
        assert_eq!(left, [Action::SeekBack]);
        let shifted = taken(&s, vec![press(Key::ArrowLeft, Modifiers::SHIFT)]);
        assert_eq!(shifted, [Action::SkipBack]);
        let ctrl = taken(&s, vec![press(Key::ArrowLeft, Modifiers::CTRL)]);
        assert_eq!(ctrl, [Action::PreviousEvent]);

        // A + typed with Shift is still the plain +
        let plus = taken(&s, vec![press(Key::Plus, Modifiers::SHIFT)]);
        assert_eq!(plus, [Action::Faster]);
    }

    #[test]
    fn the_wheel_and_the_middle_button_run_their_bindings() {
        let mut s = Shortcuts::default();
        let down = taken_over(&s, vec![wheel(-3.), wheel(-2.)], true);
        assert_eq!(down, [Action::NextPlayer]);
        let up = taken_over(&s, vec![wheel(1.)], true);
        assert_eq!(up, [Action::PreviousPlayer]);

        // Away from the picture the wheel only scrolls
        assert!(taken(&s, vec![wheel(1.)]).is_empty());

        assert!(taken(&s, middle_click()).is_empty());
        s.set(Action::Zoom, 1, Some(Binding::plain(Input::Mouse3)));
        assert_eq!(taken(&s, middle_click()), [Action::Zoom]);
    }

    /// What choosing a binding for an action that `flies` takes from a
    /// frame of `events` with `held` down, `before` held the frame before
    fn chosen(
        flies: bool,
        events: Vec<Event>,
        held: Modifiers,
        before: Modifiers,
    ) -> Option<Binding> {
        chosen_over(flies, events, held, before, false)
    }

    /// As [`chosen`], with the pointer over the key being chosen or not
    fn chosen_over(
        flies: bool,
        events: Vec<Event>,
        held: Modifiers,
        before: Modifiers,
        hovered: bool,
    ) -> Option<Binding> {
        let ctx = egui::Context::default();
        let mut all = vec![Event::ModifiersChanged(held)];
        all.extend(events);
        let input = egui::RawInput {
            events: all,
            ..Default::default()
        };
        let mut chosen = None;
        let mut output = ctx.run_ui(input, |ui| {
            chosen = Some(choose(ui.ctx(), flies, before, hovered));
        });
        output.textures_delta.clear();
        match chosen.flatten() {
            Some(Chosen::Binding(binding)) => Some(binding),
            _ => None,
        }
    }

    #[test]
    fn a_binding_is_chosen_with_its_modifiers_or_for_the_free_camera_without() {
        // egui sends the modifiers' own keys before the key held with them
        let shift_ctrl = Modifiers::SHIFT | Modifiers::CTRL;
        let pressed = vec![
            press(Key::ControlLeft, Modifiers::CTRL),
            press(Key::ShiftLeft, shift_ctrl),
            press(Key::G, shift_ctrl),
        ];
        let got = chosen(false, pressed.clone(), shift_ctrl, Modifiers::NONE);
        assert_eq!(got.map(Binding::name).as_deref(), Some("Ctrl+Shift+G"));
        let ctrl_alone = vec![press(Key::ControlLeft, Modifiers::CTRL)];
        let got = chosen(false, ctrl_alone.clone(), Modifiers::CTRL, Modifiers::NONE);
        assert_eq!(got, None);
        let got = chosen(true, ctrl_alone, Modifiers::CTRL, Modifiers::NONE);
        assert_eq!(got.map(Binding::name).as_deref(), Some("Ctrl"));
        let got = chosen(
            true,
            vec![press(Key::G, shift_ctrl)],
            shift_ctrl,
            shift_ctrl,
        );
        assert_eq!(got.map(Binding::name).as_deref(), Some("G"));

        // Escape gives up rather than binding
        let escape = vec![press(Key::Escape, Modifiers::NONE)];
        assert_eq!(
            chosen(false, escape, Modifiers::NONE, Modifiers::NONE),
            None
        );

        // A modifier newly held is the free camera's alone
        let got = chosen(true, Vec::new(), Modifiers::ALT, Modifiers::NONE);
        assert_eq!(got.map(Binding::name).as_deref(), Some("Alt"));
        assert_eq!(
            chosen(false, Vec::new(), Modifiers::ALT, Modifiers::NONE),
            None
        );
        assert_eq!(
            chosen(true, Vec::new(), Modifiers::ALT, Modifiers::ALT),
            None
        );
    }

    #[test]
    fn the_wheel_is_chosen_over_its_key_and_never_for_the_free_camera() {
        let none = Modifiers::NONE;
        let got = chosen_over(false, vec![wheel(-1.)], none, none, true);
        assert_eq!(got.map(Binding::name).as_deref(), Some("WheelDown"));
        assert_eq!(
            chosen_over(false, vec![wheel(-1.)], none, none, false),
            None
        );
        assert_eq!(chosen_over(true, vec![wheel(-1.)], none, none, true), None);

        let got = chosen(true, middle_click(), none, none);
        assert_eq!(got.map(Binding::name).as_deref(), Some("Mouse3"));
    }

    #[test]
    fn a_binding_moves_to_the_action_given_it() {
        let mut s = Shortcuts::default();
        let z = Binding::plain(Input::Key(Key::Z));
        assert_eq!(s.set(Action::Pause, 1, Some(z)), Some(Action::Zoom));
        assert_eq!(s.of(Action::Zoom), [None, None]);
        assert_eq!(s.of(Action::Pause)[1], Some(z));

        // The free camera's Space is its own
        let space = Binding::plain(Input::Key(Key::Space));
        assert_eq!(
            s.set(Action::Rewind, 0, Some(space)),
            Some(Action::PlayPause)
        );
        assert_eq!(s.of(Action::FlyUp)[0], Some(space));

        // A slot emptied leaves the other first
        s.set(Action::Faster, 0, None);
        assert_eq!(
            s.of(Action::Faster),
            [Some(Binding::plain(Input::Key(Key::Equals))), None]
        );
    }

    #[test]
    fn shortcuts_are_read_over_the_defaults() {
        let mut s = Shortcuts::default();
        s.set(Action::Zoom, 0, Some(Binding::shift(Key::Z)));
        s.set(Action::Names, 0, None);
        let text = serde_json::to_string(&s).unwrap();
        assert_eq!(serde_json::from_str::<Shortcuts>(&text).unwrap(), s);

        // Unknown actions and keys are left out, missing ones are the
        // defaults
        let read: Shortcuts =
            serde_json::from_str(r#"{"zoom": ["Ctrl+Nonsense"], "warp": ["W"]}"#).unwrap();
        assert_eq!(read.of(Action::Zoom), [None, None]);
        assert_eq!(read.of(Action::Names), Action::Names.defaults());
        let broken: Shortcuts = serde_json::from_str("42").unwrap();
        assert_eq!(broken, Shortcuts::default());
    }

    #[test]
    fn bindings_are_named_as_they_are_read() {
        let names = [
            "Ctrl+Shift+Left",
            "Mouse3",
            "Mouse4",
            "WheelUp",
            "Shift",
            "Alt+F11",
            "Plus",
        ];
        for name in names {
            let binding = Binding::from_name(name).unwrap();
            assert_eq!(binding.name(), name);
        }
        assert_eq!(Binding::from_name("Hyper+A"), None);
    }
}

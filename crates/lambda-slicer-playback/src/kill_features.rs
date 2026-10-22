//! Whether a Gauss kill went through a wall, reconstructed from the shot
//! the server fired (GUI redesign plan §10.1.5)
//!
//! The server's `CGauss::Fire` traces the beam from the eye along the aim.
//! A player in the way takes the beam's damage and the beam goes on through
//! them. The world met at a glancing angle, less than 60 degrees off its
//! plane, reflects the beam with a blast of the energy it loses. Met more
//! squarely, a charged beam tries once to punch through, which it does
//! when the wall is thinner than its energy. It loses that much, a blast at
//! the far side deals the energy left, and the beam goes on beyond. The
//! beam stops at 10 energy or after ten hits.
//!
//! A kill is through a wall when every way the shot could have hurt the
//! victim needs the punch, the exit blast or the beam beyond the wall, and
//! one of them surely reaches the victim. The victim is where the snapshot
//! of the shot's frame has them, which can be off from where the server
//! traced them by lag compensation, so a way counts as possible within
//! `POSITION_SLOP` of the victim and sure only well inside their box.
//! Players are traced as boxes, not the hitboxes the server traces, so a
//! way is sure only through a core of the box that a player's body fills
//! standing or crouched. The world is traced through its point hull, and
//! brush entities such as doors where they stand, turned as they are
//! turned, and they block a blast or a beam from the victim as the world
//! does. A beam meeting a brush entity, whose damage and reflection this
//! cannot tell, leaves the kill unknown, as does a victim out of view or a
//! map that is not there. Nothing here is a flag the server recorded

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use lambda_slicer_core::analysis::context::Kill;
use lambda_slicer_core::analysis::events::RecordedEvent;
use lambda_slicer_core::analysis::weapons::{Actor, Weapon, WeaponOccurrence};
use lambda_slicer_core::container::Limits;
use lambda_slicer_core::message::Message;

use crate::MapHulls;
use crate::events::{ClientEvent, angle_vectors, eye};
use crate::playback::Playback;
use crate::scene::{Camera, Entity};
use crate::time::SourceTime;
use crate::trace::Hull;

/// How far the server traces a beam, and the most things one shot meets
const BEAM_LENGTH: f32 = 8192.;
const MAX_HITS: usize = 10;

/// The energy at or under which the beam stops
const SPENT: f32 = 10.;

/// A hit whose angle off the plane's normal has a cosine under this
/// reflects
const REFLECT_BELOW: f32 = 0.5;

/// How far past a reflection the beam goes on from, and past where it
/// meets a wall it looks for the far side
const STEP_OFF: f32 = 8.;

/// A blast's radius for its damage, a reflection's and the exit's in
/// multiplayer, and how far the server lifts its centre
const REFLECT_RADIUS: f32 = 2.5;
const EXIT_RADIUS: f32 = 1.75;
const BLAST_LIFT: f32 = 1.;

/// A standing player's box about their origin, and the core about their
/// origin their body fills both standing and crouched, a crouched player's
/// box reaching 18 above and below it
const PLAYER_HALF: [f32; 3] = [16., 16., 36.];
const CORE_HALF: [f32; 3] = [8., 8., 16.];

/// How far the victim may have been from where the snapshot has them when
/// the server traced the shot
pub const POSITION_SLOP: f32 = 32.;

/// A shot's frame is the first group at or after its time, so close
const TIME_EPSILON: f64 = 1e-4;

/// How much server time before the shot's snapshot the one before may be
/// and still place a victim the shot's own snapshot left out
const PREVIOUS_SNAPSHOT: f64 = 0.1;

/// How long before its death message a Gauss kill's shot can be, as the
/// analysis links shots to kills
const SHOT_WINDOW: f32 = 0.3;

/// `SOLID_BSP`
const SOLID_BSP: i64 = 4;

/// One Gauss shot the server fired, as its event gives it
#[derive(Clone, Debug, PartialEq)]
pub struct FiredShot {
    pub segment: u32,
    /// Playback time of the frame that carried its event
    pub time: f64,
    pub shooter: u16,
    /// The shooter's eye and the direction of the aim
    pub source: [f32; 3],
    pub forward: [f32; 3],
    /// Its energy, the event's damage
    pub damage: f32,
    pub primary: bool,
}

impl FiredShot {
    /// The shot an event fired, none for an event that is not one, aimed as
    /// its shooter looked
    pub fn from_event(event: &RecordedEvent, max_clients: u8) -> Option<Self> {
        let time = SourceTime(f64::from(event.time));
        let client = ClientEvent::new(event, time, max_clients)?;

        // The gauss's stop event fires nothing
        let args = &client.args;
        if args.bparam2 != 0 {
            return None;
        }

        Some(Self {
            segment: event.segment,
            time: f64::from(event.time),
            shooter: client.entity,
            source: eye(&client),
            forward: angle_vectors(args.angles)[0],
            damage: args.fparam1,
            primary: args.bparam1 != 0,
        })
    }
}

/// A Gauss kill to classify, with every shot of the killer's that could
/// have made it
#[derive(Clone, Debug, PartialEq)]
pub struct GaussKill {
    pub victim: u16,
    pub shots: Vec<FiredShot>,
}

impl GaussKill {
    /// `kill` with the killer's Gauss shots that could have made it, none
    /// for a kill that is not a player's with the Gauss
    pub fn of(kill: &Kill, occurrences: &[WeaponOccurrence], max_clients: u8) -> Option<Self> {
        let killer = kill.killer.as_ref()?;
        if kill.weapon != Some(Weapon::Gauss) {
            return None;
        }

        let mut shots = Vec::new();
        for o in occurrences {
            let Actor::Player { identity, .. } = &o.actor else {
                continue;
            };
            let same = (identity.segment, identity.slot, identity.incarnation)
                == (killer.segment, killer.slot, killer.incarnation);
            let before = kill.server_time - o.server_time;
            let fits = o.weapon == Weapon::Gauss
                && o.shot
                && o.segment == kill.segment
                && (0. ..=SHOT_WINDOW).contains(&before);
            if same
                && fits
                && let Some(shot) = FiredShot::from_event(&o.event, max_clients)
            {
                shots.push(shot);
            }
        }

        Some(Self {
            victim: u16::from(kill.victim_entity),
            shots,
        })
    }
}

/// How a kill went through a wall
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallRoute {
    /// The blast at the wall's far side
    ExitBlast,
    /// The beam going on beyond the wall, or a reflection of it there
    Beyond,
}

/// What a kill through a wall was reconstructed from
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallDetails {
    pub route: WallRoute,
    /// How thick the wall was, in units
    pub thickness: f32,
    /// The playback time of the shot
    pub shot_time: f64,
}

/// How a kill that was not through a wall could have been made
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OtherRoute {
    /// The beam before any wall
    Direct,
    /// A reflection's blast before any wall
    Reflection,
}

/// Why a kill could not be told
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Doubt {
    /// No shot of the killer's was found for it
    NoShot,
    /// The map's world could not be read
    NoMap,
    /// The victim was out of the recording's view at the shot
    VictimUnseen,
    /// The beam met a brush entity
    BrushEntity,
    /// The shot could have killed both through a wall and otherwise
    Ambiguous,
    /// No way the shot could have hurt the victim was found
    Unexplained,
    /// Only through the wall could the shot have hurt the victim, but it
    /// did not surely reach them
    Unconfirmed,
    /// The recording stopped before the shot
    NotReached,
}

/// Whether a Gauss kill went through a wall, as far as it can be
/// reconstructed
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WallGauss {
    Yes(WallDetails),
    No(OtherRoute),
    Unknown(Doubt),
}

/// A stretch of the beam with the energy it carried
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Beam {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub damage: f32,
    /// It came out of a wall the beam punched through
    pub beyond_wall: bool,
}

/// A blast the beam set off
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blast {
    pub at: [f32; 3],
    pub damage: f32,
    pub radius: f32,
    /// At a wall's far side, or a reflection beyond it
    pub beyond_wall: bool,
    /// The blast at the far side of the wall itself
    pub exit: bool,
}

/// Everything a shot's beam did through the world
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShotPath {
    pub beams: Vec<Beam>,
    pub blasts: Vec<Blast>,
    /// The wall it went through
    pub thickness: Option<f32>,
    /// It met a brush entity, where this stops following it
    pub brush_entity: bool,
}

/// A brush entity's model, where it is and how it is turned, for telling
/// when the beam meets one
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrushEntity {
    pub model: usize,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
}

fn add(a: [f32; 3], b: [f32; 3], scale: f32) -> [f32; 3] {
    [0, 1, 2].map(|k| a[k] + b[k] * scale)
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = add(a, b, -1.);
    dot(d, d).sqrt()
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let length = dot(v, v).sqrt();
    if length == 0. {
        return v;
    }

    v.map(|c| c / length)
}

/// `point` in the space of a brush entity at `origin` turned by `angles`,
/// as the engine turns a line into a turned model's space
/// (`SV_ClipMoveToEntity`)
fn model_space(point: [f32; 3], origin: [f32; 3], angles: [f32; 3]) -> [f32; 3] {
    let offset = add(point, origin, -1.);
    if angles == [0.; 3] {
        return offset;
    }

    let [forward, right, up] = angle_vectors(angles);
    [dot(offset, forward), -dot(offset, right), dot(offset, up)]
}

/// Whether the line from `start` to `end` meets a brush entity before
/// `fraction` of the way
fn meets_brush(
    world: &Hull,
    brushes: &[BrushEntity],
    start: [f32; 3],
    end: [f32; 3],
    fraction: f32,
) -> bool {
    for brush in brushes {
        let local = |p: [f32; 3]| model_space(p, brush.origin, brush.angles);
        let trace = world.trace_model(brush.model, local(start), local(end));
        if trace.fraction < fraction {
            return true;
        }
    }

    false
}

/// Follow `shot`'s beam through the world as the server does, with the
/// blasts it sets off. Players do not turn the beam, so they are left out
pub fn trace_path(shot: &FiredShot, world: &Hull, brushes: &[BrushEntity]) -> ShotPath {
    let mut path = ShotPath::default();
    let mut damage = shot.damage;
    let mut forward = normalise(shot.forward);
    let mut source = shot.source;
    let mut dest = add(source, forward, BEAM_LENGTH);
    let mut punched = false;
    let mut beyond_wall = false;

    for _ in 0..MAX_HITS {
        if damage <= SPENT {
            break;
        }

        let met = world.trace(source, dest);
        if met.all_solid {
            break;
        }
        if meets_brush(world, brushes, source, dest, met.fraction) {
            path.brush_entity = true;
            break;
        }

        path.beams.push(Beam {
            start: source,
            end: met.end,
            damage,
            beyond_wall,
        });
        if met.fraction >= 1. {
            break;
        }

        // Glancing, so it reflects with a blast of the energy it loses
        let n = -dot(met.normal, forward);
        if n < REFLECT_BELOW {
            let blast = damage * n;
            path.blasts.push(Blast {
                at: met.end,
                damage: blast,
                radius: blast * REFLECT_RADIUS,
                beyond_wall,
                exit: false,
            });

            forward = add(forward, met.normal, 2. * n);
            source = add(met.end, forward, STEP_OFF);
            dest = add(source, forward, BEAM_LENGTH);
            let n = if n == 0. { 0.1 } else { n };
            damage *= 1. - n;
            continue;
        }

        // Square on, so one charged shot tries to punch through
        if punched || shot.primary {
            break;
        }
        punched = true;

        let ahead = world.trace(add(met.end, forward, STEP_OFF), dest);
        if ahead.all_solid {
            break;
        }
        let exit = world.trace(ahead.end, met.end).end;
        let thickness = distance(exit, met.end);
        if thickness >= damage {
            break;
        }

        damage -= thickness.max(1.);
        path.thickness = Some(thickness);
        path.blasts.push(Blast {
            at: add(exit, forward, STEP_OFF),
            damage,
            radius: damage * EXIT_RADIUS,
            beyond_wall: true,
            exit: true,
        });
        beyond_wall = true;
        source = add(exit, forward, 1.);
    }

    path
}

/// Whether the line from `start` to `end` passes through the box from
/// `min` to `max`
fn crosses_box(start: [f32; 3], end: [f32; 3], min: [f32; 3], max: [f32; 3]) -> bool {
    let (mut near, mut far) = (0f32, 1f32);
    for k in 0..3 {
        let d = end[k] - start[k];
        if d.abs() < 1e-9 {
            if start[k] < min[k] || start[k] > max[k] {
                return false;
            }
            continue;
        }

        let (a, b) = ((min[k] - start[k]) / d, (max[k] - start[k]) / d);
        near = near.max(a.min(b));
        far = far.min(a.max(b));
        if near > far {
            return false;
        }
    }

    true
}

/// The point of the line from `start` to `end` nearest `point`
fn nearest_on(start: [f32; 3], end: [f32; 3], point: [f32; 3]) -> [f32; 3] {
    let along = add(end, start, -1.);
    let length = dot(along, along);
    if length == 0. {
        return start;
    }

    let share = (dot(add(point, start, -1.), along) / length).clamp(0., 1.);
    add(start, along, share)
}

/// The box `half` wide about `centre`, grown by `slop` on every side
fn about(centre: [f32; 3], half: [f32; 3], slop: f32) -> ([f32; 3], [f32; 3]) {
    let min = [0, 1, 2].map(|k| centre[k] - half[k] - slop);
    let max = [0, 1, 2].map(|k| centre[k] + half[k] + slop);
    (min, max)
}

/// Whether nothing of the world or its brush entities stands between two
/// points
fn in_sight(world: &Hull, brushes: &[BrushEntity], from: [f32; 3], to: [f32; 3]) -> bool {
    world.trace(from, to).fraction >= 1. && !meets_brush(world, brushes, from, to, 1.)
}

/// What one shot could have done to a victim at `centre`
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reach {
    /// It could have hurt them without the wall, and how
    pub other: Option<OtherRoute>,
    /// It could have hurt them through the wall, and how
    pub wall: Option<WallRoute>,
    /// It surely hurt them through the wall
    pub wall_sure: bool,
}

/// What `path` could have done to a victim whose origin the snapshot has
/// at `centre`, with `brushes` standing where they were
pub fn reach(path: &ShotPath, world: &Hull, brushes: &[BrushEntity], centre: [f32; 3]) -> Reach {
    let mut out = Reach::default();
    let (near_min, near_max) = about(centre, PLAYER_HALF, POSITION_SLOP);
    let (core_min, core_max) = about(centre, CORE_HALF, 0.);

    for beam in &path.beams {
        if !crosses_box(beam.start, beam.end, near_min, near_max) {
            continue;
        }

        // Moving the victim within the slop never takes them through a
        // wall, so a beam beyond one is out of reach
        let nearest = nearest_on(beam.start, beam.end, centre);
        if !in_sight(world, brushes, centre, nearest) {
            continue;
        }

        if beam.beyond_wall {
            out.wall = Some(WallRoute::Beyond);
            out.wall_sure |= crosses_box(beam.start, beam.end, core_min, core_max);
        } else {
            out.other = Some(OtherRoute::Direct);
        }
    }

    for blast in &path.blasts {
        let at = add(blast.at, [0., 0., 1.], BLAST_LIFT);
        let away = distance(at, centre);
        if away > blast.radius + POSITION_SLOP || !in_sight(world, brushes, at, centre) {
            continue;
        }

        let sure = away + POSITION_SLOP < blast.radius;
        match (blast.beyond_wall, blast.exit) {
            (true, true) => {
                out.wall.get_or_insert(WallRoute::ExitBlast);
                out.wall_sure |= sure;
            }
            (true, false) => {
                out.wall.get_or_insert(WallRoute::Beyond);
                out.wall_sure |= sure;
            }
            (false, _) => {
                out.other.get_or_insert(OtherRoute::Reflection);
            }
        }
    }

    out
}

/// What one shot of a kill came to, once its frame was reached
#[derive(Clone, Debug, PartialEq)]
pub enum ShotFinding {
    Reached { reach: Reach, path: ShotPath },
    Doubt(Doubt),
}

/// A kill's verdict from what each of its shots came to. It went through a
/// wall when only through a wall could any shot have hurt the victim and
/// one surely did, and not through a wall when no shot could have hurt them
/// through one but one could otherwise
pub fn decide(findings: &[(f64, ShotFinding)]) -> WallGauss {
    if findings.is_empty() {
        return WallGauss::Unknown(Doubt::NoShot);
    }

    let mut wall: Option<WallDetails> = None;
    let mut wall_possible = false;
    let mut other: Option<OtherRoute> = None;
    for (time, finding) in findings {
        let (reach, path) = match finding {
            ShotFinding::Reached { reach, path } => (reach, path),
            ShotFinding::Doubt(doubt) => return WallGauss::Unknown(*doubt),
        };

        // A beam that met a brush entity may have done anything beyond it
        if path.brush_entity {
            return WallGauss::Unknown(Doubt::BrushEntity);
        }

        other = other.or(reach.other);
        wall_possible |= reach.wall.is_some();
        if reach.wall_sure
            && let Some(route) = reach.wall
        {
            wall = Some(WallDetails {
                route,
                thickness: path.thickness.unwrap_or(0.),
                shot_time: *time,
            });
        }
    }

    if wall_possible && other.is_some() {
        return WallGauss::Unknown(Doubt::Ambiguous);
    }
    if let Some(details) = wall {
        return WallGauss::Yes(details);
    }
    if wall_possible {
        return WallGauss::Unknown(Doubt::Unconfirmed);
    }

    match other {
        Some(route) => WallGauss::No(route),
        None => WallGauss::Unknown(Doubt::Unexplained),
    }
}

/// The world's brush entities in a snapshot, which beams are not followed
/// through
fn brush_entities(entities: &[Entity]) -> Vec<BrushEntity> {
    let mut brushes = Vec::new();
    for entity in entities {
        if entity.solid != SOLID_BSP {
            continue;
        }

        let model = entity.model.as_deref().and_then(|m| m.strip_prefix('*'));
        if let Some(model) = model.and_then(|m| m.parse().ok()) {
            brushes.push(BrushEntity {
                model,
                origin: entity.origin,
                angles: entity.angles,
            });
        }
    }
    brushes
}

/// Where the victim's origin was in the snapshot of a shot's frame, none
/// when they were out of view. The recording player is where the frame
/// recorded their own origin, standing or crouched, as the snapshot leaves
/// their entity unplaced
fn victim_origin(victim: u16, entities: &[Entity], camera: &Camera) -> Option<[f32; 3]> {
    if camera.view_entity == Some(victim) && !camera.spectating {
        return Some(camera.player_origin);
    }

    // A victim the snapshot left out, or kept from an older one, was out
    // of view
    let latest = entities
        .iter()
        .map(|e| e.updated.0)
        .fold(f32::MIN, f32::max);
    let seen = entities
        .iter()
        .find(|e| e.id.number == victim && e.updated.0 == latest);
    seen.map(|e| e.origin)
}

/// What one shot comes to in the snapshot of its frame, or in the snapshot
/// `before` it for a victim gone from its own
fn find(
    shot: &FiredShot,
    victim: u16,
    entities: &[Entity],
    before: &[Entity],
    camera: &Camera,
    world: Option<&Hull>,
) -> ShotFinding {
    let Some(world) = world else {
        return ShotFinding::Doubt(Doubt::NoMap);
    };

    // A victim the shot killed outright, gibbed, can be gone from the
    // snapshot of its frame, while the one before still has them where
    // the server traced them
    let seen =
        victim_origin(victim, entities, camera).or_else(|| victim_origin(victim, before, camera));
    let Some(victim) = seen else {
        return ShotFinding::Doubt(Doubt::VictimUnseen);
    };

    let brushes = brush_entities(entities);
    let path = trace_path(shot, world, &brushes);
    let reach = reach(&path, world, &brushes, victim);
    ShotFinding::Reached { reach, path }
}

/// How far the classification of a demo's kills has got
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    pub examined: usize,
    pub total: usize,
}

/// Classify `kills` by playing the demo at `path` through once, in the
/// order of its records, so later sessions that reuse times are told
/// apart. `hulls` gives each map's world. `report` hears each kill's
/// verdict as it is decided, by its place in `kills`. Stops between groups
/// once `cancel` is set
pub fn classify(
    path: &Path,
    hulls: MapHulls,
    kills: &[GaussKill],
    cancel: &AtomicBool,
    mut report: impl FnMut(usize, WallGauss, Progress),
) -> Result<(), Message> {
    let total = kills.len();
    let mut findings: Vec<Vec<(f64, ShotFinding)>> = vec![Vec::new(); total];
    let mut left: Vec<usize> = kills.iter().map(|k| k.shots.len()).collect();
    let mut examined = 0;

    // Kills without a shot are decided at once
    for (index, kill) in kills.iter().enumerate() {
        if kill.shots.is_empty() {
            examined += 1;
            report(
                index,
                WallGauss::Unknown(Doubt::NoShot),
                Progress { examined, total },
            );
        }
    }

    // Every shot in the order the recording reaches it
    let mut shots: Vec<(usize, &FiredShot)> = Vec::new();
    for (index, kill) in kills.iter().enumerate() {
        for shot in &kill.shots {
            shots.push((index, shot));
        }
    }
    shots.sort_by(|a, b| {
        (a.1.segment, a.1.time)
            .partial_cmp(&(b.1.segment, b.1.time))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut playback = Playback::open(path, Limits::default())?;
    playback.set_map_hulls(hulls);

    let mut next = 0;
    while next < shots.len() {
        if cancel.load(Ordering::Relaxed) {
            return Ok(());
        }

        // On to the shot's frame. A session that ended before the shot's
        // time has no frame for it
        let (index, shot) = shots[next];
        let position = playback.position();
        let passed = position.is_some_and(|p| p.segment > shot.segment);
        let reached = position
            .is_some_and(|p| p.segment == shot.segment && p.time.0 + TIME_EPSILON >= shot.time);
        if !passed && !reached && playback.step() {
            continue;
        }

        let finding = if reached {
            let world = playback.map_world();
            let scene = playback.scene();
            let victim = kills[index].victim;
            let before = match scene.previous_snapshot() {
                Some((gap, entities)) if (0. ..=PREVIOUS_SNAPSHOT).contains(&gap) => entities,
                _ => &[],
            };
            find(
                shot,
                victim,
                &scene.entities,
                before,
                &scene.camera,
                world.as_deref(),
            )
        } else {
            ShotFinding::Doubt(Doubt::NotReached)
        };

        findings[index].push((shot.time, finding));
        left[index] -= 1;
        if left[index] == 0 {
            examined += 1;
            let verdict = decide(&findings[index]);
            report(index, verdict, Progress { examined, total });
        }
        next += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::EntityId;
    use crate::trace::{Child, SOLID};

    /// Open space with a wall `thickness` thick from x = 100, and a floor
    /// far below
    fn wall(thickness: f32) -> Hull {
        Hull {
            planes: vec![
                ([1., 0., 0.], 100.),
                ([1., 0., 0.], 100. + thickness),
                ([0., 0., 1.], -500.),
            ],
            nodes: vec![
                (0, [Child::Node(1), Child::Node(2)]),
                (1, [Child::Node(2), Child::Contents(SOLID)]),
                (2, [Child::Contents(-1), Child::Contents(SOLID)]),
            ],
            models: vec![Child::Node(0)],
            node_faces: Vec::new(),
            faces: Vec::new(),
        }
    }

    /// A charged shot of `damage` along x from the origin, or a primary one
    fn shot(damage: f32, primary: bool) -> FiredShot {
        FiredShot {
            segment: 0,
            time: 1.,
            shooter: 1,
            source: [0., 0., 0.],
            forward: [1., 0., 0.],
            damage,
            primary,
        }
    }

    fn reached(path: ShotPath, world: &Hull, victim: [f32; 3]) -> (f64, ShotFinding) {
        let reach = reach(&path, world, &[], victim);
        (1., ShotFinding::Reached { reach, path })
    }

    #[test]
    fn a_charged_beam_punches_through_a_thin_wall_with_a_blast_beyond() {
        let world = wall(16.);
        let path = trace_path(&shot(200., false), &world, &[]);

        assert_eq!(path.beams.len(), 2);
        assert!(!path.beams[0].beyond_wall && path.beams[1].beyond_wall);
        let thickness = path.thickness.unwrap();
        assert!((thickness - 16.).abs() < 0.5, "{thickness}");

        // The blast deals what is left once the wall is paid for
        let exit = path.blasts.iter().find(|b| b.exit).unwrap();
        assert!((exit.damage - (200. - thickness)).abs() < 0.01);
        assert!((exit.radius - exit.damage * EXIT_RADIUS).abs() < 0.01);
        assert!((exit.at[0] - (116. + STEP_OFF)).abs() < 0.5);
    }

    #[test]
    fn primary_shots_and_thick_walls_stop_the_beam() {
        let world = wall(16.);
        let primary = trace_path(&shot(20., true), &world, &[]);
        assert_eq!(primary.beams.len(), 1);
        assert_eq!(primary.thickness, None);

        // A wall thicker than the beam's energy holds it
        let thick = wall(120.);
        let weak = trace_path(&shot(100., false), &thick, &[]);
        assert_eq!(weak.thickness, None);
        assert!(weak.blasts.is_empty());
    }

    #[test]
    fn a_victim_behind_the_wall_is_killed_through_it() {
        let world = wall(16.);
        let path = trace_path(&shot(200., false), &world, &[]);

        // Hit by the beam beyond the wall
        let behind = reached(path.clone(), &world, [300., 0., 0.]);
        let WallGauss::Yes(details) = decide(&[behind]) else {
            panic!("not through the wall");
        };
        assert_eq!(details.route, WallRoute::Beyond);

        // Off the beam's line but in the exit blast
        let beside = reached(path, &world, [160., 80., 0.]);
        let WallGauss::Yes(details) = decide(&[beside]) else {
            panic!("not through the wall");
        };
        assert_eq!(details.route, WallRoute::ExitBlast);
    }

    #[test]
    fn a_direct_hit_before_the_wall_is_not_through_it() {
        let world = wall(16.);
        let path = trace_path(&shot(200., false), &world, &[]);

        // In the open before the wall, and far from the far side's blast
        let before = reached(path, &world, [-200., 0., 0.]);
        let ahead = reached(
            trace_path(&shot(200., false), &world, &[]),
            &world,
            [40., 0., 0.],
        );
        assert_eq!(decide(&[ahead]), WallGauss::No(OtherRoute::Direct));

        // Behind the shooter nothing reaches, which is not a guess either
        assert_eq!(decide(&[before]), WallGauss::Unknown(Doubt::Unexplained));
    }

    #[test]
    fn a_victim_hugging_the_wall_on_either_side_is_ambiguous() {
        let world = wall(16.);
        let path = trace_path(&shot(200., false), &world, &[]);

        // Right before the wall, the beam hits them and the exit blast
        // reaches round the wall's edge only through solid, so it is direct
        let near_side = reached(path.clone(), &world, [80., 0., 0.]);
        assert_eq!(decide(&[near_side]), WallGauss::No(OtherRoute::Direct));

        // Another shot in the same moment with a clear line to a victim
        // behind the wall leaves that victim in doubt
        let through = reached(path, &world, [300., 0., 0.]);
        let across = FiredShot {
            source: [300., -300., 0.],
            forward: [0., 1., 0.],
            ..shot(20., true)
        };
        let direct = reached(trace_path(&across, &world, &[]), &world, [300., 0., 0.]);
        assert!(matches!(
            decide(std::slice::from_ref(&through)),
            WallGauss::Yes(_)
        ));
        assert_eq!(
            decide(&[through, direct]),
            WallGauss::Unknown(Doubt::Ambiguous)
        );
    }

    #[test]
    fn a_reflection_only_kill_is_not_through_a_wall() {
        // The floor at z = -500 met at a glancing angle reflects the beam
        let world = Hull {
            planes: vec![([0., 0., 1.], -500.)],
            nodes: vec![(0, [Child::Contents(-1), Child::Contents(SOLID)])],
            models: vec![Child::Node(0)],
            node_faces: Vec::new(),
            faces: Vec::new(),
        };
        let down = FiredShot {
            forward: normalise([1., 0., -0.3]),
            ..shot(200., false)
        };
        let path = trace_path(&down, &world, &[]);
        assert!(!path.blasts.is_empty());
        assert!(path.blasts.iter().all(|b| !b.beyond_wall));

        // Next to where it met the floor
        let bounce = path.blasts[0].at;
        let victim = [bounce[0] + 20., bounce[1], bounce[2] + 40.];
        let verdict = decide(&[reached(path, &world, victim)]);
        assert!(matches!(verdict, WallGauss::No(_)), "{verdict:?}");
    }

    #[test]
    fn a_missing_map_a_hidden_victim_or_no_shot_stay_unknown() {
        assert_eq!(decide(&[]), WallGauss::Unknown(Doubt::NoShot));

        let entities: Vec<Entity> = Vec::new();
        let camera = Camera::default();
        let finding = find(&shot(200., false), 2, &entities, &[], &camera, None);
        assert_eq!(finding, ShotFinding::Doubt(Doubt::NoMap));

        let world = wall(16.);
        let finding = find(&shot(200., false), 2, &entities, &[], &camera, Some(&world));
        assert_eq!(finding, ShotFinding::Doubt(Doubt::VictimUnseen));
        assert_eq!(
            decide(&[(1., finding)]),
            WallGauss::Unknown(Doubt::VictimUnseen)
        );
    }

    #[test]
    fn a_victim_gone_from_the_shots_snapshot_is_placed_by_the_one_before() {
        let world = wall(16.);
        let camera = Camera::default();
        let behind = Entity {
            id: EntityId {
                number: 2,
                ..EntityId::default()
            },
            origin: [300., 0., 0.],
            ..Entity::default()
        };

        // Killed outright, so only the snapshot before has them
        let shot = shot(200., false);
        let finding = find(&shot, 2, &[], &[behind], &camera, Some(&world));
        let ShotFinding::Reached { reach, .. } = finding else {
            panic!("{finding:?}");
        };
        assert!(reach.wall_sure);
    }

    #[test]
    fn a_beam_meeting_a_brush_entity_is_not_followed() {
        // A brush model the size of the wall, standing in front of it
        let world = wall(16.);
        let door = BrushEntity {
            model: 0,
            origin: [-50., 0., 0.],
            angles: [0.; 3],
        };
        let path = trace_path(&shot(200., false), &world, &[door]);
        assert!(path.brush_entity);

        let finding = reached(path, &world, [300., 0., 0.]);
        assert_eq!(decide(&[finding]), WallGauss::Unknown(Doubt::BrushEntity));
    }

    #[test]
    fn a_turned_brush_entity_is_met_where_it_is_turned_to() {
        // The wall's model turned half round about the origin stands from
        // x = -116 to -100, behind the shot
        let world = wall(16.);
        let turned = BrushEntity {
            model: 0,
            origin: [0.; 3],
            angles: [0., 180., 0.],
        };
        assert!(!trace_path(&shot(200., false), &world, &[turned]).brush_entity);

        let back = FiredShot {
            forward: [-1., 0., 0.],
            ..shot(200., false)
        };
        assert!(trace_path(&back, &world, &[turned]).brush_entity);
    }

    #[test]
    fn a_brush_entity_between_the_exit_blast_and_the_victim_blocks_it() {
        let world = wall(16.);
        let path = trace_path(&shot(200., false), &world, &[]);
        let victim = [160., 80., 0.];
        assert!(reach(&path, &world, &[], victim).wall_sure);

        // The wall's model turned a quarter round, standing across from
        // y = 40 to 56 between the blast and them
        let door = BrushEntity {
            model: 0,
            origin: [0., -60., 0.],
            angles: [0., 90., 0.],
        };
        let blocked = reach(&path, &world, &[door], victim);
        assert_eq!(blocked.wall, None);
    }

    #[test]
    fn the_sure_core_fits_a_crouched_victim() {
        // A beam beyond the wall passing 20 over a victim's origin could
        // miss them crouched, so it is possible but not sure. They stand
        // well out of the exit blast
        let world = wall(16.);
        let path = trace_path(&shot(200., false), &world, &[]);
        let reach = reach(&path, &world, &[], [700., 0., -20.]);
        assert_eq!(reach.wall, Some(WallRoute::Beyond));
        assert!(!reach.wall_sure);
    }

    #[test]
    fn one_shot_recorded_twice_or_killing_two_is_told_per_victim() {
        let world = wall(16.);
        let path = trace_path(&shot(200., false), &world, &[]);

        // The same shot recorded twice tells the same
        let behind = reached(path.clone(), &world, [300., 0., 0.]);
        assert!(matches!(
            decide(&[behind.clone(), behind]),
            WallGauss::Yes(_)
        ));

        // Through a player before the wall and on through the wall to
        // another, only the second is through the wall
        let front = reached(path.clone(), &world, [40., 0., 0.]);
        assert_eq!(decide(&[front]), WallGauss::No(OtherRoute::Direct));
        let back = reached(path, &world, [300., 0., 0.]);
        assert!(matches!(decide(&[back]), WallGauss::Yes(_)));
    }
}

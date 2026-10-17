# Weapon analysis profile `hl-ag-1`

`lambda-slicer analyze DEMO [--json FILE] [--occurrences]` reports weapon actions, charge and beam
intervals and actors (`analysis` module, implementation plan §5). The rules form a versioned
profile, named in every report, so a change to them changes the version.

## Evidence

Every event the recording holds is kept as a `RecordedEvent`:

- **source:** a standalone demo `Event` record (the recording client's own predicted event), an
  `svc_event` or an `svc_event_reliable`;
- the event script path, and for demo records the outer dispatch flags, separate from the
  argument flags;
- the complete arguments (entity, origin, angles, ducking, `fparam1/2`, `iparam1/2`,
  `bparam1/2`; velocity and args flags only for demo records, since network events do not send
  them);
- playback time, server time and delay;
- the actor resolution (explicit `entindex`, position in the packet's complete entity array,
  or unresolved) and the player occupying that slot when the event arrived.

Each weapon action (`WeaponOccurrence`) has exactly one recorded event behind it, embedded in the
report. Nothing is merged across records.

## Actions

Matching compares event script names without case or separator differences. The recorded bytes
stay in the evidence.

| Script | Weapon | Action | Shot |
|---|---|---|---|
| `crowbar.sc` | crowbar | swing, not a confirmed hit | no |
| `glock1.sc`, `glock2.sc` | glock | fire (primary, rapid) | yes |
| `python.sc` | python | fire | yes |
| `mp5.sc`, `mp52.sc` | mp5 | fire (bullets), launch (grenade) | yes |
| `crossbow1.sc`, `crossbow2.sc` | crossbow | fire (unzoomed, zoomed) | yes |
| `shotgun1.sc`, `shotgun2.sc` | shotgun | fire (single, double) | yes |
| `rpg.sc` | rpg | launch | yes |
| `gauss.sc`, `bparam2 != 0` | gauss | charge stop | no |
| `gauss.sc`, `bparam2 = 0` | gauss | fire (primary when `bparam1 != 0`, else charged) | yes |
| `gaussspin.sc` | gauss | charge activity (intervals) | no |
| `egon_fire.sc` | egon | beam start (`bparam1 != 0`), sustain (intervals) | no |
| `egon_stop.sc` | egon | beam stop | no |
| `firehornet.sc` | hivehand | fire | yes |
| `tripfire.sc` | tripmine | placement attempt, sent before the server checks it | no |
| `snarkfire.sc` | snark | release attempt, sent before the server checks it | no |

The gauss rows follow the server (OpenAG `dlls/gauss.cpp`). A shot sends `gauss.sc` with
`bparam2 = 0` and the damage parameter in `fparam1`, which is not the damage dealt. Every shot is
followed 10 ms later by a reliable stop with `bparam2 = 1`, which is also sent on holster.
Counting every `gauss.sc` record would therefore roughly double the shots.

Handgrenade and satchel have no weapon event script. They appear only as the recorder's weapon and
in kills.

## Intervals

Gauss charging and egon beams become intervals per weapon, session and actor:

- a `gaussspin` with `bparam1 = 0` or an egon beam start opens one with a known start, and a
  later update without an open interval opens one with an unseen start;
- updates extend it, and a gauss shot or stop or an egon stop ends it;
- updates more than 1 s apart, or the end of the recording, end it with an unseen end.

## Kills

`DeathMsg` is killer entity, victim entity and a weapon string (OpenAG
`dlls/multiplay_gamerules.cpp`). The killer slot holds a player only when a player killed;
otherwise it is 0. The string is the killer's active weapon, or the inflictor's class name without
its `weapon_`, `monster_` or `func_` prefix, or `world`, or `teammate`. A player operating a mounted
gun is the killer, with the gun's name. Singleplayer sends no death messages at all.

Each kill keeps the killer and victim occupants when the message arrived, the raw string, and a
cause:

| Cause | Rule |
|---|---|
| player weapon | killer is another player, and the name is a player weapon or its projectile (below) |
| player, other | killer is another player, with another name. `grenade` is the mp5 grenade and the hand grenade alike |
| mounted gun | killer is a player, name `tank`, `tanklaser`, `tankrocket` or `tankmortar` (`func_tank`) |
| suicide | killer is the victim, keeping the weapon when the name is one |
| teammate | name `teammate`, which the server sends instead of the weapon |
| world | killer 0, `world` or `worldspawn` (falls and other world damage) |
| environment | killer 0, a `func_` class (doors, platforms, trains, rotating parts, unmanned guns, chargers and so on), `mortar`, or any `trigger_` or `env_` name |
| monster | killer 0, a `monster_` class (grunts, turrets, sentries, aliens and so on), or `hornet` or `snark` with no player owning them |
| unknown | anything else |

Class names come from OpenAG `dlls`. Player weapon names:

| String | Weapon |
|---|---|
| `crowbar`, `9mmhandgun`, `357`, `9mmAR`, `shotgun`, `gauss`, `egon`, `handgrenade`, `tripmine`, `satchel`, `snark` | the weapon of that name |
| `crossbow` or `bolt`, `rpg` or `rpg_rocket`, `hornetgun` or `hornet` | crossbow, rpg, hivehand |
| `tau_cannon`, `gluon gun` | gauss, egon (renamed in later versions) |

A kill is not a shot, and its time is when the death was announced, not when the weapon was used.
The death feed goes to every client, so it covers all players, unlike their events. Kills per
weapon count only kills of other players with a player weapon.

**Survey of the 2,000-demo sample (2026-10-16):** player kills by weapon names make up most of the
feed (`gauss` 8,651, `crossbow` 3,690, `shotgun` 2,461, `9mmAR` 2,311 and so on). Other causes seen:
`grenade` by players (1,074), suicides (`grenade` 123, `rpg_rocket` 72, `gauss` 34, `world` 21 and
others), `teammate` (177), `worldspawn` (73) and `trigger_hurt` (62) with no killer, `tank`
operated by players (8), `env_laser`, `tracktrain`, `train` and `rotating` with no killer, one
`houndeye`, and one `snark` with no owner.

## The recorder's weapon

`CurWeapon` (state, signed weapon ID, signed clip, `cl_dll/ammo.cpp`) is sent only to the recording
player. A weapon switch is recorded whenever a message with state 1 names a different weapon than
the last one. State 0 updates another weapon's clip and does not select it.

## Coverage

| Kind | When | Notes |
|---|---|---|
| multiplayer view | more than one client, not HLTV | other players' events reach the client only while the server counts them as visible, so their counts are lower bounds. Without local events, the recorder's own actions are missing unless the server sent them |
| singleplayer | one client | other actors are monsters or world entities, and there is no death feed |
| HLTV | spectator proxy | no recording player, and events follow the proxy's view |
| unknown | decoding stopped before the server info | nothing can be said about its events |

The report also counts local and network events.

The recorder spectates while its client data `iuser1` holds an observer mode (`OBS_*` in
`common/const.h`, with `iuser2` naming the watched player), or while the demo info spectator flag
of a spectator client is set. The report sums playback time while spectating, each network record
weighted by the time until the next one, and gives the fraction of playback spent spectating.
Above one half the coverage notes that the recorder mostly spectated.

**Measured (2026-10-16):** the demo info flag alone was never set in the first 356 demos of the
sample, while observer mode covers the AG `spectate` command. `kuvick_vs_fedor_stalky.dem` is
observed in first person (mode 4) throughout, and recorded bhop runs of other players read 83% to
99% spectating. Across the 2,000-demo sample, 93 demos spend some time spectating and 62 spend
more than half.

## Headshots

Half-Life's death notice has no headshot flag, and demos record no hit locations. The server does
pick a dying player's animation from the body part of the last hit (`GetDeathActivity`), and every
snapshot carries that animation as the entity's `sequence`. In the standard `player.mdl`
sequences 18 to 24 are the deaths, and 22 `headshot` and 23 `die_spin` both have
`ACT_DIE_HEADSHOT`, one picked at random. A kill is a headshot when the victim's sequence in the
first packet after the death notice is 22 or 23.

- Only a player's direct hit counts: crowbar, glock, python, MP5 bullets, shotgun, crossbow, gauss
  and egon, whose damage goes through `TraceAttack`. Explosions and hornets damage without a body
  part, so the animation would come from an earlier hit, and those kills are not headshots.
- A victim the recording does not see within a second, or one gibbed and left in an aiming pose,
  leaves the headshot unknown. Other players' headshot counts are therefore incomplete and only the
  recording player's are shown, except in a duel, where every kill by the opponent is of the
  recording player, whose death the recording always shows. A headshot that was seen is certain,
  so the headshot kill filters use every one.

**Measured (2026-10-17):** `stalkx_5o5_asasasdgwgw.dem` has 49 headshot kills of 249 kills whose
victim was seen, 22 of them with the MP5 and 16 with the glock. Of 22 rocket kills whose victim was seen, 3
showed the headshot death from an earlier hit, which the weapon rule leaves out.

## Accuracy

Demos record no hits, but a hit through `TraceAttack` makes the server send a blood sprite
(`TE_BLOODSPRITE`) at the impact point to clients that can see it. A shot is a hit when blood
appears along its aim line between 0.05 s before and 0.25 s after it, within 16 units plus the
weapon's spread (12 degrees for the shotgun, 6 for the MP5 and glock, 3 otherwise).

- **Aim.** The recording player's aim is exact, from its own events. Other players' events carry the
  model's pitch, a third of the view pitch, so their aim is rebuilt from the snapshot pose at the
  shot, pitch times minus three, eye 28 units above the origin. Shots with no pose within 0.1 s,
  usually by players out of view, are not counted.
- **Chance.** Several players shooting one target put the same blood on more than one aim line.
  Shifting every blood sprite 1.5 s later and matching again counts those chance matches, and
  accuracy is hits less chance matches over shots. Giving each blood sprite to only the closest
  shot was tried and is worse, since rebuilt aims then take the recorder's real hits.
- **Weapons.** Glock, python, MP5 bullets, shotgun, zoomed crossbow and the gauss beam's direct hit.
  Explosions, gauss reflections and punch-through and hornets deal damage without blood, and
  unzoomed bolts fly too long to match, so they are left out. Gauss accuracy is direct beam hits
  only.
- **Shown.** Only the recording player's accuracy is shown, with at least 50 counted shots. Other
  players' kills and shots often happen outside what the recording client receives, so their
  figures are incomplete as well as approximate.

**Measured (2026-10-17)** with `aim_probe`, comparing kill recall, the share of visible direct-hit
kills with a matched hit by the killer in the half second before, and chance matches.

| Demo | Aim | Accuracy | Chance | Kills with a hit |
|---|---|---|---|---|
| `stalkx_5o5_asasasdgwgw.dem` | recorder, exact | 45.4% | 7.3% | 56 of 56 |
| | others, rebuilt | 33.7% | 5.0% | 118 of 151 |
| `20230909_224143_stalkyard.dem` | recorder, exact | 44.0% | 6.1% | 83 of 83 |
| | others, rebuilt | 31.7% | 8.1% | 104 of 143 |

The exact aim finds a hit before every kill. The rebuilt aim finds one before about three kills in
four, and duels do no better (8 of 13, 51 of 51, 35 of 44 and 74 of 85 in four), so other players'
accuracy is not shown. The other probes are
`blood_probe` (blood sizes and heights) and `death_probe` (death animations).

## Highlights

Highlights are filters for finding moments worth a closer look, such as midair or fast kills and
stretches packed with kills on both sides. They are approximate and meant for searching, not as
evidence.

### Motion

The recorder's motion is exact, from client data (`flags & FL_ONGROUND`, `velocity`). Other
players are inferred from their entity origins, timed by the server clock with repeated snapshots
dropped.

- **Airborne** when a quadratic fit of height over the last 0.2 s (at least 4 samples spanning
  0.1 s, no gap above 0.1 s) has an acceleration within 25% of minus the server gravity and a
  residual of at most 0.5 units. **On ground** when the acceleration is within 0.2 g of zero.
  Otherwise unclear.
- **Horizontal speed** from the displacement over the last 0.1 s.
- A player outside the recorder's view is not visible.

**Measured (2026-10-16)** by inferring the recorder's own motion from its client data origins and
comparing it with the exact values over `20230909_224143_stalkyard.dem`,
`naz_crossfire_pb_01.dem` and `20180208_000722_crossfire.dem`: of 62,428 snapshots inferred
airborne, 97.0% were airborne. 52.8% of airborne snapshots were found airborne and 1.6% on
ground, the rest unclear. 1.1% of on-ground snapshots were found airborne. The horizontal speed
error is 13.2 u/s at the median, 96.9 at the 90th percentile and 472.4 at the 99th, where
speed changes sharply.

### Kill context

Each player kill is linked to the killer's latest shot that may cause it, at most the weapon's
flight time before the kill by server time.

| Shot | Flight limit |
|---|---|
| crossbow unzoomed | 3 s |
| rpg | 6 s |
| mp5 grenade launch | 4 s |
| hivehand | 3 s |
| anything else | 0.3 s |

A `grenade` kill links to an mp5 grenade launch, since hand grenades leave no event. An `9mmAR`
kill links only to mp5 fire. The killer's motion is taken at the linked shot, or at the kill
without one, the victim's at the kill, and the distance between the two.

### Action windows

Every 60 s window starting at a frag is scored over the frags of its two busiest sides. Sides are
teams when both players have different teams, otherwise killers. Frags are kills by another
player, excluding suicides and teammates. With `a` and `b` frags the score is

```
a^(3/4) + b^(3/4) - max(|a - b|^(2/5), 1)
```

times 1 + 0.1 per weapon beyond the first. Even exchanges outscore one-sided runs: 8-7 scores
8.06, 5-5 scores 5.69 and 15-0 scores 4.67. The best five windows that do not overlap are
reported.

### Filters

`analyze --kills` lists kills with their context. `--weapon NAME` keeps one weapon string,
`--victim-airborne` and `--killer-airborne` keep kills where that player was inferred airborne,
and `--min-speed U` keeps kills where either player moved horizontally at least `U` units per second.
For example `--weapon crossbow --victim-airborne` finds midair crossbow kills, and
`--weapon grenade --victim-airborne --killer-airborne` finds midair grenade kills by midair
players.

## Actors

| Label | Meaning |
|---|---|
| player | a player slot with its occupant when the event arrived |
| unknown player (entity N) | a player entity whose occupant was not known then |
| entity N | in singleplayer, an entity outside the player range, such as a monster |
| unknown actor | no entity, a packet index that could not be resolved, or in multiplayer an entity outside the player range |

Every weapon script in the profile belongs to a player weapon. In multiplayer, an event that names an
entity outside the player range therefore has an unknown shooter, with attribution
`non_player_entity`. The shot still counts, and the record keeps the entity. **Observed** on two
kinds of server, both for players out of the recorder's view:

- `[LLHL]_dabo_27092026_031629.dem` (16 slots): events carry explicit `entindex` 185 to 188,
  masked numbers for out-of-view players.
- `20180208_000722_crossfire.dem` (16 slots, a two-player duel): the opponent's events carry no
  `entindex`, and a packet index equal to the number of entities numbered below 256 in the
  packet (11,103 of 12,337 packet-resolved events). It lands on unrelated entities such as a
  laser beam, where a ReHLDS server would send the packet's entity count and the real entity.
  The 2021 vengeance match never shows this.

## Local and network records

The recording player's shots come from its own predicted demo events. In the 2,000-demo sample,
57,950 of its shots are local events and 79 are network events, and no network shot has a local
event of the same weapon within 0.1 s. The 79 are all in singleplayer speedruns, where the
player's events arrive only from the server. So no shot is recorded twice, and nothing is merged.
The recorder does receive its own gauss stops twice (local and reliable from the server), which
are not shots.

## Counts

The report gives, per weapon, **shots** (fire and launch actions), **records** (every recorded
event of that weapon, updates included) and intervals, and per weapon, action and mode the number
of actions and how many have a known player.

**Measured (2026-10-16):**

- `naz_crossfire_pb_01.dem` also has 231 kills, 200 of them by gauss (both players), 1 by
  `grenade` (unmapped), and 156 weapon switches by the recorder.
- `run_108.dem` (singleplayer, `c1a0e`) has no events and one weapon switch, and is labelled
  singleplayer.

- `naz_crossfire_pb_01.dem`: 72,820 gauss records hold 550 shots (531 charged, 19 primary), 595
  stops and 545 charge intervals, 5 of them with an unseen end. Every action has a known player.
  These shot counts are in the fixture manifest.
- Fixtures for the plan's acceptance cases, with shots per weapon, unattributed shots and
  recording kind: `20230909_224143_stalkyard.dem` (every scripted action, 5,126 shots, all
  attributed), `run_92.dem` (singleplayer, 47 shots) and `20180208_000722_crossfire.dem` (1,028
  of 2,278 shots with an unknown shooter).
- `20210418_004941_vengeance.dem`, a five-player team match: 501 gauss shots out of 74,796
  records, other players' actions from network events resolved through both explicit entities and
  packet entity arrays, and every action with a known player. 235 of 740 charge intervals have an
  unseen start, mostly other players charging out of view. The recorder receives its own gauss
  stops twice (102 local, 70 reliable from the server). Its shots appear only as local events.

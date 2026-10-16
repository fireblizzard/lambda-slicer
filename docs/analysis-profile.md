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

The report also counts local and network events.

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

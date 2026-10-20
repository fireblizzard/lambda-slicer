# English, the source catalog. Every other catalog has the same messages
# with the same arguments

app-title = Lambda Slicer

## Toolbar

toolbar-open = Open demo…
toolbar-add-folder = Add folder…
toolbar-rescan = Rescan
toolbar-cancel = Cancel
toolbar-settings = Settings
toolbar-rescan-busy = A scan is already running.
toolbar-rescan-nothing = Add a folder or open a demo to scan.
toolbar-cancel-idle = No scan is running.
dialog-demos = Half-Life demos

## Settings

settings-title = Settings
settings-language = Language
settings-unreviewed = This translation is a draft that no native speaker has reviewed yet.
settings-workers = Analysis workers
settings-workers-hint = More workers analyse faster but use more processor time.
settings-cache = Library cache
settings-folders = Library folders
settings-folders-empty = No folders added yet.
settings-remove = Remove

## Library

library-empty = Add a folder or open a demo to get started.
library-no-matches = No demos match these filters.
library-no-cache = There is no place to keep the library cache on this system.
library-demos = { $count ->
    [one] { $count } demo
   *[other] { $count } demos
}
library-percent = { $percent }%
library-demos-of = { $shown } of { $total ->
    [one] { $total } demo
   *[other] { $total } demos
}
library-total = { $count ->
    [one] { $count } demo in the library
   *[other] { $count } demos in the library
}
filters-title = Filters
filters-find = Find
filters-pov = Point of view
filters-quality = Quality
filters-recorded = Recorded
pov-any = Any
pov-playing = Playing
pov-spectating = Spectating
help-pov = Playing means the recording spent at most half of its playback spectating, and Spectating means more than half.
columns-button = Columns
list-back = Back to the library table
sort-by = Sort by
sort-ascending = Ascending
sort-descending = Descending
score-more-sides = { $count ->
    [one] { $count } more side
   *[other] { $count } more sides
}
intensity-help = Intensity { $score }. The bar compares it with the highest in the library, { $max }.
value-unknown = Not known
path-copy = Copy path
path-show = Show in folder
path-show-failed = The folder could not be opened ({ $detail }). Copy the path instead.

filter-search = Search paths
filter-map = Map
filter-player = Player
filter-weapon-any = Any weapon
filter-min-minutes = At least (minutes)
filter-spectating-any = Playing or spectating
filter-spectating-mostly = Mostly spectating
filter-spectating-playing = Mostly playing
filter-clear = Clear filters

column-status = Status
column-name = Demo
column-map = Map
column-duration = Length
column-gauss = Gauss
column-kills = Kills
column-action = Intensity
column-spectating = Spectating

status-complete = Analysed
status-partial = Partial
status-failed = Failed
status-unsupported = Unsupported
status-no-use = No weapon use

kind-multiplayer-view = Player's view
kind-singleplayer = Singleplayer
kind-hltv = Spectator proxy
kind-unknown = Unknown kind

details-recorder = Recorded by { $name }
details-shots = { $count ->
    [one] { $count } shot
   *[other] { $count } shots
}
details-kills = { $count ->
    [one] { $count } kill
   *[other] { $count } kills
}
details-action = Highest intensity { NUMBER($score, minimumFractionDigits: 2) }

## Scanning

scan-progress = Found { $found }, done { $done } ({ $cached } cached, { $analysed } analysed), { $megabytes } MB read
scan-cancelling = Stopping…
scan-cancelled = Stopped with { $left } left to scan.
scan-finished = Done: { $done } demos, { $analysed } analysed, in { $seconds } s.
scan-problems = { $count ->
    [one] { $count } problem
   *[other] { $count } problems
}
problems-title = Scan problems
problem-changed = The file changed while it was analysed, so nothing was stored.
drop-hint = Drop demos or folders to add them
drop-not-demo = Not a demo: { $names }

## Messages from the analysis core, by code

analysis-panicked = The analysis stopped on an internal error: { $detail }
cache-cancelled = The analysis was cancelled, so nothing was stored.
cache-database = The library cache cannot be used: { $detail }
cache-directory = The library cache folder cannot be created: { $detail }
container-bad-magic = This is not a Half-Life demo.
container-frame-bad-time = The recording is damaged at frame { $frame } of part { $entry }, which has an invalid time.
container-frame-missing-terminal = Part { $entry } of the recording ends without its closing record.
container-frame-negative-length = The recording is damaged at frame { $frame } of part { $entry }.
container-frame-over-limit = Frame { $frame } of part { $entry } is larger than allowed.
container-frame-truncated = The recording is cut off at frame { $frame } of part { $entry }.
container-frame-unknown-tag = The recording is damaged at frame { $frame } of part { $entry }, which has an unknown type.
container-io = The file cannot be read: { $detail }
container-no-directory = The recording was never finished, so it has no table of contents.
container-range = The recording's table of contents points outside the file.
container-too-short = The file is too short to be a demo.
coverage-hltv = Spectator proxy recording. There is no recording player, and events follow the proxy's view.
coverage-mostly-spectating = The recorder spectated for { $percent }% of the playback, so most events are other players' as the spectated view showed them.
coverage-no-local-events = No local events. The recorder's own actions are missing unless the server sent them.
coverage-singleplayer = Singleplayer. Other actors are monsters or world entities, and there is no death feed.
coverage-unknown = The recording did not decode far enough to tell what kind it is.
coverage-visible-only = Other players' events reach this client only while the server counts them as visible, so their counts are lower bounds.
decode-failed = Decoding stopped at { $time } (frame { $frame }): { $detail }
export-active-effect = { $effect } is under way at the start and cannot be carried over yet.
export-client-dropped-packets = The game dropped late packets at { $time }, which a cut cannot reproduce yet.
export-cross-segment = A new session starts at { $time }, inside the range.
export-destination-exists = { $path } already exists. Choose another name.
export-empty-range = The selected range holds nothing to export.
export-io = { $path } cannot be written: { $detail }
export-missing-delta-base = The recording lacks state needed before the start (sequence { $sequence }).
export-missing-parent = The folder { $path } does not exist.
export-range-beyond-end = The end { $end } is past the end of the recording, { $segment_end }.
export-same-as-source = The destination is the source demo itself.
export-source-changed = { $path } changed during the export.
export-source-decode-failed = The source demo does not decode: { $detail }
export-source-failed-crosscheck = The source demo failed its cross-check, so its reconstructed state is not trusted.
export-unknown-persistent-message = { $name } before the start sets state a cut cannot carry over yet.
export-unproven-state = The state at { $time } rests on an inferred base and is not proven.
export-unrepresentable-time-field = A time at the start cannot be written faithfully: { $detail }
export-unsupported = Not supported: { $detail }
export-validation-failed = The exported file failed validation: { $detail }
io-not-found = The file or folder was not found.
io-other = { $detail }
io-permission-denied = Access was denied.
io-unexpected-end = The file ended unexpectedly.
protocol-unsupported = Demo protocol { $demo }/{ $network } is not supported, only 5/48.
protocol-unsupported-network = The server switched to network protocol { $network }, which is not supported.

## Demo view

demo-close = Close
demo-loading = Analysing { $name }…
show-shots = Shots
show-actions = Other actions
show-kills = Kills
show-switches = Weapon switches
lane-unknown = Unknown player
lane-recorder = { $name } (recorder)
event-kill = Killed { $victim }
event-switch = Weapon switch
action-swing = Swing
action-fire = Fire
action-launch = Launch
action-charge-activity = Charging
action-charge-stop = Charge stop
action-beam-start = Beam start
action-beam-sustain = Beam
action-beam-stop = Beam stop
action-placement-attempt = Placement attempt
action-release-attempt = Release attempt
mode-primary = primary
mode-charged = charged
mode-zoomed = zoomed
mode-unzoomed = unzoomed
mode-grenade = grenade
mode-bullets = bullets
mode-single = single
mode-double = double
mode-rapid = rapid
demo-events = { $count ->
    [one] { $count } event
   *[other] { $count } events
}

column-date = Date
column-score = Score
filter-from = From
filter-to = To
date-from-name = Date from the file name
date-from-modified = Date from when the file was last modified, which copying or downloading can change
timeline-best-action = Golden minute
timeline-window = #{ $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Final scores, at the last kill { $time }
scores-at = Scores at { $time }
scores-show-final = Show final scores
scores-frags = Frags
scores-deaths = Deaths

viewer-play = Play
viewer-pause = Pause
viewer-volume = Volume
viewer-no-sound = No sound
viewer-play-backward = Rewind
viewer-step-back = Previous frame
viewer-step-forward = Next frame
viewer-rate = { $rate }×
viewer-shortcut = Shortcut: { $key }
key-space = Space
viewer-map-failed = The map { $map } could not be loaded
viewer-stopped = Playback stopped early: { $reason }
viewer-missing = Files that could not be read: { $count }
event-previous = Previous event
event-next = Next event
event-play = Play event
timeline-title = Timeline
timeline-lanes = Player lanes
timeline-zoom-in = Zoom in
timeline-zoom-out = Zoom out
timeline-fit-all = Fit the whole demo
timeline-gestures = Click a mark to select its event, or anywhere else to go there. Drag the ruler or the playhead to scrub, and drag elsewhere to move along. Scroll to zoom around the pointer, and double-click to see the whole demo. Drag a clip's edge to move it, or click the clip to edit it.
timeline-density = The shaded bars show how many of the shown events fall in each stretch, against the busiest stretch of the demo.
timeline-density-count = { $count ->
    [one] One shown event here
   *[other] { $count } shown events here
}
timeline-session-count = Session { $session }: { $count }
timeline-choose = Events at this point
legend-clip = Clip
event-session = Session { $session }
event-unreachable = This event is in a later session that reuses the same times, so the viewer can't go to it.
event-hidden = The selected event is hidden by the filters.
highlight-jump = Jump to highlight…
highlight-help = Highlights rank the whole demo by its action, whatever the filters show.
kills-with = Kills with
refine-midair = Midair
refine-speed = Speed
refine-wallgauss = Wallgauss
refine-clear = Clear
refine-airborne = Airborne
refine-either = Either
midair-both = Both
midair-one = Only one
help-midair-either = The killer, the victim or both were in the air. The killer is checked at the shot linked to the kill, or at the kill without one, and the victim at the death.
help-midair-both = The killer and the victim were both in the air, each when checked.
help-midair-one = Exactly one of them was in the air, and the other is known to be on the ground.
speed-at-least = At least
speed-unit = ups
speed-killer = Killer
speed-victim = Victim
help-speed = Horizontal speed in units per second, across the ground. The killer is measured at the shot linked to the kill, or at the kill without one, and the victim at the death. Other players' speeds are estimates.
wallgauss-pending = This version can't tell yet which Gauss kills went through a wall, so no kills match. Clear the refinement to see every kill again.
wallgauss-weapon = Wallgauss only covers Gauss kills, and the weapon filter shows { $weapon }.
events-unknown = { $count ->
    [one] One kill lacks the data to tell
   *[other] { $count } kills lack the data to tell
}
help-unknown = Movement the recording couldn't see counts as unknown, never as on the ground or standing still, so these kills are left out.
clip-it = Clip it
badge-headshot = HS
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = suicide
badge-teamkill = team kill
badge-world = world
air-both = Both were in the air when checked.
air-killer = The killer was in the air when checked.
air-victim = The victim was in the air when checked.
air-other-unknown = Whether the other was is unknown.
speed-horizontal = Horizontal speed, across the ground.
motion-line = { $who }: { $air }, { $speed }. { $how }, { $when }.
motion-who-killer = Killer
motion-who-victim = Victim
motion-air-yes = in the air
motion-air-no = on the ground
motion-air-unknown = not known whether in the air
motion-speed-unknown = speed unknown
motion-exact = Recorded by the game
motion-trajectory = Estimated from movement
motion-unclear = Movement unclear
motion-not-visible = Out of view
motion-at-shot = at the linked shot
motion-at-kill = at the death
motion-none = The analysis has no movement for this kill.
legend-shot = Shot
legend-charge = Charging
legend-kill = Kill
legend-death = Death
legend-switch = Weapon switch

column-headshots = Headshots
filter-headshots = Headshot kills
event-headshot = headshot
scores-headshots = HS

column-accuracy = Accuracy
filter-min-accuracy = Accuracy at least (%)
scores-accuracy = Acc.

help-headshots = Kills by a hit to the head, read from the death animation the victim plays. Explosions don’t count. Only the recording player’s are shown, because other players often kill out of the recording’s view. In a duel the opponent’s are shown too, since all their kills are of the recording player.
help-headshots-column = The recording player’s kills by a hit to the head, read from the death animation the victim plays. Explosions don’t count. Other players’ kills aren’t counted, because they often happen out of the recording’s view.
help-accuracy = The share of shots that hit, from the blood a hit leaves along the aim, minus matches that happen by chance. It covers the glock, python, MP5 bullets, shotgun, zoomed crossbow and the gauss beam’s direct hits. Explosions and gauss reflections leave no blood and aren’t counted. Shown for the recording player only, after at least 50 shots.
help-headshot-filter = Demos with at least one kill by a hit to the head that the recording saw, by any player.
help-accuracy-filter = Demos where the recording player hit at least this share of their shots.

clips-title = Clips
clips-new = New clip
clips-add-event = Clip { $start } – { $end }
clips-add-event-none = Clip the selected event
clips-select-event = Select an event in the list or on the timeline first.
clips-other-session = A clip cannot start here, as an earlier session of the recording has the same times.
clips-add-window = Clip this intense moment
clips-empty = No clips yet. Select an event or one of the most intense moments, then add a clip around it. How much a clip keeps before and after an event is set in Settings. New clip adds one whose start and end you type yourself.
clip-default-name = Clip { $number }
clip-name = Name
clip-length = Length { $duration }
clip-show = Show
clip-remove = Remove
clip-loop = Loop
clip-time-help = Seconds (96), minutes and seconds (1:36), or hours, minutes and seconds (00:01:36.250)
clip-exact = The export starts and ends exactly here.
clip-effective = The export cuts { $start } – { $end } ({ $duration }), where the recording’s frames fall.
settings-clip-padding = Clip padding
settings-preroll = Seconds before the event
settings-postroll = Seconds after the event
export-start-first-group = A cut cannot start on the recording’s very first frame. Start a moment later.
export-start-during-loading = The start is while the game is still loading a map. Start a moment later.

export-all = Export all clips
export-one = Export
export-cancel = Stop exporting
export-folder = Exports go to { $folder }.
export-folder-beside = Exports go beside the demo, in { $folder }.
export-waiting = Waiting to export…
export-running = Exporting… This reads the whole demo and can take a while.
export-cancelled = Not exported, as the export was stopped.
export-saved = Saved { $path }. Snapshots checked against the original: { $count }.
export-playdemo = In the game’s console:
export-copy-first = Copy { $name } into your mod folder, the one with liblist.gam, then in the game’s console:
export-copy = Copy
settings-export-folder = Export folder
settings-export-choose = Choose…
settings-viewer = Demo viewer
settings-viewer-game = Half-Life folder
settings-viewer-game-none = Not chosen
settings-viewer-game-hint = The folder that holds valve. The demo viewer plays demos with the game's maps, models and sounds from it.
settings-executable = Game executable
settings-executable-hint = Finds the Half-Life folder. The viewer reads the game's files itself and never starts the game.
settings-executable-not-set = Not set, so the viewer has no picture or sound
settings-executable-optional = Not set, and not needed while the Half-Life folder holds the game
settings-executable-missing = { $path } is no longer there. The viewer still reads the Half-Life folder.
settings-game-override = This run uses { $path }, given with --game. A choice here applies from the next run.
settings-browse = Browse…
dialog-executables = Programs
setup-title = Set up the demo viewer
setup-body = Picture and sound need Half-Life's game files. Show Lambda Slicer where the game is installed and it reads them from there.
setup-locate-exe = Locate hl.exe…
setup-locate-folder = Locate Half-Life…
setup-settings = Settings
setup-still-works = The timeline, events and clips keep working without them.
setup-relocate = { $folder } no longer holds Half-Life. Locate it again.
setup-not-executable = { $name } is not { $executable }. Choose the game's own executable, not a shortcut.
setup-not-installed = { $folder } holds no Half-Life installation with a valve folder.
tab-events = Events
tab-clips = Clips
tab-scoreboard = Scoreboard
list-hide = Hide demo list
list-show = Show demo list
demo-details = Details
viewer-start = Back to the start
viewer-mute = Mute
viewer-unmute = Unmute
viewer-picture = Picture and viewer settings
viewer-needs-files = Needs the game's files
viewer-no-sound-why = No sound: { $reason }
settings-tab-general = General
settings-section-folders = Folders
settings-folders-hint = The folders the library scans for demos, subfolders included.
settings-clip-padding-hint = How much a clip made around an event keeps before and after it.
settings-section-picture = Picture
settings-section-sound = Sound
settings-gamma = Gamma
settings-brightness = Brightness
settings-texgamma = Texture gamma
settings-lightgamma = Light gamma
settings-overbright = Overbright lighting
settings-gamma-hint = The game's settings of the same names (gamma, brightness, texgamma, lightgamma, gl_overbright). Set them as in the game to see the same brightness.
settings-gamma-defaults = Game defaults
settings-fov-recorded = Field of view as recorded
settings-fov = Field of view
settings-viewmodel = Show the weapon in view
settings-see-through = See players through walls
settings-see-through-items = Also other models, such as projectiles and items
settings-see-through-hint = Players behind walls show as silhouettes in their top colour, which team games set by team. Only what the demo holds can show. The server sends the players near the recorder, and servers that hide players the recorder cannot see leave nothing to show.
viewer-see-through = See through walls
settings-export-beside = Beside each demo
settings-export-hint = Inside your mod folder, the one with liblist.gam such as Half-Life\ag, clips play with a single playdemo command.

clips-store-moved = The clips file { $path } could not be read, so it was kept as { $moved } and clips start empty. { $detail }
clips-store-unreadable = The clips file { $path } could not be read: { $detail }. Clips are not saved this session, so nothing in it is lost.
clips-save-failed = Clips could not be saved to { $path }: { $detail }
notice-dismiss = Dismiss

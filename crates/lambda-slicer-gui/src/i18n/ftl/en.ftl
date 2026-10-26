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
settings-workers = Demos analysed at once
settings-workers-hint = Analysing more at once finishes sooner but makes the computer work harder.
settings-cache = Where the library is kept
settings-start-tab = Tab at start
settings-start-tab-hint = The library tab shown each time the app starts. Clicking another tab still shows it until you close the app.
settings-sort = Order at start
settings-sort-hint = How the demos are sorted each time the app starts. Clicking a column's heading still sorts by it until you close the app.
settings-sprees = Killing sprees
settings-sprees-hint = A player's last kill before dying shows a badge with how many kills they made in a row, when it's at least this many. Dying in any way ends a spree, and so does the map ending or the player leaving. Team kills don't count.
settings-spree-least = At least (kills)
settings-start-filters = Filters at start
settings-start-filters-hint = The library starts with these filters on each time the app opens, so short demos you never want to see stay out of the way. You can still change the filters until you close the app.
settings-hide-unplayed-maps = In demos of several maps, leave out maps with no kills recorded for less than 5 minutes
settings-folders = Library folders
settings-folders-empty = No folders added yet.
settings-remove = Remove

## Library

library-empty = Add a folder or open a demo to get started.
library-no-matches = No demos match these filters.
library-no-cache = There is nowhere on this computer to keep the library.
library-demos = { $count ->
    [one] { $count } demo
   *[other] { $count } demos
}
library-percent = { $percent }%
library-demos-of = { $shown } of { $total ->
    [one] { $total } demo
   *[other] { $total } demos
}
library-matches = { $count ->
    [one] { $count } match
   *[other] { $count } matches
}
library-matches-of = { $shown } of { $total ->
    [one] { $total } match
   *[other] { $total } matches
}
library-no-games = None of the demos has a match, or five minutes of play without one.
library-no-games-filtered = No matches meet these filters.
library-total = { $count ->
    [one] { $count } demo in the library
   *[other] { $count } demos in the library
}
filters-title = Filters
filters-find = Find
filters-pov = Point of view
filters-quality = Quality
filters-recorded = Recorded
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
path-copy = Copy file location
path-show = Show in folder
path-show-failed = The folder could not be opened ({ $detail }). Copy the file location instead.

filter-search = Search file names and folders
filter-map = Map
filter-player = Player
help-player-filter = Type part of a name to find demos where anyone used it. Pick a player from the list to find the demos they played in under any name, told apart by their SteamID. Players picked must all have played, not only watched. Bots and some older demos have no SteamID, so those players can only be found by name.
player-remove = Take off the filter
player-also = also { $names }
player-demos = { $count ->
    [one] { $count } demo
   *[other] { $count } demos
}
filter-weapon-any = Any weapon
filter-min-minutes = At least (minutes)
filter-min-kills = At least (kills)
filter-min-spree = Spree of at least (kills)
filter-players = Players
filter-teams = Teams
filter-bots = Bots
bots-any = Any
bots-few = Few
bots-none = None
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
kind-hltv = HLTV
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

scan-progress = Found { $found }, done { $done } ({ $cached } already known, { $analysed } analysed), { $megabytes } MB read
scan-cancelling = Stopping…
scan-cancelled = Stopped with { $left } left to scan.
scan-finished = Done: { $done ->
    [one] { $done } demo
   *[other] { $done } demos
}, { $analysed } analysed, in { $seconds } s.
scan-problems = { $count ->
    [one] { $count } problem
   *[other] { $count } problems
}
problems-title = Scan problems
problem-changed = The file changed while it was analysed, so nothing was stored.
drop-hint = Drop demos or folders to add them
drop-not-demo = Not a demo: { $names }

## Messages from the analysis core, by code

analysis-panicked = The analysis stopped because of a fault in Lambda Slicer: { $detail }
cache-cancelled = The analysis was cancelled, so nothing was stored.
cache-database = The saved library can't be used: { $detail }
cache-directory = The folder for the saved library can't be created: { $detail }
container-bad-magic = This is not a Half-Life demo.
container-frame-bad-time = The recording is damaged at frame { $frame } of part { $entry }, which has a broken time.
container-frame-missing-terminal = Part { $entry } of the recording doesn't end properly.
container-frame-negative-length = The recording is damaged at frame { $frame } of part { $entry }.
container-frame-over-limit = Frame { $frame } of part { $entry } is larger than allowed.
container-frame-truncated = The recording is cut off at frame { $frame } of part { $entry }.
container-frame-unknown-tag = The recording is damaged at frame { $frame } of part { $entry }, which holds something Lambda Slicer doesn't recognise.
container-io = The file cannot be read: { $detail }
container-no-directory = The recording was never finished, so it has no table of contents.
container-range = The recording's table of contents points outside the file.
container-too-short = The file is too short to be a demo.
coverage-hltv = HLTV recording. No player recorded it, so events follow what HLTV was watching.
coverage-mostly-spectating = The recorder spectated for { $percent }% of the playback, so most events are other players', as seen while spectating them.
coverage-no-local-events = The recorder's own actions weren't saved in this demo, so only the ones the server sent show.
coverage-singleplayer = Singleplayer. Everything else is monsters or parts of the map, and there is no kill feed.
coverage-unknown = The recording couldn't be read far enough to tell what kind it is.
coverage-visible-only = The game only gets other players' actions while the server thinks the recorder can see them, so their counts may be lower than what really happened.
decode-failed = Reading the recording stopped at { $time } (frame { $frame }): { $detail }
export-active-effect = { $effect } is still going on at the start, and a clip can't carry it over yet.
export-client-dropped-packets = At { $time } the game threw away data that arrived too late, which a clip can't copy yet.
export-clock-starts-over = The demo's clock started over at { $time }, after a map change or reconnect inside the clip, and a clip can't go across that.
export-destination-exists = { $path } already exists. Choose another name.
export-empty-range = The clip holds nothing to export.
export-io = { $path } cannot be written: { $detail }
export-missing-delta-base = The recording is missing data from before the start that the clip needs (number { $sequence }).
export-missing-parent = The folder { $path } does not exist.
export-range-beyond-end = The end { $end } is past the end of the recording, { $segment_end }.
export-same-as-source = The export would replace the demo it comes from.
export-source-changed = { $path } changed during the export.
export-source-decode-failed = The demo can't be read: { $detail }
export-source-failed-crosscheck = When checked, what Lambda Slicer worked out from this demo didn't match the demo itself, so clips from it can't be trusted.
export-unknown-persistent-message = The { $name } message isn't supported in clips yet.
export-unproven-state = At { $time } the recording relies on data that had to be guessed, so the clip can't be made exactly.
export-unrepresentable-time-field = A time at the start can't be saved exactly: { $detail }
export-unsupported = Not supported: { $detail }
export-validation-failed = The exported clip didn't pass its final check: { $detail }
io-not-found = The file or folder was not found.
io-other = { $detail }
io-permission-denied = Access was denied.
io-unexpected-end = The file ended unexpectedly.
protocol-unsupported = This demo is from a game version that isn't supported (protocol { $demo }/{ $network }). Only 5/47 and 5/48 are.
protocol-unsupported-network = The server switched to a game version that isn't supported (protocol { $network }).

## Demo view

demo-close = Close
demo-loading = Analysing { $name }…
show-shots = Shots
show-actions = Other actions
show-kills = Kills
show-switches = Weapon switches
show-chat = Chat
show-match = Match
lane-unknown = Unknown player
lane-recorder = { $name } (recorder)
event-kill = Killed { $victim }
event-switch = Weapon switch
event-copy = Copy
event-copy-kill = { $killer } killed { $victim } with { $weapon }
event-copy-teamkill = { $killer } killed their teammate { $victim } with { $weapon }
event-copy-suicide = { $victim } killed themselves with { $weapon }
event-copy-death = { $victim } died from { $weapon }
event-vote-called = Vote called on { $subject }
event-vote-cast = Vote on { $subject }: { $change } ({ $tally })
event-vote-more-yes = { $count ->
    [one] one more yes
   *[other] { $count } more yes
}
event-vote-fewer-yes = { $count ->
    [one] one less yes
   *[other] { $count } fewer yes
}
event-vote-more-no = { $count ->
    [one] one more no
   *[other] { $count } more no
}
event-vote-fewer-no = { $count ->
    [one] one less no
   *[other] { $count } fewer no
}
event-vote-tally = { $yes } yes, { $no } no, { $undecided } undecided
event-vote-accepted = Vote on { $subject } accepted
event-vote-denied = Vote on { $subject } denied
event-timeout-called = Timeout called
event-timeout-under-way = Timeout started
event-timeout-over = Timeout over
event-paused = Game paused
event-resumed = Game resumed
event-sudden-death = Sudden death started
event-sudden-death-over = Sudden death over
event-map-changed = Map changed to { $map }
event-spectating = Started spectating
event-playing = Stopped spectating
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

column-date = Date
column-file-date = File date
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
scores-spectators = Spectators
scores-follow = Click to watch through their eyes
pane-fill = Give it the whole side panel
pane-share = Share the side panel half and half
scores-frags = Frags
scores-deaths = Deaths

viewer-play = Play
viewer-pause = Pause
viewer-volume = Volume
viewer-no-sound = No sound
viewer-play-backward = Rewind
viewer-step-back = Previous frame
viewer-step-forward = Next frame
viewer-position = Position in the demo
viewer-skip-back = Back 10 seconds
viewer-skip-forward = Forward 10 seconds
viewer-rate = { $rate }×
viewer-slower = Slower
viewer-faster = Faster
viewer-rate-normal = Back to normal speed
viewer-shortcut = Shortcut: { $key }
key-space = Space
key-ctrl = Ctrl
key-shift = Shift
key-alt = Alt
key-left = Left
key-right = Right
key-up = Up
key-down = Down
key-home = Home
key-end = End
key-page-up = Page Up
key-page-down = Page Down
key-enter = Enter
key-backspace = Backspace
key-delete = Delete
key-insert = Insert
key-tab = Tab
key-mouse-3 = Mouse 3
key-mouse-4 = Mouse 4
key-mouse-5 = Mouse 5
key-wheel-up = Wheel up
key-wheel-down = Wheel down
viewer-map-failed = The map { $map } could not be loaded
viewer-stopped = Playback stopped early: { $reason }
viewer-missing = Files that could not be read: { $count }
event-previous = Previous event
event-next = Next event
event-details = Kill details
event-play = Play event
timeline-title = Timeline
timeline-lanes = Player lanes
timeline-zoom-in = Zoom in
timeline-zoom-out = Zoom out
timeline-fit-all = Fit the whole demo
timeline-gestures = Click a mark to select its event, or anywhere else to go there. Drag along the times at the top, or drag the line that marks the current moment, to move through the demo. Drag anywhere else to slide the view. Scroll to zoom in and out where the mouse is, and double-click to see the whole demo. Drag a clip's edge to change where it starts or ends, or click the clip to edit it.
timeline-density = The shaded bars show how many of the events shown happen at each point, compared with the busiest point of the demo.
timeline-density-count = { $count ->
    [one] One shown event here
   *[other] { $count } shown events here
}
timeline-session-count = Part { $session }: { $count }
timeline-choose = Events at this point
legend-clip = Clip
event-session = Part { $session }
event-unreachable = This event comes after a map change or reconnect, where the demo's clock started over, so the viewer can't go to it.
event-hidden = The selected event is hidden by the filters.
highlight-jump = Jump to highlight…
highlight-help = Highlights rank the whole demo by its action, whatever the filters show.
kills-with = Kills with
refine-midair = Midair
refine-speed = Speed
refine-wallgauss = Wallgauss
refine-multikill = Multikill
refine-clear = Clear
refine-airborne = Airborne
refine-either = Either
midair-both = Both
midair-one = Only one
help-midair-either = The killer, the victim or both were in the air. The killer is checked at the shot linked to the kill, or at the kill without one, and the victim at the death. Someone just jumping, less than 50 units above the floor, doesn't count. This needs the game folder set, to know where the floor is.
help-midair-both = The killer and the victim were both in the air, each when checked.
help-midair-one = Exactly one of them was in the air, and the other is known to be on the ground.
speed-at-least = At least
speed-unit = ups
speed-killer = Killer
speed-victim = Victim
help-multikill = Several players killed with one shot, such as a blast that reaches them all or a gauss beam through them. Every kill of that shot shows it.
help-speed = Speed across the ground in ups, as a speedometer shows it, leaving out movement up and down. The killer is measured at the shot linked to the kill, or at the kill without one, and the victim at the death. Other players' speeds are estimates.
wallgauss-weapon = Wallgauss only covers Gauss kills, and the weapon filter shows { $weapon }.
wallgauss-checking = Checking wallgauss… { $examined } of { $total } kills
wallgauss-needs-files = Wallgauss needs the game's files to know where the map's walls are.
wallgauss-failed = Wallgauss could not be checked. { $reason }
help-wallgauss = Gauss kills through a wall, found by following each shot through the map the way the server does. A kill that can't be decided, such as one whose victim was out of the recording's view, counts as unknown and is left out.
badge-wall = WG
wall-yes-exit = Wallgauss, worked out from the shot: the blast where the beam came out of a wall { $thickness } units thick.
wall-yes-beyond = Wallgauss, worked out from the shot: the beam hit after going through a wall { $thickness } units thick.
wall-limits = Worked out from the shot and where the players were, as the server doesn't record it. A player is 32 units wide and counts as a box, and the victim may have been a body width or two away from where the demo shows them.
wall-no-direct = Not a wallgauss: the beam could have hit the victim before any wall.
wall-no-reflection = Not a wallgauss: the blast of a reflection could have hit the victim before any wall.
wall-doubt-no-shot = Wallgauss unknown: the killer's shot is not in the recording.
wall-doubt-no-map = Wallgauss unknown: the map's files could not be read.
wall-doubt-unseen = Wallgauss unknown: the victim was out of the recording's view.
wall-doubt-brush = Wallgauss unknown: the beam hit a door, a lift or another special part of the map, which isn't checked.
wall-doubt-ambiguous = Wallgauss unknown: the shot could have killed both through a wall and without one.
wall-doubt-unexplained = Wallgauss unknown: no way the shot reached the victim was found.
wall-doubt-unconfirmed = Wallgauss unknown: only through a wall could the shot have reached the victim, but not surely.
wall-doubt-not-reached = Wallgauss unknown: the recording stops before the shot.
wall-pending = Wallgauss not checked yet.
events-unknown = { $count ->
    [one] One kill lacks the data to tell
   *[other] { $count } kills lack the data to tell
}
help-unknown = Movement the recording couldn't see counts as unknown, never as on the ground or standing still, so these kills are left out.
events-filters = Filters
events-filters-on = Filters ({ $count })
events-kinds-none = No events
events-follow = Keep the list on the current moment while the demo plays. Scrolling the list turns this off.
clip-it = Clip it
badge-headshot = HS
badge-double-kill = double kill
badge-triple-kill = triple kill
badge-quadra-kill = quadra kill
badge-penta-kill = penta kill
badge-multikill = { $count } kills
badge-spree = ×{ $count }
badge-spree-help = { $count } kills in a row without dying
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = suicide
badge-teamkill = team kill
badge-world = world
badge-clip = clip
clip-of = Clip of { $name }
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
column-multikill = Multikill
column-spree = Spree
filter-headshots = Headshot kills
filter-midair = Midair kills
event-headshot = headshot
scores-headshots = HS

column-accuracy = Accuracy
column-players = Players
column-teams = Teams
help-players-column = Players in the game, not spectators, counted when a match started after its countdown. In a game with teams only players on a team count. A demo shows the most in any complete match, and a demo with no complete match the most at any moment.
help-teams-column = Teams with a player on them, counted as the players are. A game without teams has none.
filter-min-accuracy = Accuracy at least (%)
scores-accuracy = Acc.
scores-kd = K/D
scores-ping = Ping
help-kd = Kills per death, the frags divided by the deaths. A player who never died shows their frags.
help-ping-loss = Each player’s ping in milliseconds, then their packet loss, as the server last sent them. It sends them only while the recording player holds the scoreboard key, so they stay a dash until then.

help-headshots = Kills by a hit to the head, read from the death animation the victim plays. Explosions don’t count. Only the recording player’s are shown, because other players often kill out of the recording’s view. In a duel the opponent’s are shown too, since all their kills are of the recording player.
help-headshots-column = The recording player’s kills by a hit to the head, read from the death animation the victim plays. Explosions don’t count. Other players’ kills aren’t counted, because they often happen out of the recording’s view.
help-multikill-column = The most players one shot killed in the demo, by anyone, such as 3 for a triple kill.
help-spree-column = The longest killing spree in the demo, by anyone: the most kills a player made in a row without dying.
help-date-column = When the match was recorded. It's read from the file name when the name has a date like 20230910_214553, as automatic recordings do. Otherwise it's the file date, which copying or downloading can change.
help-file-date = When the file was saved. For an exported clip, that's when it was exported.
help-accuracy = The share of shots that hit, from the blood a hit leaves along the aim, minus matches that happen by chance. It covers the glock, python, MP5 bullets, shotgun, zoomed crossbow and the gauss beam’s direct hits. Explosions and gauss reflections leave no blood and aren’t counted. Shown for the recording player only, after at least 50 shots.
help-headshot-filter = Demos with at least one kill by a hit to the head that the recording saw, by any player.
help-midair-filter = Demos with at least one kill where the killer or the victim was in the air, at least 50 units above the floor. Without the game folder set, a plain jump counts too.
help-spree-filter = Demos where a player killed at least this many in a row without dying. Dying in any way ends a spree, and so does the map ending or the player leaving. Team kills don't count.
help-accuracy-filter = Demos where the recording player hit at least this share of their shots.
help-bots-filter = Few keeps only demos and matches with fewer bots than people playing, so a 1v1 against a bot is left out, but not a busy server with a few bots filling in. None keeps only those with no bots at all. A bot is a player the game marks as one, or one without a CD key.

clips-new = New clip
clips-select-event = Select an event in the list or on the timeline first.
clips-other-session = A clip can't start here, because the demo's clock started over at a map change or reconnect just before.
clips-add-window = Clip this intense moment
clip-default-name = Clip { $number }
clip-name = Name
clip-length = Length { $duration }
clip-remove = Remove
clip-loop = Loop
clip-time-help = Seconds (96), minutes and seconds (1:36), or hours, minutes and seconds (00:01:36.250)
clips-empty-hint = No clips yet. Select an event and press Clip it, or press New clip to start one at the current moment.
clips-new-help = A ten-second clip from the current moment.
clip-in = Start
clip-out = End
clip-set-in = Start the clip at the current moment
clip-set-out = End the clip at the current moment
clip-no-playhead = The viewer isn't available, so there is no current moment to use.
clip-play = Play the clip once
clip-unplayable = This clip comes after a map change or reconnect, where the demo's clock started over, so the viewer can't play it. Exporting it still cuts the right moment.
clip-show-on-timeline = Show on the timeline
export-destination = Change where exports go
export-skipped = Skipped. { $reason }
export-earlier-range = This export is of { $start } – { $end }, the clip as it was before you changed it.
scores-player = Player
scores-none = The recording has no scores.
clip-effective = The export runs { $start } – { $end } ({ $duration }), as close as the recording allows.
settings-clip-padding = Clip padding
settings-preroll = Seconds before the event
settings-postroll = Seconds after the event
export-start-first-group = A clip can't start at the very beginning of the recording. Start a moment later.
export-stopped = The export was stopped, so no clip was written.
export-start-during-loading = The start is while the game is still loading a map. Start a moment later.

export-all = Export all clips
export-one = Export
export-cancel = Stop exporting
export-folder = Exports go to { $folder }.
export-folder-beside = Exports go beside the demo, in { $folder }.
export-waiting = Waiting to export…
export-running = Exporting… This reads the whole demo and can take a while.
export-searching = The clip can't start right there. Looking for a start nearby that works…
export-nearest-earlier = It can start at { $time } instead, { $seconds } s earlier.
export-nearest-later = It can start at { $time } instead, { $seconds } s later.
export-start-there = Start there and export
export-cancelled = Not exported, as the export was stopped.
export-saved = Saved { $path }. Moments checked against the original: { $count }.
export-playdemo = In the game’s console:
export-copy-first = Copy { $name } into your mod folder, the one with liblist.gam, then in the game’s console:
export-copy = Copy
settings-export-folder = Export folder
settings-export-choose = Choose…
settings-file-names = File names
settings-clip-name-files = Use the clip's name for the file
settings-demo-name-first = Start with the demo's name
settings-viewer = Demo viewer
settings-viewer-game = Half-Life folder
settings-viewer-game-none = Not chosen
settings-viewer-game-hint = The folder that holds valve. The demo viewer plays demos with the game's maps, models and sounds from it.
settings-game-override = This time Lambda Slicer uses { $path }, as it was started with --game. A choice here applies the next time it starts.
settings-more-games = More Half-Life folders
settings-more-games-hint = Other places where Half-Life is installed, such as one with lots of maps. When a map, model, sprite or sound isn't in the Half-Life folder above, the demo player looks in these, from the top down. Changing them makes the library check its demos again for midair kills, with the maps they add.
settings-more-games-empty = None added yet.
settings-move-up = Move up
settings-move-down = Move down
settings-game-content = Game content
settings-game-content-hint = These start as you have them in the game. Changing them here only changes the demo player, never the game. With more than one on, the game's order is used: addon content first, then localized content, then HD models, then the game's own files.
settings-hd-models = Use HD models
settings-hd-models-hint = Looks in the _hd folders first, such as valve_hd, for the high quality models. Starts as the game's "Use high quality models" option.
settings-addon-content = Use custom addon content
settings-addon-content-hint = Looks in the _addon folders first, such as valve_addon, for content you added to the game. Starts as the game's "Allow custom addon content" option.
settings-localized-content = Use localized content
settings-localized-content-hint = Looks in the folders of the language Steam runs Half-Life in first, such as valve_spanish for Spanish. The game always uses them, and English has none.
settings-steam-language = Steam runs Half-Life in { $language }.
settings-steam-english = Steam runs Half-Life in English, which has no folders of its own.
dialog-executables = Programs
setup-title = Set up the demo viewer
setup-body = Picture and sound need Half-Life's game files. Show Lambda Slicer where the game is installed and it reads them from there.
setup-locate-exe = Locate hl.exe…
setup-locate-folder = Locate Half-Life…
setup-settings = Settings
setup-still-works = The timeline, events and clips keep working without them.
setup-relocate = { $folder } no longer holds Half-Life. Locate it again.
setup-not-executable = { $name } is not { $executable }. Choose the game's own program, not a shortcut.
setup-not-installed = { $folder } holds no Half-Life installation with a valve folder.
tab-events = Events
tab-clips = Clips
tab-scoreboard = Scoreboard
tab-demos = Demos
tab-matches = Matches
filter-status = Status
library-all = All
library-favourites = Favourites
library-no-favourites = No favourite demos yet. Right-click a demo and choose Add to favourites.
menu-favourite-add = Add to favourites
menu-favourite-remove = Remove from favourites
favourite-star = Favourite
library-hidden = Hidden
library-no-hidden = No hidden demos. Right-click a demo and choose Hide to keep it out of the list.
menu-hide = Hide
menu-unhide = Unhide
filter-status-all = All
help-status-column = How much of the match the demo recorded. Point at a row's status to see what it means.
help-match-players = Players in the game, not spectators, when the match started after its countdown, or at its first kill when it was already on as the recording began. In free play, the most at once.
help-match-teams = Teams with a player on them, counted as the players are. A game without teams has none.
help-match-multikill = The most players one shot killed in the match, by anyone, such as 3 for a triple kill.
help-match-spree = The longest killing spree that ended in the match, by anyone: the most kills a player made in a row without dying.
match-complete = Complete
match-joined-late = Joined late
match-interrupted = Interrupted
match-cut-off = Cut off
match-free-play = Free play
help-match-complete = Recorded from the countdown to the end of the match.
help-match-joined-late = The match was already on when the recording began, and was recorded to its end.
help-match-interrupted = Another match started, the match was called off, or the map changed before the match ended.
help-match-cut-off = The recording ended before the match did.
help-match-free-play = Five minutes or more of play with no match on.
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
settings-tab-library = Library
settings-tab-clips = Clips
settings-tab-hud = HUD
settings-search = Search settings
settings-search-none = No settings match.
settings-tab-shortcuts = Shortcuts
settings-shortcuts-hint = Click a key to change it, then press the new key or mouse button, turn the mouse wheel with the pointer on it, or press Escape to give up. Right-click a key to remove it. A key another action already has moves here. The mouse wheel's shortcuts work over the demo's picture.
settings-shortcuts-defaults = Default keys
settings-shortcut-press = Press a key…
settings-shortcut-change = Click to change, right-click to clear
settings-shortcut-moved = { $key } was taken from “{ $action }”.
shortcut-none = no key
shortcuts-anywhere = Anywhere
shortcuts-demo = Demo
shortcuts-playback = Playback
shortcuts-view = View
shortcuts-players = POVs
shortcuts-free-camera = Free camera
shortcut-open-demo = Open a demo
shortcut-settings = Open or close the settings
shortcut-back = Back through the demos and matches you opened
shortcut-forward = Forward through the demos and matches you opened
shortcut-previous-event = Previous event
shortcut-next-event = Next event
shortcut-clip-start = Set the chosen clip's start to now
shortcut-clip-end = Set the chosen clip's end to now
shortcut-play-pause = Play or pause
shortcut-rewind = Rewind
shortcut-pause = Pause
shortcut-seek-back = 5 seconds back
shortcut-seek-forward = 5 seconds forward
shortcut-skip-back = 10 seconds back
shortcut-skip-forward = 10 seconds forward
shortcut-frame-back = One frame back
shortcut-frame-forward = One frame forward
shortcut-slower = Slower
shortcut-faster = Faster
shortcut-start = Back to the start
shortcut-fullscreen = Full screen
shortcut-zoom = Zoom in or out
shortcut-see-through = See players through walls
shortcut-fullbright = Fullbright
shortcut-names = Player names
shortcut-free-camera = Free camera
shortcut-recorder = The recorder's POV
shortcut-follow = POV of the map's best player number { $place }
shortcut-previous-player = Previous player's POV
shortcut-next-player = Next player's POV
shortcut-fly-forward = Forward
shortcut-fly-back = Back
shortcut-fly-left = Left
shortcut-fly-right = Right
shortcut-fly-up = Up
shortcut-fly-down = Down
shortcut-fly-fast = Faster
settings-section-game = Half-Life
settings-section-view = View
settings-language-system = System language ({ $name })
settings-section-folders = Folders
settings-folders-hint = The folders the library scans for demos, subfolders included.
settings-clip-padding-hint = How much a clip made around an event keeps before and after it.
settings-section-picture = Picture
settings-section-sound = Sound
settings-section-list = Library list
settings-section-effects = Effects
settings-section-vitals = Health and ammo
settings-section-scores = Scores
settings-section-chat = Chat
settings-section-speeds = Speed and fps
settings-section-server = From the server
settings-section-colours = Colours and text size
settings-gamma = Gamma
settings-brightness = Brightness
settings-texgamma = Texture gamma
settings-lightgamma = Light gamma
settings-overbright = Overbright lighting
settings-gamma-hint = The game's settings of the same names (gamma, brightness, texgamma, lightgamma, gl_overbright). Set them as in the game to see the same brightness.
settings-gamma-defaults = Game defaults
settings-fov-recorded = Field of view as recorded
settings-fov = Field of view
settings-zoom-fov = Zoomed field of view
settings-viewmodel = Show the weapon in view
settings-weapon-bob-recorded = Weapon bob as recorded
settings-weapon-bob = Weapon bob (cl_bob)
settings-crosshair = Show the crosshair
settings-chat = Show the chat
settings-chat-places = Show messages that give a location
settings-chat-places-hint = Team messages from binds that give the sender's location with %l or %L. Unticked, they're left out of the chat as if they were never sent.
settings-chat-long-jump = Show messages that say "got LJ" or "no LJ"
settings-chat-long-jump-hint = Team messages from binds that say whether the sender has the long jump module. Unticked, they're left out of the chat as if they were never sent.
settings-vitals = Show health, armour, ammo and pickups
settings-vitals-size = Size of health, armour, ammo and pickups
settings-free-vitals = Show health, armour, ammo and pickups with the free camera too
settings-scores = Show the scores at the top left
settings-score-rows = Most rows of scores
settings-scoreboard = Show scoreboard while the recorder holds its key, and at intermission
settings-scoreboard-off = Off
settings-scoreboard-fullscreen = Only in fullscreen
settings-scoreboard-always = On
settings-server-settings = Show the server's settings at the top right
settings-text-size = Size of the chat, kill feed and server's settings
settings-hud-color = HUD colour
settings-hud-color-hint = The colour of the health, armour, ammo, pickups, weapon menu, speeds and recorder's fps, and of AG's timer, votes and server settings, like hud_color in OpenAG. Low health and weapons out of ammo stay red, and the server's own HUD text keeps its colour.
settings-text-color = Text colour
settings-text-color-default = Game's colour
settings-text-color-hint = The colour of chat messages, of lines no player sent, and of the position and view angles, like con_color in the game. Players' names keep their team's colour.
settings-hud-messages = Show text from the server, such as bhop and KZ timers
settings-see-through = See players through walls
settings-see-through-items = Also other models, such as projectiles and items
settings-player-names = Show players' names over them
settings-show-position = Show position and view angles
settings-speedometer = Show the speedometer
settings-jump-speed = Show the last jump speed
settings-jump-speed-hint = How fast the recorder was going as they last left the ground, shown under the speedometer until their next jump.
settings-recorder-fps = Show the recorder's fps
settings-recorder-fps-hint = The frames a second the game ran at for whoever recorded the demo, at the top left, averaged over every half second. Not how fast this app draws it.
settings-speed-axes = Speed counts
settings-speed-horizontal = Horizontal
settings-speed-vertical = Vertical
settings-speed-both = Both (3D)
viewer-jump-speed = Jump { $speed }
settings-show-position-hint = Where the picture is seen from, in the game's units, and the way it looks, in degrees up or down (pitch) and round (yaw). With the free camera on they are the free camera's. Useful to tell someone exactly where to look.
viewer-position-line = Position { $x } { $y } { $z }
viewer-fps = { $fps } fps
viewer-angles-line = Angles { $pitch } { $yaw }
settings-player-names-hint = Each player in view has their name over their head, in their team's colour, from the free camera too. Players behind walls show theirs only while players are seen through walls.
settings-see-through-hint = Players behind walls show as silhouettes in their top colour, which team games set by team. Only what the demo holds can show. The server sends the players near the recorder, and servers that hide players the recorder cannot see leave nothing to show.
settings-section-decals = Decals
settings-decals = Most kept
settings-decals-hint = Bullet holes, blood and other marks left during the game. Only the newest ones stay, and the oldest go as new ones are made. Marks that are part of the map always stay.
settings-restore = Restore defaults…
settings-restore-title = Restore defaults
settings-restore-list = These settings go back to how they were at first. The Half-Life folder and the library folders stay as they are.
settings-restore-none = Every setting is already at its default.
settings-restore-columns = Library columns shown
settings-restore-layout = Open panels and their sizes
settings-clear-library = Clear library…
settings-clear-library-title = Clear library
settings-clear-library-text = { $count ->
    [one] Lambda Slicer forgets what it found in { $count } demo and scans the library folders again, which can take a while. Your demo files, clips and settings stay as they are.
   *[other] Lambda Slicer forgets what it found in { $count } demos and scans the library folders again, which can take a while. Your demo files, clips and settings stay as they are.
}
viewer-zoom = Zoom in
viewer-free-camera = Free camera
viewer-free-camera-help = Fly around the map apart from the recording player. { $forward }, { $left }, { $back } and { $right } move, { $up } goes up, { $down } goes down and { $fast } goes faster. Hold the right mouse button and drag to look around.
viewer-fullscreen = Full screen
viewer-leave-fullscreen = Leave full screen
viewer-see-through = See through walls
viewer-player-names = Player names
viewer-fullbright = Light up everything
viewer-fullbright-help = Shows the whole map fully lit, so dark corners and the players in them are easy to see.
viewer-paused = Paused
viewer-map-change = The server is changing the map to { $map }
viewer-pov-recorder = { $name } (recorder)
viewer-pov-recorder-unnamed = Recorder
viewer-pov-help = Whose eyes to watch through, the recorder or this map's best players by score, each with its key beside it. Previous player: { $previous }. Next player: { $next }.
viewer-scoreboard-ping-loss = Ping/loss
viewer-scoreboard-bot = BOT
viewer-scoreboard-players = { $count ->
    [one] ({ $count } player)
   *[other] ({ $count } players)
}
viewer-pov-unseen = The demo doesn't show { $name } here, so you're seeing the recorder's view
settings-events-leave-free = Leave the free camera when choosing an event
settings-events-leave-free-hint = Choosing a kill shows it through the killer's eyes, or the victim's, or the recorder's. With this off, the free camera stays where it is, and turning it off goes to that view.
settings-particles = Show particles
settings-particles-hint = Sparks, the bits flying out of explosions and the dust where bullets hit.
settings-export-beside = Beside each demo
settings-export-hint = Inside your mod folder, the one with liblist.gam such as Half-Life\ag, clips play with a single playdemo command.
settings-file-names-hint = An exported clip is named after its demo and its start and end times, unless you use the clip's name. The game can only open files named with English letters, numbers, - and ., so anything else becomes _. A clip name with none of these is named by its times instead.

clips-store-moved = The clips file { $path } could not be read, so it was kept as { $moved } and clips start empty. { $detail }
clips-store-unreadable = The clips file { $path } could not be read: { $detail }. Until Lambda Slicer is closed, clips aren't saved, so nothing in the file is lost.
clips-save-failed = Clips could not be saved to { $path }: { $detail }
notice-dismiss = Dismiss

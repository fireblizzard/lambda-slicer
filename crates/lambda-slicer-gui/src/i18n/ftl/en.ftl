# English, the source catalog. Every other catalog has the same messages
# with the same arguments

app-title = Lambda Slicer

## Toolbar

toolbar-open = Open demo…
toolbar-add-folder = Add folder…
toolbar-rescan = Rescan
toolbar-cancel = Cancel
toolbar-settings = Settings
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
export-missing-parent = The folder for { $path } does not exist.
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
timeline-fit = Fit
show-shots = Shots
show-actions = Other actions
show-kills = Kills
show-switches = Weapon switches
lane-unknown = Unknown player
lane-recorder = { $name } (recorder)
column-time = Time
column-player = Player
column-weapon = Weapon
column-event = Event
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
timeline-windows = Most intense moments
timeline-window = #{ $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Final scores, at the last kill { $time }
scores-at = Scores at { $time }
scores-show-final = Show final scores
scores-frags = Frags
scores-deaths = Deaths

show-timeline = Show timeline
legend-shot = Shot
legend-charge = Charging
legend-kill = Kill
legend-death = Death
legend-switch = Weapon switch
timeline-help = Each row is a player. Scroll to zoom, drag to move, double-click to see everything, click a mark to select it.

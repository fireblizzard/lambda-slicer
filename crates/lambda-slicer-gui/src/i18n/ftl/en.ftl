# English, the source catalog. Every other catalog has the same messages
# with the same arguments

app-title = Lambda Slicer

toolbar-open = Open demo…
toolbar-add-folder = Add folder…
toolbar-rescan = Rescan
toolbar-cancel = Cancel
toolbar-settings = Settings

settings-title = Settings
settings-language = Language
settings-unreviewed = This translation is a draft that no native speaker has reviewed yet.
settings-workers = Analysis workers
settings-workers-hint = More workers analyse faster but use more processor time.
settings-cache = Library cache
settings-folders = Library folders
settings-folders-empty = No folders added yet.
settings-remove = Remove
settings-close = Close

library-empty = Add a folder or open a demo to get started.
library-demos = { $count ->
    [one] { $count } demo
   *[other] { $count } demos
}
library-kills = { $count ->
    [one] { $count } kill
   *[other] { $count } kills
}
library-action = Action score { NUMBER($score, minimumFractionDigits: 2) }
library-spectating = Spectating { $percent }%

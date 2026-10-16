# Deutsch. Ungeprüfter Entwurf

app-title = Lambda Slicer

toolbar-open = Demo öffnen…
toolbar-add-folder = Ordner hinzufügen…
toolbar-rescan = Neu scannen
toolbar-cancel = Abbrechen
toolbar-settings = Einstellungen

settings-title = Einstellungen
settings-language = Sprache
settings-unreviewed = Diese Übersetzung ist ein Entwurf, den noch keine Muttersprachlerin und kein Muttersprachler geprüft hat.
settings-workers = Analyse-Threads
settings-workers-hint = Mehr Threads analysieren schneller, belasten aber den Prozessor stärker.
settings-cache = Bibliothekscache
settings-folders = Bibliotheksordner
settings-folders-empty = Noch keine Ordner hinzugefügt.
settings-remove = Entfernen
settings-close = Schließen

library-empty = Füge einen Ordner hinzu oder öffne eine Demo, um loszulegen.
library-demos = { $count ->
    [one] { $count } Demo
   *[other] { $count } Demos
}
library-kills = { $count ->
    [one] { $count } Abschuss
   *[other] { $count } Abschüsse
}
library-action = Action-Wert { NUMBER($score, minimumFractionDigits: 2) }
library-spectating = Zuschauer { $percent } %

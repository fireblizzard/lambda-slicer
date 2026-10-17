# Deutsch. Ungeprüfter Entwurf

app-title = Lambda Slicer

## Werkzeugleiste

toolbar-open = Demo öffnen…
toolbar-add-folder = Ordner hinzufügen…
toolbar-rescan = Neu scannen
toolbar-cancel = Abbrechen
toolbar-settings = Einstellungen
dialog-demos = Half-Life-Demos

## Einstellungen

settings-title = Einstellungen
settings-language = Sprache
settings-unreviewed = Diese Übersetzung ist ein Entwurf, den noch keine Muttersprachlerin und kein Muttersprachler geprüft hat.
settings-workers = Analyse-Threads
settings-workers-hint = Mehr Threads analysieren schneller, belasten aber den Prozessor stärker.
settings-cache = Bibliothekscache
settings-folders = Bibliotheksordner
settings-folders-empty = Noch keine Ordner hinzugefügt.
settings-remove = Entfernen

## Bibliothek

library-empty = Füge einen Ordner hinzu oder öffne eine Demo, um loszulegen.
library-no-matches = Keine Demo passt zu diesen Filtern.
library-no-cache = Auf diesem System gibt es keinen Ort für den Bibliothekscache.
library-demos = { $count ->
    [one] { $count } Demo
   *[other] { $count } Demos
}
library-percent = { $percent } %

filter-search = Pfade durchsuchen
filter-map = Map
filter-player = Spieler
filter-weapon-any = Beliebige Waffe
filter-min-minutes = Mindestens (Minuten)
filter-spectating-any = Spielend oder zuschauend
filter-spectating-mostly = Meist zuschauend
filter-spectating-playing = Meist spielend
filter-clear = Filter zurücksetzen

column-status = Status
column-name = Demo
column-map = Map
column-duration = Länge
column-gauss = Gauss
column-kills = Abschüsse
column-action = Intensität
column-spectating = Zuschauer

status-complete = Analysiert
status-partial = Teilweise
status-failed = Fehlgeschlagen
status-unsupported = Nicht unterstützt
status-no-use = Kein Waffeneinsatz

kind-multiplayer-view = Spieleransicht
kind-singleplayer = Einzelspieler
kind-hltv = Zuschauer-Proxy
kind-unknown = Unbekannte Art

details-recorder = Aufgenommen von { $name }
details-shots = { $count ->
    [one] { $count } Schuss
   *[other] { $count } Schüsse
}
details-kills = { $count ->
    [one] { $count } Abschuss
   *[other] { $count } Abschüsse
}
details-action = Höchste Intensität { NUMBER($score, minimumFractionDigits: 2) }

## Scannen

scan-progress = Gefunden { $found }, fertig { $done } ({ $cached } aus dem Cache, { $analysed } analysiert), { $megabytes } MB gelesen
scan-cancelling = Wird angehalten…
scan-cancelled = Angehalten, { $left } noch nicht gescannt.
scan-finished = Fertig: { $done } Demos, { $analysed } analysiert, in { $seconds } s.
scan-problems = { $count ->
    [one] { $count } Problem
   *[other] { $count } Probleme
}
problems-title = Probleme beim Scannen
problem-changed = Die Datei hat sich während der Analyse geändert, daher wurde nichts gespeichert.
drop-hint = Demos oder Ordner hier ablegen, um sie hinzuzufügen
drop-not-demo = Keine Demo: { $names }

## Meldungen des Analysekerns nach Code

analysis-panicked = Die Analyse wurde durch einen internen Fehler angehalten: { $detail }
cache-cancelled = Die Analyse wurde abgebrochen, daher wurde nichts gespeichert.
cache-database = Der Bibliothekscache kann nicht verwendet werden: { $detail }
cache-directory = Der Cache-Ordner kann nicht angelegt werden: { $detail }
container-bad-magic = Das ist keine Half-Life-Demo.
container-frame-bad-time = Die Aufnahme ist bei Frame { $frame } von Teil { $entry } beschädigt, der eine ungültige Zeit hat.
container-frame-missing-terminal = Teil { $entry } der Aufnahme endet ohne seinen Abschlussdatensatz.
container-frame-negative-length = Die Aufnahme ist bei Frame { $frame } von Teil { $entry } beschädigt.
container-frame-over-limit = Frame { $frame } von Teil { $entry } ist größer als erlaubt.
container-frame-truncated = Die Aufnahme bricht bei Frame { $frame } von Teil { $entry } ab.
container-frame-unknown-tag = Die Aufnahme ist bei Frame { $frame } von Teil { $entry } beschädigt, der einen unbekannten Typ hat.
container-io = Die Datei kann nicht gelesen werden: { $detail }
container-no-directory = Die Aufnahme wurde nie abgeschlossen und hat daher kein Inhaltsverzeichnis.
container-range = Das Inhaltsverzeichnis der Aufnahme zeigt außerhalb der Datei.
container-too-short = Die Datei ist zu kurz für eine Demo.
coverage-hltv = Aufnahme eines Zuschauer-Proxys. Es gibt keinen aufnehmenden Spieler, und die Ereignisse folgen der Sicht des Proxys.
coverage-mostly-spectating = Die aufnehmende Person hat { $percent } % der Wiedergabe zugeschaut, daher stammen die meisten Ereignisse von anderen Spielern, so wie die beobachtete Ansicht sie zeigte.
coverage-no-local-events = Keine lokalen Ereignisse. Die eigenen Aktionen der aufnehmenden Person fehlen, außer denen, die der Server geschickt hat.
coverage-singleplayer = Einzelspieler. Die anderen Akteure sind Monster oder Objekte der Map, und es gibt keine Abschussliste.
coverage-unknown = Die Aufnahme wurde nicht weit genug dekodiert, um ihre Art zu bestimmen.
coverage-visible-only = Ereignisse anderer Spieler erreichen diesen Client nur, solange der Server sie als sichtbar zählt, daher sind ihre Zahlen Untergrenzen.
decode-failed = Die Dekodierung hielt bei { $time } an (Frame { $frame }): { $detail }
export-active-effect = { $effect } läuft zu Beginn noch und kann noch nicht übernommen werden.
export-client-dropped-packets = Das Spiel hat bei { $time } verspätete Pakete verworfen, was ein Schnitt noch nicht nachbilden kann.
export-cross-segment = Bei { $time } beginnt innerhalb des Bereichs eine neue Sitzung.
export-destination-exists = { $path } existiert bereits. Wähle einen anderen Namen.
export-empty-range = Der gewählte Bereich enthält nichts zum Exportieren.
export-io = { $path } kann nicht geschrieben werden: { $detail }
export-missing-delta-base = Der Aufnahme fehlt ein Zustand, der vor dem Beginn nötig ist (Sequenz { $sequence }).
export-missing-parent = Der Ordner { $path } existiert nicht.
export-range-beyond-end = Das Ende { $end } liegt nach dem Ende der Aufnahme, { $segment_end }.
export-same-as-source = Das Ziel ist die Quelldemo selbst.
export-source-changed = { $path } hat sich während des Exports geändert.
export-source-decode-failed = Die Quelldemo lässt sich nicht dekodieren: { $detail }
export-source-failed-crosscheck = Die Quelldemo hat ihre Gegenprüfung nicht bestanden, daher ist ihr rekonstruierter Zustand nicht vertrauenswürdig.
export-unknown-persistent-message = { $name } vor dem Beginn setzt einen Zustand, den ein Schnitt noch nicht übernehmen kann.
export-unproven-state = Der Zustand bei { $time } beruht auf einer erschlossenen Basis und ist nicht belegt.
export-unrepresentable-time-field = Eine Zeit am Anfang kann nicht originalgetreu geschrieben werden: { $detail }
export-unsupported = Nicht unterstützt: { $detail }
export-validation-failed = Die exportierte Datei hat die Prüfung nicht bestanden: { $detail }
io-not-found = Die Datei oder der Ordner wurde nicht gefunden.
io-other = { $detail }
io-permission-denied = Zugriff verweigert.
io-unexpected-end = Die Datei endete unerwartet.
protocol-unsupported = Das Demo-Protokoll { $demo }/{ $network } wird nicht unterstützt, nur 5/48.
protocol-unsupported-network = Der Server hat auf das nicht unterstützte Netzwerkprotokoll { $network } gewechselt.

## Demoansicht

demo-close = Schließen
demo-loading = { $name } wird analysiert…
timeline-fit = Einpassen
show-shots = Schüsse
show-actions = Andere Aktionen
show-kills = Abschüsse
show-switches = Waffenwechsel
lane-unknown = Unbekannter Spieler
lane-recorder = { $name } (Aufnahme)
column-time = Zeit
column-player = Spieler
column-weapon = Waffe
column-event = Ereignis
event-kill = Hat { $victim } abgeschossen
event-switch = Waffenwechsel
action-swing = Schlag
action-fire = Schuss
action-launch = Abschuss
action-charge-activity = Aufladen
action-charge-stop = Ladeende
action-beam-start = Strahlbeginn
action-beam-sustain = Strahl
action-beam-stop = Strahlende
action-placement-attempt = Platzierungsversuch
action-release-attempt = Auslöseversuch
mode-primary = primär
mode-charged = aufgeladen
mode-zoomed = mit Zoom
mode-unzoomed = ohne Zoom
mode-grenade = Granate
mode-bullets = Kugeln
mode-single = einzeln
mode-double = doppelt
mode-rapid = schnell
demo-events = { $count ->
    [one] { $count } Ereignis
   *[other] { $count } Ereignisse
}

column-date = Datum
column-score = Ergebnis
filter-from = Von
filter-to = Bis
date-from-name = Datum aus dem Dateinamen
date-from-modified = Datum der letzten Änderung der Datei, das sich beim Kopieren oder Herunterladen ändern kann
timeline-best-action = Goldene Minute
timeline-windows = Intensivste Momente
timeline-window = Nr. { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Endstand beim letzten Abschuss { $time }
scores-at = Stand bei { $time }
scores-show-final = Endstand zeigen
scores-frags = Frags
scores-deaths = Tode

show-timeline = Zeitleiste zeigen
legend-shot = Schuss
legend-charge = Aufladen
legend-kill = Abschuss
legend-death = Tod
legend-switch = Waffenwechsel
timeline-help = Jede Zeile ist ein Spieler. Scrollen zoomt, Ziehen verschiebt, Doppelklick zeigt alles, ein Klick auf eine Markierung wählt sie aus.

column-headshots = Kopfschüsse
filter-headshots = Abschüsse per Kopfschuss
event-headshot = Kopfschuss
scores-headshots = KS

column-accuracy = Treffsicherheit
filter-min-accuracy = Treffsicherheit mindestens (%)
scores-accuracy = Treff.

help-headshots = Abschüsse durch einen Treffer am Kopf, erkannt an der Todesanimation des Opfers. Explosionen zählen nicht. Gezeigt werden nur die der aufnehmenden Person, weil andere Spieler oft außerhalb ihres Blickfelds abschießen. Im Duell auch die des Gegners, da alle seine Abschüsse die aufnehmende Person treffen.
help-headshots-column = Abschüsse der aufnehmenden Person durch einen Treffer am Kopf, erkannt an der Todesanimation des Opfers. Explosionen zählen nicht. Abschüsse anderer Spieler zählen nicht, weil sie oft außerhalb des Blickfelds der Aufnahme geschehen.
help-accuracy = Der Anteil der Schüsse, die trafen, anhand des Bluts, das ein Treffer entlang der Schusslinie hinterlässt, abzüglich zufälliger Übereinstimmungen. Er umfasst Glock, Python, MP5-Kugeln, Schrotflinte, Armbrust mit Zoom und direkte Treffer des Gauss-Strahls. Explosionen und Gauss-Reflexionen hinterlassen kein Blut und zählen nicht. Nur für die aufnehmende Person und erst ab 50 Schüssen.
help-headshot-filter = Demos mit mindestens einem Kopfschuss-Abschuss, den die Aufnahme zeigt, von beliebigen Spielern.
help-accuracy-filter = Demos, in denen die aufnehmende Person mindestens diesen Anteil ihrer Schüsse traf.

clips-title = Clips
clips-add-event = Clip { $start } – { $end }
clips-add-event-none = Clip vom gewählten Ereignis
clips-select-event = Wähle zuerst ein Ereignis in der Liste oder auf der Zeitleiste.
clips-other-session = Vorerst lässt sich nur die erste Sitzung der Aufnahme exportieren.
clips-add-window = Clip von diesem intensiven Moment
clips-empty = Noch keine Clips. Wähle ein Ereignis oder einen der intensivsten Momente und lege einen Clip darum an. Wie viel ein Clip vor und nach einem Ereignis enthält, stellst du in den Einstellungen ein.
clip-default-name = Clip { $number }
clip-name = Name
clip-length = Länge { $duration }
clip-show = Zeigen
clip-remove = Entfernen
clip-time-help = Sekunden (96), Minuten und Sekunden (1:36) oder Stunden, Minuten und Sekunden (00:01:36.250)
clip-exact = Der Export beginnt und endet genau hier.
clip-effective = Der Export schneidet { $start } – { $end } ({ $duration }), wie die Frames der Aufnahme fallen.
timeline-clip-help = Clips sind die hellblauen Bänder. Ziehe den Rand eines Bandes, um ihn zu verschieben.
settings-clip-padding = Puffer um Clips
settings-preroll = Sekunden vor dem Ereignis
settings-postroll = Sekunden nach dem Ereignis
export-start-first-group = Ein Schnitt kann nicht auf dem allerersten Frame der Aufnahme beginnen. Beginne etwas später.
export-start-later-session = Der Anfang liegt in einer späteren Sitzung der Aufnahme, in der ein Schnitt noch nicht beginnen kann.

export-all = Alle Clips exportieren
export-one = Exportieren
export-cancel = Export stoppen
export-folder = Exporte landen in { $folder }.
export-folder-beside = Exporte landen neben der Demo, in { $folder }.
export-waiting = Wartet auf den Export…
export-running = Wird exportiert… Dabei wird die ganze Demo gelesen, das kann dauern.
export-cancelled = Nicht exportiert, da der Export gestoppt wurde.
export-saved = Gespeichert unter { $path }. Mit dem Original verglichene Snapshots: { $count }.
export-playdemo = In der Spielkonsole:
export-copy-first = Kopiere { $name } in deinen Mod-Ordner, den mit liblist.gam, dann in der Spielkonsole:
export-copy = Kopieren
settings-export-folder = Exportordner
settings-export-choose = Wählen…
settings-export-beside = Neben jeder Demo
settings-export-hint = Im Mod-Ordner, dem mit liblist.gam wie Half-Life\ag, spielen Clips mit einem einzigen playdemo-Befehl.

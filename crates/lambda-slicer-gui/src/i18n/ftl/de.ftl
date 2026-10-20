# Deutsch. Ungeprüfter Entwurf

app-title = Lambda Slicer

## Werkzeugleiste

toolbar-open = Demo öffnen…
toolbar-add-folder = Ordner hinzufügen…
toolbar-rescan = Neu scannen
toolbar-cancel = Abbrechen
toolbar-settings = Einstellungen
toolbar-rescan-busy = Ein Scan läuft bereits.
toolbar-rescan-nothing = Füge einen Ordner hinzu oder öffne eine Demo zum Scannen.
toolbar-cancel-idle = Es läuft kein Scan.
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
library-demos-of = { $shown } von { $total ->
    [one] { $total } Demo
   *[other] { $total } Demos
}
library-total = { $count ->
    [one] { $count } Demo in der Bibliothek
   *[other] { $count } Demos in der Bibliothek
}
filters-title = Filter
filters-find = Suchen
filters-pov = Perspektive
filters-quality = Qualität
filters-recorded = Aufgenommen
pov-any = Alle
pov-playing = Spielend
pov-spectating = Zuschauend
help-pov = „Spielend“ heißt, die Aufnahme war höchstens die Hälfte der Wiedergabe im Zuschauermodus, „Zuschauend“ heißt mehr als die Hälfte.
columns-button = Spalten
list-back = Zurück zur Bibliothekstabelle
sort-by = Sortieren nach
sort-ascending = Aufsteigend
sort-descending = Absteigend
score-more-sides = { $count ->
    [one] { $count } weitere Seite
   *[other] { $count } weitere Seiten
}
intensity-help = Intensität { $score }. Der Balken vergleicht sie mit der höchsten der Bibliothek, { $max }.
value-unknown = Unbekannt
path-copy = Pfad kopieren
path-show = Im Ordner zeigen
path-show-failed = Der Ordner ließ sich nicht öffnen ({ $detail }). Kopiere stattdessen den Pfad.

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
show-shots = Schüsse
show-actions = Andere Aktionen
show-kills = Abschüsse
show-switches = Waffenwechsel
lane-unknown = Unbekannter Spieler
lane-recorder = { $name } (Aufnahme)
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
timeline-window = Nr. { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Endstand beim letzten Abschuss { $time }
scores-at = Stand bei { $time }
scores-show-final = Endstand zeigen
scores-frags = Frags
scores-deaths = Tode

viewer-play = Abspielen
viewer-pause = Pause
viewer-volume = Lautstärke
viewer-no-sound = Kein Ton
viewer-play-backward = Zurückspulen
viewer-step-back = Vorheriger Frame
viewer-step-forward = Nächster Frame
viewer-rate = { $rate }×
viewer-shortcut = Tastenkürzel: { $key }
key-space = Leertaste
viewer-map-failed = Die Karte { $map } konnte nicht geladen werden
viewer-stopped = Die Wiedergabe hat vorzeitig angehalten: { $reason }
viewer-missing = Dateien, die nicht gelesen werden konnten: { $count }
event-previous = Vorheriges Ereignis
event-next = Nächstes Ereignis
event-play = Ereignis abspielen
timeline-title = Zeitleiste
timeline-lanes = Spielerzeilen
timeline-zoom-in = Vergrößern
timeline-zoom-out = Verkleinern
timeline-fit-all = Ganze Demo einpassen
timeline-gestures = Klicke auf eine Markierung, um ihr Ereignis zu wählen, oder irgendwo anders, um dorthin zu springen. Ziehe das Lineal oder den Abspielkopf zum Spulen, und ziehe anderswo, um die Ansicht zu verschieben. Scrolle, um um den Mauszeiger herum zu zoomen, und doppelklicke, um die ganze Demo zu sehen. Ziehe den Rand eines Clips, um ihn zu verschieben, oder klicke auf den Clip, um ihn zu bearbeiten.
timeline-density = Die schattierten Balken zeigen, wie viele der angezeigten Ereignisse in jeden Abschnitt fallen, gemessen am dichtesten Abschnitt der Demo.
timeline-density-count = { $count ->
    [one] Ein angezeigtes Ereignis hier
   *[other] { $count } angezeigte Ereignisse hier
}
timeline-session-count = Sitzung { $session }: { $count }
timeline-choose = Ereignisse an dieser Stelle
legend-clip = Clip
event-session = Sitzung { $session }
event-unreachable = Dieses Ereignis liegt in einer späteren Sitzung, die dieselben Zeiten erneut verwendet, daher kann die Demo-Ansicht nicht dorthin springen.
event-hidden = Das ausgewählte Ereignis ist durch die Filter ausgeblendet.
highlight-jump = Zu Highlight springen…
highlight-help = Highlights bewerten die ganze Demo nach ihrer Action, egal was die Filter zeigen.
kills-with = Abschüsse mit
refine-midair = Midair
refine-speed = Speed
refine-wallgauss = Wallgauss
refine-clear = Zurücksetzen
refine-airborne = In der Luft
refine-either = Beliebig
midair-both = Beide
midair-one = Nur einer
help-midair-either = Der Schütze, das Opfer oder beide waren in der Luft. Der Schütze wird beim mit dem Abschuss verknüpften Schuss geprüft, ohne einen solchen beim Abschuss, und das Opfer beim Tod.
help-midair-both = Schütze und Opfer waren beide in der Luft, jeweils zum Zeitpunkt der Prüfung.
help-midair-one = Genau einer von beiden war in der Luft, und vom anderen ist bekannt, dass er am Boden war.
speed-at-least = Mindestens
speed-unit = ups
speed-killer = Schütze
speed-victim = Opfer
help-speed = Horizontale Geschwindigkeit in Units pro Sekunde, parallel zum Boden. Der Schütze wird beim mit dem Abschuss verknüpften Schuss gemessen, ohne einen solchen beim Abschuss, und das Opfer beim Tod. Die Geschwindigkeiten anderer Spieler sind Schätzungen.
wallgauss-weapon = Wallgauss betrifft nur Gauss-Abschüsse, und der Waffenfilter zeigt { $weapon }.
wallgauss-checking = Wallgauss wird geprüft… { $examined } von { $total } Abschüssen
wallgauss-needs-files = Wallgauss braucht die Spieldateien, um Schüsse durch die Karte zu verfolgen.
wallgauss-failed = Wallgauss konnte nicht geprüft werden. { $reason }
help-wallgauss = Gauss-Abschüsse durch eine Wand, ermittelt, indem jeder Schuss wie auf dem Server durch die Karte verfolgt wird. Ein Abschuss, der sich nicht klären lässt, etwa weil das Opfer außerhalb des Blickfelds der Aufnahme war, gilt als unbekannt und wird ausgelassen.
badge-wall = WALL
wall-yes-exit = Wallgauss per Rekonstruktion erkannt: die Explosion, wo der Strahl eine { $thickness } Units dicke Wand verließ.
wall-yes-beyond = Wallgauss per Rekonstruktion erkannt: der Strahl, nachdem er eine { $thickness } Units dicke Wand durchdrungen hatte.
wall-limits = Aus dem Schuss und dem Snapshot rekonstruiert, nicht vom Server aufgezeichnet. Spieler werden als Boxen verfolgt, und das Opfer war womöglich einige Dutzend Units von der Stelle entfernt, die der Snapshot zeigt.
wall-no-direct = Kein Wallgauss: Der Strahl hätte das Opfer vor jeder Wand treffen können.
wall-no-reflection = Kein Wallgauss: Die Explosion einer Reflexion hätte das Opfer vor jeder Wand treffen können.
wall-doubt-no-shot = Wallgauss unbekannt: Der Schuss des Schützen ist nicht in der Aufnahme.
wall-doubt-no-map = Wallgauss unbekannt: Die Dateien der Karte konnten nicht gelesen werden.
wall-doubt-unseen = Wallgauss unbekannt: Das Opfer war außerhalb des Blickfelds der Aufnahme.
wall-doubt-brush = Wallgauss unbekannt: Der Strahl traf eine Tür, eine Plattform oder eine andere Brush-Entity, die nicht verfolgt wird.
wall-doubt-ambiguous = Wallgauss unbekannt: Der Schuss hätte sowohl durch eine Wand als auch ohne töten können.
wall-doubt-unexplained = Wallgauss unbekannt: Es wurde kein Weg gefunden, wie der Schuss das Opfer erreichte.
wall-doubt-unconfirmed = Wallgauss unbekannt: Der Schuss kann das Opfer nur durch eine Wand erreicht haben, doch das ist nicht sicher.
wall-doubt-not-reached = Wallgauss unbekannt: Die Aufnahme endet vor dem Schuss.
wall-pending = Wallgauss noch nicht geprüft.
events-unknown = { $count ->
    [one] Einem Abschuss fehlen die Daten zur Beurteilung
   *[other] { $count } Abschüssen fehlen die Daten zur Beurteilung
}
help-unknown = Bewegung, die die Aufnahme nicht sehen konnte, zählt als unbekannt, nie als am Boden oder stillstehend, daher werden diese Abschüsse ausgelassen.
clip-it = Clippen
badge-headshot = KS
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = Suizid
badge-teamkill = Teamkill
badge-world = Welt
air-both = Beide waren bei der Prüfung in der Luft.
air-killer = Der Schütze war bei der Prüfung in der Luft.
air-victim = Das Opfer war bei der Prüfung in der Luft.
air-other-unknown = Ob der andere es auch war, ist unbekannt.
speed-horizontal = Horizontale Geschwindigkeit, parallel zum Boden.
motion-line = { $who }: { $air }, { $speed }. { $how }, { $when }.
motion-who-killer = Schütze
motion-who-victim = Opfer
motion-air-yes = in der Luft
motion-air-no = am Boden
motion-air-unknown = unklar, ob in der Luft
motion-speed-unknown = Geschwindigkeit unbekannt
motion-exact = Vom Spiel aufgezeichnet
motion-trajectory = Aus der Bewegung geschätzt
motion-unclear = Bewegung unklar
motion-not-visible = Außer Sicht
motion-at-shot = beim verknüpften Schuss
motion-at-kill = beim Tod
motion-none = Die Analyse hat keine Bewegungsdaten für diesen Abschuss.
legend-shot = Schuss
legend-charge = Aufladen
legend-kill = Abschuss
legend-death = Tod
legend-switch = Waffenwechsel

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

clips-new = Neuer Clip
clips-select-event = Wähle zuerst ein Ereignis in der Liste oder auf der Zeitleiste.
clips-other-session = Hier kann kein Clip beginnen, da eine frühere Sitzung der Aufnahme dieselben Zeiten hat.
clips-add-window = Clip von diesem intensiven Moment
clip-default-name = Clip { $number }
clip-name = Name
clip-length = Länge { $duration }
clip-remove = Entfernen
clip-loop = Schleife
clip-time-help = Sekunden (96), Minuten und Sekunden (1:36) oder Stunden, Minuten und Sekunden (00:01:36.250)
clips-empty-hint = Noch keine Clips. Wähle ein Ereignis und drücke Clippen, oder lege mit Neuer Clip einen am Abspielkopf an.
clips-new-help = Ein Clip von zehn Sekunden ab dem Abspielkopf.
clip-in = In
clip-out = Out
clip-set-in = Anfang auf den Abspielkopf setzen
clip-set-out = Ende auf den Abspielkopf setzen
clip-no-playhead = Es gibt keinen Abspielkopf, von dem eine Zeit kommen könnte.
clip-play = Clip einmal abspielen
clip-show-on-timeline = Auf der Zeitleiste zeigen
export-destination = Exportordner ändern
export-skipped = Übersprungen. { $reason }
export-earlier-range = Dieser Export umfasst { $start } – { $end }, den Bereich vor der Bearbeitung des Clips.
scores-player = Spieler
scores-none = Die Aufnahme enthält keinen Punktestand.
clip-effective = Der Export schneidet { $start } – { $end } ({ $duration }), wie die Frames der Aufnahme fallen.
settings-clip-padding = Puffer um Clips
settings-preroll = Sekunden vor dem Ereignis
settings-postroll = Sekunden nach dem Ereignis
export-start-first-group = Ein Schnitt kann nicht auf dem allerersten Frame der Aufnahme beginnen. Beginne etwas später.
export-start-during-loading = Der Anfang fällt in das Laden einer Map. Beginne etwas später.

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
settings-viewer = Demo-Ansicht
settings-viewer-game = Half-Life-Ordner
settings-viewer-game-none = Nicht gewählt
settings-viewer-game-hint = Der Ordner mit valve. Die Demo-Ansicht spielt Demos mit den Karten, Modellen und Sounds des Spiels daraus.
settings-executable = Spieldatei
settings-executable-hint = Findet den Half-Life-Ordner. Die Demo-Ansicht liest die Dateien des Spiels selbst und startet das Spiel nie.
settings-executable-not-set = Nicht gesetzt, daher zeigt die Demo-Ansicht weder Bild noch Ton
settings-executable-optional = Nicht gesetzt und nicht nötig, solange der Half-Life-Ordner das Spiel enthält
settings-executable-missing = { $path } ist nicht mehr da. Die Demo-Ansicht liest weiter den Half-Life-Ordner.
settings-game-override = Dieser Lauf nutzt { $path }, angegeben mit --game. Eine Wahl hier gilt ab dem nächsten Lauf.
settings-browse = Durchsuchen…
dialog-executables = Programme
setup-title = Demo-Ansicht einrichten
setup-body = Bild und Ton brauchen die Spieldateien von Half-Life. Zeig Lambda Slicer, wo das Spiel installiert ist, dann liest es sie von dort.
setup-locate-exe = hl.exe suchen…
setup-locate-folder = Half-Life suchen…
setup-settings = Einstellungen
setup-still-works = Zeitleiste, Ereignisse und Clips funktionieren auch ohne sie.
setup-relocate = { $folder } enthält Half-Life nicht mehr. Such es erneut.
setup-not-executable = { $name } ist nicht { $executable }. Wähle die Spieldatei selbst, keine Verknüpfung.
setup-not-installed = { $folder } enthält keine Half-Life-Installation mit valve-Ordner.
tab-events = Ereignisse
tab-clips = Clips
tab-scoreboard = Punktestand
list-hide = Demoliste ausblenden
list-show = Demoliste einblenden
demo-details = Details
viewer-start = Zurück zum Anfang
viewer-mute = Stumm
viewer-unmute = Ton an
viewer-picture = Bild- und Ansichtseinstellungen
viewer-needs-files = Braucht die Spieldateien
viewer-no-sound-why = Kein Ton: { $reason }
settings-tab-general = Allgemein
settings-section-folders = Ordner
settings-folders-hint = Die Ordner, die die Bibliothek nach Demos durchsucht, Unterordner eingeschlossen.
settings-clip-padding-hint = Wie viel ein Clip um ein Ereignis davor und danach behält.
settings-section-picture = Bild
settings-section-sound = Ton
settings-gamma = Gamma
settings-brightness = Helligkeit
settings-texgamma = Textur-Gamma
settings-lightgamma = Licht-Gamma
settings-overbright = Überhelle Beleuchtung
settings-gamma-hint = Die gleichnamigen Einstellungen des Spiels (gamma, brightness, texgamma, lightgamma, gl_overbright). Stelle sie wie im Spiel ein, um dieselbe Helligkeit zu sehen.
settings-gamma-defaults = Standardwerte des Spiels
settings-fov-recorded = Sichtfeld wie aufgenommen
settings-fov = Sichtfeld
settings-viewmodel = Waffe in der Hand zeigen
settings-see-through = Spieler durch Wände sehen
settings-see-through-items = Auch andere Modelle, etwa Geschosse und Gegenstände
settings-see-through-hint = Spieler hinter Wänden erscheinen als Umrisse in ihrer oberen Farbe, die Teamspiele nach Team setzen. Es kann nur zu sehen sein, was die Demo enthält. Der Server schickt die Spieler in der Nähe des Aufnehmenden, und Server, die Spieler verbergen, die der Aufnehmende nicht sehen kann, lassen nichts zu zeigen.
viewer-see-through = Durch Wände sehen
settings-export-beside = Neben jeder Demo
settings-export-hint = Im Mod-Ordner, dem mit liblist.gam wie Half-Life\ag, spielen Clips mit einem einzigen playdemo-Befehl.

clips-store-moved = Die Clip-Datei { $path } konnte nicht gelesen werden. Sie wurde als { $moved } aufbewahrt, und die Clips beginnen leer. { $detail }
clips-store-unreadable = Die Clip-Datei { $path } konnte nicht gelesen werden: { $detail }. Clips werden in dieser Sitzung nicht gespeichert, damit nichts daraus verloren geht.
clips-save-failed = Clips konnten nicht in { $path } gespeichert werden: { $detail }
notice-dismiss = Schließen

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
settings-workers = Gleichzeitig analysierte Demos
settings-workers-hint = Mehr gleichzeitig zu analysieren geht schneller, lässt den Computer aber härter arbeiten.
settings-cache = Speicherort der Bibliothek
settings-start-tab = Tab beim Start
settings-start-tab-hint = Der Tab der Bibliothek, der bei jedem Start von Lambda Slicer angezeigt wird. Ein anderer Tab, auf den du klickst, bleibt trotzdem offen, bis du Lambda Slicer schließt.
settings-sort = Reihenfolge beim Start
settings-sort-hint = Wie die Demos bei jedem Start von Lambda Slicer sortiert werden. Ein Klick auf die Überschrift einer Spalte sortiert trotzdem danach, bis du Lambda Slicer schließt.
settings-sprees = Killstreaks
settings-sprees-hint = Der letzte Abschuss eines Spielers vor seinem Tod bekommt ein Abzeichen mit der Zahl seiner Abschüsse am Stück, wenn es mindestens so viele sind. Jeder Tod beendet die Killstreak, egal wie, genauso wie das Ende der Map oder wenn der Spieler das Spiel verlässt. Teamkills zählen nicht.
settings-spree-least = Mindestens (Abschüsse)
settings-start-filters =Filter beim Start
settings-start-filters-hint = Bei jedem Start von Lambda Slicer sind diese Filter in der Bibliothek eingeschaltet, damit kurze Demos, die du nie sehen willst, nicht im Weg sind. Du kannst die Filter trotzdem ändern, bis du Lambda Slicer schließt.
settings-hide-unplayed-maps = Bei Demos mit mehreren Maps die Maps weglassen, auf denen es keine Abschüsse gab und die weniger als 5 Minuten aufgenommen wurden
settings-folders = Bibliotheksordner
settings-folders-empty = Noch keine Ordner hinzugefügt.
settings-remove = Entfernen

## Bibliothek

library-empty = Füge einen Ordner hinzu oder öffne eine Demo, um loszulegen.
library-no-matches = Keine Demo passt zu diesen Filtern.
library-no-cache = Auf diesem Computer gibt es keinen Ort, um die Bibliothek aufzubewahren.
library-demos = { $count ->
    [one] { $count } Demo
   *[other] { $count } Demos
}
library-percent = { $percent } %
library-demos-of = { $shown } von { $total ->
    [one] { $total } Demo
   *[other] { $total } Demos
}
library-matches = { $count ->
    [one] { $count } Match
   *[other] { $count } Matches
}
library-matches-of = { $shown } von { $total ->
    [one] { $total } Match
   *[other] { $total } Matches
}
library-no-games = Keine der Demos enthält ein Match oder fünf Minuten Spiel ohne Match.
library-no-games-filtered = Kein Match passt zu diesen Filtern.
library-total = { $count ->
    [one] { $count } Demo in der Bibliothek
   *[other] { $count } Demos in der Bibliothek
}
filters-title = Filter
filters-find = Suchen
filters-pov = Perspektive
filters-quality = Qualität
filters-recorded = Aufgenommen
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
path-copy = Speicherort kopieren
path-show = Im Ordner zeigen
path-show-failed = Der Ordner ließ sich nicht öffnen ({ $detail }). Kopiere stattdessen den Speicherort.

filter-search = Dateinamen und Ordner durchsuchen
filter-map = Map
filter-player = Spieler
help-player-filter = Tippe einen Teil eines Namens ein, um Demos zu finden, in denen jemand ihn benutzt hat. Wähle einen Spieler aus der Liste, um die Demos zu finden, in denen er unter irgendeinem Namen gespielt hat, erkannt an seiner SteamID. Alle gewählten Spieler müssen mitgespielt und nicht nur zugeschaut haben. Bots und manche ältere Demos haben keine SteamID, daher findet man diese Spieler nur über den Namen.
player-remove = Aus dem Filter nehmen
player-also = auch { $names }
player-demos = { $count ->
    [one] { $count } Demo
   *[other] { $count } Demos
}
filter-weapon-any = Beliebige Waffe
filter-min-minutes = Mindestens (Minuten)
filter-min-kills = Mindestens (Abschüsse)
filter-min-spree = Killstreak von mindestens (Abschüsse)
filter-players = Spieler
filter-teams = Teams
filter-bots = Bots
bots-any = Egal
bots-few = Wenige
bots-none = Keine
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
kind-hltv = HLTV
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

scan-progress = Gefunden { $found }, fertig { $done } ({ $cached } schon bekannt, { $analysed } analysiert), { $megabytes } MB gelesen
scan-cancelling = Wird angehalten…
scan-cancelled = Angehalten, { $left } noch nicht gescannt.
scan-finished = Fertig: { $done ->
    [one] { $done } Demo
   *[other] { $done } Demos
}, { $analysed } analysiert, in { $seconds } s.
scan-problems = { $count ->
    [one] { $count } Problem
   *[other] { $count } Probleme
}
problems-title = Probleme beim Scannen
problem-changed = Die Datei hat sich während der Analyse geändert, daher wurde nichts gespeichert.
drop-hint = Demos oder Ordner hier ablegen, um sie hinzuzufügen
drop-not-demo = Keine Demo: { $names }

## Meldungen des Analysekerns nach Code

analysis-panicked = Die Analyse wurde durch einen Fehler in Lambda Slicer angehalten: { $detail }
cache-cancelled = Die Analyse wurde abgebrochen, daher wurde nichts gespeichert.
cache-database = Die gespeicherte Bibliothek kann nicht verwendet werden: { $detail }
cache-directory = Der Ordner für die gespeicherte Bibliothek kann nicht angelegt werden: { $detail }
container-bad-magic = Das ist keine Half-Life-Demo.
container-frame-bad-time = Die Aufnahme ist bei Frame { $frame } von Teil { $entry } beschädigt, der eine kaputte Zeitangabe hat.
container-frame-missing-terminal = Teil { $entry } der Aufnahme endet nicht richtig.
container-frame-negative-length = Die Aufnahme ist bei Frame { $frame } von Teil { $entry } beschädigt.
container-frame-over-limit = Frame { $frame } von Teil { $entry } ist größer als erlaubt.
container-frame-truncated = Die Aufnahme bricht bei Frame { $frame } von Teil { $entry } ab.
container-frame-unknown-tag = Die Aufnahme ist bei Frame { $frame } von Teil { $entry } beschädigt, der etwas enthält, das Lambda Slicer nicht kennt.
container-io = Die Datei kann nicht gelesen werden: { $detail }
container-no-directory = Die Aufnahme wurde nie abgeschlossen und hat daher kein Inhaltsverzeichnis.
container-range = Das Inhaltsverzeichnis der Aufnahme zeigt außerhalb der Datei.
container-too-short = Die Datei ist zu kurz für eine Demo.
coverage-hltv = HLTV-Aufnahme. Kein Spieler hat sie aufgenommen, daher folgen die Ereignisse dem, was HLTV gerade beobachtet hat.
coverage-mostly-spectating = Die aufnehmende Person hat { $percent } % der Wiedergabe zugeschaut, daher stammen die meisten Ereignisse von anderen Spielern, so wie man sie beim Zuschauen gesehen hat.
coverage-no-local-events = Die eigenen Aktionen der aufnehmenden Person wurden in dieser Demo nicht gespeichert, daher erscheinen nur die, die der Server geschickt hat.
coverage-singleplayer = Einzelspieler. Alles andere sind Monster oder Teile der Map, und es gibt keinen Killfeed.
coverage-unknown = Die Aufnahme ließ sich nicht weit genug lesen, um zu erkennen, was für eine Aufnahme es ist.
coverage-visible-only = Das Spiel bekommt die Aktionen anderer Spieler nur, solange der Server meint, dass die aufnehmende Person sie sehen kann, daher können ihre Zahlen niedriger sein als das, was wirklich passiert ist.
decode-failed = Das Lesen der Aufnahme hielt bei { $time } an (Frame { $frame }): { $detail }
export-active-effect = { $effect } läuft zu Beginn noch, und ein Clip kann das noch nicht übernehmen.
export-client-dropped-packets = Bei { $time } hat das Spiel zu spät angekommene Daten weggeworfen, was ein Clip noch nicht nachmachen kann.
export-cross-segment = Bei { $time }, mitten im Clip, wurde die Map gewechselt oder der Spieler hat sich neu verbunden, und darüber kann ein Clip nicht hinweggehen.
export-destination-exists = { $path } existiert bereits. Wähle einen anderen Namen.
export-empty-range = Der Clip enthält nichts zum Exportieren.
export-io = { $path } kann nicht geschrieben werden: { $detail }
export-missing-delta-base = Der Aufnahme fehlen Daten von vor dem Beginn, die der Clip braucht (Nummer { $sequence }).
export-missing-parent = Der Ordner { $path } existiert nicht.
export-range-beyond-end = Das Ende { $end } liegt nach dem Ende der Aufnahme, { $segment_end }.
export-same-as-source = Der Export würde die Demo ersetzen, aus der er stammt.
export-source-changed = { $path } hat sich während des Exports geändert.
export-source-decode-failed = Die Demo lässt sich nicht lesen: { $detail }
export-source-failed-crosscheck = Bei der Prüfung passte das, was Lambda Slicer aus dieser Demo ermittelt hat, nicht zur Demo selbst, daher kann man Clips daraus nicht vertrauen.
export-unknown-persistent-message = Die Nachricht { $name } wird in Clips noch nicht unterstützt.
export-unproven-state = Bei { $time } stützt sich die Aufnahme auf Daten, die geraten werden mussten, daher lässt sich der Clip nicht genau erstellen.
export-unrepresentable-time-field = Eine Zeit am Anfang lässt sich nicht genau speichern: { $detail }
export-unsupported = Nicht unterstützt: { $detail }
export-validation-failed = Der exportierte Clip hat die abschließende Prüfung nicht bestanden: { $detail }
io-not-found = Die Datei oder der Ordner wurde nicht gefunden.
io-other = { $detail }
io-permission-denied = Zugriff verweigert.
io-unexpected-end = Die Datei endete unerwartet.
protocol-unsupported = Diese Demo stammt von einer Spielversion, die nicht unterstützt wird (Protokoll { $demo }/{ $network }). Nur 5/47 und 5/48 werden unterstützt.
protocol-unsupported-network = Der Server hat auf eine Spielversion gewechselt, die nicht unterstützt wird (Protokoll { $network }).

## Demoansicht

demo-close = Schließen
demo-loading = { $name } wird analysiert…
show-shots = Schüsse
show-actions = Andere Aktionen
show-kills = Abschüsse
show-switches = Waffenwechsel
show-chat = Chat
show-match = Match
lane-unknown = Unbekannter Spieler
lane-recorder = { $name } (Aufnahme)
event-kill = Hat { $victim } abgeschossen
event-switch = Waffenwechsel
event-copy = Kopieren
event-copy-kill = { $killer } hat { $victim } mit { $weapon } getötet
event-copy-teamkill = { $killer } hat { $victim } aus dem eigenen Team mit { $weapon } getötet
event-copy-suicide = { $victim } hat sich mit { $weapon } selbst getötet
event-copy-death = { $victim } ist durch { $weapon } gestorben
event-vote-called = Abstimmung über { $subject } gestartet
event-vote-cast = Abstimmung über { $subject }: { $change } ({ $tally })
event-vote-more-yes = { $count ->
    [one] eine Ja-Stimme mehr
   *[other] { $count } Ja-Stimmen mehr
}
event-vote-fewer-yes = { $count ->
    [one] eine Ja-Stimme weniger
   *[other] { $count } Ja-Stimmen weniger
}
event-vote-more-no = { $count ->
    [one] eine Nein-Stimme mehr
   *[other] { $count } Nein-Stimmen mehr
}
event-vote-fewer-no = { $count ->
    [one] eine Nein-Stimme weniger
   *[other] { $count } Nein-Stimmen weniger
}
event-vote-tally = { $yes } Ja, { $no } Nein, { $undecided } unentschlossen
event-vote-accepted = Abstimmung über { $subject } angenommen
event-vote-denied = Abstimmung über { $subject } abgelehnt
event-timeout-called = Timeout angefordert
event-timeout-under-way = Timeout gestartet
event-timeout-over = Timeout vorbei
event-paused = Spiel pausiert
event-resumed = Spiel fortgesetzt
event-sudden-death = Sudden Death gestartet
event-sudden-death-over = Sudden Death vorbei
event-map-changed = Map gewechselt zu { $map }
event-spectating = Zuschauen begonnen
event-playing = Zuschauen beendet
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

column-date = Datum
column-file-date = Dateidatum
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
scores-spectators = Zuschauer
scores-follow = Klicke, um durch seine Augen zuzuschauen
pane-fill = Ganzen Seitenbereich nutzen
pane-share = Seitenbereich halb und halb teilen
scores-frags = Frags
scores-deaths = Tode

viewer-play = Abspielen
viewer-pause = Pause
viewer-volume = Lautstärke
viewer-no-sound = Kein Ton
viewer-play-backward = Zurückspulen
viewer-step-back = Vorheriger Frame
viewer-step-forward = Nächster Frame
viewer-position = Position in der Demo
viewer-skip-back = 10 Sekunden zurück
viewer-skip-forward = 10 Sekunden vor
viewer-rate = { $rate }×
viewer-slower = Langsamer
viewer-faster = Schneller
viewer-rate-normal = Zurück zur normalen Geschwindigkeit
viewer-shortcut = Tastenkürzel: { $key }
key-space = Leertaste
key-shift-left = Shift+Links
key-shift-right = Shift+Rechts
viewer-map-failed = Die Karte { $map } konnte nicht geladen werden
viewer-stopped = Die Wiedergabe hat vorzeitig angehalten: { $reason }
viewer-missing = Dateien, die nicht gelesen werden konnten: { $count }
event-previous = Vorheriges Ereignis
event-next = Nächstes Ereignis
event-details = Abschussdetails
event-play = Ereignis abspielen
timeline-title = Zeitleiste
timeline-lanes = Spielerzeilen
timeline-zoom-in = Vergrößern
timeline-zoom-out = Verkleinern
timeline-fit-all = Ganze Demo einpassen
timeline-gestures = Klicke auf eine Markierung, um ihr Ereignis zu wählen, oder irgendwo anders, um dorthin zu springen. Ziehe an den Zeiten oben oder an der Linie, die den aktuellen Moment markiert, um dich durch die Demo zu bewegen. Ziehe irgendwo anders, um die Ansicht zu verschieben. Scrolle, um dort hinein- und herauszuzoomen, wo die Maus ist, und doppelklicke, um die ganze Demo zu sehen. Ziehe den Rand eines Clips, um zu ändern, wo er beginnt oder endet, oder klicke auf den Clip, um ihn zu bearbeiten.
timeline-density = Die schattierten Balken zeigen, wie viele der angezeigten Ereignisse an jeder Stelle passieren, verglichen mit der Stelle der Demo, an der am meisten los ist.
timeline-density-count = { $count ->
    [one] Ein angezeigtes Ereignis hier
   *[other] { $count } angezeigte Ereignisse hier
}
timeline-session-count = Teil { $session }: { $count }
timeline-choose = Ereignisse an dieser Stelle
legend-clip = Clip
event-session = Teil { $session }
event-unreachable = Dieses Ereignis kommt nach einem Mapwechsel oder Reconnect, bei dem die Uhr der Demo wieder bei null angefangen hat, daher kann die Demo-Ansicht nicht dorthin springen.
event-hidden = Das ausgewählte Ereignis ist durch die Filter ausgeblendet.
highlight-jump = Zu Highlight springen…
highlight-help = Highlights bewerten die ganze Demo nach ihrer Action, egal was die Filter zeigen.
kills-with = Abschüsse mit
refine-midair = Midair
refine-speed = Speed
refine-wallgauss = Wallgauss
refine-multikill = Multikill
refine-clear = Zurücksetzen
refine-airborne = In der Luft
refine-either = Beliebig
midair-both = Beide
midair-one = Nur einer
help-midair-either = Der Schütze, das Opfer oder beide waren in der Luft. Der Schütze wird beim mit dem Abschuss verknüpften Schuss geprüft, ohne einen solchen beim Abschuss, und das Opfer beim Tod. Wer nur springt, weniger als 50 Units über dem Boden, zählt nicht. Dafür muss der Half-Life-Ordner gewählt sein, um zu wissen, wo der Boden ist.
help-midair-both = Schütze und Opfer waren beide in der Luft, jeweils zum Zeitpunkt der Prüfung.
help-midair-one = Genau einer von beiden war in der Luft, und vom anderen ist bekannt, dass er am Boden war.
speed-at-least = Mindestens
speed-unit = ups
speed-killer = Schütze
speed-victim = Opfer
help-multikill = Mehrere Spieler mit einem einzigen Schuss getötet, etwa durch eine Explosion, die alle erreicht, oder einen Gauss-Strahl, der durch sie hindurchgeht. Jeder Abschuss dieses Schusses zeigt es an.
help-speed = Geschwindigkeit über den Boden in ups, wie ein Speedometer sie anzeigt, ohne Bewegung nach oben und unten. Der Schütze wird beim mit dem Abschuss verknüpften Schuss gemessen, ohne einen solchen beim Abschuss, und das Opfer beim Tod. Die Geschwindigkeiten anderer Spieler sind Schätzungen.
wallgauss-weapon = Wallgauss betrifft nur Gauss-Abschüsse, und der Waffenfilter zeigt { $weapon }.
wallgauss-checking = Wallgauss wird geprüft… { $examined } von { $total } Abschüssen
wallgauss-needs-files = Wallgauss braucht die Spieldateien, um zu wissen, wo die Wände der Map sind.
wallgauss-failed = Wallgauss konnte nicht geprüft werden. { $reason }
help-wallgauss = Gauss-Abschüsse durch eine Wand, ermittelt, indem jeder Schuss so durch die Map verfolgt wird, wie es der Server tut. Ein Abschuss, der sich nicht entscheiden lässt, etwa weil das Opfer außerhalb des Blickfelds der Aufnahme war, gilt als unbekannt und wird ausgelassen.
badge-wall = WG
wall-yes-exit = Wallgauss, aus dem Schuss ermittelt: die Explosion, wo der Strahl aus einer { $thickness } Units dicken Wand herauskam.
wall-yes-beyond = Wallgauss, aus dem Schuss ermittelt: Der Strahl traf, nachdem er durch eine { $thickness } Units dicke Wand gegangen war.
wall-limits = Aus dem Schuss und den Positionen der Spieler ermittelt, da der Server das nicht aufzeichnet. Ein Spieler ist 32 Units breit und zählt als Box, und das Opfer war womöglich ein oder zwei Körperbreiten von der Stelle entfernt, die die Demo zeigt.
wall-no-direct = Kein Wallgauss: Der Strahl hätte das Opfer vor jeder Wand treffen können.
wall-no-reflection = Kein Wallgauss: Die Explosion einer Reflexion hätte das Opfer vor jeder Wand treffen können.
wall-doubt-no-shot = Wallgauss unbekannt: Der Schuss des Schützen ist nicht in der Aufnahme.
wall-doubt-no-map = Wallgauss unbekannt: Die Dateien der Karte konnten nicht gelesen werden.
wall-doubt-unseen = Wallgauss unbekannt: Das Opfer war außerhalb des Blickfelds der Aufnahme.
wall-doubt-brush = Wallgauss unbekannt: Der Strahl traf eine Tür, einen Aufzug oder einen anderen besonderen Teil der Map, der nicht geprüft wird.
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
events-filters = Filter
events-filters-on = Filter ({ $count })
events-kinds-none = Keine Ereignisse
events-follow = Die Liste beim aktuellen Moment halten, während die Demo läuft. Scrollen in der Liste schaltet das aus.
clip-it = Clippen
badge-headshot = KS
badge-double-kill = Doublekill
badge-triple-kill = Triplekill
badge-quadra-kill = Quadrakill
badge-penta-kill = Pentakill
badge-multikill = { $count } Abschüsse
badge-spree = ×{ $count }
badge-spree-help = { $count ->
    [one] { $count } Abschuss am Stück, ohne zu sterben
   *[other] { $count } Abschüsse am Stück, ohne zu sterben
}
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = Suizid
badge-teamkill = Teamkill
badge-world = Welt
badge-clip = Clip
clip-of = Clip aus { $name }
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
column-multikill = Multikill
column-spree = Killstreak
filter-headshots = Abschüsse per Kopfschuss
filter-midair = Midair-Abschüsse
event-headshot = Kopfschuss
scores-headshots = KS

column-accuracy = Treffsicherheit
column-players = Spieler
column-teams = Teams
help-players-column = Spieler im Spiel, keine Zuschauer, gezählt, wenn ein Match nach seinem Countdown begann. In einem Teamspiel zählen nur Spieler in einem Team. Eine Demo zeigt die höchste Zahl in einem vollständigen Match, und eine Demo ohne vollständiges Match die höchste Zahl zu irgendeinem Zeitpunkt.
help-teams-column = Teams mit mindestens einem Spieler, gezählt wie die Spieler. Ein Spiel ohne Teams hat keine.
filter-min-accuracy = Treffsicherheit mindestens (%)
scores-accuracy = Treff.
scores-kd = K/D
scores-ping = Ping
help-kd = Frags pro Tod, also die Frags geteilt durch die Tode. Wer nie gestorben ist, bekommt seine Frags angezeigt.
help-ping-loss = Ping jedes Spielers in Millisekunden, danach sein Loss (verlorene Pakete), so wie der Server sie zuletzt geschickt hat. Er schickt sie nur, solange der aufnehmende Spieler die Scoreboard-Taste hält, deshalb steht bis dahin ein Strich da.

help-headshots = Abschüsse durch einen Treffer am Kopf, erkannt an der Todesanimation des Opfers. Explosionen zählen nicht. Gezeigt werden nur die der aufnehmenden Person, weil andere Spieler oft außerhalb ihres Blickfelds abschießen. Im Duell auch die des Gegners, da alle seine Abschüsse die aufnehmende Person treffen.
help-headshots-column = Abschüsse der aufnehmenden Person durch einen Treffer am Kopf, erkannt an der Todesanimation des Opfers. Explosionen zählen nicht. Abschüsse anderer Spieler zählen nicht, weil sie oft außerhalb des Blickfelds der Aufnahme geschehen.
help-multikill-column = Die meisten Spieler, die jemand in der Demo mit einem einzigen Schuss getötet hat, etwa 3 bei einem Triplekill.
help-spree-column = Die längste Killstreak der Demo, egal von wem: die meisten Abschüsse, die ein Spieler am Stück gemacht hat, ohne zu sterben.
help-date-column = Wann das Match aufgenommen wurde. Das Datum kommt aus dem Dateinamen, wenn der Name ein Datum wie 20230910_214553 enthält, wie bei den Aufnahmen, die das Spiel von selbst macht. Sonst ist es das Dateidatum, das sich beim Kopieren oder Herunterladen ändern kann.
help-file-date = Wann die Datei gespeichert wurde. Bei einem exportierten Clip ist das der Zeitpunkt des Exports.
help-accuracy = Der Anteil der Schüsse, die trafen, anhand des Bluts, das ein Treffer entlang der Schusslinie hinterlässt, abzüglich zufälliger Übereinstimmungen. Er umfasst Glock, Python, MP5-Kugeln, Schrotflinte, Armbrust mit Zoom und direkte Treffer des Gauss-Strahls. Explosionen und Gauss-Reflexionen hinterlassen kein Blut und zählen nicht. Nur für die aufnehmende Person und erst ab 50 Schüssen.
help-headshot-filter = Demos mit mindestens einem Kopfschuss-Abschuss, den die Aufnahme zeigt, von beliebigen Spielern.
help-midair-filter = Demos mit mindestens einem Abschuss, bei dem der Schütze oder das Opfer in der Luft war, mindestens 50 Units über dem Boden. Ohne gewählten Half-Life-Ordner zählt auch ein einfacher Sprung.
help-spree-filter = Demos, in denen ein Spieler mindestens so viele Abschüsse am Stück gemacht hat, ohne zu sterben. Jeder Tod beendet die Killstreak, egal wie, genauso wie das Ende der Map oder wenn der Spieler das Spiel verlässt. Teamkills zählen nicht.
help-accuracy-filter = Demos, in denen die aufnehmende Person mindestens diesen Anteil ihrer Schüsse traf.
help-bots-filter = Wenige behält nur Demos und Matches, in denen weniger Bots als echte Spieler waren. Ein 1v1 gegen einen Bot fällt also weg, ein voller Server mit ein paar Bots als Lückenfüller aber nicht. Keine behält nur die ganz ohne Bots. Ein Bot ist ein Spieler, den das Spiel als Bot kennzeichnet, oder einer ohne CD-Key.

clips-new = Neuer Clip
clips-select-event = Wähle zuerst ein Ereignis in der Liste oder auf der Zeitleiste.
clips-other-session = Hier kann kein Clip beginnen, weil die Uhr der Demo nach einem Mapwechsel oder Reconnect wieder bei null angefangen hat und ein früherer Teil dieselben Zeiten hat.
clips-add-window = Clip von diesem intensiven Moment
clip-default-name = Clip { $number }
clip-name = Name
clip-length = Länge { $duration }
clip-remove = Entfernen
clip-loop = Schleife
clip-time-help = Sekunden (96), Minuten und Sekunden (1:36) oder Stunden, Minuten und Sekunden (00:01:36.250)
clips-empty-hint = Noch keine Clips. Wähle ein Ereignis und drücke Clippen, oder drücke Neuer Clip, um einen ab dem aktuellen Moment zu starten.
clips-new-help = Ein Clip von zehn Sekunden ab dem aktuellen Moment.
clip-in = Anfang
clip-out = Ende
clip-set-in = Clip ab dem aktuellen Moment starten
clip-set-out = Clip beim aktuellen Moment beenden
clip-no-playhead = Die Demo-Ansicht ist nicht verfügbar, daher gibt es keinen aktuellen Moment, den du nehmen kannst.
clip-play = Clip einmal abspielen
clip-show-on-timeline = Auf der Zeitleiste zeigen
export-destination = Exportordner ändern
export-skipped = Übersprungen. { $reason }
export-earlier-range = Dieser Export umfasst { $start } – { $end }, den Clip so, wie er war, bevor du ihn geändert hast.
scores-player = Spieler
scores-none = Die Aufnahme enthält keinen Punktestand.
clip-effective = Der Export umfasst { $start } – { $end } ({ $duration }), so genau, wie die Aufnahme es zulässt.
settings-clip-padding = Puffer um Clips
settings-preroll = Sekunden vor dem Ereignis
settings-postroll = Sekunden nach dem Ereignis
export-start-first-group = Ein Clip kann nicht ganz am Anfang der Aufnahme beginnen. Beginne etwas später.
export-stopped = Der Export wurde gestoppt, daher wurde kein Clip gespeichert.
export-start-during-loading = Der Anfang fällt in das Laden einer Map. Beginne etwas später.

export-all = Alle Clips exportieren
export-one = Exportieren
export-cancel = Export stoppen
export-folder = Exporte landen in { $folder }.
export-folder-beside = Exporte landen neben der Demo, in { $folder }.
export-waiting = Wartet auf den Export…
export-running = Wird exportiert… Dabei wird die ganze Demo gelesen, das kann dauern.
export-searching = Der Clip kann nicht genau dort beginnen. Suche einen Start in der Nähe, der funktioniert…
export-nearest-earlier = Er kann stattdessen bei { $time } beginnen, { $seconds } s früher.
export-nearest-later = Er kann stattdessen bei { $time } beginnen, { $seconds } s später.
export-start-there = Dort beginnen und exportieren
export-cancelled = Nicht exportiert, da der Export gestoppt wurde.
export-saved = Gespeichert unter { $path }. Mit dem Original verglichene Momente: { $count }.
export-playdemo = In der Spielkonsole:
export-copy-first = Kopiere { $name } in deinen Mod-Ordner, den mit liblist.gam, dann in der Spielkonsole:
export-copy = Kopieren
settings-export-folder = Exportordner
settings-export-choose = Wählen…
settings-file-names = Dateinamen
settings-clip-name-files = Namen des Clips für die Datei verwenden
settings-demo-name-first = Mit dem Namen der Demo beginnen
settings-viewer = Demo-Ansicht
settings-viewer-game = Half-Life-Ordner
settings-viewer-game-none = Nicht gewählt
settings-viewer-game-hint = Der Ordner mit valve. Die Demo-Ansicht spielt Demos mit den Karten, Modellen und Sounds des Spiels daraus.
settings-game-override = Diesmal nutzt Lambda Slicer { $path }, weil es mit --game gestartet wurde. Eine Wahl hier gilt ab dem nächsten Start.
settings-more-games = Weitere Half-Life-Ordner
settings-more-games-hint = Andere Orte, an denen Half-Life installiert ist, etwa einer mit vielen Karten. Liegt eine Karte, ein Modell, ein Sprite oder ein Sound nicht im Half-Life-Ordner oben, sucht die Demo-Ansicht in diesen Ordnern, von oben nach unten. Nach einer Änderung prüft die Bibliothek ihre Demos erneut auf Midair-Abschüsse, mit den Karten, die diese Ordner mitbringen.
settings-more-games-empty = Noch keine hinzugefügt.
settings-move-up = Nach oben
settings-move-down = Nach unten
settings-game-content = Spielinhalte
settings-game-content-hint = Diese Optionen stehen am Anfang so, wie du sie im Spiel eingestellt hast. Eine Änderung hier wirkt nur in der Demo-Ansicht, nie im Spiel. Ist mehr als eine an, gilt die Reihenfolge des Spiels: zuerst Addon-Inhalte, dann lokalisierte Inhalte, dann HD-Modelle, dann die eigenen Dateien des Spiels.
settings-hd-models = HD-Modelle verwenden
settings-hd-models-hint = Sucht die hochwertigen Modelle zuerst in den _hd-Ordnern, etwa valve_hd. Steht am Anfang wie die Spieloption „Use high quality models“.
settings-addon-content = Eigene Addon-Inhalte verwenden
settings-addon-content-hint = Sucht Inhalte, die du dem Spiel hinzugefügt hast, zuerst in den _addon-Ordnern, etwa valve_addon. Steht am Anfang wie die Spieloption „Allow custom addon content“.
settings-localized-content = Lokalisierte Inhalte verwenden
settings-localized-content-hint = Sucht zuerst in den Ordnern der Sprache, in der Steam Half-Life startet, etwa valve_spanish für Spanisch. Das Spiel nutzt sie immer, und für Englisch gibt es keine.
settings-steam-language = Steam startet Half-Life in dieser Sprache: { $language }.
settings-steam-english = Steam startet Half-Life auf Englisch, das keine eigenen Ordner hat.
dialog-executables = Programme
setup-title = Demo-Ansicht einrichten
setup-body = Bild und Ton brauchen die Spieldateien von Half-Life. Zeig Lambda Slicer, wo das Spiel installiert ist, dann liest es sie von dort.
setup-locate-exe = hl.exe suchen…
setup-locate-folder = Half-Life suchen…
setup-settings = Einstellungen
setup-still-works = Zeitleiste, Ereignisse und Clips funktionieren auch ohne sie.
setup-relocate = { $folder } enthält Half-Life nicht mehr. Such es erneut.
setup-not-executable = { $name } ist nicht { $executable }. Wähle das Programm des Spiels selbst, keine Verknüpfung.
setup-not-installed = { $folder } enthält keine Half-Life-Installation mit valve-Ordner.
tab-events = Ereignisse
tab-clips = Clips
tab-scoreboard = Punktestand
tab-demos = Demos
tab-matches = Matches
filter-status = Status
library-all = Alle
library-favourites = Favoriten
library-no-favourites = Noch keine Demos unter den Favoriten. Klicke mit der rechten Maustaste auf eine Demo und wähle „Zu Favoriten hinzufügen“.
menu-favourite-add = Zu Favoriten hinzufügen
menu-favourite-remove = Aus Favoriten entfernen
favourite-star = Favorit
library-hidden = Ausgeblendet
library-no-hidden = Keine ausgeblendeten Demos. Klicke mit der rechten Maustaste auf eine Demo und wähle „Ausblenden“, damit sie nicht mehr in der Liste steht.
menu-hide = Ausblenden
menu-unhide = Einblenden
filter-status-all = Alle
help-status-column = Wie viel vom Match die Demo aufgenommen hat. Zeige auf den Status einer Zeile, um zu sehen, was er bedeutet.
help-match-players = Spieler im Spiel, keine Zuschauer, beim Start des Matches nach dem Countdown, oder beim ersten Abschuss, wenn das Match schon lief, als die Aufnahme begann. Im freien Spiel die meisten gleichzeitig.
help-match-teams = Teams mit mindestens einem Spieler, gezählt wie die Spieler. Ein Spiel ohne Teams hat keine.
help-match-multikill = Die meisten Spieler, die jemand im Match mit einem einzigen Schuss getötet hat, etwa 3 bei einem Triplekill.
help-match-spree = Die längste Killstreak, die im Match endete, egal von wem: die meisten Abschüsse, die ein Spieler am Stück gemacht hat, ohne zu sterben.
match-complete = Vollständig
match-joined-late = Spät eingestiegen
match-interrupted = Unterbrochen
match-cut-off = Abgeschnitten
match-free-play = Freies Spiel
help-match-complete = Vom Countdown bis zum Ende des Matches aufgenommen.
help-match-joined-late = Das Match lief schon, als die Aufnahme begann, und wurde bis zum Ende aufgenommen.
help-match-interrupted = Ein anderes Match begann, das Match wurde abgebrochen oder die Map wurde gewechselt, bevor das Match zu Ende war.
help-match-cut-off = Die Aufnahme endete vor dem Match.
help-match-free-play = Fünf Minuten oder mehr Spiel, ohne dass ein Match lief.
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
settings-tab-library = Bibliothek
settings-tab-clips = Clips
settings-section-game = Half-Life
settings-section-view = Ansicht
settings-language-system = Sprache des Computers ({ $name })
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
settings-zoom-fov = Sichtfeld beim Heranzoomen
settings-viewmodel = Waffe in der Hand zeigen
settings-weapon-bob = Waffenschwanken (cl_bob)
settings-weapon-bob-recorded = Waffenschwanken wie aufgenommen
settings-crosshair = Crosshair zeigen
settings-chat = Chat zeigen
settings-vitals = Leben, Rüstung, Munition und Pickups zeigen
settings-vitals-size = Größe von Leben, Rüstung, Munition und Pickups
settings-free-vitals = Leben, Rüstung, Munition und Pickups auch mit der freien Kamera anzeigen
settings-scores = Punktestand oben links zeigen
settings-score-rows = Höchstzahl der Zeilen im Punktestand
settings-scoreboard = Scoreboard zeigen, solange der aufnehmende Spieler die Taste dafür hält, und am Ende der Map
settings-scoreboard-off = Aus
settings-scoreboard-fullscreen = Nur im Vollbild
settings-scoreboard-always = An
settings-server-settings = Servereinstellungen oben rechts zeigen
settings-text-size = Größe von Chat, Killfeed und Servereinstellungen
settings-hud-messages = Text vom Server zeigen, etwa Timer für bhop und KZ
settings-see-through = Spieler durch Wände sehen
settings-see-through-items = Auch andere Modelle, etwa Geschosse und Gegenstände
settings-player-names = Namen der Spieler über ihnen zeigen
settings-show-position = Position und Blickwinkel zeigen
settings-speedometer = Speedometer zeigen
settings-jump-speed = Geschwindigkeit des letzten Sprungs zeigen
settings-jump-speed-hint = Wie schnell der aufnehmende Spieler war, als er zuletzt vom Boden abhob. Steht bis zu seinem nächsten Sprung unter dem Speedometer.
settings-recorder-fps = FPS des aufnehmenden Spielers zeigen
settings-recorder-fps-hint = Die Bilder pro Sekunde, mit denen das Spiel beim aufnehmenden Spieler lief, oben links, gemittelt über jede halbe Sekunde. Nicht, wie schnell diese App es zeichnet.
settings-speed-axes = Geschwindigkeit messen
settings-speed-horizontal = Horizontal
settings-speed-vertical = Vertikal
settings-speed-both = Beides (3D)
viewer-jump-speed = Sprung { $speed }
settings-show-position-hint = Von wo das Bild gesehen wird, in Spieleinheiten, und wohin es schaut, in Grad nach oben oder unten (Pitch) und zur Seite (Yaw). Mit eingeschalteter freier Kamera sind es die Werte der freien Kamera. Praktisch, um jemandem genau zu sagen, wohin er schauen soll.
viewer-position-line = Position { $x } { $y } { $z }
viewer-fps = { $fps } FPS
viewer-angles-line = Winkel { $pitch } { $yaw }
settings-player-names-hint =Jeder Spieler im Blick hat seinen Namen über dem Kopf, in der Farbe seines Teams, auch in der freien Kamera. Spieler hinter Wänden zeigen ihren nur, solange Spieler durch Wände zu sehen sind.
settings-see-through-hint = Spieler hinter Wänden erscheinen als Umrisse in ihrer oberen Farbe, die Teamspiele nach Team setzen. Es kann nur zu sehen sein, was die Demo enthält. Der Server schickt die Spieler in der Nähe des Aufnehmenden, und Server, die Spieler verbergen, die der Aufnehmende nicht sehen kann, lassen nichts zu zeigen.
settings-section-decals = Decals
settings-decals = Höchstens
settings-decals-hint = Einschusslöcher, Blut und andere Spuren, die im Spiel entstehen. Nur die neuesten bleiben, und die ältesten verschwinden, wenn neue entstehen. Spuren, die zur Map gehören, bleiben immer.
settings-restore = Auf Standard zurücksetzen…
settings-restore-title = Auf Standard zurücksetzen
settings-restore-list = Diese Einstellungen werden wieder so, wie sie am Anfang waren. Der Half-Life-Ordner und die Bibliotheksordner bleiben, wie sie sind.
settings-restore-none = Alle Einstellungen sind schon auf Standard.
settings-restore-columns = Angezeigte Spalten der Bibliothek
settings-restore-layout = Geöffnete Bereiche und ihre Größe
settings-clear-library = Bibliothek leeren…
settings-clear-library-title = Bibliothek leeren
settings-clear-library-text = { $count ->
    [one] Lambda Slicer vergisst, was es in { $count } Demo gefunden hat, und scannt die Bibliotheksordner neu, was eine Weile dauern kann. Deine Demo-Dateien, Clips und Einstellungen bleiben, wie sie sind.
   *[other] Lambda Slicer vergisst, was es in { $count } Demos gefunden hat, und scannt die Bibliotheksordner neu, was eine Weile dauern kann. Deine Demo-Dateien, Clips und Einstellungen bleiben, wie sie sind.
}
viewer-zoom = Heranzoomen
viewer-free-camera = Freie Kamera
viewer-free-camera-help = Flieg frei über die Map, unabhängig vom aufnehmenden Spieler. Mit W, A, S und D bewegst du dich, Leertaste geht nach oben, Strg nach unten und Umschalt macht dich schneller. Halte die rechte Maustaste gedrückt und zieh, um dich umzusehen.
viewer-fullscreen = Vollbild
viewer-leave-fullscreen = Vollbild beenden
viewer-see-through = Durch Wände sehen
viewer-player-names = Spielernamen
viewer-fullbright = Alles ausleuchten
viewer-fullbright-help = Zeigt die ganze Map voll ausgeleuchtet, damit dunkle Ecken und die Spieler darin gut zu sehen sind.
viewer-paused = Pausiert
viewer-map-change = Der Server wechselt die Map zu { $map }
viewer-pov-recorder = { $name } (Aufnahme)
viewer-pov-recorder-unnamed = Aufnehmender Spieler
viewer-pov-help = Durch wessen Augen du zuschaust. ` springt zurück zum aufnehmenden Spieler, 1 bis 9 und 0 wählen die besten Spieler dieser Map nach Punkten, und das Mausrad über dem Bild oder Bild auf und Bild ab wechseln zum vorherigen oder nächsten Spieler.
viewer-scoreboard-ping-loss = Ping/Loss
viewer-scoreboard-bot = BOT
viewer-scoreboard-players = { $count ->
    [one] ({ $count } Spieler)
   *[other] ({ $count } Spieler)
}
viewer-pov-unseen = Die Demo zeigt { $name } hier nicht, darum siehst du die Sicht des aufnehmenden Spielers
settings-events-leave-free = Beim Auswählen eines Ereignisses die freie Kamera verlassen
settings-events-leave-free-hint = Ein ausgewählter Abschuss wird durch die Augen des Schützen, des Opfers oder des aufnehmenden Spielers gezeigt. Ist das aus, bleibt die freie Kamera, wo sie ist, und wenn du sie ausschaltest, kommst du zu dieser Sicht.
settings-export-beside = Neben jeder Demo
settings-export-hint = Im Mod-Ordner, dem mit liblist.gam wie Half-Life\ag, spielen Clips mit einem einzigen playdemo-Befehl.
settings-file-names-hint = Ein exportierter Clip wird nach seiner Demo und seiner Anfangs- und Endzeit benannt, außer du verwendest den Namen des Clips. Das Spiel kann nur Dateien öffnen, deren Namen aus Buchstaben des englischen Alphabets, Zahlen, - und . bestehen, daher wird alles andere zu _. Enthält der Name eines Clips keins dieser Zeichen, wird die Datei stattdessen nach den Zeiten benannt.

clips-store-moved = Die Clip-Datei { $path } konnte nicht gelesen werden. Sie wurde als { $moved } aufbewahrt, und die Clips beginnen leer. { $detail }
clips-store-unreadable = Die Clip-Datei { $path } konnte nicht gelesen werden: { $detail }. Bis Lambda Slicer geschlossen wird, werden Clips nicht gespeichert, damit nichts in der Datei verloren geht.
clips-save-failed = Clips konnten nicht in { $path } gespeichert werden: { $detail }
notice-dismiss = Schließen

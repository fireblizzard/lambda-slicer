# Italiano. Bozza non revisionata

app-title = Lambda Slicer

## Barra degli strumenti

toolbar-open = Apri demo…
toolbar-add-folder = Aggiungi cartella…
toolbar-rescan = Analizza di nuovo
toolbar-cancel = Annulla
toolbar-settings = Impostazioni
toolbar-rescan-busy = Un'analisi è già in corso.
toolbar-rescan-nothing = Aggiungi una cartella o apri una demo da analizzare.
toolbar-cancel-idle = Nessuna analisi in corso.
dialog-demos = Demo di Half-Life

## Impostazioni

settings-title = Impostazioni
settings-language = Lingua
settings-unreviewed = Questa traduzione è una bozza che nessun madrelingua ha ancora revisionato.
settings-workers = Processi di analisi
settings-workers-hint = Più processi analizzano più in fretta, ma usano più tempo del processore.
settings-cache = Cache della libreria
settings-folders = Cartelle della libreria
settings-folders-empty = Nessuna cartella aggiunta.
settings-remove = Rimuovi

## Libreria

library-empty = Aggiungi una cartella o apri una demo per iniziare.
library-no-matches = Nessuna demo corrisponde a questi filtri.
library-no-cache = Su questo sistema non c’è un posto per la cache della libreria.
library-demos = { $count ->
    [one] { $count } demo
    [many] { $count } di demo
   *[other] { $count } demo
}
library-percent = { $percent }%
library-demos-of = { $shown } su { $total ->
    [one] { $total } demo
    [many] { $total } di demo
   *[other] { $total } demo
}
library-total = { $count ->
    [one] { $count } demo nella libreria
    [many] { $count } di demo nella libreria
   *[other] { $count } demo nella libreria
}
filters-title = Filtri
filters-find = Cerca
filters-pov = Punto di vista
filters-quality = Qualità
filters-recorded = Registrata
pov-any = Tutte
pov-playing = In gioco
pov-spectating = Spettatore
help-pov = «In gioco» significa che la registrazione è stata spettatrice al massimo per metà della riproduzione, «Spettatore» per più di metà.
columns-button = Colonne
list-back = Torna alla tabella della libreria
sort-by = Ordina per
sort-ascending = Crescente
sort-descending = Decrescente
score-more-sides = { $count ->
    [one] { $count } parte in più
    [many] { $count } di parti in più
   *[other] { $count } parti in più
}
intensity-help = Intensità { $score }. La barra la confronta con la più alta della libreria, { $max }.
value-unknown = Non noto
path-copy = Copia percorso
path-show = Mostra nella cartella
path-show-failed = Impossibile aprire la cartella ({ $detail }). Copia invece il percorso.

filter-search = Cerca nei percorsi
filter-map = Mappa
filter-player = Giocatore
filter-weapon-any = Qualsiasi arma
filter-min-minutes = Almeno (minuti)
filter-spectating-any = In gioco o spettatore
filter-spectating-mostly = Soprattutto spettatore
filter-spectating-playing = Soprattutto in gioco
filter-clear = Azzera i filtri

column-status = Stato
column-name = Demo
column-map = Mappa
column-duration = Durata
column-gauss = Gauss
column-kills = Uccisioni
column-action = Intensità
column-spectating = Spettatore

status-complete = Analizzata
status-partial = Parziale
status-failed = Non riuscita
status-unsupported = Non supportata
status-no-use = Nessun uso di armi

kind-multiplayer-view = Vista del giocatore
kind-singleplayer = Giocatore singolo
kind-hltv = Proxy spettatore
kind-unknown = Tipo sconosciuto

details-recorder = Registrata da { $name }
details-shots = { $count ->
    [one] { $count } colpo
    [many] { $count } di colpi
   *[other] { $count } colpi
}
details-kills = { $count ->
    [one] { $count } uccisione
    [many] { $count } di uccisioni
   *[other] { $count } uccisioni
}
details-action = Intensità massima { NUMBER($score, minimumFractionDigits: 2) }

## Scansione

scan-progress = Trovate { $found }, fatte { $done } ({ $cached } dalla cache, { $analysed } analizzate), { $megabytes } MB letti
scan-cancelling = Interruzione…
scan-cancelled = Interrotta, { $left } ancora da scansionare.
scan-finished = Fatto: { $done } demo, { $analysed } analizzate, in { $seconds } s.
scan-problems = { $count ->
    [one] { $count } problema
    [many] { $count } di problemi
   *[other] { $count } problemi
}
problems-title = Problemi della scansione
problem-changed = Il file è cambiato durante l’analisi, quindi non è stato salvato nulla.
drop-hint = Trascina qui demo o cartelle per aggiungerle
drop-not-demo = Non è una demo: { $names }

## Messaggi del motore di analisi, per codice

analysis-panicked = L’analisi si è fermata per un errore interno: { $detail }
cache-cancelled = L’analisi è stata annullata, quindi non è stato salvato nulla.
cache-database = Impossibile usare la cache della libreria: { $detail }
cache-directory = Impossibile creare la cartella della cache: { $detail }
container-bad-magic = Questa non è una demo di Half-Life.
container-frame-bad-time = La registrazione è danneggiata al fotogramma { $frame } della parte { $entry }, che ha un tempo non valido.
container-frame-missing-terminal = La parte { $entry } della registrazione finisce senza il record di chiusura.
container-frame-negative-length = La registrazione è danneggiata al fotogramma { $frame } della parte { $entry }.
container-frame-over-limit = Il fotogramma { $frame } della parte { $entry } è più grande del consentito.
container-frame-truncated = La registrazione si interrompe al fotogramma { $frame } della parte { $entry }.
container-frame-unknown-tag = La registrazione è danneggiata al fotogramma { $frame } della parte { $entry }, di tipo sconosciuto.
container-io = Impossibile leggere il file: { $detail }
container-no-directory = La registrazione non è mai stata completata, quindi non ha un indice.
container-range = L’indice della registrazione punta fuori dal file.
container-too-short = Il file è troppo corto per essere una demo.
coverage-hltv = Registrazione di un proxy spettatore. Non c’è un giocatore che registra e gli eventi seguono la vista del proxy.
coverage-mostly-spectating = Chi registrava è stato spettatore per il { $percent }% della riproduzione, quindi la maggior parte degli eventi è di altri giocatori, come li mostrava la vista osservata.
coverage-no-local-events = Nessun evento locale. Mancano le azioni di chi registrava, tranne quelle inviate dal server.
coverage-singleplayer = Giocatore singolo. Gli altri attori sono mostri o elementi della mappa, e non c’è l’elenco delle uccisioni.
coverage-unknown = La registrazione non è stata decodificata abbastanza da capirne il tipo.
coverage-visible-only = Gli eventi degli altri giocatori arrivano a questo client solo finché il server li considera visibili, quindi i loro conteggi sono minimi.
decode-failed = La decodifica si è fermata a { $time } (fotogramma { $frame }): { $detail }
export-active-effect = { $effect } è in corso all’inizio e non può ancora essere riportato.
export-client-dropped-packets = Il gioco ha scartato pacchetti in ritardo a { $time }, cosa che un taglio non sa ancora riprodurre.
export-cross-segment = Una nuova sessione inizia a { $time }, dentro l’intervallo.
export-destination-exists = { $path } esiste già. Scegli un altro nome.
export-empty-range = L’intervallo scelto non contiene nulla da esportare.
export-io = Impossibile scrivere { $path }: { $detail }
export-missing-delta-base = Alla registrazione manca uno stato necessario prima dell’inizio (sequenza { $sequence }).
export-missing-parent = La cartella { $path } non esiste.
export-range-beyond-end = La fine { $end } è oltre la fine della registrazione, { $segment_end }.
export-same-as-source = La destinazione è la demo di origine stessa.
export-source-changed = { $path } è cambiato durante l’esportazione.
export-source-decode-failed = La demo di origine non si decodifica: { $detail }
export-source-failed-crosscheck = La demo di origine non ha superato il controllo incrociato, quindi il suo stato ricostruito non è affidabile.
export-unknown-persistent-message = { $name } prima dell’inizio imposta uno stato che un taglio non sa ancora riportare.
export-unproven-state = Lo stato a { $time } si basa su una base dedotta e non è dimostrato.
export-unrepresentable-time-field = Un tempo all’inizio non può essere scritto fedelmente: { $detail }
export-unsupported = Non supportato: { $detail }
export-validation-failed = Il file esportato non ha superato la convalida: { $detail }
io-not-found = File o cartella non trovati.
io-other = { $detail }
io-permission-denied = Accesso negato.
io-unexpected-end = Il file è finito inaspettatamente.
protocol-unsupported = Il protocollo demo { $demo }/{ $network } non è supportato, solo 5/48.
protocol-unsupported-network = Il server è passato al protocollo di rete { $network }, che non è supportato.

## Vista della demo

demo-close = Chiudi
demo-loading = Analisi di { $name }…
show-shots = Colpi
show-actions = Altre azioni
show-kills = Uccisioni
show-switches = Cambi d’arma
lane-unknown = Giocatore sconosciuto
lane-recorder = { $name } (registra)
event-kill = Ha ucciso { $victim }
event-switch = Cambio d’arma
action-swing = Colpo corpo a corpo
action-fire = Sparo
action-launch = Lancio
action-charge-activity = Carica
action-charge-stop = Fine carica
action-beam-start = Inizio raggio
action-beam-sustain = Raggio
action-beam-stop = Fine raggio
action-placement-attempt = Tentativo di piazzamento
action-release-attempt = Tentativo di rilascio
mode-primary = primario
mode-charged = caricato
mode-zoomed = con zoom
mode-unzoomed = senza zoom
mode-grenade = granata
mode-bullets = proiettili
mode-single = singolo
mode-double = doppio
mode-rapid = rapido
demo-events = { $count ->
    [one] { $count } evento
    [many] { $count } di eventi
   *[other] { $count } eventi
}

column-date = Data
column-score = Punteggio
filter-from = Dal
filter-to = Al
date-from-name = Data presa dal nome del file
date-from-modified = Data dell’ultima modifica del file, che copiarlo o scaricarlo può cambiare
timeline-best-action = Minuto d’oro
timeline-window = N. { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Punteggi finali, all’ultima uccisione { $time }
scores-at = Punteggi a { $time }
scores-show-final = Mostra i punteggi finali
scores-frags = Frag
scores-deaths = Morti

viewer-play = Riproduci
viewer-pause = Pausa
viewer-volume = Volume
viewer-no-sound = Nessun audio
viewer-play-backward = Riavvolgi
viewer-step-back = Frame precedente
viewer-step-forward = Frame successivo
viewer-rate = { $rate }×
viewer-shortcut = Scorciatoia: { $key }
key-space = Spazio
viewer-map-failed = Impossibile caricare la mappa { $map }
viewer-stopped = La riproduzione si è interrotta prima della fine: { $reason }
viewer-missing = File che non è stato possibile leggere: { $count }
event-previous = Evento precedente
event-next = Evento successivo
event-play = Riproduci evento
timeline-title = Timeline
timeline-lanes = Righe giocatori
timeline-zoom-in = Ingrandisci
timeline-zoom-out = Riduci
timeline-fit-all = Adatta tutta la demo
timeline-gestures = Clicca su un segno per selezionarne l’evento, o in qualsiasi altro punto per andarci. Trascina il righello o la testina per scorrere, e trascina altrove per spostarti. Scorri per lo zoom attorno al puntatore, e fai doppio clic per vedere tutta la demo. Trascina il bordo di una clip per spostarlo, o clicca sulla clip per modificarla.
timeline-density = Le barre ombreggiate mostrano quanti degli eventi mostrati cadono in ogni tratto, rispetto al tratto più fitto della demo.
timeline-density-count = { $count ->
    [one] Un evento mostrato qui
    [many] { $count } di eventi mostrati qui
   *[other] { $count } eventi mostrati qui
}
timeline-session-count = Sessione { $session }: { $count }
timeline-choose = Eventi in questo punto
legend-clip = Clip
event-session = Sessione { $session }
event-unreachable = Questo evento è in una sessione successiva che riusa gli stessi tempi, quindi il visore non può raggiungerlo.
event-hidden = I filtri nascondono l’evento selezionato.
highlight-jump = Vai a un highlight…
highlight-help = Gli highlight classificano tutta la demo in base all’azione, qualunque cosa mostrino i filtri.
kills-with = Uccisioni con
refine-midair = Midair
refine-speed = Velocità
refine-wallgauss = Wallgauss
refine-clear = Azzera
refine-airborne = In aria
refine-either = Almeno uno
midair-both = Entrambi
midair-one = Solo uno
help-midair-either = Il killer, la vittima o entrambi erano in aria. Il killer è controllato al colpo legato all’uccisione, o all’uccisione se non c’è, e la vittima alla morte.
help-midair-both = Il killer e la vittima erano entrambi in aria, ciascuno al momento del controllo.
help-midair-one = Esattamente uno dei due era in aria, e dell’altro si sa che era a terra.
speed-at-least = Almeno
speed-unit = ups
speed-killer = Killer
speed-victim = Vittima
help-speed = Velocità orizzontale in unità al secondo, parallela al suolo. Il killer è misurato al colpo legato all’uccisione, o all’uccisione se non c’è, e la vittima alla morte. Le velocità degli altri giocatori sono stime.
wallgauss-weapon = Wallgauss riguarda solo le uccisioni col gauss, e il filtro arma mostra { $weapon }.
wallgauss-checking = Controllo Wallgauss… { $examined } di { $total } uccisioni
wallgauss-needs-files = Wallgauss ha bisogno dei file del gioco per tracciare i colpi nella mappa.
wallgauss-failed = Impossibile controllare Wallgauss. { $reason }
help-wallgauss = Uccisioni col gauss attraverso un muro, ricavate tracciando ogni colpo nella mappa come fa il server. Un’uccisione che non si può stabilire, come una in cui la vittima era fuori dalla vista della registrazione, conta come sconosciuta e resta esclusa.
badge-wall = WALL
wall-yes-exit = Wallgauss identificato per ricostruzione: l’esplosione dove il raggio è uscito da un muro spesso { $thickness } unità.
wall-yes-beyond = Wallgauss identificato per ricostruzione: il raggio dopo aver attraversato un muro spesso { $thickness } unità.
wall-limits = Ricostruito dal colpo e dallo snapshot, non registrato dal server. I giocatori sono tracciati come scatole, e la vittima poteva trovarsi a qualche decina di unità da dove la mostra lo snapshot.
wall-no-direct = Non è un Wallgauss: il raggio poteva colpire la vittima prima di qualsiasi muro.
wall-no-reflection = Non è un Wallgauss: l’esplosione di un rimbalzo poteva colpire la vittima prima di qualsiasi muro.
wall-doubt-no-shot = Wallgauss sconosciuto: il colpo del killer non è nella registrazione.
wall-doubt-no-map = Wallgauss sconosciuto: impossibile leggere i file della mappa.
wall-doubt-unseen = Wallgauss sconosciuto: la vittima era fuori dalla vista della registrazione.
wall-doubt-brush = Wallgauss sconosciuto: il raggio ha incontrato una porta, una piattaforma o un’altra brush entity, che non viene tracciata.
wall-doubt-ambiguous = Wallgauss sconosciuto: il colpo poteva uccidere sia attraverso un muro sia senza.
wall-doubt-unexplained = Wallgauss sconosciuto: non è stato trovato alcun modo in cui il colpo abbia raggiunto la vittima.
wall-doubt-unconfirmed = Wallgauss sconosciuto: il colpo poteva raggiungere la vittima solo attraverso un muro, ma non è certo.
wall-doubt-not-reached = Wallgauss sconosciuto: la registrazione si ferma prima del colpo.
wall-pending = Wallgauss non ancora controllato.
events-unknown = { $count ->
    [one] A un’uccisione mancano i dati per stabilirlo
    [many] A { $count } di uccisioni mancano i dati per stabilirlo
   *[other] A { $count } uccisioni mancano i dati per stabilirlo
}
help-unknown = Il movimento che la registrazione non ha potuto vedere conta come sconosciuto, mai come a terra o fermo, quindi queste uccisioni restano escluse.
clip-it = Clippa
badge-headshot = CT
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = suicidio
badge-teamkill = teamkill
badge-world = mondo
air-both = Entrambi erano in aria al controllo.
air-killer = Il killer era in aria al controllo.
air-victim = La vittima era in aria al controllo.
air-other-unknown = Non si sa se lo fosse anche l’altro.
speed-horizontal = Velocità orizzontale, parallela al suolo.
motion-line = { $who }: { $air }, { $speed }. { $how }, { $when }.
motion-who-killer = Killer
motion-who-victim = Vittima
motion-air-yes = in aria
motion-air-no = a terra
motion-air-unknown = non si sa se in aria
motion-speed-unknown = velocità sconosciuta
motion-exact = Registrato dal gioco
motion-trajectory = Stimato dal movimento
motion-unclear = Movimento incerto
motion-not-visible = Fuori vista
motion-at-shot = al colpo collegato
motion-at-kill = alla morte
motion-none = L’analisi non ha movimento per questa uccisione.
legend-shot = Colpo
legend-charge = Carica
legend-kill = Uccisione
legend-death = Morte
legend-switch = Cambio d’arma

column-headshots = Colpi alla testa
filter-headshots = Uccisioni con colpo alla testa
event-headshot = colpo alla testa
scores-headshots = CT

column-accuracy = Precisione
filter-min-accuracy = Precisione minima (%)
scores-accuracy = Prec.

help-headshots = Uccisioni con un colpo alla testa, riconosciute dall’animazione di morte della vittima. Le esplosioni non contano. Si mostrano solo quelle di chi registra, perché gli altri giocatori uccidono spesso fuori dalla sua vista. In un duello anche quelle dell’avversario, dato che colpiscono tutte chi registra.
help-headshots-column = Uccisioni di chi registra con un colpo alla testa, riconosciute dall’animazione di morte della vittima. Le esplosioni non contano. Quelle degli altri giocatori non contano, perché avvengono spesso fuori dalla vista della registrazione.
help-accuracy = La quota di colpi andati a segno, in base al sangue che un colpo lascia lungo la linea di mira, meno le corrispondenze casuali. Comprende glock, python, proiettili dell’MP5, fucile a pompa, balestra con zoom e i colpi diretti del raggio gauss. Esplosioni e rimbalzi del gauss non lasciano sangue e non contano. Mostrata solo per chi registra, dopo almeno 50 colpi.
help-headshot-filter = Demo con almeno un’uccisione con colpo alla testa visibile nella registrazione, di qualsiasi giocatore.
help-accuracy-filter = Demo in cui chi registra ha colpito almeno questa quota dei suoi colpi.

clips-title = Clip
clips-new = Nuova clip
clips-add-event = Clip { $start } – { $end }
clips-add-event-none = Clip dell’evento scelto
clips-select-event = Scegli prima un evento nell’elenco o sulla linea del tempo.
clips-other-session = Una clip non può iniziare qui, perché una sessione precedente della registrazione ha gli stessi tempi.
clips-add-window = Clip di questo momento intenso
clips-empty = Ancora nessuna clip. Scegli un evento o uno dei momenti più intensi, poi aggiungi una clip attorno. Quanto una clip tiene prima e dopo un evento si regola nelle Impostazioni. Nuova clip ne aggiunge una di cui scrivi tu inizio e fine.
clip-default-name = Clip { $number }
clip-name = Nome
clip-length = Durata { $duration }
clip-show = Mostra
clip-remove = Rimuovi
clip-loop = Ripeti
clip-time-help = Secondi (96), minuti e secondi (1:36) oppure ore, minuti e secondi (00:01:36.250)
clip-exact = L’esportazione inizia e finisce esattamente qui.
clip-effective = L’esportazione taglia { $start } – { $end } ({ $duration }), secondo i fotogrammi della registrazione.
settings-clip-padding = Margine delle clip
settings-preroll = Secondi prima dell’evento
settings-postroll = Secondi dopo l’evento
export-start-first-group = Un taglio non può iniziare dal primissimo fotogramma della registrazione. Inizia un po’ più tardi.
export-start-during-loading = L’inizio cade mentre il gioco sta ancora caricando una mappa. Inizia un po’ più tardi.

export-all = Esporta tutte le clip
export-one = Esporta
export-cancel = Ferma l’esportazione
export-folder = Le esportazioni vanno in { $folder }.
export-folder-beside = Le esportazioni vanno accanto alla demo, in { $folder }.
export-waiting = In attesa di esportazione…
export-running = Esportazione in corso… La demo viene letta tutta e può volerci un po’.
export-cancelled = Non esportata, perché l’esportazione è stata fermata.
export-saved = Salvata in { $path }. Snapshot confrontati con l’originale: { $count }.
export-playdemo = Nella console del gioco:
export-copy-first = Copia { $name } nella cartella della tua mod, quella con liblist.gam, poi nella console del gioco:
export-copy = Copia
settings-export-folder = Cartella di esportazione
settings-export-choose = Scegli…
settings-viewer = Visualizzatore di demo
settings-viewer-game = Cartella di Half-Life
settings-viewer-game-none = Non scelta
settings-viewer-game-hint = La cartella che contiene valve. Il visualizzatore riproduce le demo con le mappe, i modelli e i suoni del gioco che vi si trovano.
settings-executable = Eseguibile del gioco
settings-executable-hint = Serve a trovare la cartella di Half-Life. Il visore legge da sé i file del gioco e non avvia mai il gioco.
settings-executable-not-set = Non impostato, quindi il visore non ha immagine né suono
settings-executable-optional = Non impostato, e non serve finché la cartella di Half-Life contiene il gioco
settings-executable-missing = { $path } non c’è più. Il visore continua a leggere la cartella di Half-Life.
settings-game-override = Questa esecuzione usa { $path }, dato con --game. Una scelta fatta qui vale dalla prossima.
settings-browse = Sfoglia…
dialog-executables = Programmi
setup-title = Configura il visore delle demo
setup-body = Immagine e suono hanno bisogno dei file di Half-Life. Mostra a Lambda Slicer dove è installato il gioco e li leggerà da lì.
setup-locate-exe = Trova hl.exe…
setup-locate-folder = Trova Half-Life…
setup-settings = Impostazioni
setup-still-works = Timeline, eventi e clip funzionano anche senza.
setup-relocate = { $folder } non contiene più Half-Life. Trovalo di nuovo.
setup-not-executable = { $name } non è { $executable }. Scegli l’eseguibile del gioco, non un collegamento.
setup-not-installed = { $folder } non contiene un’installazione di Half-Life con la cartella valve.
tab-events = Eventi
tab-clips = Clip
tab-scoreboard = Punteggi
list-hide = Nascondi l’elenco delle demo
list-show = Mostra l’elenco delle demo
demo-details = Dettagli
viewer-start = Torna all’inizio
viewer-mute = Disattiva audio
viewer-unmute = Riattiva audio
viewer-picture = Immagine e impostazioni del visore
viewer-needs-files = Servono i file del gioco
viewer-no-sound-why = Nessun suono: { $reason }
settings-tab-general = Generale
settings-section-folders = Cartelle
settings-folders-hint = Le cartelle in cui la libreria cerca le demo, sottocartelle comprese.
settings-clip-padding-hint = Quanto una clip creata attorno a un evento conserva prima e dopo di esso.
settings-section-picture = Immagine
settings-section-sound = Audio
settings-gamma = Gamma
settings-brightness = Luminosità
settings-texgamma = Gamma delle texture
settings-lightgamma = Gamma della luce
settings-overbright = Illuminazione sovraesposta
settings-gamma-hint = Le impostazioni del gioco con lo stesso nome (gamma, brightness, texgamma, lightgamma, gl_overbright). Impostale come nel gioco per vedere la stessa luminosità.
settings-gamma-defaults = Valori del gioco
settings-fov-recorded = Campo visivo come registrato
settings-fov = Campo visivo
settings-viewmodel = Mostra l’arma in mano
settings-see-through = Vedi i giocatori attraverso i muri
settings-see-through-items = Anche altri modelli, come proiettili e oggetti
settings-see-through-hint = I giocatori dietro i muri appaiono come sagome del loro colore superiore, che i giochi a squadre impostano per squadra. Si vede solo ciò che la demo contiene. Il server invia i giocatori vicini a chi registra, e i server che nascondono i giocatori che chi registra non può vedere non lasciano nulla da mostrare.
viewer-see-through = Vedi attraverso i muri
settings-export-beside = Accanto a ogni demo
settings-export-hint = Nella cartella della tua mod, quella con liblist.gam come Half-Life\ag, le clip partono con un solo comando playdemo.

clips-store-moved = Non è stato possibile leggere il file delle clip { $path }, quindi è stato conservato come { $moved } e le clip ripartono da zero. { $detail }
clips-store-unreadable = Non è stato possibile leggere il file delle clip { $path }: { $detail }. In questa sessione le clip non vengono salvate, così nulla del suo contenuto va perso.
clips-save-failed = Non è stato possibile salvare le clip in { $path }: { $detail }
notice-dismiss = Chiudi

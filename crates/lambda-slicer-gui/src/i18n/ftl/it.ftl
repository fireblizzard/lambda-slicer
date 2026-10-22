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
settings-workers = Demo analizzate insieme
settings-workers-hint = Analizzarne di più insieme finisce prima, ma fa lavorare di più il computer.
settings-cache = Dove viene tenuta la libreria
settings-sort = Ordine all’avvio
settings-sort-hint = Come vengono ordinate le demo a ogni avvio di Lambda Slicer. Cliccando l’intestazione di una colonna si ordina comunque per quella, finché non chiudi Lambda Slicer.
settings-start-filters = Filtri all’avvio
settings-start-filters-hint = La libreria parte con questi filtri a ogni avvio di Lambda Slicer, così le demo brevi che non vuoi mai vedere restano fuori dai piedi. Puoi comunque cambiare i filtri finché non chiudi Lambda Slicer.
settings-hide-unplayed-maps = Nelle demo con più mappe, nascondi le mappe senza uccisioni e registrate per meno di 5 minuti
settings-folders = Cartelle della libreria
settings-folders-empty = Nessuna cartella aggiunta.
settings-remove = Rimuovi

## Libreria

library-empty = Aggiungi una cartella o apri una demo per iniziare.
library-no-matches = Nessuna demo corrisponde a questi filtri.
library-no-cache = Su questo computer non c’è nessun posto dove tenere la libreria.
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
path-copy = Copia posizione del file
path-show = Mostra nella cartella
path-show-failed = Impossibile aprire la cartella ({ $detail }). Copia invece la posizione del file.

filter-search = Cerca nei nomi dei file e nelle cartelle
filter-map = Mappa
filter-player = Giocatore
filter-weapon-any = Qualsiasi arma
filter-min-minutes = Almeno (minuti)
filter-min-kills = Almeno (uccisioni)
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
kind-hltv = HLTV
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

scan-progress = Trovate { $found }, fatte { $done } ({ $cached } già note, { $analysed } analizzate), { $megabytes } MB letti
scan-cancelling = Interruzione…
scan-cancelled = Interrotta, { $left } ancora da scansionare.
scan-finished = Fatto: { $done } demo, { $analysed ->
    [one] { $analysed } analizzata
   *[other] { $analysed } analizzate
}, in { $seconds } s.
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

analysis-panicked = L’analisi si è fermata per un problema in Lambda Slicer: { $detail }
cache-cancelled = L’analisi è stata annullata, quindi non è stato salvato nulla.
cache-database = Impossibile usare la libreria salvata: { $detail }
cache-directory = Impossibile creare la cartella per la libreria salvata: { $detail }
container-bad-magic = Questa non è una demo di Half-Life.
container-frame-bad-time = La registrazione è danneggiata al fotogramma { $frame } della parte { $entry }, che ha un tempo sbagliato.
container-frame-missing-terminal = La parte { $entry } della registrazione non finisce come dovrebbe.
container-frame-negative-length = La registrazione è danneggiata al fotogramma { $frame } della parte { $entry }.
container-frame-over-limit = Il fotogramma { $frame } della parte { $entry } è più grande del consentito.
container-frame-truncated = La registrazione si interrompe al fotogramma { $frame } della parte { $entry }.
container-frame-unknown-tag = La registrazione è danneggiata al fotogramma { $frame } della parte { $entry }, che contiene qualcosa che Lambda Slicer non riconosce.
container-io = Impossibile leggere il file: { $detail }
container-no-directory = La registrazione non è mai stata completata, quindi non ha un indice.
container-range = L’indice della registrazione punta fuori dal file.
container-too-short = Il file è troppo corto per essere una demo.
coverage-hltv = Registrazione HLTV. Nessun giocatore l’ha registrata, quindi gli eventi seguono ciò che HLTV stava guardando.
coverage-mostly-spectating = Chi registrava è stato spettatore per il { $percent }% della riproduzione, quindi la maggior parte degli eventi è di altri giocatori, come si vedevano mentre li seguiva da spettatore.
coverage-no-local-events = Le azioni di chi registrava non sono state salvate in questa demo, quindi si vedono solo quelle inviate dal server.
coverage-singleplayer = Giocatore singolo. Tutto il resto sono mostri o parti della mappa, e non c’è il kill feed.
coverage-unknown = Non è stato possibile leggere la registrazione abbastanza da capire di che tipo è.
coverage-visible-only = Il gioco riceve le azioni degli altri giocatori solo finché il server pensa che chi registra possa vederli, quindi i loro conteggi possono essere più bassi di quanto è successo davvero.
decode-failed = La lettura della registrazione si è fermata a { $time } (fotogramma { $frame }): { $detail }
export-active-effect = { $effect } è ancora in corso all’inizio, e una clip non può ancora portarlo con sé.
export-client-dropped-packets = A { $time } il gioco ha scartato dati arrivati troppo tardi, cosa che una clip non sa ancora copiare.
export-cross-segment = La mappa è cambiata o il giocatore si è riconnesso a { $time }, dentro la clip, e una clip non può andare oltre quel punto.
export-destination-exists = { $path } esiste già. Scegli un altro nome.
export-empty-range = La clip non contiene nulla da esportare.
export-io = Impossibile scrivere { $path }: { $detail }
export-missing-delta-base = Alla registrazione mancano dati di prima dell’inizio che servono alla clip (numero { $sequence }).
export-missing-parent = La cartella { $path } non esiste.
export-range-beyond-end = La fine { $end } è oltre la fine della registrazione, { $segment_end }.
export-same-as-source = L’esportazione sostituirebbe la demo da cui proviene.
export-source-changed = { $path } è cambiato durante l’esportazione.
export-source-decode-failed = Impossibile leggere la demo: { $detail }
export-source-failed-crosscheck = Al controllo, ciò che Lambda Slicer ha ricavato da questa demo non corrispondeva alla demo stessa, quindi le clip tratte da essa non sono affidabili.
export-unknown-persistent-message = { $name } prima dell’inizio imposta qualcosa che una clip non può ancora portare con sé.
export-unproven-state = A { $time } la registrazione si basa su dati che si sono dovuti indovinare, quindi la clip non può essere fatta in modo esatto.
export-unrepresentable-time-field = Un tempo all’inizio non può essere salvato in modo esatto: { $detail }
export-unsupported = Non supportato: { $detail }
export-validation-failed = La clip esportata non ha superato il controllo finale: { $detail }
io-not-found = File o cartella non trovati.
io-other = { $detail }
io-permission-denied = Accesso negato.
io-unexpected-end = Il file è finito inaspettatamente.
protocol-unsupported = Questa demo è di una versione del gioco non supportata (protocollo { $demo }/{ $network }). È supportata solo la 5/48.
protocol-unsupported-network = Il server è passato a una versione del gioco non supportata (protocollo { $network }).

## Vista della demo

demo-close = Chiudi
demo-loading = Analisi di { $name }…
show-shots = Colpi
show-actions = Altre azioni
show-kills = Uccisioni
show-switches = Cambi d’arma
show-chat = Chat
show-match = Partita
lane-unknown = Giocatore sconosciuto
lane-recorder = { $name } (registra)
event-kill = Ha ucciso { $victim }
event-switch = Cambio d’arma
event-vote-called = Votazione indetta su { $subject }
event-vote-cast = Votazione su { $subject }: { $change } ({ $tally })
event-vote-more-yes = { $count ->
    [one] un sì in più
    [many] { $count } sì in più
   *[other] { $count } sì in più
}
event-vote-fewer-yes = { $count ->
    [one] un sì in meno
    [many] { $count } sì in meno
   *[other] { $count } sì in meno
}
event-vote-more-no = { $count ->
    [one] un no in più
    [many] { $count } no in più
   *[other] { $count } no in più
}
event-vote-fewer-no = { $count ->
    [one] un no in meno
    [many] { $count } no in meno
   *[other] { $count } no in meno
}
event-vote-tally = { $yes } sì, { $no } no, { $undecided } indecisi
event-vote-accepted = Votazione su { $subject } approvata
event-vote-denied = Votazione su { $subject } respinta
event-timeout-called = Timeout chiesto
event-timeout-under-way = Timeout iniziato
event-timeout-over = Timeout finito
event-paused = Partita in pausa
event-resumed = Partita ripresa
event-sudden-death = Morte improvvisa iniziata
event-sudden-death-over = Morte improvvisa finita
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
column-file-date = Data del file
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
viewer-position = Posizione nella demo
viewer-skip-back = Indietro di 10 secondi
viewer-skip-forward = Avanti di 10 secondi
viewer-rate = { $rate }×
viewer-shortcut = Scorciatoia: { $key }
key-space = Spazio
key-shift-left = Shift+Sinistra
key-shift-right = Shift+Destra
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
timeline-gestures = Clicca su un segno per selezionarne l’evento, o in qualsiasi altro punto per andarci. Trascina sui tempi in alto, o trascina la linea che segna il momento attuale, per muoverti nella demo. Trascina in qualsiasi altro punto per far scorrere la vista. Usa la rotellina per ingrandire e rimpicciolire dove si trova il mouse, e fai doppio clic per vedere tutta la demo. Trascina il bordo di una clip per cambiare dove inizia o finisce, o clicca sulla clip per modificarla.
timeline-density = Le barre ombreggiate mostrano quanti degli eventi mostrati capitano in ogni punto, rispetto al punto più fitto della demo.
timeline-density-count = { $count ->
    [one] Un evento mostrato qui
    [many] { $count } di eventi mostrati qui
   *[other] { $count } eventi mostrati qui
}
timeline-session-count = Parte { $session }: { $count }
timeline-choose = Eventi in questo punto
legend-clip = Clip
event-session = Parte { $session }
event-unreachable = Questo evento viene dopo un cambio di mappa o una riconnessione, dove l’orologio della demo è ripartito da zero, quindi il visore non può raggiungerlo.
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
help-midair-either = Il killer, la vittima o entrambi erano in aria. Il killer è controllato al colpo legato all’uccisione, o all’uccisione se non c’è, e la vittima alla morte. Un semplice salto, a meno di 50 unità dal terreno, non conta. Serve aver scelto la cartella di Half-Life, per sapere dov’è il terreno.
help-midair-both = Il killer e la vittima erano entrambi in aria, ciascuno al momento del controllo.
help-midair-one = Esattamente uno dei due era in aria, e dell’altro si sa che era a terra.
speed-at-least = Almeno
speed-unit = ups
speed-killer = Killer
speed-victim = Vittima
help-speed = Velocità sul terreno in ups, come la mostra un tachimetro, senza contare il movimento in su e in giù. Il killer è misurato al colpo legato all’uccisione, o all’uccisione se non c’è, e la vittima alla morte. Le velocità degli altri giocatori sono stime.
wallgauss-weapon = Wallgauss riguarda solo le uccisioni col gauss, e il filtro arma mostra { $weapon }.
wallgauss-checking = Controllo Wallgauss… { $examined } di { $total } uccisioni
wallgauss-needs-files = Wallgauss ha bisogno dei file del gioco per sapere dove sono i muri della mappa.
wallgauss-failed = Impossibile controllare Wallgauss. { $reason }
help-wallgauss = Uccisioni col gauss attraverso un muro, trovate seguendo ogni colpo nella mappa come fa il server. Un’uccisione che non si può stabilire, come una in cui la vittima era fuori dalla vista della registrazione, conta come sconosciuta e resta esclusa.
badge-wall = WG
wall-yes-exit = Wallgauss, ricavato dal colpo: l’esplosione dove il raggio è uscito da un muro spesso { $thickness } unità.
wall-yes-beyond = Wallgauss, ricavato dal colpo: il raggio ha colpito dopo aver attraversato un muro spesso { $thickness } unità.
wall-limits = Ricavato dal colpo e da dove si trovavano i giocatori, perché il server non lo registra. Un giocatore è largo 32 unità e conta come una scatola, e la vittima poteva trovarsi a uno o due corpi di distanza da dove la mostra la demo.
wall-no-direct = Non è un Wallgauss: il raggio poteva colpire la vittima prima di qualsiasi muro.
wall-no-reflection = Non è un Wallgauss: l’esplosione di un rimbalzo poteva colpire la vittima prima di qualsiasi muro.
wall-doubt-no-shot = Wallgauss sconosciuto: il colpo del killer non è nella registrazione.
wall-doubt-no-map = Wallgauss sconosciuto: impossibile leggere i file della mappa.
wall-doubt-unseen = Wallgauss sconosciuto: la vittima era fuori dalla vista della registrazione.
wall-doubt-brush = Wallgauss sconosciuto: il raggio ha colpito una porta, un ascensore o un’altra parte speciale della mappa, che non viene controllata.
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
events-follow = Tieni l’elenco sul momento attuale mentre la demo è in riproduzione. Scorrere l’elenco lo disattiva.
clip-it = Clippa
badge-headshot = CT
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = suicidio
badge-teamkill = teamkill
badge-world = mondo
badge-clip = clip
clip-of = Clip di { $name }
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
filter-midair = Uccisioni in midair
event-headshot = colpo alla testa
scores-headshots = CT

column-accuracy = Precisione
filter-min-accuracy = Precisione minima (%)
scores-accuracy = Prec.

help-headshots = Uccisioni con un colpo alla testa, riconosciute dall’animazione di morte della vittima. Le esplosioni non contano. Si mostrano solo quelle di chi registra, perché gli altri giocatori uccidono spesso fuori dalla sua vista. In un duello anche quelle dell’avversario, dato che colpiscono tutte chi registra.
help-headshots-column = Uccisioni di chi registra con un colpo alla testa, riconosciute dall’animazione di morte della vittima. Le esplosioni non contano. Quelle degli altri giocatori non contano, perché avvengono spesso fuori dalla vista della registrazione.
help-date-column = Quando è stata registrata la partita. La data è presa dal nome del file se il nome contiene una data come 20230910_214553, come nelle registrazioni automatiche del gioco. Altrimenti è la data del file, che copiarlo o scaricarlo può cambiare.
help-file-date = Quando il file è stato salvato. Per una clip esportata, è quando è stata esportata.
help-accuracy = La quota di colpi andati a segno, in base al sangue che un colpo lascia lungo la linea di mira, meno le corrispondenze casuali. Comprende glock, python, proiettili dell’MP5, fucile a pompa, balestra con zoom e i colpi diretti del raggio gauss. Esplosioni e rimbalzi del gauss non lasciano sangue e non contano. Mostrata solo per chi registra, dopo almeno 50 colpi.
help-headshot-filter = Demo con almeno un’uccisione con colpo alla testa visibile nella registrazione, di qualsiasi giocatore.
help-midair-filter = Demo con almeno un’uccisione in cui il killer o la vittima era in aria, ad almeno 50 unità dal terreno. Senza aver scelto la cartella di Half-Life, conta anche un semplice salto.
help-accuracy-filter = Demo in cui chi registra ha colpito almeno questa quota dei suoi colpi.

clips-new = Nuova clip
clips-select-event = Scegli prima un evento nell’elenco o sulla linea del tempo.
clips-other-session = Una clip non può iniziare qui, perché l’orologio della demo è ripartito da zero dopo un cambio di mappa o una riconnessione, e una parte precedente ha gli stessi tempi.
clips-add-window = Clip di questo momento intenso
clip-default-name = Clip { $number }
clip-name = Nome
clip-length = Durata { $duration }
clip-remove = Rimuovi
clip-loop = Ripeti
clip-time-help = Secondi (96), minuti e secondi (1:36) oppure ore, minuti e secondi (00:01:36.250)
clips-empty-hint = Ancora nessuna clip. Scegli un evento e premi Clippa, oppure premi Nuova clip per iniziarne una dal momento attuale.
clips-new-help = Una clip di dieci secondi dal momento attuale.
clip-in = Inizio
clip-out = Fine
clip-set-in = Inizia la clip dal momento attuale
clip-set-out = Finisci la clip al momento attuale
clip-no-playhead = Il visore non è disponibile, quindi non c’è un momento attuale da usare.
clip-play = Riproduci la clip una volta
clip-show-on-timeline = Mostra sulla timeline
export-destination = Cambia la cartella di esportazione
export-skipped = Saltata. { $reason }
export-earlier-range = Questa esportazione è di { $start } – { $end }, la clip com’era prima che la cambiassi.
scores-player = Giocatore
scores-none = La registrazione non ha punteggi.
clip-effective = L’esportazione copre { $start } – { $end } ({ $duration }), il più vicino che la registrazione permette.
settings-clip-padding = Margine delle clip
settings-preroll = Secondi prima dell’evento
settings-postroll = Secondi dopo l’evento
export-start-first-group = Una clip non può iniziare proprio all’inizio della registrazione. Inizia un po’ più tardi.
export-start-during-loading = L’inizio cade mentre il gioco sta ancora caricando una mappa. Inizia un po’ più tardi.

export-all = Esporta tutte le clip
export-one = Esporta
export-cancel = Ferma l’esportazione
export-folder = Le esportazioni vanno in { $folder }.
export-folder-beside = Le esportazioni vanno accanto alla demo, in { $folder }.
export-waiting = In attesa di esportazione…
export-running = Esportazione in corso… La demo viene letta tutta e può volerci un po’.
export-cancelled = Non esportata, perché l’esportazione è stata fermata.
export-saved = Salvata in { $path }. Momenti confrontati con l’originale: { $count }.
export-playdemo = Nella console del gioco:
export-copy-first = Copia { $name } nella cartella della tua mod, quella con liblist.gam, poi nella console del gioco:
export-copy = Copia
settings-export-folder = Cartella di esportazione
settings-export-choose = Scegli…
settings-file-names = Nomi dei file
settings-clip-name-files = Usa il nome della clip per il file
settings-demo-name-first = Inizia con il nome della demo
settings-viewer = Visualizzatore di demo
settings-viewer-game = Cartella di Half-Life
settings-viewer-game-none = Non scelta
settings-viewer-game-hint = La cartella che contiene valve. Il visualizzatore riproduce le demo con le mappe, i modelli e i suoni del gioco che vi si trovano.
settings-executable = Programma del gioco
settings-executable-hint = Serve a trovare la cartella di Half-Life. Il visore legge da sé i file del gioco e non avvia mai il gioco.
settings-executable-not-set = Non impostato, quindi il visore non ha immagine né suono
settings-executable-optional = Non impostato, e non serve finché la cartella di Half-Life contiene il gioco
settings-executable-missing = { $path } non c’è più. Il visore continua a leggere la cartella di Half-Life.
settings-game-override = Questa volta Lambda Slicer usa { $path }, perché è stato avviato con --game. Una scelta fatta qui vale dal prossimo avvio.
settings-browse = Sfoglia…
dialog-executables = Programmi
setup-title = Configura il visore delle demo
setup-body = Immagine e suono hanno bisogno dei file di Half-Life. Mostra a Lambda Slicer dove è installato il gioco e li leggerà da lì.
setup-locate-exe = Trova hl.exe…
setup-locate-folder = Trova Half-Life…
setup-settings = Impostazioni
setup-still-works = Timeline, eventi e clip funzionano anche senza.
setup-relocate = { $folder } non contiene più Half-Life. Trovalo di nuovo.
setup-not-executable = { $name } non è { $executable }. Scegli il programma del gioco stesso, non un collegamento.
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
settings-tab-library = Libreria
settings-tab-clips = Clip
settings-section-game = Half-Life
settings-section-view = Vista
settings-language-system = Lingua del computer ({ $name })
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
settings-zoom-fov = Campo visivo con lo zoom
settings-viewmodel = Mostra l’arma in mano
settings-crosshair = Mostra il mirino
settings-chat = Mostra la chat
settings-vitals = Mostra vita, armatura, munizioni e oggetti raccolti
settings-vitals-size = Dimensione di vita, armatura, munizioni e oggetti raccolti
settings-scores = Mostra i punteggi in alto a sinistra
settings-score-rows = Numero massimo di righe dei punteggi
settings-server-settings = Mostra le impostazioni del server in alto a destra
settings-hud-messages = Mostra i testi del server, come i timer di bhop e KZ
settings-see-through = Vedi i giocatori attraverso i muri
settings-see-through-items = Anche altri modelli, come proiettili e oggetti
settings-see-through-hint = I giocatori dietro i muri appaiono come sagome del loro colore superiore, che i giochi a squadre impostano per squadra. Si vede solo ciò che la demo contiene. Il server invia i giocatori vicini a chi registra, e i server che nascondono i giocatori che chi registra non può vedere non lasciano nulla da mostrare.
settings-section-decals = Decal
settings-decals = Massimo
settings-decals-hint = Fori di proiettile, sangue e altri segni lasciati durante la partita. Restano solo i più recenti, e i più vecchi spariscono quando ne compaiono di nuovi. I segni che fanno parte della mappa restano sempre.
settings-restore = Ripristina predefiniti…
settings-restore-title = Ripristina predefiniti
settings-restore-list = Queste impostazioni tornano com’erano all’inizio. La cartella di Half-Life e le cartelle della libreria restano come sono.
settings-restore-none = Tutte le impostazioni sono già quelle predefinite.
settings-restore-columns = Colonne mostrate nella libreria
settings-restore-layout = Pannelli aperti e loro dimensioni
settings-clear-library = Svuota la libreria…
settings-clear-library-title = Svuota la libreria
settings-clear-library-text = { $count ->
    [one] Lambda Slicer dimentica quello che ha trovato in { $count } demo e analizza di nuovo le cartelle della libreria, il che può richiedere un po’ di tempo. I tuoi file delle demo, le tue clip e le tue impostazioni restano come sono.
    [many] Lambda Slicer dimentica quello che ha trovato in { $count } di demo e analizza di nuovo le cartelle della libreria, il che può richiedere un po’ di tempo. I tuoi file delle demo, le tue clip e le tue impostazioni restano come sono.
   *[other] Lambda Slicer dimentica quello che ha trovato in { $count } demo e analizza di nuovo le cartelle della libreria, il che può richiedere un po’ di tempo. I tuoi file delle demo, le tue clip e le tue impostazioni restano come sono.
}
viewer-zoom = Zoom avanti
viewer-see-through = Vedi attraverso i muri
viewer-paused = In pausa
viewer-map-change = Il server sta passando alla mappa { $map }
settings-export-beside = Accanto a ogni demo
settings-export-hint = Nella cartella della tua mod, quella con liblist.gam come Half-Life\ag, le clip partono con un solo comando playdemo.
settings-file-names-hint = Una clip esportata prende il nome dalla sua demo e dai suoi tempi di inizio e fine, a meno che tu non usi il nome della clip. Il gioco apre solo file con nomi fatti di lettere dell’alfabeto inglese, numeri, - e ., quindi tutto il resto diventa _. Se il nome di una clip non contiene nessuno di questi caratteri, il file prende il nome dai suoi tempi.

clips-store-moved = Non è stato possibile leggere il file delle clip { $path }, quindi è stato conservato come { $moved } e le clip ripartono da zero. { $detail }
clips-store-unreadable = Non è stato possibile leggere il file delle clip { $path }: { $detail }. Finché Lambda Slicer non viene chiuso, le clip non vengono salvate, così nulla del contenuto del file va perso.
clips-save-failed = Non è stato possibile salvare le clip in { $path }: { $detail }
notice-dismiss = Chiudi

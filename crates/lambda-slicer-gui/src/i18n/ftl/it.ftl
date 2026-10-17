# Italiano. Bozza non revisionata

app-title = Lambda Slicer

## Barra degli strumenti

toolbar-open = Apri demo…
toolbar-add-folder = Aggiungi cartella…
toolbar-rescan = Analizza di nuovo
toolbar-cancel = Annulla
toolbar-settings = Impostazioni
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
timeline-fit = Adatta
show-shots = Colpi
show-actions = Altre azioni
show-kills = Uccisioni
show-switches = Cambi d’arma
lane-unknown = Giocatore sconosciuto
lane-recorder = { $name } (registra)
column-time = Tempo
column-player = Giocatore
column-weapon = Arma
column-event = Evento
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
timeline-windows = Momenti più intensi
timeline-window = N. { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Punteggi finali, all’ultima uccisione { $time }
scores-at = Punteggi a { $time }
scores-show-final = Mostra i punteggi finali
scores-frags = Frag
scores-deaths = Morti

show-timeline = Mostra la timeline
legend-shot = Colpo
legend-charge = Carica
legend-kill = Uccisione
legend-death = Morte
legend-switch = Cambio d’arma
timeline-help = Ogni riga è un giocatore. Scorri per lo zoom, trascina per spostarti, doppio clic per vedere tutto, clic su un segno per selezionarlo.

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
clips-add-event = Clip { $start } – { $end }
clips-add-event-none = Clip dell’evento scelto
clips-select-event = Scegli prima un evento nell’elenco o sulla linea del tempo.
clips-other-session = Per ora si può esportare solo la prima sessione della registrazione.
clips-add-window = Clip di questo momento intenso
clips-empty = Ancora nessuna clip. Scegli un evento o uno dei momenti più intensi, poi aggiungi una clip attorno. Quanto una clip tiene prima e dopo un evento si regola nelle Impostazioni.
clip-default-name = Clip { $number }
clip-name = Nome
clip-length = Durata { $duration }
clip-show = Mostra
clip-remove = Rimuovi
clip-time-help = Secondi (96), minuti e secondi (1:36) oppure ore, minuti e secondi (00:01:36.250)
clip-exact = L’esportazione inizia e finisce esattamente qui.
clip-effective = L’esportazione taglia { $start } – { $end } ({ $duration }), secondo i fotogrammi della registrazione.
timeline-clip-help = Le clip sono le bande azzurre. Trascina il bordo di una banda per spostarlo.
settings-clip-padding = Margine delle clip
settings-preroll = Secondi prima dell’evento
settings-postroll = Secondi dopo l’evento
export-start-first-group = Un taglio non può iniziare dal primissimo fotogramma della registrazione. Inizia un po’ più tardi.
export-start-later-session = L’inizio cade in una sessione successiva della registrazione, da cui un taglio non può ancora iniziare.

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
settings-export-beside = Accanto a ogni demo
settings-export-hint = Nella cartella della tua mod, quella con liblist.gam come Half-Life\ag, le clip partono con un solo comando playdemo.

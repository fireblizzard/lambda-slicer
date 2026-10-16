# Italiano. Bozza non revisionata

app-title = Lambda Slicer

toolbar-open = Apri demo…
toolbar-add-folder = Aggiungi cartella…
toolbar-rescan = Analizza di nuovo
toolbar-cancel = Annulla
toolbar-settings = Impostazioni

settings-title = Impostazioni
settings-language = Lingua
settings-unreviewed = Questa traduzione è una bozza che nessun madrelingua ha ancora revisionato.
settings-workers = Processi di analisi
settings-workers-hint = Più processi analizzano più in fretta, ma usano più tempo del processore.
settings-cache = Cache della libreria
settings-folders = Cartelle della libreria
settings-folders-empty = Nessuna cartella aggiunta.
settings-remove = Rimuovi
settings-close = Chiudi

library-empty = Aggiungi una cartella o apri una demo per iniziare.
library-demos = { $count ->
    [one] { $count } demo
    [many] { $count } di demo
   *[other] { $count } demo
}
library-kills = { $count ->
    [one] { $count } uccisione
    [many] { $count } di uccisioni
   *[other] { $count } uccisioni
}
library-action = Punteggio azione { NUMBER($score, minimumFractionDigits: 2) }
library-spectating = Spettatore { $percent }%

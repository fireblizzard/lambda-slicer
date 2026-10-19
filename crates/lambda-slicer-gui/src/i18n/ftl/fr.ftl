# Français. Brouillon non relu

app-title = Lambda Slicer

## Barre d’outils

toolbar-open = Ouvrir une démo…
toolbar-add-folder = Ajouter un dossier…
toolbar-rescan = Analyser à nouveau
toolbar-cancel = Annuler
toolbar-settings = Paramètres
dialog-demos = Démos Half-Life

## Paramètres

settings-title = Paramètres
settings-language = Langue
settings-unreviewed = Cette traduction est un brouillon qu’aucun locuteur natif n’a encore relu.
settings-workers = Processus d’analyse
settings-workers-hint = Davantage de processus analysent plus vite, mais sollicitent davantage le processeur.
settings-cache = Cache de la bibliothèque
settings-folders = Dossiers de la bibliothèque
settings-folders-empty = Aucun dossier ajouté pour l’instant.
settings-remove = Retirer

## Bibliothèque

library-empty = Ajoutez un dossier ou ouvrez une démo pour commencer.
library-no-matches = Aucune démo ne correspond à ces filtres.
library-no-cache = Ce système n’offre aucun emplacement pour le cache de la bibliothèque.
library-demos = { $count ->
    [one] { $count } démo
    [many] { $count } de démos
   *[other] { $count } démos
}
library-percent = { $percent } %

filter-search = Rechercher dans les chemins
filter-map = Carte
filter-player = Joueur
filter-weapon-any = N’importe quelle arme
filter-min-minutes = Au moins (minutes)
filter-spectating-any = En jeu ou en spectateur
filter-spectating-mostly = Surtout en spectateur
filter-spectating-playing = Surtout en jeu
filter-clear = Effacer les filtres

column-status = État
column-name = Démo
column-map = Carte
column-duration = Durée
column-gauss = Gauss
column-kills = Éliminations
column-action = Intensité
column-spectating = Spectateur

status-complete = Analysée
status-partial = Partielle
status-failed = Échec
status-unsupported = Non prise en charge
status-no-use = Aucun usage d’arme

kind-multiplayer-view = Vue du joueur
kind-singleplayer = Solo
kind-hltv = Proxy spectateur
kind-unknown = Type inconnu

details-recorder = Enregistrée par { $name }
details-shots = { $count ->
    [one] { $count } tir
    [many] { $count } de tirs
   *[other] { $count } tirs
}
details-kills = { $count ->
    [one] { $count } élimination
    [many] { $count } d’éliminations
   *[other] { $count } éliminations
}
details-action = Intensité maximale { NUMBER($score, minimumFractionDigits: 2) }

## Analyse des dossiers

scan-progress = { $found } trouvées, { $done } traitées ({ $cached } en cache, { $analysed } analysées), { $megabytes } Mo lus
scan-cancelling = Arrêt…
scan-cancelled = Arrêté, { $left } restant à analyser.
scan-finished = Terminé : { $done } démos, { $analysed } analysées, en { $seconds } s.
scan-problems = { $count ->
    [one] { $count } problème
    [many] { $count } de problèmes
   *[other] { $count } problèmes
}
problems-title = Problèmes rencontrés
problem-changed = Le fichier a changé pendant l’analyse, donc rien n’a été enregistré.
drop-hint = Déposez des démos ou des dossiers pour les ajouter
drop-not-demo = Pas une démo : { $names }

## Messages du moteur d’analyse, par code

analysis-panicked = L’analyse s’est arrêtée sur une erreur interne : { $detail }
cache-cancelled = L’analyse a été annulée, donc rien n’a été enregistré.
cache-database = Impossible d’utiliser le cache de la bibliothèque : { $detail }
cache-directory = Impossible de créer le dossier du cache : { $detail }
container-bad-magic = Ce n’est pas une démo Half-Life.
container-frame-bad-time = L’enregistrement est endommagé à l’image { $frame } de la partie { $entry }, dont le temps n’est pas valide.
container-frame-missing-terminal = La partie { $entry } de l’enregistrement se termine sans son enregistrement de clôture.
container-frame-negative-length = L’enregistrement est endommagé à l’image { $frame } de la partie { $entry }.
container-frame-over-limit = L’image { $frame } de la partie { $entry } dépasse la taille autorisée.
container-frame-truncated = L’enregistrement s’interrompt à l’image { $frame } de la partie { $entry }.
container-frame-unknown-tag = L’enregistrement est endommagé à l’image { $frame } de la partie { $entry }, d’un type inconnu.
container-io = Impossible de lire le fichier : { $detail }
container-no-directory = L’enregistrement n’a jamais été terminé, il n’a donc pas de sommaire.
container-range = Le sommaire de l’enregistrement pointe en dehors du fichier.
container-too-short = Le fichier est trop court pour être une démo.
coverage-hltv = Enregistrement d’un proxy spectateur. Il n’y a pas de joueur qui enregistre, et les événements suivent la vue du proxy.
coverage-mostly-spectating = La personne qui enregistrait a été spectatrice pendant { $percent } % de la lecture, donc la plupart des événements sont ceux d’autres joueurs, tels que la vue observée les montrait.
coverage-no-local-events = Aucun événement local. Les actions de la personne qui enregistrait manquent, sauf celles que le serveur a envoyées.
coverage-singleplayer = Solo. Les autres acteurs sont des monstres ou des éléments de la carte, et il n’y a pas de fil des éliminations.
coverage-unknown = L’enregistrement n’a pas été décodé assez loin pour en connaître le type.
coverage-visible-only = Les événements des autres joueurs n’arrivent à ce client que tant que le serveur les juge visibles, leurs nombres sont donc des minimums.
decode-failed = Le décodage s’est arrêté à { $time } (image { $frame }) : { $detail }
export-active-effect = { $effect } est en cours au début et ne peut pas encore être repris.
export-client-dropped-packets = Le jeu a écarté des paquets en retard à { $time }, ce qu’une découpe ne sait pas encore reproduire.
export-cross-segment = Une nouvelle session commence à { $time }, dans la plage.
export-destination-exists = { $path } existe déjà. Choisissez un autre nom.
export-empty-range = La plage choisie ne contient rien à exporter.
export-io = Impossible d’écrire { $path } : { $detail }
export-missing-delta-base = Il manque à l’enregistrement un état nécessaire avant le début (séquence { $sequence }).
export-missing-parent = Le dossier { $path } n’existe pas.
export-range-beyond-end = La fin { $end } dépasse la fin de l’enregistrement, { $segment_end }.
export-same-as-source = La destination est la démo source elle-même.
export-source-changed = { $path } a changé pendant l’export.
export-source-decode-failed = La démo source ne se décode pas : { $detail }
export-source-failed-crosscheck = La démo source a échoué à sa vérification croisée, son état reconstruit n’est donc pas fiable.
export-unknown-persistent-message = { $name } avant le début fixe un état qu’une découpe ne sait pas encore reprendre.
export-unproven-state = L’état à { $time } repose sur une base déduite et n’est pas prouvé.
export-unrepresentable-time-field = Un temps au début ne peut pas être écrit fidèlement : { $detail }
export-unsupported = Non pris en charge : { $detail }
export-validation-failed = Le fichier exporté a échoué à la validation : { $detail }
io-not-found = Le fichier ou le dossier est introuvable.
io-other = { $detail }
io-permission-denied = Accès refusé.
io-unexpected-end = Le fichier s’est terminé de façon inattendue.
protocol-unsupported = Le protocole de démo { $demo }/{ $network } n’est pas pris en charge, seulement 5/48.
protocol-unsupported-network = Le serveur est passé au protocole réseau { $network }, qui n’est pas pris en charge.

## Vue de la démo

demo-close = Fermer
demo-loading = Analyse de { $name }…
timeline-fit = Ajuster
show-shots = Tirs
show-actions = Autres actions
show-kills = Éliminations
show-switches = Changements d’arme
lane-unknown = Joueur inconnu
lane-recorder = { $name } (enregistre)
column-time = Temps
column-player = Joueur
column-weapon = Arme
column-event = Événement
event-kill = A éliminé { $victim }
event-switch = Changement d’arme
action-swing = Coup
action-fire = Tir
action-launch = Lancer
action-charge-activity = Charge
action-charge-stop = Fin de charge
action-beam-start = Début du rayon
action-beam-sustain = Rayon
action-beam-stop = Fin du rayon
action-placement-attempt = Tentative de pose
action-release-attempt = Tentative de lâcher
mode-primary = principal
mode-charged = chargé
mode-zoomed = zoomé
mode-unzoomed = sans zoom
mode-grenade = grenade
mode-bullets = balles
mode-single = simple
mode-double = double
mode-rapid = rapide
demo-events = { $count ->
    [one] { $count } événement
    [many] { $count } d’événements
   *[other] { $count } événements
}

column-date = Date
column-score = Score
filter-from = Du
filter-to = Au
date-from-name = Date tirée du nom du fichier
date-from-modified = Date de dernière modification du fichier, qu’une copie ou un téléchargement peut changer
timeline-best-action = Minute en or
timeline-windows = Moments les plus intenses
timeline-window = N° { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Scores finaux, à la dernière élimination { $time }
scores-at = Scores à { $time }
scores-show-final = Voir les scores finaux
scores-frags = Frags
scores-deaths = Morts

show-timeline = Afficher la frise
viewer-play = Lecture
viewer-pause = Pause
viewer-volume = Volume
viewer-play-backward = Lecture arrière
viewer-step-back = Pas en arrière
viewer-step-forward = Pas en avant
viewer-rate = { $rate }×
viewer-shortcut = Raccourci : { $key }
key-space = Espace
viewer-map-failed = Impossible de charger la carte { $map }
viewer-stopped = La lecture s’est arrêtée avant la fin : { $reason }
event-previous = Événement précédent
event-next = Événement suivant
event-play = Lire l’événement
legend-shot = Tir
legend-charge = Charge
legend-kill = Élimination
legend-death = Mort
legend-switch = Changement d’arme
timeline-help = Chaque ligne est un joueur. La molette zoome, glisser déplace, un double-clic montre tout, un clic sur une marque la sélectionne.

column-headshots = Tirs à la tête
filter-headshots = Éliminations d’un tir à la tête
event-headshot = tir à la tête
scores-headshots = TT

column-accuracy = Précision
filter-min-accuracy = Précision minimale (%)
scores-accuracy = Préc.

help-headshots = Éliminations par un tir à la tête, reconnues à l’animation de mort de la victime. Les explosions ne comptent pas. Seules celles de la personne qui enregistre sont affichées, car les autres joueurs éliminent souvent hors de sa vue. En duel, celles de l’adversaire aussi, puisque toutes visent la personne qui enregistre.
help-headshots-column = Éliminations de la personne qui enregistre par un tir à la tête, reconnues à l’animation de mort de la victime. Les explosions ne comptent pas. Celles des autres joueurs ne comptent pas, car elles ont souvent lieu hors de la vue de l’enregistrement.
help-accuracy = La part des tirs qui ont touché, d’après le sang qu’un impact laisse sur la ligne de visée, moins les correspondances dues au hasard. Elle couvre le glock, le python, les balles du MP5, le fusil à pompe, l’arbalète avec zoom et les impacts directs du rayon gauss. Les explosions et les rebonds du gauss ne laissent pas de sang et ne comptent pas. Affichée pour la personne qui enregistre seulement, après au moins 50 tirs.
help-headshot-filter = Démos avec au moins une élimination d’un tir à la tête visible dans l’enregistrement, par n’importe quel joueur.
help-accuracy-filter = Démos où la personne qui enregistre a touché au moins cette part de ses tirs.

clips-title = Clips
clips-new = Nouveau clip
clips-add-event = Clip { $start } – { $end }
clips-add-event-none = Clip de l’événement choisi
clips-select-event = Choisis d’abord un événement dans la liste ou sur la frise.
clips-other-session = Un clip ne peut pas commencer ici, car une session antérieure de l’enregistrement a les mêmes temps.
clips-add-window = Clip de ce moment intense
clips-empty = Aucun clip pour l’instant. Choisis un événement ou l’un des moments les plus intenses, puis ajoute un clip autour. Ce qu’un clip garde avant et après un événement se règle dans les Paramètres. Nouveau clip en ajoute un dont tu saisis toi-même le début et la fin.
clip-default-name = Clip { $number }
clip-name = Nom
clip-length = Durée { $duration }
clip-show = Montrer
clip-remove = Retirer
clip-loop = Boucle
clip-time-help = Secondes (96), minutes et secondes (1:36) ou heures, minutes et secondes (00:01:36.250)
clip-exact = L’export commence et se termine exactement ici.
clip-effective = L’export coupe { $start } – { $end } ({ $duration }), selon les images de l’enregistrement.
timeline-clip-help = Les clips sont les bandes bleu clair. Fais glisser le bord d’une bande pour le déplacer.
settings-clip-padding = Marge des clips
settings-preroll = Secondes avant l’événement
settings-postroll = Secondes après l’événement
export-start-first-group = Une coupe ne peut pas commencer sur la toute première image de l’enregistrement. Commence un peu plus tard.
export-start-during-loading = Le début tombe pendant le chargement d’une carte. Commence un peu plus tard.

export-all = Exporter tous les clips
export-one = Exporter
export-cancel = Arrêter l’export
export-folder = Les exports vont dans { $folder }.
export-folder-beside = Les exports vont à côté de la démo, dans { $folder }.
export-waiting = En attente d’export…
export-running = Export en cours… La démo est lue en entier, cela peut prendre un moment.
export-cancelled = Non exporté, car l’export a été arrêté.
export-saved = Enregistré : { $path }. Instantanés vérifiés avec l’original : { $count }.
export-playdemo = Dans la console du jeu :
export-copy-first = Copie { $name } dans le dossier de ton mod, celui qui contient liblist.gam, puis dans la console du jeu :
export-copy = Copier
settings-export-folder = Dossier d’export
settings-export-choose = Choisir…
settings-export-beside = À côté de chaque démo
settings-export-hint = Dans le dossier de ton mod, celui qui contient liblist.gam comme Half-Life\ag, les clips se lancent avec une seule commande playdemo.

clips-store-moved = Le fichier de clips { $path } n’a pas pu être lu, il a donc été conservé sous { $moved } et les clips repartent de zéro. { $detail }
clips-store-unreadable = Le fichier de clips { $path } n’a pas pu être lu : { $detail }. Les clips ne sont pas enregistrés pendant cette session, donc rien de son contenu n’est perdu.
clips-save-failed = Les clips n’ont pas pu être enregistrés dans { $path } : { $detail }
notice-dismiss = Fermer

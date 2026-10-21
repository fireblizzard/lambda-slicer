# Français. Brouillon non relu

app-title = Lambda Slicer

## Barre d’outils

toolbar-open = Ouvrir une démo…
toolbar-add-folder = Ajouter un dossier…
toolbar-rescan = Analyser à nouveau
toolbar-cancel = Annuler
toolbar-settings = Paramètres
toolbar-rescan-busy = Une analyse est déjà en cours.
toolbar-rescan-nothing = Ajoutez un dossier ou ouvrez une démo à analyser.
toolbar-cancel-idle = Aucune analyse en cours.
dialog-demos = Démos Half-Life

## Paramètres

settings-title = Paramètres
settings-language = Langue
settings-unreviewed = Cette traduction est un brouillon qu’aucun locuteur natif n’a encore relu.
settings-workers = Démos analysées en même temps
settings-workers-hint = En analyser plus en même temps finit plus vite, mais fait davantage travailler l’ordinateur.
settings-cache = Où la bibliothèque est gardée
settings-folders = Dossiers de la bibliothèque
settings-folders-empty = Aucun dossier ajouté pour l’instant.
settings-remove = Retirer

## Bibliothèque

library-empty = Ajoutez un dossier ou ouvrez une démo pour commencer.
library-no-matches = Aucune démo ne correspond à ces filtres.
library-no-cache = Il n’y a aucun endroit sur cet ordinateur pour garder la bibliothèque.
library-demos = { $count ->
    [one] { $count } démo
    [many] { $count } de démos
   *[other] { $count } démos
}
library-percent = { $percent } %
library-demos-of = { $shown } sur { $total ->
    [one] { $total } démo
    [many] { $total } de démos
   *[other] { $total } démos
}
library-total = { $count ->
    [one] { $count } démo dans la bibliothèque
    [many] { $count } de démos dans la bibliothèque
   *[other] { $count } démos dans la bibliothèque
}
filters-title = Filtres
filters-find = Rechercher
filters-pov = Point de vue
filters-quality = Qualité
filters-recorded = Enregistrée
pov-any = Toutes
pov-playing = En jeu
pov-spectating = Spectateur
help-pov = « En jeu » veut dire que l’enregistrement a passé au plus la moitié de sa lecture en spectateur, et « Spectateur » plus de la moitié.
columns-button = Colonnes
list-back = Retour au tableau de la bibliothèque
sort-by = Trier par
sort-ascending = Croissant
sort-descending = Décroissant
score-more-sides = { $count ->
    [one] { $count } camp de plus
    [many] { $count } de camps de plus
   *[other] { $count } camps de plus
}
intensity-help = Intensité { $score }. La barre la compare à la plus haute de la bibliothèque, { $max }.
value-unknown = Inconnu
path-copy = Copier l’emplacement du fichier
path-show = Afficher dans le dossier
path-show-failed = Impossible d’ouvrir le dossier ({ $detail }). Copie plutôt l’emplacement du fichier.

filter-search = Rechercher dans les noms de fichiers et les dossiers
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
kind-hltv = HLTV
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

scan-progress = { $found } trouvées, { $done } traitées ({ $cached } déjà connues, { $analysed } analysées), { $megabytes } Mo lus
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

analysis-panicked = L’analyse s’est arrêtée à cause d’un problème dans Lambda Slicer : { $detail }
cache-cancelled = L’analyse a été annulée, donc rien n’a été enregistré.
cache-database = Impossible d’utiliser la bibliothèque enregistrée : { $detail }
cache-directory = Impossible de créer le dossier de la bibliothèque enregistrée : { $detail }
container-bad-magic = Ce n’est pas une démo Half-Life.
container-frame-bad-time = L’enregistrement est endommagé à l’image { $frame } de la partie { $entry }, dont le temps est faux.
container-frame-missing-terminal = La partie { $entry } de l’enregistrement ne se termine pas correctement.
container-frame-negative-length = L’enregistrement est endommagé à l’image { $frame } de la partie { $entry }.
container-frame-over-limit = L’image { $frame } de la partie { $entry } dépasse la taille autorisée.
container-frame-truncated = L’enregistrement s’interrompt à l’image { $frame } de la partie { $entry }.
container-frame-unknown-tag = L’enregistrement est endommagé à l’image { $frame } de la partie { $entry }, qui contient quelque chose que Lambda Slicer ne reconnaît pas.
container-io = Impossible de lire le fichier : { $detail }
container-no-directory = L’enregistrement n’a jamais été terminé, il n’a donc pas de sommaire.
container-range = Le sommaire de l’enregistrement pointe en dehors du fichier.
container-too-short = Le fichier est trop court pour être une démo.
coverage-hltv = Enregistrement HLTV. Aucun joueur ne l’a enregistré, donc les événements suivent ce que HLTV regardait.
coverage-mostly-spectating = La personne qui enregistrait a été spectatrice pendant { $percent } % de la lecture, donc la plupart des événements sont ceux d’autres joueurs, tels qu’on les voyait en les regardant en spectateur.
coverage-no-local-events = Les actions de la personne qui enregistrait n’ont pas été gardées dans cette démo, donc seules celles que le serveur a envoyées s’affichent.
coverage-singleplayer = Solo. Tout le reste, ce sont des monstres ou des éléments de la carte, et il n’y a pas de kill feed.
coverage-unknown = L’enregistrement n’a pas pu être lu assez loin pour savoir de quel type il est.
coverage-visible-only = Le jeu ne reçoit les actions des autres joueurs que tant que le serveur pense que la personne qui enregistre peut les voir, donc leurs nombres peuvent être plus bas que ce qui s’est vraiment passé.
decode-failed = La lecture de l’enregistrement s’est arrêtée à { $time } (image { $frame }) : { $detail }
export-active-effect = { $effect } est toujours en cours au début, et un clip ne sait pas encore le reprendre.
export-client-dropped-packets = À { $time }, le jeu a jeté des données arrivées trop tard, ce qu’un clip ne sait pas encore copier.
export-cross-segment = La carte a changé ou le joueur s’est reconnecté à { $time }, dans le clip, et un clip ne peut pas passer par-dessus.
export-destination-exists = { $path } existe déjà. Choisissez un autre nom.
export-empty-range = Le clip ne contient rien à exporter.
export-io = Impossible d’écrire { $path } : { $detail }
export-missing-delta-base = Il manque à l’enregistrement des données d’avant le début dont le clip a besoin (numéro { $sequence }).
export-missing-parent = Le dossier { $path } n’existe pas.
export-range-beyond-end = La fin { $end } dépasse la fin de l’enregistrement, { $segment_end }.
export-same-as-source = L’export remplacerait la démo d’où il vient.
export-source-changed = { $path } a changé pendant l’export.
export-source-decode-failed = La démo ne peut pas être lue : { $detail }
export-source-failed-crosscheck = À la vérification, ce que Lambda Slicer a déduit de cette démo ne correspondait pas à la démo elle-même, donc on ne peut pas se fier aux clips qui en sont tirés.
export-unknown-persistent-message = { $name } avant le début règle quelque chose qu’un clip ne sait pas encore reprendre.
export-unproven-state = À { $time }, l’enregistrement dépend de données qu’il a fallu deviner, donc le clip ne peut pas être fait exactement.
export-unrepresentable-time-field = Un temps au début ne peut pas être enregistré exactement : { $detail }
export-unsupported = Non pris en charge : { $detail }
export-validation-failed = Le clip exporté n’a pas passé sa vérification finale : { $detail }
io-not-found = Le fichier ou le dossier est introuvable.
io-other = { $detail }
io-permission-denied = Accès refusé.
io-unexpected-end = Le fichier s’est terminé de façon inattendue.
protocol-unsupported = Cette démo vient d’une version du jeu qui n’est pas prise en charge (protocole { $demo }/{ $network }). Seule la 5/48 l’est.
protocol-unsupported-network = Le serveur est passé à une version du jeu qui n’est pas prise en charge (protocole { $network }).

## Vue de la démo

demo-close = Fermer
demo-loading = Analyse de { $name }…
show-shots = Tirs
show-actions = Autres actions
show-kills = Éliminations
show-switches = Changements d’arme
lane-unknown = Joueur inconnu
lane-recorder = { $name } (enregistre)
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
timeline-window = N° { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Scores finaux, à la dernière élimination { $time }
scores-at = Scores à { $time }
scores-show-final = Voir les scores finaux
scores-frags = Frags
scores-deaths = Morts

viewer-play = Lecture
viewer-pause = Pause
viewer-volume = Volume
viewer-no-sound = Pas de son
viewer-play-backward = Rembobiner
viewer-step-back = Image précédente
viewer-step-forward = Image suivante
viewer-rate = { $rate }×
viewer-shortcut = Raccourci : { $key }
key-space = Espace
viewer-map-failed = Impossible de charger la carte { $map }
viewer-stopped = La lecture s’est arrêtée avant la fin : { $reason }
viewer-missing = Fichiers illisibles : { $count }
event-previous = Événement précédent
event-next = Événement suivant
event-play = Lire l’événement
timeline-title = Frise
timeline-lanes = Lignes par joueur
timeline-zoom-in = Zoom avant
timeline-zoom-out = Zoom arrière
timeline-fit-all = Ajuster toute la démo
timeline-gestures = Clique sur une marque pour sélectionner son événement, ou ailleurs pour t’y rendre. Fais glisser sur les temps en haut, ou fais glisser la ligne qui marque le moment actuel, pour avancer ou reculer dans la démo. Fais glisser ailleurs pour faire défiler la vue. La molette zoome et dézoome là où se trouve la souris, et un double-clic montre toute la démo. Fais glisser le bord d’un clip pour changer où il commence ou finit, ou clique sur le clip pour le modifier.
timeline-density = Les barres ombrées montrent combien des événements affichés ont lieu à chaque endroit, par rapport à l’endroit le plus chargé de la démo.
timeline-density-count = { $count ->
    [one] Un événement affiché ici
    [many] { $count } d’événements affichés ici
   *[other] { $count } événements affichés ici
}
timeline-session-count = Partie { $session } : { $count }
timeline-choose = Événements à cet endroit
legend-clip = Clip
event-session = Partie { $session }
event-unreachable = Cet événement vient après un changement de carte ou une reconnexion, où l’horloge de la démo est repartie de zéro, le lecteur ne peut donc pas s’y rendre.
event-hidden = Les filtres masquent l’événement sélectionné.
highlight-jump = Aller à un temps fort…
highlight-help = Les temps forts classent toute la démo selon son action, quoi que montrent les filtres.
kills-with = Éliminations avec
refine-midair = Midair
refine-speed = Vitesse
refine-wallgauss = Wallgauss
refine-clear = Effacer
refine-airborne = En l’air
refine-either = Au moins un
midair-both = Les deux
midair-one = Un seul
help-midair-either = Le tueur, la victime ou les deux étaient en l’air. Le tueur est vérifié au tir lié à l’élimination, ou à l’élimination s’il n’y en a pas, et la victime à sa mort. Un simple saut, à moins de 50 unités au-dessus du sol, ne compte pas. Il faut avoir choisi le dossier de Half-Life pour savoir où est le sol.
help-midair-both = Le tueur et la victime étaient tous deux en l’air, chacun au moment de sa vérification.
help-midair-one = Exactement l’un des deux était en l’air, et on sait que l’autre était au sol.
speed-at-least = Au moins
speed-unit = ups
speed-killer = Tueur
speed-victim = Victime
help-speed = Vitesse au sol en ups, comme l’affiche un compteur de vitesse, sans compter les mouvements vers le haut et vers le bas. Le tueur est mesuré au tir lié à l’élimination, ou à l’élimination s’il n’y en a pas, et la victime à sa mort. Les vitesses des autres joueurs sont des estimations.
wallgauss-weapon = Wallgauss ne concerne que les éliminations au gauss, et le filtre d’arme affiche { $weapon }.
wallgauss-checking = Vérification Wallgauss… { $examined } sur { $total } éliminations
wallgauss-needs-files = Wallgauss a besoin des fichiers du jeu pour savoir où sont les murs de la carte.
wallgauss-failed = Impossible de vérifier Wallgauss. { $reason }
help-wallgauss = Éliminations au gauss à travers un mur, trouvées en suivant chaque tir à travers la carte comme le fait le serveur. Une élimination impossible à trancher, par exemple quand la victime était hors de la vue de l’enregistrement, compte comme inconnue et est exclue.
badge-wall = WG
wall-yes-exit = Wallgauss, déduit du tir : l’explosion là où le rayon est ressorti d’un mur de { $thickness } unités d’épaisseur.
wall-yes-beyond = Wallgauss, déduit du tir : le rayon a touché après avoir traversé un mur de { $thickness } unités d’épaisseur.
wall-limits = Déduit du tir et de la position des joueurs, car le serveur ne l’enregistre pas. Un joueur fait 32 unités de large et compte comme une boîte, et la victime pouvait se trouver à une ou deux largeurs de corps de l’endroit où la démo la montre.
wall-no-direct = Pas un Wallgauss : le rayon pouvait toucher la victime avant tout mur.
wall-no-reflection = Pas un Wallgauss : l’explosion d’un rebond pouvait toucher la victime avant tout mur.
wall-doubt-no-shot = Wallgauss inconnu : le tir du tueur n’est pas dans l’enregistrement.
wall-doubt-no-map = Wallgauss inconnu : les fichiers de la carte n’ont pas pu être lus.
wall-doubt-unseen = Wallgauss inconnu : la victime était hors de la vue de l’enregistrement.
wall-doubt-brush = Wallgauss inconnu : le rayon a touché une porte, un ascenseur ou une autre partie spéciale de la carte, qui n’est pas vérifiée.
wall-doubt-ambiguous = Wallgauss inconnu : le tir a pu tuer aussi bien à travers un mur que sans.
wall-doubt-unexplained = Wallgauss inconnu : aucun chemin du tir jusqu’à la victime n’a été trouvé.
wall-doubt-unconfirmed = Wallgauss inconnu : le tir n’a pu atteindre la victime qu’à travers un mur, mais ce n’est pas sûr.
wall-doubt-not-reached = Wallgauss inconnu : l’enregistrement s’arrête avant le tir.
wall-pending = Wallgauss pas encore vérifié.
events-unknown = { $count ->
    [one] Une élimination manque de données pour trancher
    [many] { $count } d’éliminations manquent de données pour trancher
   *[other] { $count } éliminations manquent de données pour trancher
}
help-unknown = Un mouvement que l’enregistrement n’a pas pu voir compte comme inconnu, jamais comme au sol ou à l’arrêt, donc ces éliminations sont exclues.
clip-it = Clipper
badge-headshot = TT
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = suicide
badge-teamkill = teamkill
badge-world = monde
air-both = Les deux étaient en l’air lors de la vérification.
air-killer = Le tueur était en l’air lors de la vérification.
air-victim = La victime était en l’air lors de la vérification.
air-other-unknown = On ne sait pas si l’autre l’était.
speed-horizontal = Vitesse horizontale, parallèlement au sol.
motion-line = { $who } : { $air }, { $speed }. { $how }, { $when }.
motion-who-killer = Tueur
motion-who-victim = Victime
motion-air-yes = en l’air
motion-air-no = au sol
motion-air-unknown = en l’air ou non, on ne sait pas
motion-speed-unknown = vitesse inconnue
motion-exact = Enregistré par le jeu
motion-trajectory = Estimé d’après le mouvement
motion-unclear = Mouvement incertain
motion-not-visible = Hors de vue
motion-at-shot = au tir lié
motion-at-kill = à la mort
motion-none = L’analyse n’a aucun mouvement pour cette élimination.
legend-shot = Tir
legend-charge = Charge
legend-kill = Élimination
legend-death = Mort
legend-switch = Changement d’arme

column-headshots = Tirs à la tête
filter-headshots = Éliminations d’un tir à la tête
filter-midair = Éliminations en midair
event-headshot = tir à la tête
scores-headshots = TT

column-accuracy = Précision
filter-min-accuracy = Précision minimale (%)
scores-accuracy = Préc.

help-headshots = Éliminations par un tir à la tête, reconnues à l’animation de mort de la victime. Les explosions ne comptent pas. Seules celles de la personne qui enregistre sont affichées, car les autres joueurs éliminent souvent hors de sa vue. En duel, celles de l’adversaire aussi, puisque toutes visent la personne qui enregistre.
help-headshots-column = Éliminations de la personne qui enregistre par un tir à la tête, reconnues à l’animation de mort de la victime. Les explosions ne comptent pas. Celles des autres joueurs ne comptent pas, car elles ont souvent lieu hors de la vue de l’enregistrement.
help-accuracy = La part des tirs qui ont touché, d’après le sang qu’un impact laisse sur la ligne de visée, moins les correspondances dues au hasard. Elle couvre le glock, le python, les balles du MP5, le fusil à pompe, l’arbalète avec zoom et les impacts directs du rayon gauss. Les explosions et les rebonds du gauss ne laissent pas de sang et ne comptent pas. Affichée pour la personne qui enregistre seulement, après au moins 50 tirs.
help-headshot-filter = Démos avec au moins une élimination d’un tir à la tête visible dans l’enregistrement, par n’importe quel joueur.
help-midair-filter = Démos avec au moins une élimination où le tueur ou la victime était en l’air, à 50 unités ou plus au-dessus du sol. Si le dossier de Half-Life n’a pas été choisi, un simple saut compte aussi.
help-accuracy-filter = Démos où la personne qui enregistre a touché au moins cette part de ses tirs.

clips-new = Nouveau clip
clips-select-event = Choisis d’abord un événement dans la liste ou sur la frise.
clips-other-session = Un clip ne peut pas commencer ici, car l’horloge de la démo est repartie de zéro après un changement de carte ou une reconnexion, et une partie précédente a les mêmes temps.
clips-add-window = Clip de ce moment intense
clip-default-name = Clip { $number }
clip-name = Nom
clip-length = Durée { $duration }
clip-remove = Retirer
clip-loop = Boucle
clip-time-help = Secondes (96), minutes et secondes (1:36) ou heures, minutes et secondes (00:01:36.250)
clips-empty-hint = Aucun clip pour l’instant. Choisis un événement et appuie sur Clipper, ou appuie sur Nouveau clip pour en commencer un au moment actuel.
clips-new-help = Un clip de dix secondes à partir du moment actuel.
clip-in = Début
clip-out = Fin
clip-set-in = Commencer le clip au moment actuel
clip-set-out = Finir le clip au moment actuel
clip-no-playhead = Le lecteur n’est pas disponible, il n’y a donc pas de moment actuel à utiliser.
clip-play = Lire le clip une fois
clip-show-on-timeline = Montrer sur la frise
export-destination = Changer le dossier d’export
export-skipped = Ignoré. { $reason }
export-earlier-range = Cet export couvre { $start } – { $end }, le clip tel qu’il était avant que tu le modifies.
scores-player = Joueur
scores-none = L’enregistrement n’a pas de scores.
clip-effective = L’export couvre { $start } – { $end } ({ $duration }), au plus près de ce que permet l’enregistrement.
settings-clip-padding = Marge des clips
settings-preroll = Secondes avant l’événement
settings-postroll = Secondes après l’événement
export-start-first-group = Un clip ne peut pas commencer tout au début de l’enregistrement. Commence un peu plus tard.
export-start-during-loading = Le début tombe pendant le chargement d’une carte. Commence un peu plus tard.

export-all = Exporter tous les clips
export-one = Exporter
export-cancel = Arrêter l’export
export-folder = Les exports vont dans { $folder }.
export-folder-beside = Les exports vont à côté de la démo, dans { $folder }.
export-waiting = En attente d’export…
export-running = Export en cours… La démo est lue en entier, cela peut prendre un moment.
export-cancelled = Non exporté, car l’export a été arrêté.
export-saved = Enregistré : { $path }. Moments vérifiés avec l’original : { $count }.
export-playdemo = Dans la console du jeu :
export-copy-first = Copie { $name } dans le dossier de ton mod, celui qui contient liblist.gam, puis dans la console du jeu :
export-copy = Copier
settings-export-folder = Dossier d’export
settings-export-choose = Choisir…
settings-viewer = Visionneuse de démos
settings-viewer-game = Dossier de Half-Life
settings-viewer-game-none = Non choisi
settings-viewer-game-hint = Le dossier qui contient valve. La visionneuse lit les démos avec les cartes, modèles et sons du jeu qu'il contient.
settings-executable = Programme du jeu
settings-executable-hint = Sert à trouver le dossier Half-Life. Le lecteur lit lui-même les fichiers du jeu et ne lance jamais le jeu.
settings-executable-not-set = Non défini, donc le lecteur n’a ni image ni son
settings-executable-optional = Non défini, et inutile tant que le dossier Half-Life contient le jeu
settings-executable-missing = { $path } n’est plus là. Le lecteur lit toujours le dossier Half-Life.
settings-game-override = Cette fois, Lambda Slicer utilise { $path }, car il a été lancé avec --game. Un choix fait ici s’applique au prochain lancement.
settings-browse = Parcourir…
dialog-executables = Programmes
setup-title = Configurer le lecteur de démos
setup-body = L’image et le son ont besoin des fichiers de Half-Life. Montre à Lambda Slicer où le jeu est installé et il les lira de là.
setup-locate-exe = Trouver hl.exe…
setup-locate-folder = Trouver Half-Life…
setup-settings = Paramètres
setup-still-works = La chronologie, les événements et les clips marchent sans eux.
setup-relocate = { $folder } ne contient plus Half-Life. Retrouve-le.
setup-not-executable = { $name } n’est pas { $executable }. Choisis le programme du jeu lui-même, pas un raccourci.
setup-not-installed = { $folder } ne contient aucune installation de Half-Life avec un dossier valve.
tab-events = Événements
tab-clips = Clips
tab-scoreboard = Tableau des scores
list-hide = Masquer la liste des démos
list-show = Afficher la liste des démos
demo-details = Détails
viewer-start = Revenir au début
viewer-mute = Couper le son
viewer-unmute = Remettre le son
viewer-picture = Image et réglages du lecteur
viewer-needs-files = Nécessite les fichiers du jeu
viewer-no-sound-why = Pas de son : { $reason }
settings-tab-general = Général
settings-tab-library = Bibliothèque
settings-tab-clips = Clips
settings-section-game = Half-Life
settings-section-view = Vue
settings-language-system = Langue de l’ordinateur ({ $name })
settings-section-folders = Dossiers
settings-folders-hint = Les dossiers où la bibliothèque cherche des démos, sous-dossiers compris.
settings-clip-padding-hint = Ce qu'un clip créé autour d'un événement garde avant et après lui.
settings-section-picture = Image
settings-section-sound = Son
settings-gamma = Gamma
settings-brightness = Luminosité
settings-texgamma = Gamma des textures
settings-lightgamma = Gamma de la lumière
settings-overbright = Éclairage surexposé
settings-gamma-hint = Les réglages du jeu du même nom (gamma, brightness, texgamma, lightgamma, gl_overbright). Règle-les comme dans le jeu pour voir la même luminosité.
settings-gamma-defaults = Valeurs du jeu
settings-fov-recorded = Champ de vision enregistré
settings-fov = Champ de vision
settings-viewmodel = Afficher l’arme en main
settings-see-through = Voir les joueurs à travers les murs
settings-see-through-items = Aussi les autres modèles, comme les projectiles et les objets
settings-see-through-hint = Les joueurs derrière les murs apparaissent en silhouettes de leur couleur du haut, que les jeux en équipe fixent par équipe. Seul ce que contient la démo peut apparaître. Le serveur envoie les joueurs proches de celui qui enregistre, et les serveurs qui cachent les joueurs qu’il ne peut pas voir ne laissent rien à montrer.
viewer-see-through = Voir à travers les murs
settings-export-beside = À côté de chaque démo
settings-export-hint = Dans le dossier de ton mod, celui qui contient liblist.gam comme Half-Life\ag, les clips se lancent avec une seule commande playdemo.

clips-store-moved = Le fichier de clips { $path } n’a pas pu être lu, il a donc été conservé sous { $moved } et les clips repartent de zéro. { $detail }
clips-store-unreadable = Le fichier de clips { $path } n’a pas pu être lu : { $detail }. Jusqu’à la fermeture de Lambda Slicer, les clips ne sont pas enregistrés, donc rien de ce que contient le fichier n’est perdu.
clips-save-failed = Les clips n’ont pas pu être enregistrés dans { $path } : { $detail }
notice-dismiss = Fermer

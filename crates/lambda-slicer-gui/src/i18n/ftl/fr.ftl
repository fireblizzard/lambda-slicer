# Français. Brouillon non relu

app-title = Lambda Slicer

toolbar-open = Ouvrir une démo…
toolbar-add-folder = Ajouter un dossier…
toolbar-rescan = Analyser à nouveau
toolbar-cancel = Annuler
toolbar-settings = Paramètres

settings-title = Paramètres
settings-language = Langue
settings-unreviewed = Cette traduction est un brouillon qu’aucun locuteur natif n’a encore relu.
settings-workers = Processus d’analyse
settings-workers-hint = Davantage de processus analysent plus vite, mais sollicitent davantage le processeur.
settings-cache = Cache de la bibliothèque
settings-folders = Dossiers de la bibliothèque
settings-folders-empty = Aucun dossier ajouté pour l’instant.
settings-remove = Retirer
settings-close = Fermer

library-empty = Ajoutez un dossier ou ouvrez une démo pour commencer.
library-demos = { $count ->
    [one] { $count } démo
    [many] { $count } de démos
   *[other] { $count } démos
}
library-kills = { $count ->
    [one] { $count } élimination
    [many] { $count } d’éliminations
   *[other] { $count } éliminations
}
library-action = Score d’action { NUMBER($score, minimumFractionDigits: 2) }
library-spectating = Spectateur { $percent } %

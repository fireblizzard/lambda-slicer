# Suomi. Tarkistamaton luonnos

app-title = Lambda Slicer

toolbar-open = Avaa demo…
toolbar-add-folder = Lisää kansio…
toolbar-rescan = Skannaa uudelleen
toolbar-cancel = Peruuta
toolbar-settings = Asetukset

settings-title = Asetukset
settings-language = Kieli
settings-unreviewed = Tämä käännös on luonnos, jota kukaan äidinkielinen puhuja ei ole vielä tarkistanut.
settings-workers = Analyysisäikeet
settings-workers-hint = Useampi säie analysoi nopeammin mutta kuormittaa suoritinta enemmän.
settings-cache = Kirjaston välimuisti
settings-folders = Kirjaston kansiot
settings-folders-empty = Kansioita ei ole vielä lisätty.
settings-remove = Poista
settings-close = Sulje

library-empty = Aloita lisäämällä kansio tai avaamalla demo.
library-demos = { $count ->
    [one] { $count } demo
   *[other] { $count } demoa
}
library-kills = { $count ->
    [one] { $count } tappo
   *[other] { $count } tappoa
}
library-action = Toimintapisteet { NUMBER($score, minimumFractionDigits: 2) }
library-spectating = Katsojana { $percent } %

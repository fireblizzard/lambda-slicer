# Lietuvių. Neperžiūrėtas juodraštis

app-title = Lambda Slicer

toolbar-open = Atidaryti įrašą…
toolbar-add-folder = Pridėti aplanką…
toolbar-rescan = Nuskaityti iš naujo
toolbar-cancel = Atšaukti
toolbar-settings = Nustatymai

settings-title = Nustatymai
settings-language = Kalba
settings-unreviewed = Šis vertimas yra juodraštis, kurio dar neperžiūrėjo gimtakalbis.
settings-workers = Analizės gijos
settings-workers-hint = Daugiau gijų analizuoja greičiau, bet labiau apkrauna procesorių.
settings-cache = Bibliotekos podėlis
settings-folders = Bibliotekos aplankai
settings-folders-empty = Aplankų dar nepridėta.
settings-remove = Pašalinti
settings-close = Uždaryti

library-empty = Norėdami pradėti, pridėkite aplanką arba atidarykite įrašą.
library-demos = { $count ->
    [one] { $count } įrašas
    [few] { $count } įrašai
    [many] { $count } įrašo
   *[other] { $count } įrašų
}
library-kills = { $count ->
    [one] { $count } nužudymas
    [few] { $count } nužudymai
    [many] { $count } nužudymo
   *[other] { $count } nužudymų
}
library-action = Veiksmo balas { NUMBER($score, minimumFractionDigits: 2) }
library-spectating = Stebima { $percent } %

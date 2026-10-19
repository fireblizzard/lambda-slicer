# Lietuvių. Neperžiūrėtas juodraštis

app-title = Lambda Slicer

## Įrankių juosta

toolbar-open = Atidaryti įrašą…
toolbar-add-folder = Pridėti aplanką…
toolbar-rescan = Nuskaityti iš naujo
toolbar-cancel = Atšaukti
toolbar-settings = Nustatymai
dialog-demos = Half-Life įrašai

## Nustatymai

settings-title = Nustatymai
settings-language = Kalba
settings-unreviewed = Šis vertimas yra juodraštis, kurio dar neperžiūrėjo gimtakalbis.
settings-workers = Analizės gijos
settings-workers-hint = Daugiau gijų analizuoja greičiau, bet labiau apkrauna procesorių.
settings-cache = Bibliotekos podėlis
settings-folders = Bibliotekos aplankai
settings-folders-empty = Aplankų dar nepridėta.
settings-remove = Pašalinti

## Biblioteka

library-empty = Norėdami pradėti, pridėkite aplanką arba atidarykite įrašą.
library-no-matches = Šių filtrų neatitinka joks įrašas.
library-no-cache = Šioje sistemoje nėra kur laikyti bibliotekos podėlio.
library-demos = { $count ->
    [one] { $count } įrašas
    [few] { $count } įrašai
    [many] { $count } įrašo
   *[other] { $count } įrašų
}
library-percent = { $percent } %

filter-search = Ieškoti keliuose
filter-map = Žemėlapis
filter-player = Žaidėjas
filter-weapon-any = Bet kuris ginklas
filter-min-minutes = Bent (minučių)
filter-spectating-any = Žaidžiant ar stebint
filter-spectating-mostly = Daugiausia stebint
filter-spectating-playing = Daugiausia žaidžiant
filter-clear = Išvalyti filtrus

column-status = Būsena
column-name = Įrašas
column-map = Žemėlapis
column-duration = Trukmė
column-gauss = Gausas
column-kills = Nužudymai
column-action = Intensyvumas
column-spectating = Stebėjimas

status-complete = Išanalizuotas
status-partial = Iš dalies
status-failed = Nepavyko
status-unsupported = Nepalaikomas
status-no-use = Ginklai nenaudoti

kind-multiplayer-view = Žaidėjo vaizdas
kind-singleplayer = Vieno žaidėjo
kind-hltv = Stebėtojo tarpinis serveris
kind-unknown = Nežinoma rūšis

details-recorder = Įrašė { $name }
details-shots = { $count ->
    [one] { $count } šūvis
    [few] { $count } šūviai
    [many] { $count } šūvio
   *[other] { $count } šūvių
}
details-kills = { $count ->
    [one] { $count } nužudymas
    [few] { $count } nužudymai
    [many] { $count } nužudymo
   *[other] { $count } nužudymų
}
details-action = Didžiausias intensyvumas { NUMBER($score, minimumFractionDigits: 2) }

## Nuskaitymas

scan-progress = Rasta { $found }, baigta { $done } (iš podėlio { $cached }, išanalizuota { $analysed }), perskaityta { $megabytes } MB
scan-cancelling = Stabdoma…
scan-cancelled = Sustabdyta, liko nuskaityti { $left }.
scan-finished = Baigta: įrašų { $done }, išanalizuota { $analysed }, per { $seconds } s.
scan-problems = { $count ->
    [one] { $count } problema
    [few] { $count } problemos
    [many] { $count } problemos
   *[other] { $count } problemų
}
problems-title = Nuskaitymo problemos
problem-changed = Failas pasikeitė analizės metu, todėl niekas neišsaugota.
drop-hint = Numeskite įrašus arba aplankus, kad juos pridėtumėte
drop-not-demo = Ne įrašas: { $names }

## Analizės branduolio pranešimai pagal kodą

analysis-panicked = Analizė sustojo dėl vidinės klaidos: { $detail }
cache-cancelled = Analizė atšaukta, todėl niekas neišsaugota.
cache-database = Negalima naudoti bibliotekos podėlio: { $detail }
cache-directory = Negalima sukurti podėlio aplanko: { $detail }
container-bad-magic = Tai ne Half-Life įrašas.
container-frame-bad-time = Įrašas sugadintas { $entry } dalies { $frame } kadre, kurio laikas netinkamas.
container-frame-missing-terminal = Įrašo { $entry } dalis baigiasi be uždaromojo įrašo.
container-frame-negative-length = Įrašas sugadintas { $entry } dalies { $frame } kadre.
container-frame-over-limit = { $entry } dalies { $frame } kadras didesnis nei leidžiama.
container-frame-truncated = Įrašas nutrūksta { $entry } dalies { $frame } kadre.
container-frame-unknown-tag = Įrašas sugadintas { $entry } dalies { $frame } kadre, kurio tipas nežinomas.
container-io = Failo nepavyksta perskaityti: { $detail }
container-no-directory = Įrašas niekada nebuvo užbaigtas, todėl neturi turinio.
container-range = Įrašo turinys rodo už failo ribų.
container-too-short = Failas per trumpas, kad būtų įrašas.
coverage-hltv = Stebėtojo tarpinio serverio įrašas. Įrašančio žaidėjo nėra, o įvykiai seka tarpinio serverio vaizdą.
coverage-mostly-spectating = Įrašantysis stebėjo { $percent } % atkūrimo, todėl dauguma įvykių yra kitų žaidėjų, tokie, kokius rodė stebimas vaizdas.
coverage-no-local-events = Nėra vietinių įvykių. Įrašančiojo veiksmų trūksta, išskyrus tuos, kuriuos atsiuntė serveris.
coverage-singleplayer = Vieno žaidėjo. Kiti veikėjai yra monstrai ar žemėlapio objektai, o nužudymų sąrašo nėra.
coverage-unknown = Įrašo nepavyko iškoduoti tiek, kad būtų aišku, kokios jis rūšies.
coverage-visible-only = Kitų žaidėjų įvykiai pasiekia šį klientą tik tol, kol serveris juos laiko matomais, todėl jų skaičiai yra apatinės ribos.
decode-failed = Iškodavimas sustojo ties { $time } (kadras { $frame }): { $detail }
export-active-effect = Pradžioje vyksta { $effect }, ir jo dar negalima perkelti.
export-client-dropped-packets = Žaidimas ties { $time } atmetė pavėlavusius paketus, ko iškarpa dar negali atkurti.
export-cross-segment = Ties { $time }, intervalo viduje, prasideda nauja sesija.
export-destination-exists = { $path } jau yra. Pasirinkite kitą pavadinimą.
export-empty-range = Pasirinktame intervale nėra ką eksportuoti.
export-io = Nepavyksta įrašyti { $path }: { $detail }
export-missing-delta-base = Įraše trūksta būsenos, reikalingos prieš pradžią (seka { $sequence }).
export-missing-parent = Aplanko { $path } nėra.
export-range-beyond-end = Pabaiga { $end } yra po įrašo pabaigos, { $segment_end }.
export-same-as-source = Paskirtis yra pats šaltinio įrašas.
export-source-changed = { $path } pasikeitė eksporto metu.
export-source-decode-failed = Šaltinio įrašo nepavyksta iškoduoti: { $detail }
export-source-failed-crosscheck = Šaltinio įrašas neišlaikė kryžminės patikros, todėl atkurta jo būsena nepatikima.
export-unknown-persistent-message = { $name } prieš pradžią nustato būseną, kurios iškarpa dar negali perkelti.
export-unproven-state = Būsena ties { $time } remiasi numanoma baze ir nėra įrodyta.
export-unrepresentable-time-field = Laiko pradžioje negalima tiksliai įrašyti: { $detail }
export-unsupported = Nepalaikoma: { $detail }
export-validation-failed = Eksportuotas failas neišlaikė patikros: { $detail }
io-not-found = Failas ar aplankas nerastas.
io-other = { $detail }
io-permission-denied = Prieiga uždrausta.
io-unexpected-end = Failas netikėtai baigėsi.
protocol-unsupported = Įrašo protokolas { $demo }/{ $network } nepalaikomas, tik 5/48.
protocol-unsupported-network = Serveris perėjo prie nepalaikomo tinklo protokolo { $network }.

## Įrašo peržiūra

demo-close = Uždaryti
demo-loading = Analizuojamas { $name }…
timeline-fit = Talpinti
show-shots = Šūviai
show-actions = Kiti veiksmai
show-kills = Nužudymai
show-switches = Ginklų keitimai
lane-unknown = Nežinomas žaidėjas
lane-recorder = { $name } (įrašė)
column-time = Laikas
column-player = Žaidėjas
column-weapon = Ginklas
column-event = Įvykis
event-kill = Nužudė { $victim }
event-switch = Ginklo keitimas
action-swing = Smūgis
action-fire = Šūvis
action-launch = Paleidimas
action-charge-activity = Krovimas
action-charge-stop = Krovimo pabaiga
action-beam-start = Spindulio pradžia
action-beam-sustain = Spindulys
action-beam-stop = Spindulio pabaiga
action-placement-attempt = Padėjimo bandymas
action-release-attempt = Paleidimo bandymas
mode-primary = pagrindinis
mode-charged = įkrautas
mode-zoomed = su taikikliu
mode-unzoomed = be taikiklio
mode-grenade = granata
mode-bullets = kulkos
mode-single = pavienis
mode-double = dvigubas
mode-rapid = greitas
demo-events = { $count ->
    [one] { $count } įvykis
    [few] { $count } įvykiai
    [many] { $count } įvykio
   *[other] { $count } įvykių
}

column-date = Data
column-score = Rezultatas
filter-from = Nuo
filter-to = Iki
date-from-name = Data iš failo pavadinimo
date-from-modified = Paskutinio failo pakeitimo data, kurią kopijavimas ar atsisiuntimas gali pakeisti
timeline-best-action = Auksinė minutė
timeline-windows = Intensyviausios akimirkos
timeline-window = Nr. { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Galutiniai rezultatai, ties paskutiniu nužudymu { $time }
scores-at = Rezultatai ties { $time }
scores-show-final = Rodyti galutinius rezultatus
scores-frags = Fragai
scores-deaths = Mirtys

show-timeline = Rodyti laiko juostą
viewer-play = Leisti
viewer-pause = Pristabdyti
viewer-volume = Garsumas
viewer-no-sound = Nėra garso
viewer-play-backward = Atsukti
viewer-step-back = Ankstesnis kadras
viewer-step-forward = Kitas kadras
viewer-rate = { $rate }×
viewer-shortcut = Spartusis klavišas: { $key }
key-space = Tarpas
viewer-map-failed = Nepavyko įkelti žemėlapio { $map }
viewer-stopped = Atkūrimas sustojo anksčiau laiko: { $reason }
viewer-missing = Failai, kurių nepavyko perskaityti: { $count }
event-previous = Ankstesnis įvykis
event-next = Kitas įvykis
event-play = Leisti įvykį
legend-shot = Šūvis
legend-charge = Krovimas
legend-kill = Nužudymas
legend-death = Mirtis
legend-switch = Ginklo keitimas
timeline-help = Kiekviena eilutė yra žaidėjas. Slinkite, kad priartintumėte, tempkite, kad pastumtumėte, dukart spustelėkite, kad matytumėte viską, spustelėkite žymę, kad ją pasirinktumėte.

column-headshots = Šūviai į galvą
filter-headshots = Nužudymai šūviu į galvą
event-headshot = į galvą
scores-headshots = ŠG

column-accuracy = Taiklumas
filter-min-accuracy = Taiklumas bent (%)
scores-accuracy = Taikl.

help-headshots = Nužudymai pataikius į galvą, atpažinti iš aukos mirties animacijos. Sprogimai neskaičiuojami. Rodomi tik įrašančiojo nužudymai, nes kiti žaidėjai dažnai žudo už įrašo matomumo ribų. Dvikovoje rodomi ir priešininko, nes visi jo nužudymai yra įrašančiojo.
help-headshots-column = Įrašančiojo nužudymai pataikius į galvą, atpažinti iš aukos mirties animacijos. Sprogimai neskaičiuojami. Kitų žaidėjų nužudymai neskaičiuojami, nes dažnai įvyksta už įrašo matomumo ribų.
help-accuracy = Pataikiusių šūvių dalis, nustatyta pagal kraują, kurį pataikymas palieka taikymo linijoje, atėmus atsitiktinius sutapimus. Įtraukiami glock, python, MP5 kulkos, šautuvas, arbaletas su taikikliu ir tiesioginiai gauso spindulio pataikymai. Sprogimai ir gauso atšokimai kraujo nepalieka ir neskaičiuojami. Rodoma tik įrašančiajam ir tik po bent 50 šūvių.
help-headshot-filter = Įrašai, kuriuose yra bent vienas įraše matomas nužudymas į galvą, bet kurio žaidėjo.
help-accuracy-filter = Įrašai, kuriuose įrašantysis pataikė bent tokią šūvių dalį.

clips-title = Klipai
clips-new = Naujas klipas
clips-add-event = Klipas { $start } – { $end }
clips-add-event-none = Pasirinkto įvykio klipas
clips-select-event = Pirmiausia pasirinkite įvykį sąraše arba laiko juostoje.
clips-other-session = Klipas negali prasidėti čia, nes ankstesnė įrašo sesija turi tuos pačius laikus.
clips-add-window = Šios intensyvios akimirkos klipas
clips-empty = Klipų dar nėra. Pasirinkite įvykį arba vieną intensyviausių akimirkų ir pridėkite aplink jį klipą. Kiek klipas apima prieš įvykį ir po jo, nustatoma Nustatymuose. Naujas klipas prideda klipą, kurio pradžią ir pabaigą įvedate patys.
clip-default-name = Klipas { $number }
clip-name = Pavadinimas
clip-length = Trukmė { $duration }
clip-show = Rodyti
clip-remove = Pašalinti
clip-loop = Kartoti
clip-time-help = Sekundės (96), minutės ir sekundės (1:36) arba valandos, minutės ir sekundės (00:01:36.250)
clip-exact = Eksportas prasidės ir baigsis lygiai čia.
clip-effective = Eksportas iškirps { $start } – { $end } ({ $duration }) pagal įrašo kadrus.
timeline-clip-help = Klipai yra šviesiai mėlynos juostos. Tempkite juostos kraštą, kad jį pastumtumėte.
settings-clip-padding = Klipo paraštė
settings-preroll = Sekundės prieš įvykį
settings-postroll = Sekundės po įvykio
export-start-first-group = Iškarpa negali prasidėti nuo pačio pirmojo įrašo kadro. Pradėkite šiek tiek vėliau.
export-start-during-loading = Pradžia patenka į žemėlapio įkėlimą. Pradėkite šiek tiek vėliau.

export-all = Eksportuoti visus klipus
export-one = Eksportuoti
export-cancel = Stabdyti eksportą
export-folder = Eksportuoti failai keliauja į { $folder }.
export-folder-beside = Eksportuoti failai keliauja šalia įrašo, į { $folder }.
export-waiting = Laukia eksporto…
export-running = Eksportuojama… Perskaitomas visas įrašas, tai gali užtrukti.
export-cancelled = Neeksportuota, nes eksportas sustabdytas.
export-saved = Išsaugota: { $path }. Su originalu patikrintų momentinių kopijų: { $count }.
export-playdemo = Žaidimo konsolėje:
export-copy-first = Nukopijuokite { $name } į modo aplanką, kuriame yra liblist.gam, tada žaidimo konsolėje:
export-copy = Kopijuoti
settings-export-folder = Eksporto aplankas
settings-export-choose = Pasirinkti…
settings-viewer = Įrašų peržiūra
settings-viewer-game = Half-Life aplankas
settings-viewer-game-none = Nepasirinktas. Pasirinkite aplanką su valve, kad žiūrėtumėte įrašus peržiūroje.
settings-gamma = Gama
settings-brightness = Ryškumas
settings-texgamma = Tekstūrų gama
settings-lightgamma = Šviesos gama
settings-overbright = Perryškintas apšvietimas
settings-gamma-hint = Tokiais pat pavadinimais žaidimo nustatymai (gamma, brightness, texgamma, lightgamma, gl_overbright). Nustatykite juos kaip žaidime, kad matytumėte tą patį ryškumą.
settings-gamma-defaults = Žaidimo numatytosios
settings-fov-recorded = Matymo laukas kaip įrašyta
settings-fov = Matymo laukas
settings-viewmodel = Rodyti ginklą rankoje
settings-see-through = Matyti žaidėjus pro sienas
settings-see-through-items = Taip pat kitus modelius, pvz., sviedinius ir daiktus
settings-see-through-hint = Žaidėjai už sienų rodomi kaip siluetai jų viršutine spalva, kurią komandiniai žaidimai nustato pagal komandą. Gali būti rodoma tik tai, ką turi įrašas. Serveris siunčia įrašančiajam artimus žaidėjus, o serveriai, slepiantys žaidėjus, kurių įrašantysis nemato, nepalieka ko rodyti.
viewer-see-through = Matyti pro sienas
settings-export-beside = Šalia kiekvieno įrašo
settings-export-hint = Modo aplanke, kuriame yra liblist.gam, pvz., Half-Life\ag, klipai paleidžiami viena playdemo komanda.

clips-store-moved = Klipų failo { $path } nepavyko perskaityti, todėl jis išsaugotas kaip { $moved }, o klipų sąrašas pradedamas iš naujo. { $detail }
clips-store-unreadable = Klipų failo { $path } nepavyko perskaityti: { $detail }. Šioje sesijoje klipai neišsaugomi, todėl niekas iš jo nedings.
clips-save-failed = Klipų nepavyko išsaugoti į { $path }: { $detail }
notice-dismiss = Uždaryti

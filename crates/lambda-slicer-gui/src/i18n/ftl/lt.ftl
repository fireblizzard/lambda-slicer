# Lietuvių. Neperžiūrėtas juodraštis

app-title = Lambda Slicer

## Įrankių juosta

toolbar-open = Atidaryti įrašą…
toolbar-add-folder = Pridėti aplanką…
toolbar-rescan = Nuskaityti iš naujo
toolbar-cancel = Atšaukti
toolbar-settings = Nustatymai
toolbar-rescan-busy = Nuskaitymas jau vyksta.
toolbar-rescan-nothing = Pridėkite aplanką arba atidarykite įrašą, kad būtų ką nuskaityti.
toolbar-cancel-idle = Niekas nenuskaitoma.
dialog-demos = Half-Life įrašai

## Nustatymai

settings-title = Nustatymai
settings-language = Kalba
settings-unreviewed = Šis vertimas yra juodraštis, kurio dar neperžiūrėjo gimtakalbis.
settings-workers = Kiek įrašų analizuoti vienu metu
settings-workers-hint = Analizuojant daugiau vienu metu, baigiama greičiau, bet kompiuteris labiau apkraunamas.
settings-cache = Kur laikoma biblioteka
settings-folders = Bibliotekos aplankai
settings-folders-empty = Aplankų dar nepridėta.
settings-remove = Pašalinti

## Biblioteka

library-empty = Norėdami pradėti, pridėkite aplanką arba atidarykite įrašą.
library-no-matches = Šių filtrų neatitinka joks įrašas.
library-no-cache = Šiame kompiuteryje nėra kur laikyti bibliotekos.
library-demos = { $count ->
    [one] { $count } įrašas
    [few] { $count } įrašai
    [many] { $count } įrašo
   *[other] { $count } įrašų
}
library-percent = { $percent } %
library-demos-of = { $shown } iš { $total ->
    [one] { $total } įrašo
    [few] { $total } įrašų
    [many] { $total } įrašo
   *[other] { $total } įrašų
}
library-total = { $count ->
    [one] Bibliotekoje { $count } įrašas
    [few] Bibliotekoje { $count } įrašai
    [many] Bibliotekoje { $count } įrašo
   *[other] Bibliotekoje { $count } įrašų
}
filters-title = Filtrai
filters-find = Paieška
filters-pov = Požiūris
filters-quality = Kokybė
filters-recorded = Įrašyta
pov-any = Visi
pov-playing = Žaidžiant
pov-spectating = Stebint
help-pov = „Žaidžiant“ reiškia, kad įrašas stebėjo ne daugiau kaip pusę atkūrimo, o „Stebint“ – daugiau nei pusę.
columns-button = Stulpeliai
list-back = Atgal į bibliotekos lentelę
sort-by = Rikiuoti pagal
sort-ascending = Didėjančiai
sort-descending = Mažėjančiai
score-more-sides = { $count ->
    [one] dar { $count } pusė
    [few] dar { $count } pusės
    [many] dar { $count } pusės
   *[other] dar { $count } pusių
}
intensity-help = Intensyvumas { $score }. Juosta jį lygina su didžiausiu bibliotekoje, { $max }.
value-unknown = Nežinoma
path-copy = Kopijuoti failo vietą
path-show = Rodyti aplanke
path-show-failed = Nepavyko atidaryti aplanko ({ $detail }). Nukopijuokite failo vietą.

filter-search = Ieškoti failų pavadinimuose ir aplankuose
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
kind-hltv = HLTV
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

scan-progress = Rasta { $found }, baigta { $done } (jau žinomų { $cached }, išanalizuota { $analysed }), perskaityta { $megabytes } MB
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

analysis-panicked = Analizė sustojo dėl Lambda Slicer gedimo: { $detail }
cache-cancelled = Analizė atšaukta, todėl niekas neišsaugota.
cache-database = Negalima naudoti išsaugotos bibliotekos: { $detail }
cache-directory = Negalima sukurti išsaugotos bibliotekos aplanko: { $detail }
container-bad-magic = Tai ne Half-Life įrašas.
container-frame-bad-time = Įrašas sugadintas { $entry } dalies { $frame } kadre, kurio laikas sugadintas.
container-frame-missing-terminal = Įrašo { $entry } dalis tinkamai nesibaigia.
container-frame-negative-length = Įrašas sugadintas { $entry } dalies { $frame } kadre.
container-frame-over-limit = { $entry } dalies { $frame } kadras didesnis nei leidžiama.
container-frame-truncated = Įrašas nutrūksta { $entry } dalies { $frame } kadre.
container-frame-unknown-tag = Įrašas sugadintas { $entry } dalies { $frame } kadre, kuriame yra kažkas, ko Lambda Slicer neatpažįsta.
container-io = Failo nepavyksta perskaityti: { $detail }
container-no-directory = Įrašas niekada nebuvo užbaigtas, todėl neturi turinio.
container-range = Įrašo turinys rodo už failo ribų.
container-too-short = Failas per trumpas, kad būtų įrašas.
coverage-hltv = HLTV įrašas. Jo neįrašė joks žaidėjas, todėl įvykiai seka tai, ką stebėjo HLTV.
coverage-mostly-spectating = Įrašantysis stebėjo { $percent } % atkūrimo, todėl dauguma įvykių yra kitų žaidėjų, tokie, kokie matėsi juos stebint.
coverage-no-local-events = Paties įrašančiojo veiksmai šiame įraše neišsaugoti, todėl rodomi tik tie, kuriuos atsiuntė serveris.
coverage-singleplayer = Vieno žaidėjo. Visa kita yra monstrai ar žemėlapio dalys, o kill feed nėra.
coverage-unknown = Įrašo nepavyko perskaityti tiek, kad būtų aišku, kokios jis rūšies.
coverage-visible-only = Žaidimas gauna kitų žaidėjų veiksmus tik tada, kai serveris mano, kad įrašantysis juos mato, todėl jų skaičiai gali būti mažesni, nei buvo iš tikrųjų.
decode-failed = Įrašo skaitymas sustojo ties { $time } (kadras { $frame }): { $detail }
export-active-effect = Pradžioje dar vyksta { $effect }, o klipas to dar negali perkelti.
export-client-dropped-packets = Ties { $time } žaidimas išmetė per vėlai atėjusius duomenis, o klipas to dar negali nukopijuoti.
export-cross-segment = Ties { $time }, klipo viduje, pasikeitė žemėlapis arba žaidėjas prisijungė iš naujo, o klipas negali tęstis per tokią vietą.
export-destination-exists = { $path } jau yra. Pasirinkite kitą pavadinimą.
export-empty-range = Klipe nėra ką eksportuoti.
export-io = Nepavyksta įrašyti { $path }: { $detail }
export-missing-delta-base = Įraše trūksta duomenų iš laiko prieš pradžią, kurių reikia klipui (numeris { $sequence }).
export-missing-parent = Aplanko { $path } nėra.
export-range-beyond-end = Pabaiga { $end } yra po įrašo pabaigos, { $segment_end }.
export-same-as-source = Eksportas pakeistų įrašą, iš kurio jis daromas.
export-source-changed = { $path } pasikeitė eksporto metu.
export-source-decode-failed = Įrašo nepavyksta perskaityti: { $detail }
export-source-failed-crosscheck = Patikrinus paaiškėjo, kad tai, ką Lambda Slicer nustatė iš šio įrašo, nesutampa su pačiu įrašu, todėl iš jo padarytais klipais negalima pasitikėti.
export-unknown-persistent-message = { $name } prieš pradžią nustato kažką, ko klipas dar negali perkelti.
export-unproven-state = Ties { $time } įrašas remiasi duomenimis, kuriuos teko spėti, todėl klipo negalima padaryti tiksliai.
export-unrepresentable-time-field = Laiko pradžioje negalima tiksliai išsaugoti: { $detail }
export-unsupported = Nepalaikoma: { $detail }
export-validation-failed = Eksportuotas klipas neišlaikė galutinės patikros: { $detail }
io-not-found = Failas ar aplankas nerastas.
io-other = { $detail }
io-permission-denied = Prieiga uždrausta.
io-unexpected-end = Failas netikėtai baigėsi.
protocol-unsupported = Šis įrašas yra iš nepalaikomos žaidimo versijos (protokolas { $demo }/{ $network }). Palaikoma tik 5/48.
protocol-unsupported-network = Serveris perėjo prie nepalaikomos žaidimo versijos (protokolas { $network }).

## Įrašo peržiūra

demo-close = Uždaryti
demo-loading = Analizuojamas { $name }…
show-shots = Šūviai
show-actions = Kiti veiksmai
show-kills = Nužudymai
show-switches = Ginklų keitimai
lane-unknown = Nežinomas žaidėjas
lane-recorder = { $name } (įrašė)
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
timeline-window = Nr. { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Galutiniai rezultatai, ties paskutiniu nužudymu { $time }
scores-at = Rezultatai ties { $time }
scores-show-final = Rodyti galutinius rezultatus
scores-frags = Fragai
scores-deaths = Mirtys

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
timeline-title = Laiko juosta
timeline-lanes = Žaidėjų eilutės
timeline-zoom-in = Priartinti
timeline-zoom-out = Nutolinti
timeline-fit-all = Talpinti visą įrašą
timeline-gestures = Spustelėkite žymę, kad pasirinktumėte jos įvykį, arba bet kur kitur, kad ten pereitumėte. Tempkite išilgai laikų viršuje arba tempkite dabartinę akimirką žyminčią liniją, kad judėtumėte per įrašą. Tempkite bet kur kitur, kad pastumtumėte vaizdą. Slinkite, kad priartintumėte ar nutolintumėte ten, kur yra pelė, ir dukart spustelėkite, kad matytumėte visą įrašą. Tempkite klipo kraštą, kad pakeistumėte, kur jis prasideda ar baigiasi, arba spustelėkite klipą, kad jį redaguotumėte.
timeline-density = Užtamsintos juostelės rodo, kiek rodomų įvykių vyksta kiekvienoje vietoje, palyginti su judriausia įrašo vieta.
timeline-density-count = { $count ->
    [one] Čia { $count } rodomas įvykis
    [few] Čia { $count } rodomi įvykiai
    [many] Čia { $count } rodomo įvykio
   *[other] Čia { $count } rodomų įvykių
}
timeline-session-count = Dalis { $session }: { $count }
timeline-choose = Įvykiai šioje vietoje
legend-clip = Klipas
event-session = Dalis { $session }
event-unreachable = Šis įvykis yra po žemėlapio pakeitimo ar prisijungimo iš naujo, kai įrašo laikrodis prasidėjo iš naujo, todėl peržiūra negali į jį pereiti.
event-hidden = Pasirinktą įvykį slepia filtrai.
highlight-jump = Pereiti prie geriausios akimirkos…
highlight-help = Geriausios akimirkos rikiuoja visą įrašą pagal veiksmą, nepaisant to, ką rodo filtrai.
kills-with = Nužudymai su
refine-midair = Midair
refine-speed = Greitis
refine-wallgauss = Wallgauss
refine-clear = Išvalyti
refine-airborne = Ore
refine-either = Bet kuris
midair-both = Abu
midair-one = Tik vienas
help-midair-either = Žudikas, auka arba abu buvo ore. Žudikas tikrinamas su nužudymu susieto šūvio metu arba, jei šūvio nėra, nužudymo metu, o auka mirties metu.
help-midair-both = Žudikas ir auka abu buvo ore, kiekvienas savo tikrinimo metu.
help-midair-one = Ore buvo lygiai vienas iš jų, o apie kitą žinoma, kad jis buvo ant žemės.
speed-at-least = Bent
speed-unit = ups
speed-killer = Žudikas
speed-victim = Auka
help-speed = Greitis išilgai žemės ups vienetais, kaip rodo spidometras, neįskaitant judėjimo aukštyn ir žemyn. Žudiko greitis matuojamas su nužudymu susieto šūvio metu arba, jei šūvio nėra, nužudymo metu, o aukos mirties metu. Kitų žaidėjų greičiai yra įverčiai.
wallgauss-weapon = Wallgauss apima tik gauso nužudymus, o ginklo filtras rodo { $weapon }.
wallgauss-checking = Tikrinamas Wallgauss… { $examined } iš { $total } nužudymų
wallgauss-needs-files = Wallgauss tikrinimui reikia žaidimo failų, kad būtų žinoma, kur yra žemėlapio sienos.
wallgauss-failed = Nepavyko patikrinti Wallgauss. { $reason }
help-wallgauss = Gauso nužudymai per sieną, rasti sekant kiekvieną šūvį per žemėlapį taip, kaip tai daro serveris. Nužudymas, kurio nepavyksta nustatyti, pavyzdžiui, kai auka buvo už įrašo matomumo ribų, laikomas nežinomu ir neįtraukiamas.
badge-wall = WALL
wall-yes-exit = Wallgauss, nustatytas pagal šūvį: sprogimas ten, kur spindulys išėjo iš { $thickness } vienetų storio sienos.
wall-yes-beyond = Wallgauss, nustatytas pagal šūvį: spindulys pataikė, praėjęs per { $thickness } vienetų storio sieną.
wall-limits = Nustatyta pagal šūvį ir žaidėjų vietas, nes serveris to neįrašo. Žaidėjas yra 32 vienetų pločio ir laikomas dėže, o auka galėjo būti per vieną ar du kūno pločius nuo vietos, kurią rodo įrašas.
wall-no-direct = Ne Wallgauss: spindulys galėjo pataikyti į auką prieš bet kokią sieną.
wall-no-reflection = Ne Wallgauss: atšokimo sprogimas galėjo pataikyti į auką prieš bet kokią sieną.
wall-doubt-no-shot = Wallgauss nežinomas: žudiko šūvio įraše nėra.
wall-doubt-no-map = Wallgauss nežinomas: nepavyko perskaityti žemėlapio failų.
wall-doubt-unseen = Wallgauss nežinomas: auka buvo už įrašo matomumo ribų.
wall-doubt-brush = Wallgauss nežinomas: spindulys pataikė į duris, liftą ar kitą ypatingą žemėlapio dalį, kuri netikrinama.
wall-doubt-ambiguous = Wallgauss nežinomas: šūvis galėjo nužudyti ir per sieną, ir be jos.
wall-doubt-unexplained = Wallgauss nežinomas: nerastas joks būdas, kaip šūvis pasiekė auką.
wall-doubt-unconfirmed = Wallgauss nežinomas: šūvis galėjo pasiekti auką tik per sieną, bet tai neužtikrinta.
wall-doubt-not-reached = Wallgauss nežinomas: įrašas baigiasi prieš šūvį.
wall-pending = Wallgauss dar nepatikrintas.
events-unknown = { $count ->
    [one] { $count } nužudymas neturi duomenų tam nustatyti
    [few] { $count } nužudymai neturi duomenų tam nustatyti
    [many] { $count } nužudymo neturi duomenų tam nustatyti
   *[other] { $count } nužudymų neturi duomenų tam nustatyti
}
help-unknown = Judėjimas, kurio įrašas negalėjo matyti, laikomas nežinomu, o ne buvimu ant žemės ar stovėjimu vietoje, todėl šie nužudymai neįtraukiami.
clip-it = Į klipą
badge-headshot = ŠG
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = savižudybė
badge-teamkill = teamkill
badge-world = pasaulis
air-both = Abu buvo ore tikrinimo metu.
air-killer = Žudikas buvo ore tikrinimo metu.
air-victim = Auka buvo ore tikrinimo metu.
air-other-unknown = Ar ore buvo kitas, nežinoma.
speed-horizontal = Horizontalusis greitis, lygiagrečiai žemei.
motion-line = { $who }: { $air }, { $speed }. { $how }, { $when }.
motion-who-killer = Žudikas
motion-who-victim = Auka
motion-air-yes = ore
motion-air-no = ant žemės
motion-air-unknown = nežinoma, ar ore
motion-speed-unknown = greitis nežinomas
motion-exact = Užfiksavo žaidimas
motion-trajectory = Įvertinta pagal judėjimą
motion-unclear = Judėjimas neaiškus
motion-not-visible = Už matomumo ribų
motion-at-shot = susieto šūvio metu
motion-at-kill = mirties metu
motion-none = Analizėje nėra šio nužudymo judėjimo duomenų.
legend-shot = Šūvis
legend-charge = Krovimas
legend-kill = Nužudymas
legend-death = Mirtis
legend-switch = Ginklo keitimas

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

clips-new = Naujas klipas
clips-select-event = Pirmiausia pasirinkite įvykį sąraše arba laiko juostoje.
clips-other-session = Klipas negali prasidėti čia, nes po žemėlapio pakeitimo ar prisijungimo iš naujo įrašo laikrodis prasidėjo iš naujo, o ankstesnė dalis turi tuos pačius laikus.
clips-add-window = Šios intensyvios akimirkos klipas
clip-default-name = Klipas { $number }
clip-name = Pavadinimas
clip-length = Trukmė { $duration }
clip-remove = Pašalinti
clip-loop = Kartoti
clip-time-help = Sekundės (96), minutės ir sekundės (1:36) arba valandos, minutės ir sekundės (00:01:36.250)
clips-empty-hint = Klipų dar nėra. Pasirinkite įvykį ir paspauskite „Į klipą“ arba paspauskite „Naujas klipas“, kad pradėtumėte klipą nuo dabartinės akimirkos.
clips-new-help = Dešimties sekundžių klipas nuo dabartinės akimirkos.
clip-in = Pradžia
clip-out = Pabaiga
clip-set-in = Pradėti klipą nuo dabartinės akimirkos
clip-set-out = Baigti klipą ties dabartine akimirka
clip-no-playhead = Peržiūra nepasiekiama, todėl nėra dabartinės akimirkos, kurią būtų galima naudoti.
clip-play = Leisti klipą vieną kartą
clip-show-on-timeline = Rodyti laiko juostoje
export-destination = Keisti eksporto aplanką
export-skipped = Praleista. { $reason }
export-earlier-range = Šis eksportas apima { $start } – { $end }, tokį klipą, koks jis buvo prieš jums jį pakeičiant.
scores-player = Žaidėjas
scores-none = Įraše nėra rezultatų.
clip-effective = Eksportas apima { $start } – { $end } ({ $duration }), taip tiksliai, kaip leidžia įrašas.
settings-clip-padding = Klipo paraštė
settings-preroll = Sekundės prieš įvykį
settings-postroll = Sekundės po įvykio
export-start-first-group = Klipas negali prasidėti pačioje įrašo pradžioje. Pradėkite šiek tiek vėliau.
export-start-during-loading = Pradžia patenka į žemėlapio įkėlimą. Pradėkite šiek tiek vėliau.

export-all = Eksportuoti visus klipus
export-one = Eksportuoti
export-cancel = Stabdyti eksportą
export-folder = Eksportuoti failai keliauja į { $folder }.
export-folder-beside = Eksportuoti failai keliauja šalia įrašo, į { $folder }.
export-waiting = Laukia eksporto…
export-running = Eksportuojama… Perskaitomas visas įrašas, tai gali užtrukti.
export-cancelled = Neeksportuota, nes eksportas sustabdytas.
export-saved = Išsaugota: { $path }. Su originalu patikrintų akimirkų: { $count }.
export-playdemo = Žaidimo konsolėje:
export-copy-first = Nukopijuokite { $name } į modo aplanką, kuriame yra liblist.gam, tada žaidimo konsolėje:
export-copy = Kopijuoti
settings-export-folder = Eksporto aplankas
settings-export-choose = Pasirinkti…
settings-viewer = Įrašų peržiūra
settings-viewer-game = Half-Life aplankas
settings-viewer-game-none = Nepasirinktas
settings-viewer-game-hint = Aplankas, kuriame yra valve. Peržiūra rodo įrašus su žaidimo žemėlapiais, modeliais ir garsais iš jo.
settings-executable = Žaidimo programa
settings-executable-hint = Padeda rasti Half-Life aplanką. Peržiūra pati skaito žaidimo failus ir niekada nepaleidžia žaidimo.
settings-executable-not-set = Nenustatyta, todėl peržiūroje nėra nei vaizdo, nei garso
settings-executable-optional = Nenustatyta ir nereikia, kol Half-Life aplanke yra žaidimas
settings-executable-missing = { $path } nebėra. Peržiūra toliau skaito Half-Life aplanką.
settings-game-override = Šį kartą Lambda Slicer naudoja { $path }, nes buvo paleista su --game. Čia pasirinkta galios nuo kito paleidimo.
settings-browse = Naršyti…
dialog-executables = Programos
setup-title = Nustatykite įrašų peržiūrą
setup-body = Vaizdui ir garsui reikia Half-Life žaidimo failų. Parodykite Lambda Slicer, kur įdiegtas žaidimas, ir ji skaitys juos iš ten.
setup-locate-exe = Rasti hl.exe…
setup-locate-folder = Rasti Half-Life…
setup-settings = Nustatymai
setup-still-works = Laiko juosta, įvykiai ir klipai veikia ir be jų.
setup-relocate = { $folder } nebeturi Half-Life. Raskite jį iš naujo.
setup-not-executable = { $name } nėra { $executable }. Pasirinkite pačią žaidimo programą, ne nuorodą.
setup-not-installed = { $folder } neturi Half-Life diegimo su valve aplanku.
tab-events = Įvykiai
tab-clips = Klipai
tab-scoreboard = Rezultatai
list-hide = Slėpti įrašų sąrašą
list-show = Rodyti įrašų sąrašą
demo-details = Išsami informacija
viewer-start = Į pradžią
viewer-mute = Nutildyti
viewer-unmute = Įjungti garsą
viewer-picture = Vaizdo ir peržiūros nustatymai
viewer-needs-files = Reikia žaidimo failų
viewer-no-sound-why = Nėra garso: { $reason }
settings-tab-general = Bendra
settings-section-folders = Aplankai
settings-folders-hint = Aplankai, kuriuose biblioteka ieško įrašų, įskaitant poaplankius.
settings-clip-padding-hint = Kiek klipas, sukurtas aplink įvykį, pasilieka prieš jį ir po jo.
settings-section-picture = Vaizdas
settings-section-sound = Garsas
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
clips-store-unreadable = Klipų failo { $path } nepavyko perskaityti: { $detail }. Kol Lambda Slicer neuždaryta, klipai neišsaugomi, todėl niekas iš failo nedings.
clips-save-failed = Klipų nepavyko išsaugoti į { $path }: { $detail }
notice-dismiss = Uždaryti

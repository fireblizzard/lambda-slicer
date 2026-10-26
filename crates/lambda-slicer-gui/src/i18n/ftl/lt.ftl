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
settings-start-tab = Skirtukas paleidus
settings-start-tab-hint = Bibliotekos skirtukas, rodomas kaskart paleidus Lambda Slicer. Spustelėjus kitą skirtuką, vis tiek rodomas jis, kol uždarysite Lambda Slicer.
settings-sort = Tvarka paleidus
settings-sort-hint = Kaip įrašai rikiuojami kaskart paleidus Lambda Slicer. Spustelėjus stulpelio antraštę, vis tiek rikiuojama pagal jį, kol uždarysite Lambda Slicer.
settings-sprees = Nužudymų serijos
settings-sprees-hint = Paskutinis žaidėjo nužudymas prieš mirtį gauna ženkliuką, rodantį, kiek nužudymų jis padarė iš eilės, jei jų yra bent tiek. Bet kokia mirtis nutraukia seriją, kaip ir žemėlapio pabaiga ar žaidėjo išėjimas. Savo komandos žaidėjų nužudymai neskaičiuojami.
settings-spree-least = Bent (nužudymų)
settings-start-filters =Filtrai paleidus
settings-start-filters-hint = Kaskart paleidus Lambda Slicer, bibliotekoje įjungiami šie filtrai, kad trumpi įrašai, kurių niekada nenorite matyti, netrukdytų. Filtrus vis tiek galite keisti, kol uždarysite Lambda Slicer.
settings-hide-unplayed-maps = Kelių žemėlapių įrašuose nerodyti žemėlapių be nužudymų, įrašytų trumpiau nei 5 minutes
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
library-matches = { $count ->
    [one] { $count } rungtynės
    [few] { $count } rungtynės
    [many] { $count } rungtynių
   *[other] { $count } rungtynių
}
library-matches-of = { $shown } iš { $total ->
    [one] { $total } rungtynių
    [few] { $total } rungtynių
    [many] { $total } rungtynių
   *[other] { $total } rungtynių
}
library-no-games = Nė viename įraše nėra nei rungtynių, nei penkių minučių žaidimo be jų.
library-no-games-filtered = Šių filtrų neatitinka jokios rungtynės.
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
help-player-filter = Įveskite vardo dalį, kad rastumėte įrašus, kuriuose kas nors jį naudojo. Pasirinkite žaidėją iš sąrašo, kad rastumėte įrašus, kuriuose jis žaidė bet kokiu vardu, atpažįstamas pagal SteamID. Visi pasirinkti žaidėjai turi būti žaidę, ne tik stebėję. Botai ir kai kurie senesni įrašai neturi SteamID, todėl tokius žaidėjus galima rasti tik pagal vardą.
player-remove = Pašalinti iš filtro
player-also = taip pat { $names }
player-demos = { $count ->
    [one] { $count } įrašas
    [few] { $count } įrašai
    [many] { $count } įrašo
   *[other] { $count } įrašų
}
filter-weapon-any = Bet kuris ginklas
filter-min-minutes = Bent (minučių)
filter-min-kills = Bent (nužudymų)
filter-min-spree = Serija bent (nužudymų)
filter-players = Žaidėjai
filter-teams = Komandos
filter-bots = Botai
bots-any = Bet kiek
bots-few = Nedaug
bots-none = Nė vieno
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
export-unknown-persistent-message = Pranešimas { $name } klipuose dar nepalaikomas.
export-unproven-state = Ties { $time } įrašas remiasi duomenimis, kuriuos teko spėti, todėl klipo negalima padaryti tiksliai.
export-unrepresentable-time-field = Laiko pradžioje negalima tiksliai išsaugoti: { $detail }
export-unsupported = Nepalaikoma: { $detail }
export-validation-failed = Eksportuotas klipas neišlaikė galutinės patikros: { $detail }
io-not-found = Failas ar aplankas nerastas.
io-other = { $detail }
io-permission-denied = Prieiga uždrausta.
io-unexpected-end = Failas netikėtai baigėsi.
protocol-unsupported = Šis įrašas yra iš nepalaikomos žaidimo versijos (protokolas { $demo }/{ $network }). Palaikomos tik 5/47 ir 5/48.
protocol-unsupported-network = Serveris perėjo prie nepalaikomos žaidimo versijos (protokolas { $network }).

## Įrašo peržiūra

demo-close = Uždaryti
demo-loading = Analizuojamas { $name }…
show-shots = Šūviai
show-actions = Kiti veiksmai
show-kills = Nužudymai
show-switches = Ginklų keitimai
show-chat = Čatas
show-match = Rungtynės
lane-unknown = Nežinomas žaidėjas
lane-recorder = { $name } (įrašė)
event-kill = Nužudė { $victim }
event-switch = Ginklo keitimas
event-copy = Kopijuoti
event-copy-kill = { $killer } nukovė žaidėją { $victim } ginklu { $weapon }
event-copy-teamkill = { $killer } nukovė savo komandos žaidėją { $victim } ginklu { $weapon }
event-copy-suicide = { $victim } nusižudė ginklu { $weapon }
event-copy-death = { $victim } žuvo, priežastis: { $weapon }
event-vote-called = Pradėtas balsavimas dėl { $subject }
event-vote-cast = Balsavimas dėl { $subject }: { $change } ({ $tally })
event-vote-more-yes = { $count ->
    [one] { $count } balsu „už“ daugiau
    [few] { $count } balsais „už“ daugiau
    [many] { $count } balso „už“ daugiau
   *[other] { $count } balsų „už“ daugiau
}
event-vote-fewer-yes = { $count ->
    [one] { $count } balsu „už“ mažiau
    [few] { $count } balsais „už“ mažiau
    [many] { $count } balso „už“ mažiau
   *[other] { $count } balsų „už“ mažiau
}
event-vote-more-no = { $count ->
    [one] { $count } balsu „prieš“ daugiau
    [few] { $count } balsais „prieš“ daugiau
    [many] { $count } balso „prieš“ daugiau
   *[other] { $count } balsų „prieš“ daugiau
}
event-vote-fewer-no = { $count ->
    [one] { $count } balsu „prieš“ mažiau
    [few] { $count } balsais „prieš“ mažiau
    [many] { $count } balso „prieš“ mažiau
   *[other] { $count } balsų „prieš“ mažiau
}
event-vote-tally = { $yes } už, { $no } prieš, { $undecided } nebalsavo
event-vote-accepted = Balsavimas dėl { $subject } priimtas
event-vote-denied = Balsavimas dėl { $subject } atmestas
event-timeout-called = Paprašyta pertraukos
event-timeout-under-way = Pertrauka prasidėjo
event-timeout-over = Pertrauka baigėsi
event-paused = Žaidimas pristabdytas
event-resumed = Žaidimas tęsiamas
event-sudden-death = Prasidėjo sudden death
event-sudden-death-over = Sudden death baigėsi
event-map-changed = Žemėlapis pakeistas į { $map }
event-spectating = Pradėjo stebėti
event-playing = Baigė stebėti
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

column-date = Data
column-file-date = Failo data
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
scores-spectators = Stebėtojai
scores-follow = Spustelėkite, kad žiūrėtumėte jo akimis
pane-fill = Užimti visą šoninį skydelį
pane-share = Padalyti šoninį skydelį per pusę
scores-frags = Fragai
scores-deaths = Mirtys

viewer-play = Leisti
viewer-pause = Pristabdyti
viewer-volume = Garsumas
viewer-no-sound = Nėra garso
viewer-play-backward = Atsukti
viewer-step-back = Ankstesnis kadras
viewer-step-forward = Kitas kadras
viewer-position = Vieta įraše
viewer-skip-back = 10 sekundžių atgal
viewer-skip-forward = 10 sekundžių pirmyn
viewer-rate = { $rate }×
viewer-slower = Lėčiau
viewer-faster = Greičiau
viewer-rate-normal = Grįžti į įprastą greitį
viewer-shortcut = Spartusis klavišas: { $key }
key-space = Tarpas
key-shift-left = Shift+Kairėn
key-shift-right = Shift+Dešinėn
viewer-map-failed = Nepavyko įkelti žemėlapio { $map }
viewer-stopped = Atkūrimas sustojo anksčiau laiko: { $reason }
viewer-missing = Failai, kurių nepavyko perskaityti: { $count }
event-previous = Ankstesnis įvykis
event-next = Kitas įvykis
event-details = Nužudymo informacija
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
refine-multikill = Multikill
refine-clear = Išvalyti
refine-airborne = Ore
refine-either = Bet kuris
midair-both = Abu
midair-one = Tik vienas
help-midair-either = Žudikas, auka arba abu buvo ore. Žudikas tikrinamas su nužudymu susieto šūvio metu arba, jei šūvio nėra, nužudymo metu, o auka mirties metu. Paprastas šuolis, mažiau nei 50 vienetų virš žemės, neskaičiuojamas. Tam reikia pasirinkti Half-Life aplanką, kad būtų žinoma, kur yra žemė.
help-midair-both = Žudikas ir auka abu buvo ore, kiekvienas savo tikrinimo metu.
help-midair-one = Ore buvo lygiai vienas iš jų, o apie kitą žinoma, kad jis buvo ant žemės.
speed-at-least = Bent
speed-unit = ups
speed-killer = Žudikas
speed-victim = Auka
help-multikill = Keli žaidėjai nužudyti vienu šūviu, pavyzdžiui, sprogimu, pasiekusiu juos visus, arba gauso spinduliu, perėjusiu per juos. Tai rodo kiekvienas to šūvio nužudymas.
help-speed = Greitis išilgai žemės ups vienetais, kaip rodo spidometras, neįskaitant judėjimo aukštyn ir žemyn. Žudiko greitis matuojamas su nužudymu susieto šūvio metu arba, jei šūvio nėra, nužudymo metu, o aukos mirties metu. Kitų žaidėjų greičiai yra įverčiai.
wallgauss-weapon = Wallgauss apima tik gauso nužudymus, o ginklo filtras rodo { $weapon }.
wallgauss-checking = Tikrinamas Wallgauss… { $examined } iš { $total } nužudymų
wallgauss-needs-files = Wallgauss tikrinimui reikia žaidimo failų, kad būtų žinoma, kur yra žemėlapio sienos.
wallgauss-failed = Nepavyko patikrinti Wallgauss. { $reason }
help-wallgauss = Gauso nužudymai per sieną, rasti sekant kiekvieną šūvį per žemėlapį taip, kaip tai daro serveris. Nužudymas, kurio nepavyksta nustatyti, pavyzdžiui, kai auka buvo už įrašo matomumo ribų, laikomas nežinomu ir neįtraukiamas.
badge-wall = WG
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
events-filters = Filtrai
events-filters-on = Filtrai ({ $count })
events-kinds-none = Nėra įvykių
events-follow = Laikyti sąrašą ties dabartine akimirka, kol leidžiamas įrašas. Slenkant sąrašą tai išjungiama.
clip-it = Į klipą
badge-headshot = ŠG
badge-double-kill = double kill
badge-triple-kill = triple kill
badge-quadra-kill = quadra kill
badge-penta-kill = penta kill
badge-multikill = { $count } nužudymai
badge-spree = ×{ $count }
badge-spree-help = { $count ->
    [one] { $count } nužudymas iš eilės nemirštant
    [few] { $count } nužudymai iš eilės nemirštant
    [many] { $count } nužudymo iš eilės nemirštant
   *[other] { $count } nužudymų iš eilės nemirštant
}
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = savižudybė
badge-teamkill = teamkill
badge-world = pasaulis
badge-clip = klipas
clip-of = Klipas iš { $name }
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
column-multikill = Multikill
column-spree = Serija
filter-headshots = Nužudymai šūviu į galvą
filter-midair = Midair nužudymai
event-headshot = į galvą
scores-headshots = ŠG

column-accuracy = Taiklumas
column-players = Žaidėjai
column-teams = Komandos
help-players-column = Žaidime esantys žaidėjai, ne stebėtojai, suskaičiuoti, kai rungtynės prasidėjo po atgalinės atskaitos. Komandiniame žaidime skaičiuojami tik komandoje esantys žaidėjai. Įrašas rodo didžiausią skaičių bet kuriose pilnose rungtynėse, o įrašas be pilnų rungtynių – didžiausią skaičių bet kuriuo metu.
help-teams-column = Komandos, kuriose yra bent vienas žaidėjas, suskaičiuotos taip pat kaip žaidėjai. Žaidime be komandų jų nėra.
filter-min-accuracy = Taiklumas bent (%)
scores-accuracy = Taikl.
scores-kd = K/D
scores-ping = Pingas
help-kd = Fragai vienai mirčiai: fragai, padalyti iš mirčių. Jei žaidėjas nė karto nemirė, rodomi jo fragai.
help-ping-loss = Kiekvieno žaidėjo pingas milisekundėmis, o po jo loss, t. y. prarasti paketai, kokius serveris atsiuntė paskutinį kartą. Serveris juos siunčia tik tada, kai įrašantysis laiko rezultatų lentelės klavišą, todėl iki tol jų vietoje rodomas brūkšnys.

help-headshots = Nužudymai pataikius į galvą, atpažinti iš aukos mirties animacijos. Sprogimai neskaičiuojami. Rodomi tik įrašančiojo nužudymai, nes kiti žaidėjai dažnai žudo už įrašo matomumo ribų. Dvikovoje rodomi ir priešininko, nes visi jo nužudymai yra įrašančiojo.
help-headshots-column = Įrašančiojo nužudymai pataikius į galvą, atpažinti iš aukos mirties animacijos. Sprogimai neskaičiuojami. Kitų žaidėjų nužudymai neskaičiuojami, nes dažnai įvyksta už įrašo matomumo ribų.
help-multikill-column = Daugiausia žaidėjų, kuriuos kas nors įraše nužudė vienu šūviu, pavyzdžiui, 3, jei tai triple kill.
help-spree-column = Ilgiausia įrašo nužudymų serija, bet kurio žaidėjo: daugiausia nužudymų, kuriuos žaidėjas padarė iš eilės nemirdamas.
help-date-column = Kada buvo įrašytos rungtynės. Data imama iš failo pavadinimo, kai jame yra data, pvz., 20230910_214553, kaip žaidimo automatiškai daromuose įrašuose. Kitaip tai failo data, kurią kopijavimas ar atsisiuntimas gali pakeisti.
help-file-date = Kada failas buvo išsaugotas. Eksportuoto klipo atveju tai yra eksportavimo laikas.
help-accuracy = Pataikiusių šūvių dalis, nustatyta pagal kraują, kurį pataikymas palieka taikymo linijoje, atėmus atsitiktinius sutapimus. Įtraukiami glock, python, MP5 kulkos, šautuvas, arbaletas su taikikliu ir tiesioginiai gauso spindulio pataikymai. Sprogimai ir gauso atšokimai kraujo nepalieka ir neskaičiuojami. Rodoma tik įrašančiajam ir tik po bent 50 šūvių.
help-headshot-filter = Įrašai, kuriuose yra bent vienas įraše matomas nužudymas į galvą, bet kurio žaidėjo.
help-midair-filter = Įrašai, kuriuose yra bent vienas nužudymas, kai žudikas arba auka buvo ore, bent 50 vienetų virš žemės. Nepasirinkus Half-Life aplanko, skaičiuojamas ir paprastas šuolis.
help-spree-filter = Įrašai, kuriuose žaidėjas iš eilės nemirdamas nužudė bent tiek. Bet kokia mirtis nutraukia seriją, kaip ir žemėlapio pabaiga ar žaidėjo išėjimas. Savo komandos žaidėjų nužudymai neskaičiuojami.
help-accuracy-filter = Įrašai, kuriuose įrašantysis pataikė bent tokią šūvių dalį.
help-bots-filter = „Nedaug“ palieka tik įrašus ir rungtynes, kuriose botų buvo mažiau nei tikrų žaidėjų, todėl 1v1 prieš botą atmetamas, bet pilnas serveris, kuriame keli botai užpildo vietas, – ne. „Nė vieno“ palieka tik tuos, kuriuose nebuvo nė vieno boto. Botas – tai žaidėjas, kurį žaidimas pažymi kaip botą, arba žaidėjas be CD rakto.

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
export-stopped = Eksportas sustabdytas, todėl klipas neišsaugotas.
export-start-during-loading = Pradžia patenka į žemėlapio įkėlimą. Pradėkite šiek tiek vėliau.

export-all = Eksportuoti visus klipus
export-one = Eksportuoti
export-cancel = Stabdyti eksportą
export-folder = Eksportuoti failai keliauja į { $folder }.
export-folder-beside = Eksportuoti failai keliauja šalia įrašo, į { $folder }.
export-waiting = Laukia eksporto…
export-running = Eksportuojama… Perskaitomas visas įrašas, tai gali užtrukti.
export-searching = Klipas negali prasidėti būtent čia. Netoliese ieškoma pradžios, kuri tiktų…
export-nearest-earlier = Vietoj to jis gali prasidėti ties { $time }, { $seconds } s anksčiau.
export-nearest-later = Vietoj to jis gali prasidėti ties { $time }, { $seconds } s vėliau.
export-start-there = Pradėti ten ir eksportuoti
export-cancelled = Neeksportuota, nes eksportas sustabdytas.
export-saved = Išsaugota: { $path }. Su originalu patikrintų akimirkų: { $count }.
export-playdemo = Žaidimo konsolėje:
export-copy-first = Nukopijuokite { $name } į modo aplanką, kuriame yra liblist.gam, tada žaidimo konsolėje:
export-copy = Kopijuoti
settings-export-folder = Eksporto aplankas
settings-export-choose = Pasirinkti…
settings-file-names = Failų pavadinimai
settings-clip-name-files = Naudoti klipo pavadinimą failui
settings-demo-name-first = Pradėti nuo įrašo pavadinimo
settings-viewer = Įrašų peržiūra
settings-viewer-game = Half-Life aplankas
settings-viewer-game-none = Nepasirinktas
settings-viewer-game-hint = Aplankas, kuriame yra valve. Peržiūra rodo įrašus su žaidimo žemėlapiais, modeliais ir garsais iš jo.
settings-game-override = Šį kartą Lambda Slicer naudoja { $path }, nes buvo paleista su --game. Čia pasirinkta galios nuo kito paleidimo.
settings-more-games = Daugiau Half-Life aplankų
settings-more-games-hint = Kitos vietos, kur įdiegtas Half-Life, pavyzdžiui, aplankas su daug žemėlapių. Kai žemėlapio, modelio, spraito ar garso nėra aukščiau esančiame Half-Life aplanke, peržiūra ieško jo šiuose aplankuose iš viršaus į apačią. Juos pakeitus, biblioteka iš naujo patikrina savo įrašus dėl midair nužudymų su jų pridedamais žemėlapiais.
settings-more-games-empty = Aplankų dar nepridėta.
settings-move-up = Perkelti aukštyn
settings-move-down = Perkelti žemyn
settings-game-content = Žaidimo turinys
settings-game-content-hint = Iš pradžių šios parinktys nustatytos taip, kaip žaidime. Pakeitus jas čia, keičiasi tik peržiūra, niekada ne žaidimas. Kai įjungta daugiau nei viena, naudojama žaidimo tvarka: pirmiausia priedų turinys, tada lokalizuotas turinys, tada HD modeliai, tada paties žaidimo failai.
settings-hd-models = Naudoti HD modelius
settings-hd-models-hint = Aukštos kokybės modelių pirmiausia ieško _hd aplankuose, pavyzdžiui, valve_hd. Iš pradžių nustatyta kaip žaidimo parinktis „Use high quality models“.
settings-addon-content = Naudoti savo priedų turinį
settings-addon-content-hint = Pirmiausia ieško _addon aplankuose, pavyzdžiui, valve_addon, turinio, kurį pridėjote į žaidimą. Iš pradžių nustatyta kaip žaidimo parinktis „Allow custom addon content“.
settings-localized-content = Naudoti lokalizuotą turinį
settings-localized-content-hint = Pirmiausia ieško aplankuose tos kalbos, kuria Steam paleidžia Half-Life, pavyzdžiui, valve_spanish ispanų kalbai. Žaidimas juos naudoja visada, o anglų kalba jų neturi.
settings-steam-language = Steam paleidžia Half-Life šia kalba: { $language }.
settings-steam-english = Steam paleidžia Half-Life anglų kalba, kuri neturi savo aplankų.
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
tab-demos = Įrašai
tab-matches = Rungtynės
filter-status = Būsena
library-all = Visi
library-favourites = Mėgstami
library-no-favourites = Mėgstamų įrašų dar nėra. Spustelėkite įrašą dešiniuoju pelės mygtuku ir pasirinkite „Įtraukti į mėgstamus“.
menu-favourite-add = Įtraukti į mėgstamus
menu-favourite-remove = Pašalinti iš mėgstamų
favourite-star = Mėgstamas
library-hidden = Paslėpti
library-no-hidden = Paslėptų įrašų nėra. Spustelėkite įrašą dešiniuoju pelės mygtuku ir pasirinkite „Slėpti“, kad jo nebūtų sąraše.
menu-hide = Slėpti
menu-unhide = Rodyti
filter-status-all = Visos
help-status-column = Kiek rungtynių įrašas užfiksavo. Užveskite pelę ant eilutės būsenos, kad pamatytumėte, ką ji reiškia.
help-match-players = Žaidime esantys žaidėjai, ne stebėtojai, kai rungtynės prasidėjo po atgalinės atskaitos, arba ties pirmuoju nužudymu, jei rungtynės jau vyko, kai prasidėjo įrašas. Laisvame žaidime – daugiausia vienu metu.
help-match-teams = Komandos, kuriose yra bent vienas žaidėjas, suskaičiuotos taip pat kaip žaidėjai. Žaidime be komandų jų nėra.
help-match-multikill = Daugiausia žaidėjų, kuriuos kas nors rungtynėse nužudė vienu šūviu, pavyzdžiui, 3, jei tai triple kill.
help-match-spree = Ilgiausia rungtynėse pasibaigusi nužudymų serija, bet kurio žaidėjo: daugiausia nužudymų, kuriuos žaidėjas padarė iš eilės nemirdamas.
match-complete = Pilnos
match-joined-late = Prisijungta vėliau
match-interrupted = Nutrauktos
match-cut-off = Įrašas nutrūko
match-free-play = Laisvas žaidimas
help-match-complete = Įrašyta nuo atgalinės atskaitos iki rungtynių pabaigos.
help-match-joined-late = Rungtynės jau vyko, kai prasidėjo įrašas, ir buvo įrašytos iki pabaigos.
help-match-interrupted = Prasidėjo kitos rungtynės, rungtynės buvo atšauktos arba pasikeitė žemėlapis, kol rungtynės dar nesibaigė.
help-match-cut-off = Įrašas baigėsi anksčiau nei rungtynės.
help-match-free-play = Penkios ar daugiau minučių žaidimo, kai nevyksta rungtynės.
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
settings-tab-library = Biblioteka
settings-tab-clips = Klipai
settings-section-game = Half-Life
settings-section-view = Vaizdas
settings-language-system = Kompiuterio kalba ({ $name })
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
settings-zoom-fov = Matymo laukas priartinus
settings-viewmodel = Rodyti ginklą rankoje
settings-crosshair = Rodyti taikiklį
settings-chat = Rodyti čatą
settings-vitals = Rodyti gyvybes, šarvus, šovinius ir paimtus daiktus
settings-vitals-size = Gyvybių, šarvų, šovinių ir paimtų daiktų dydis
settings-free-vitals = Rodyti gyvybes, šarvus, šovinius ir paimtus daiktus ir su laisva kamera
settings-scores = Rodyti rezultatus viršuje kairėje
settings-score-rows = Daugiausia rezultatų eilučių
settings-scoreboard = Rodyti rezultatų lentelę, kol įrašantysis laiko jos klavišą, ir žemėlapio pabaigoje
settings-scoreboard-off = Išjungta
settings-scoreboard-fullscreen = Tik per visą ekraną
settings-scoreboard-always = Įjungta
settings-server-settings = Rodyti serverio nustatymus viršuje dešinėje
settings-text-size = Čato, kill feed ir serverio nustatymų dydis
settings-hud-messages = Rodyti serverio tekstą, pvz., bhop ir KZ laikmačius
settings-see-through = Matyti žaidėjus pro sienas
settings-see-through-items = Taip pat kitus modelius, pvz., sviedinius ir daiktus
settings-player-names = Rodyti žaidėjų vardus virš jų
settings-show-position = Rodyti padėtį ir žiūrėjimo kampus
settings-speedometer = Rodyti spidometrą
settings-jump-speed = Rodyti paskutinio šuolio greitį
settings-jump-speed-hint = Kaip greitai judėjo įrašantysis, kai paskutinį kartą atsiplėšė nuo žemės. Rodoma po spidometru iki kito jo šuolio.
settings-recorder-fps = Rodyti įrašančiojo FPS
settings-recorder-fps-hint = Kiek kadrų per sekundę žaidimas veikė pas įrašantįjį, viršuje kairėje, kas pusę sekundės paimtas vidurkis. Ne tai, kaip greitai jį piešia ši programa.
settings-speed-axes = Matuojamas greitis
settings-speed-horizontal = Horizontalusis
settings-speed-vertical = Vertikalusis
settings-speed-both = Abu (3D)
viewer-jump-speed = Šuolis { $speed }
settings-show-position-hint = Iš kur matomas vaizdas, žaidimo vienetais, ir kur jis nukreiptas, laipsniais aukštyn ar žemyn (pitch) ir aplink (yaw). Kai įjungta laisva kamera, tai laisvos kameros reikšmės. Naudinga, kai norite kam nors tiksliai pasakyti, kur žiūrėti.
viewer-position-line = Padėtis { $x } { $y } { $z }
viewer-fps = { $fps } FPS
viewer-angles-line = Kampai { $pitch } { $yaw }
settings-player-names-hint =Kiekvieno matomo žaidėjo vardas rodomas virš jo galvos jo komandos spalva, taip pat ir žiūrint laisva kamera. Žaidėjų už sienų vardai rodomi tik tol, kol žaidėjai matomi pro sienas.
settings-see-through-hint = Žaidėjai už sienų rodomi kaip siluetai jų viršutine spalva, kurią komandiniai žaidimai nustato pagal komandą. Gali būti rodoma tik tai, ką turi įrašas. Serveris siunčia įrašančiajam artimus žaidėjus, o serveriai, slepiantys žaidėjus, kurių įrašantysis nemato, nepalieka ko rodyti.
settings-section-decals = Dekalai
settings-decals = Daugiausia
settings-decals-hint = Kulkų skylės, kraujas ir kiti žaidimo metu palikti pėdsakai. Lieka tik naujausi, o seniausi dingsta atsiradus naujiems. Pėdsakai, kurie yra žemėlapio dalis, lieka visada.
settings-restore = Atkurti numatytuosius…
settings-restore-title = Atkurti numatytuosius
settings-restore-list = Šie nustatymai grįš į tokius, kokie buvo pradžioje. Half-Life aplankas ir bibliotekos aplankai liks tokie, kokie yra.
settings-restore-none = Visi nustatymai jau yra numatytieji.
settings-restore-columns = Bibliotekoje rodomi stulpeliai
settings-restore-layout = Atidaryti skydeliai ir jų dydžiai
settings-clear-library = Išvalyti biblioteką…
settings-clear-library-title = Išvalyti biblioteką
settings-clear-library-text = { $count ->
    [one] Lambda Slicer pamirš, ką rado { $count } įraše, ir iš naujo nuskaitys bibliotekos aplankus, o tai gali šiek tiek užtrukti. Jūsų įrašų failai, klipai ir nustatymai liks tokie, kokie yra.
    [few] Lambda Slicer pamirš, ką rado { $count } įrašuose, ir iš naujo nuskaitys bibliotekos aplankus, o tai gali šiek tiek užtrukti. Jūsų įrašų failai, klipai ir nustatymai liks tokie, kokie yra.
    [many] Lambda Slicer pamirš, ką rado { $count } įrašo, ir iš naujo nuskaitys bibliotekos aplankus, o tai gali šiek tiek užtrukti. Jūsų įrašų failai, klipai ir nustatymai liks tokie, kokie yra.
   *[other] Lambda Slicer pamirš, ką rado { $count } įrašų, ir iš naujo nuskaitys bibliotekos aplankus, o tai gali šiek tiek užtrukti. Jūsų įrašų failai, klipai ir nustatymai liks tokie, kokie yra.
}
viewer-zoom = Priartinti vaizdą
viewer-free-camera = Laisva kamera
viewer-free-camera-help = Skraidykite po žemėlapį atskirai nuo įrašą dariusio žaidėjo. W, A, S ir D – judėti, Space – kilti aukštyn, Ctrl – leistis žemyn, Shift – judėti greičiau. Laikykite nuspaudę dešinįjį pelės mygtuką ir tempkite, kad apsidairytumėte.
viewer-fullscreen = Visas ekranas
viewer-leave-fullscreen = Išjungti visą ekraną
viewer-see-through = Matyti pro sienas
viewer-player-names = Žaidėjų vardai
viewer-fullbright = Apšviesti viską
viewer-fullbright-help = Rodo visą žemėlapį visiškai apšviestą, kad tamsūs kampai ir juose esantys žaidėjai būtų gerai matomi.
viewer-paused = Pristabdyta
viewer-map-change = Serveris keičia žemėlapį į { $map }
viewer-pov-recorder = { $name } (įrašė)
viewer-pov-recorder-unnamed = Įrašantysis
viewer-pov-help = Kieno akimis žiūrėti. ` grąžina prie įrašančiojo, klavišai nuo 1 iki 9 ir 0 parenka geriausius šio žemėlapio žaidėjus pagal taškus, o pelės ratukas virš vaizdo arba Page Up ir Page Down perjungia į ankstesnį ar kitą žaidėją.
viewer-scoreboard-ping-loss = Pingas/loss
viewer-scoreboard-bot = BOT
viewer-scoreboard-players = { $count ->
    [one] ({ $count } žaidėjas)
    [few] ({ $count } žaidėjai)
    [many] ({ $count } žaidėjo)
   *[other] ({ $count } žaidėjų)
}
viewer-pov-unseen = Įraše čia nematyti žaidėjo { $name }, todėl matote įrašančiojo vaizdą
settings-events-leave-free = Išeiti iš laisvos kameros pasirinkus įvykį
settings-events-leave-free-hint = Pasirinktas nužudymas rodomas žudiko, aukos arba įrašančiojo akimis. Kai tai išjungta, laisva kamera lieka ten, kur yra, o ją išjungę matysite tomis akimis.
settings-export-beside = Šalia kiekvieno įrašo
settings-export-hint = Modo aplanke, kuriame yra liblist.gam, pvz., Half-Life\ag, klipai paleidžiami viena playdemo komanda.
settings-file-names-hint = Eksportuotas klipas pavadinamas pagal jo įrašą ir pradžios bei pabaigos laikus, nebent naudojate klipo pavadinimą. Žaidimas gali atverti tik failus, kurių pavadinimai sudaryti iš anglų abėcėlės raidžių, skaičių, - ir ., todėl visa kita pakeičiama į _. Jei klipo pavadinime nėra nė vieno iš šių ženklų, failas pavadinamas pagal laikus.

clips-store-moved = Klipų failo { $path } nepavyko perskaityti, todėl jis išsaugotas kaip { $moved }, o klipų sąrašas pradedamas iš naujo. { $detail }
clips-store-unreadable = Klipų failo { $path } nepavyko perskaityti: { $detail }. Kol Lambda Slicer neuždaryta, klipai neišsaugomi, todėl niekas iš failo nedings.
clips-save-failed = Klipų nepavyko išsaugoti į { $path }: { $detail }
notice-dismiss = Uždaryti

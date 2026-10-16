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
column-action = Veiksmas
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
details-action = Geriausias veiksmo balas { NUMBER($score, minimumFractionDigits: 2) }

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
export-missing-parent = Aplanko, skirto { $path }, nėra.
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

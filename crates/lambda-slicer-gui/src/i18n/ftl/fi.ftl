# Suomi. Tarkistamaton luonnos

app-title = Lambda Slicer

## Työkalurivi

toolbar-open = Avaa demo…
toolbar-add-folder = Lisää kansio…
toolbar-rescan = Skannaa uudelleen
toolbar-cancel = Peruuta
toolbar-settings = Asetukset
dialog-demos = Half-Life-demot

## Asetukset

settings-title = Asetukset
settings-language = Kieli
settings-unreviewed = Tämä käännös on luonnos, jota kukaan äidinkielinen puhuja ei ole vielä tarkistanut.
settings-workers = Analyysisäikeet
settings-workers-hint = Useampi säie analysoi nopeammin mutta kuormittaa suoritinta enemmän.
settings-cache = Kirjaston välimuisti
settings-folders = Kirjaston kansiot
settings-folders-empty = Kansioita ei ole vielä lisätty.
settings-remove = Poista

## Kirjasto

library-empty = Aloita lisäämällä kansio tai avaamalla demo.
library-no-matches = Mikään demo ei vastaa näitä suodattimia.
library-no-cache = Tässä järjestelmässä ei ole paikkaa kirjaston välimuistille.
library-demos = { $count ->
    [one] { $count } demo
   *[other] { $count } demoa
}
library-percent = { $percent } %

filter-search = Hae poluista
filter-map = Kartta
filter-player = Pelaaja
filter-weapon-any = Mikä tahansa ase
filter-min-minutes = Vähintään (minuuttia)
filter-spectating-any = Pelaten tai katsojana
filter-spectating-mostly = Enimmäkseen katsojana
filter-spectating-playing = Enimmäkseen pelaten
filter-clear = Tyhjennä suodattimet

column-status = Tila
column-name = Demo
column-map = Kartta
column-duration = Pituus
column-gauss = Gauss
column-kills = Tapot
column-action = Kiivaus
column-spectating = Katsojana

status-complete = Analysoitu
status-partial = Osittainen
status-failed = Epäonnistui
status-unsupported = Ei tuettu
status-no-use = Ei aseiden käyttöä

kind-multiplayer-view = Pelaajan näkymä
kind-singleplayer = Yksinpeli
kind-hltv = Katsojaproxy
kind-unknown = Tuntematon laji

details-recorder = Tallentaja { $name }
details-shots = { $count ->
    [one] { $count } laukaus
   *[other] { $count } laukausta
}
details-kills = { $count ->
    [one] { $count } tappo
   *[other] { $count } tappoa
}
details-action = Suurin kiivaus { NUMBER($score, minimumFractionDigits: 2) }

## Skannaus

scan-progress = Löydetty { $found }, valmiina { $done } (välimuistista { $cached }, analysoitu { $analysed }), luettu { $megabytes } Mt
scan-cancelling = Pysäytetään…
scan-cancelled = Pysäytetty, skannaamatta { $left }.
scan-finished = Valmis: { $done } demoa, analysoitu { $analysed }, aikaa { $seconds } s.
scan-problems = { $count ->
    [one] { $count } ongelma
   *[other] { $count } ongelmaa
}
problems-title = Skannauksen ongelmat
problem-changed = Tiedosto muuttui analyysin aikana, joten mitään ei tallennettu.
drop-hint = Pudota demoja tai kansioita lisätäksesi ne
drop-not-demo = Ei demo: { $names }

## Analyysiytimen viestit koodeittain

analysis-panicked = Analyysi pysähtyi sisäiseen virheeseen: { $detail }
cache-cancelled = Analyysi peruttiin, joten mitään ei tallennettu.
cache-database = Kirjaston välimuistia ei voi käyttää: { $detail }
cache-directory = Välimuistikansiota ei voi luoda: { $detail }
container-bad-magic = Tämä ei ole Half-Life-demo.
container-frame-bad-time = Tallenne on vioittunut osan { $entry } ruudussa { $frame }, jonka aika on virheellinen.
container-frame-missing-terminal = Tallenteen osa { $entry } päättyy ilman lopetustietuetta.
container-frame-negative-length = Tallenne on vioittunut osan { $entry } ruudussa { $frame }.
container-frame-over-limit = Osan { $entry } ruutu { $frame } on sallittua suurempi.
container-frame-truncated = Tallenne katkeaa osan { $entry } ruutuun { $frame }.
container-frame-unknown-tag = Tallenne on vioittunut osan { $entry } ruudussa { $frame }, jonka tyyppi on tuntematon.
container-io = Tiedostoa ei voi lukea: { $detail }
container-no-directory = Tallennetta ei koskaan viety loppuun, joten siinä ei ole sisällysluetteloa.
container-range = Tallenteen sisällysluettelo osoittaa tiedoston ulkopuolelle.
container-too-short = Tiedosto on liian lyhyt ollakseen demo.
coverage-hltv = Katsojaproxyn tallenne. Tallentavaa pelaajaa ei ole, ja tapahtumat seuraavat proxyn näkymää.
coverage-mostly-spectating = Tallentaja oli katsojana { $percent } % toistosta, joten useimmat tapahtumat ovat muiden pelaajien, sellaisina kuin seurattu näkymä ne näytti.
coverage-no-local-events = Ei paikallisia tapahtumia. Tallentajan omat toiminnot puuttuvat, paitsi palvelimen lähettämät.
coverage-singleplayer = Yksinpeli. Muut toimijat ovat hirviöitä tai kartan osia, eikä tappolistaa ole.
coverage-unknown = Tallennetta ei saatu purettua niin pitkälle, että sen laji selviäisi.
coverage-visible-only = Muiden pelaajien tapahtumat tulevat tälle asiakkaalle vain, kun palvelin pitää heitä näkyvinä, joten niiden määrät ovat alarajoja.
decode-failed = Purku pysähtyi kohdassa { $time } (ruutu { $frame }): { $detail }
export-active-effect = { $effect } on käynnissä alussa, eikä sitä voi vielä siirtää.
export-client-dropped-packets = Peli hylkäsi myöhästyneitä paketteja kohdassa { $time }, mitä leikkaus ei vielä osaa toistaa.
export-cross-segment = Uusi istunto alkaa kohdassa { $time }, valitun välin sisällä.
export-destination-exists = { $path } on jo olemassa. Valitse toinen nimi.
export-empty-range = Valitulla välillä ei ole mitään vietävää.
export-io = Kohdetta { $path } ei voi kirjoittaa: { $detail }
export-missing-delta-base = Tallenteesta puuttuu ennen alkua tarvittava tila (järjestys { $sequence }).
export-missing-parent = Kansiota { $path } ei ole.
export-range-beyond-end = Loppu { $end } on tallenteen lopun ({ $segment_end }) jälkeen.
export-same-as-source = Kohde on itse lähdedemo.
export-source-changed = { $path } muuttui viennin aikana.
export-source-decode-failed = Lähdedemoa ei voi purkaa: { $detail }
export-source-failed-crosscheck = Lähdedemo ei läpäissyt ristiintarkistusta, joten sen koottuun tilaan ei voi luottaa.
export-unknown-persistent-message = Ennen alkua oleva { $name } asettaa tilan, jota leikkaus ei vielä osaa siirtää.
export-unproven-state = Tila kohdassa { $time } perustuu pääteltyyn pohjaan eikä ole todistettu.
export-unrepresentable-time-field = Alun aikaa ei voi kirjoittaa tarkasti: { $detail }
export-unsupported = Ei tuettu: { $detail }
export-validation-failed = Viety tiedosto ei läpäissyt tarkistusta: { $detail }
io-not-found = Tiedostoa tai kansiota ei löytynyt.
io-other = { $detail }
io-permission-denied = Pääsy estetty.
io-unexpected-end = Tiedosto päättyi odottamatta.
protocol-unsupported = Demoprotokollaa { $demo }/{ $network } ei tueta, vain 5/48.
protocol-unsupported-network = Palvelin vaihtoi verkkoprotokollaan { $network }, jota ei tueta.

## Demonäkymä

demo-close = Sulje
demo-loading = Analysoidaan { $name }…
timeline-fit = Sovita
show-shots = Laukaukset
show-actions = Muut toiminnot
show-kills = Tapot
show-switches = Aseen vaihdot
lane-unknown = Tuntematon pelaaja
lane-recorder = { $name } (tallentaja)
column-time = Aika
column-player = Pelaaja
column-weapon = Ase
column-event = Tapahtuma
event-kill = Tappoi pelaajan { $victim }
event-switch = Aseen vaihto
action-swing = Lyönti
action-fire = Laukaus
action-launch = Laukaisu
action-charge-activity = Lataus
action-charge-stop = Latauksen loppu
action-beam-start = Säteen alku
action-beam-sustain = Säde
action-beam-stop = Säteen loppu
action-placement-attempt = Asetusyritys
action-release-attempt = Vapautusyritys
mode-primary = ensisijainen
mode-charged = ladattu
mode-zoomed = tähtäimellä
mode-unzoomed = ilman tähtäintä
mode-grenade = kranaatti
mode-bullets = luodit
mode-single = yksittäinen
mode-double = kaksois
mode-rapid = nopea
demo-events = { $count ->
    [one] { $count } tapahtuma
   *[other] { $count } tapahtumaa
}

column-date = Päiväys
column-score = Tulos
filter-from = Alkaen
filter-to = Asti
date-from-name = Päiväys tiedoston nimestä
date-from-modified = Tiedoston viimeisin muokkausaika, jonka kopiointi tai lataus voi muuttaa
timeline-best-action = Kultainen minuutti
timeline-windows = Kiivaimmat hetket
timeline-window = { $rank }. { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Lopputulokset viimeisen tapon kohdalla { $time }
scores-at = Tilanne kohdassa { $time }
scores-show-final = Näytä lopputulokset
scores-frags = Fragit
scores-deaths = Kuolemat

show-timeline = Näytä aikajana
viewer-play = Toista
viewer-pause = Tauko
viewer-volume = Äänenvoimakkuus
viewer-play-backward = Kelaa taaksepäin
viewer-step-back = Edellinen ruutu
viewer-step-forward = Seuraava ruutu
viewer-rate = { $rate }×
viewer-shortcut = Pikanäppäin: { $key }
key-space = Välilyönti
viewer-map-failed = Karttaa { $map } ei voitu ladata
viewer-stopped = Toisto pysähtyi ennen aikojaan: { $reason }
viewer-missing = Tiedostot, joita ei voitu lukea: { $count }
event-previous = Edellinen tapahtuma
event-next = Seuraava tapahtuma
event-play = Toista tapahtuma
legend-shot = Laukaus
legend-charge = Lataus
legend-kill = Tappo
legend-death = Kuolema
legend-switch = Aseen vaihto
timeline-help = Jokainen rivi on pelaaja. Vieritä zoomataksesi, vedä siirtääksesi, kaksoisnapsauta nähdäksesi kaiken ja napsauta merkkiä valitaksesi sen.

column-headshots = Päälaukaukset
filter-headshots = Päälaukaustapot
event-headshot = päälaukaus
scores-headshots = PL

column-accuracy = Tarkkuus
filter-min-accuracy = Tarkkuus vähintään (%)
scores-accuracy = Tark.

help-headshots = Tapot osumalla päähän, tunnistettu uhrin kuolinanimaatiosta. Räjähdykset eivät lasketa. Vain tallentajan tapot näytetään, koska muut pelaajat tappavat usein tallennuksen näkymän ulkopuolella. Kaksintaistelussa näytetään myös vastustajan tapot, koska ne kaikki kohdistuvat tallentajaan.
help-headshots-column = Tallentajan tapot osumalla päähän, tunnistettu uhrin kuolinanimaatiosta. Räjähdykset eivät lasketa. Muiden pelaajien tappoja ei lasketa, koska ne tapahtuvat usein tallennuksen näkymän ulkopuolella.
help-accuracy = Osuneiden laukausten osuus, päätelty verestä, jonka osuma jättää tähtäyslinjalle, vähennettynä sattumanvaraisilla osumilla. Mukana ovat glock, python, MP5:n luodit, haulikko, tähtäimellä ammuttu varsijousi ja gauss-säteen suorat osumat. Räjähdykset ja gaussin heijastukset eivät jätä verta eivätkä lasketa. Näytetään vain tallentajalle ja vasta vähintään 50 laukauksen jälkeen.
help-headshot-filter = Demot, joissa on vähintään yksi tallennuksessa näkyvä päälaukaustappo, kenen tahansa.
help-accuracy-filter = Demot, joissa tallentaja osui vähintään tähän osuuteen laukauksistaan.

clips-title = Klipit
clips-new = Uusi klippi
clips-add-event = Klippi { $start } – { $end }
clips-add-event-none = Klippi valitusta tapahtumasta
clips-select-event = Valitse ensin tapahtuma luettelosta tai aikajanalta.
clips-other-session = Klippi ei voi alkaa tästä, koska tallenteen aiemmalla istunnolla on samat ajat.
clips-add-window = Klippi tästä kiivaasta hetkestä
clips-empty = Klippejä ei vielä ole. Valitse tapahtuma tai jokin kiivaimmista hetkistä ja lisää sen ympärille klippi. Sen, paljonko klippi ottaa ennen tapahtumaa ja sen jälkeen, voi säätää asetuksissa. Uusi klippi lisää klipin, jonka alun ja lopun kirjoitat itse.
clip-default-name = Klippi { $number }
clip-name = Nimi
clip-length = Pituus { $duration }
clip-show = Näytä
clip-remove = Poista
clip-loop = Silmukka
clip-time-help = Sekunteina (96), minuutteina ja sekunteina (1:36) tai tunteina, minuutteina ja sekunteina (00:01:36.250)
clip-exact = Vienti alkaa ja päättyy juuri tähän.
clip-effective = Vienti leikkaa { $start } – { $end } ({ $duration }) tallenteen ruutujen mukaan.
timeline-clip-help = Klipit ovat vaaleansinisiä kaistoja. Vedä kaistan reunaa siirtääksesi sitä.
settings-clip-padding = Klipin marginaali
settings-preroll = Sekuntia ennen tapahtumaa
settings-postroll = Sekuntia tapahtuman jälkeen
export-start-first-group = Leikkaus ei voi alkaa tallenteen aivan ensimmäisestä ruudusta. Aloita hieman myöhemmin.
export-start-during-loading = Alku osuu kartan lataukseen. Aloita hieman myöhemmin.

export-all = Vie kaikki klipit
export-one = Vie
export-cancel = Pysäytä vienti
export-folder = Viedyt tiedostot menevät kansioon { $folder }.
export-folder-beside = Viedyt tiedostot menevät demon viereen, kansioon { $folder }.
export-waiting = Odottaa vientiä…
export-running = Viedään… Koko demo luetaan, mikä voi kestää hetken.
export-cancelled = Ei viety, koska vienti pysäytettiin.
export-saved = Tallennettu: { $path }. Alkuperäiseen verratut tilannekuvat: { $count }.
export-playdemo = Pelin konsolissa:
export-copy-first = Kopioi { $name } modin kansioon, jossa on liblist.gam, ja sitten pelin konsolissa:
export-copy = Kopioi
settings-export-folder = Vientikansio
settings-export-choose = Valitse…
settings-viewer = Demokatselin
settings-viewer-game = Half-Life-kansio
settings-viewer-game-none = Ei valittu. Valitse kansio, jossa valve on, katsoaksesi demoja katselimessa.
settings-gamma = Gamma
settings-brightness = Kirkkaus
settings-texgamma = Tekstuurien gamma
settings-lightgamma = Valon gamma
settings-overbright = Ylikirkas valaistus
settings-gamma-hint = Pelin samannimiset asetukset (gamma, brightness, texgamma, lightgamma, gl_overbright). Aseta ne kuten pelissä nähdäksesi saman kirkkauden.
settings-gamma-defaults = Pelin oletukset
settings-fov-recorded = Näkökenttä tallennuksen mukaan
settings-fov = Näkökenttä
settings-viewmodel = Näytä ase kädessä
settings-see-through = Näe pelaajat seinien läpi
settings-see-through-items = Myös muut mallit, kuten ammukset ja esineet
settings-see-through-hint = Seinien takana olevat pelaajat näkyvät siluetteina yläosan värissään, jonka joukkuepelit asettavat joukkueen mukaan. Näkyä voi vain se, mitä demo sisältää. Palvelin lähettää tallentajan lähellä olevat pelaajat, eivätkä palvelimet, jotka piilottavat pelaajat, joita tallentaja ei näe, jätä mitään näytettävää.
viewer-see-through = Näe seinien läpi
settings-export-beside = Kunkin demon viereen
settings-export-hint = Modin kansiossa, jossa on liblist.gam, kuten Half-Life\ag, klipit toistuvat yhdellä playdemo-komennolla.

clips-store-moved = Klippitiedostoa { $path } ei voitu lukea, joten se säilytettiin nimellä { $moved } ja klipit alkavat tyhjästä. { $detail }
clips-store-unreadable = Klippitiedostoa { $path } ei voitu lukea: { $detail }. Klippejä ei tallenneta tämän istunnon aikana, joten mitään sen sisällöstä ei katoa.
clips-save-failed = Klippejä ei voitu tallentaa kohteeseen { $path }: { $detail }
notice-dismiss = Sulje

# Suomi. Tarkistamaton luonnos

app-title = Lambda Slicer

## Työkalurivi

toolbar-open = Avaa demo…
toolbar-add-folder = Lisää kansio…
toolbar-rescan = Skannaa uudelleen
toolbar-cancel = Peruuta
toolbar-settings = Asetukset
toolbar-rescan-busy = Skannaus on jo käynnissä.
toolbar-rescan-nothing = Lisää kansio tai avaa demo skannattavaksi.
toolbar-cancel-idle = Skannaus ei ole käynnissä.
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
library-demos-of = { $shown } / { $total ->
    [one] { $total } demo
   *[other] { $total } demoa
}
library-total = { $count ->
    [one] Kirjastossa { $count } demo
   *[other] Kirjastossa { $count } demoa
}
filters-title = Suodattimet
filters-find = Etsi
filters-pov = Näkökulma
filters-quality = Laatu
filters-recorded = Tallennettu
pov-any = Kaikki
pov-playing = Pelaten
pov-spectating = Katsojana
help-pov = ”Pelaten” tarkoittaa, että tallenne oli katsojana enintään puolet toistostaan, ja ”Katsojana” yli puolet.
columns-button = Sarakkeet
list-back = Takaisin kirjaston taulukkoon
sort-by = Lajittele
sort-ascending = Nouseva
sort-descending = Laskeva
score-more-sides = { $count ->
    [one] { $count } puoli lisää
   *[other] { $count } puolta lisää
}
intensity-help = Kiivaus { $score }. Palkki vertaa sitä kirjaston suurimpaan, { $max }.
value-unknown = Ei tiedossa
path-copy = Kopioi polku
path-show = Näytä kansiossa
path-show-failed = Kansiota ei voitu avata ({ $detail }). Kopioi polku sen sijaan.

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
show-shots = Laukaukset
show-actions = Muut toiminnot
show-kills = Tapot
show-switches = Aseen vaihdot
lane-unknown = Tuntematon pelaaja
lane-recorder = { $name } (tallentaja)
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
timeline-window = { $rank }. { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Lopputulokset viimeisen tapon kohdalla { $time }
scores-at = Tilanne kohdassa { $time }
scores-show-final = Näytä lopputulokset
scores-frags = Fragit
scores-deaths = Kuolemat

viewer-play = Toista
viewer-pause = Tauko
viewer-volume = Äänenvoimakkuus
viewer-no-sound = Ei ääntä
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
timeline-title = Aikajana
timeline-lanes = Pelaajarivit
timeline-zoom-in = Lähennä
timeline-zoom-out = Loitonna
timeline-fit-all = Sovita koko demo
timeline-gestures = Napsauta merkkiä valitaksesi sen tapahtuman tai mitä tahansa muuta kohtaa siirtyäksesi sinne. Vedä viivainta tai toistokohtaa selataksesi, ja vedä muualta siirtääksesi näkymää. Vieritä zoomataksesi osoittimen ympäriltä ja kaksoisnapsauta nähdäksesi koko demon. Vedä klipin reunaa siirtääksesi sitä tai napsauta klippiä muokataksesi sitä.
timeline-density = Varjostetut palkit näyttävät, kuinka moni näytetyistä tapahtumista osuu kuhunkin jaksoon, suhteessa demon vilkkaimpaan jaksoon.
timeline-density-count = { $count ->
    [one] Yksi näytetty tapahtuma tässä
   *[other] { $count } näytettyä tapahtumaa tässä
}
timeline-session-count = Istunto { $session }: { $count }
timeline-choose = Tapahtumat tässä kohdassa
legend-clip = Klippi
event-session = Istunto { $session }
event-unreachable = Tämä tapahtuma on myöhemmässä istunnossa, joka käyttää samoja aikoja uudelleen, joten katselin ei voi siirtyä siihen.
event-hidden = Suodattimet piilottavat valitun tapahtuman.
highlight-jump = Siirry kohokohtaan…
highlight-help = Kohokohdat järjestävät koko demon toiminnan mukaan riippumatta siitä, mitä suodattimet näyttävät.
kills-with = Rajaa tappoja
refine-midair = Midair
refine-speed = Nopeus
refine-wallgauss = Wallgauss
refine-clear = Tyhjennä
refine-airborne = Ilmassa
refine-either = Kumpi vain
midair-both = Molemmat
midair-one = Vain toinen
help-midair-either = Tappaja, uhri tai molemmat olivat ilmassa. Tappaja tarkistetaan tappoon liitetyn laukauksen kohdalla, tai tapon kohdalla, jos sellaista ei ole, ja uhri kuoleman hetkellä.
help-midair-both = Tappaja ja uhri olivat molemmat ilmassa, kumpikin omalla tarkistushetkellään.
help-midair-one = Täsmälleen toinen heistä oli ilmassa, ja toisen tiedetään olleen maassa.
speed-at-least = Vähintään
speed-unit = ups
speed-killer = Tappaja
speed-victim = Uhri
help-speed = Vaakasuora nopeus yksikköinä sekunnissa, maan suuntaisesti. Tappaja mitataan tappoon liitetyn laukauksen kohdalla, tai tapon kohdalla, jos sellaista ei ole, ja uhri kuoleman hetkellä. Muiden pelaajien nopeudet ovat arvioita.
wallgauss-weapon = Wallgauss koskee vain gauss-tappoja, ja asesuodattimessa on { $weapon }.
wallgauss-checking = Wallgauss-tarkistus… { $examined } / { $total } tappoa
wallgauss-needs-files = Wallgauss tarvitsee pelitiedostot jäljittääkseen laukaukset kartan läpi.
wallgauss-failed = Wallgauss-tarkistus epäonnistui. { $reason }
help-wallgauss = Seinän läpi tehdyt gauss-tapot, selvitetty jäljittämällä jokainen laukaus kartan läpi samoin kuin palvelin. Tappo, jota ei voi ratkaista, esimerkiksi jos uhri oli tallenteen näkymän ulkopuolella, lasketaan tuntemattomaksi ja jätetään pois.
badge-wall = WALL
wall-yes-exit = Wallgauss tunnistettu rekonstruoimalla: räjähdys kohdassa, jossa säde tuli ulos { $thickness } yksikön paksuisesta seinästä.
wall-yes-beyond = Wallgauss tunnistettu rekonstruoimalla: säde sen jälkeen, kun se oli mennyt { $thickness } yksikön paksuisen seinän läpi.
wall-limits = Rekonstruoitu laukauksesta ja tilannekuvasta, ei palvelimen tallentama. Pelaajat jäljitetään laatikkoina, ja uhri saattoi olla muutaman kymmenen yksikön päässä siitä, missä tilannekuva hänet näyttää.
wall-no-direct = Ei Wallgauss: säde olisi voinut osua uhriin ennen yhtään seinää.
wall-no-reflection = Ei Wallgauss: heijastuksen räjähdys olisi voinut osua uhriin ennen yhtään seinää.
wall-doubt-no-shot = Wallgauss ei tiedossa: tappajan laukaus ei ole tallenteessa.
wall-doubt-no-map = Wallgauss ei tiedossa: kartan tiedostoja ei voitu lukea.
wall-doubt-unseen = Wallgauss ei tiedossa: uhri oli tallenteen näkymän ulkopuolella.
wall-doubt-brush = Wallgauss ei tiedossa: säde osui oveen, tasoon tai muuhun brush-entiteettiin, joita ei jäljitetä.
wall-doubt-ambiguous = Wallgauss ei tiedossa: laukaus olisi voinut tappaa sekä seinän läpi että ilman.
wall-doubt-unexplained = Wallgauss ei tiedossa: ei löytynyt tapaa, jolla laukaus olisi päässyt uhriin.
wall-doubt-unconfirmed = Wallgauss ei tiedossa: laukaus olisi voinut päästä uhriin vain seinän läpi, mutta ei varmasti.
wall-doubt-not-reached = Wallgauss ei tiedossa: tallenne loppuu ennen laukausta.
wall-pending = Wallgauss: ei vielä tarkistettu.
events-unknown = { $count ->
    [one] Yhden tapon tiedot eivät riitä ratkaisuun
   *[other] { $count } tapon tiedot eivät riitä ratkaisuun
}
help-unknown = Liike, jota tallenne ei nähnyt, lasketaan tuntemattomaksi, ei koskaan maassa olemiseksi tai paikallaan seisomiseksi, joten nämä tapot jätetään pois.
clip-it = Klippaa
badge-headshot = PL
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = itsemurha
badge-teamkill = tiimitappo
badge-world = maailma
air-both = Molemmat olivat ilmassa tarkistushetkellä.
air-killer = Tappaja oli ilmassa tarkistushetkellä.
air-victim = Uhri oli ilmassa tarkistushetkellä.
air-other-unknown = Toisesta ei tiedetä.
speed-horizontal = Vaakasuora nopeus, maan suuntaisesti.
motion-line = { $who }: { $air }, { $speed }. { $how }, { $when }.
motion-who-killer = Tappaja
motion-who-victim = Uhri
motion-air-yes = ilmassa
motion-air-no = maassa
motion-air-unknown = ei tiedetä, oliko ilmassa
motion-speed-unknown = nopeus tuntematon
motion-exact = Pelin tallentama
motion-trajectory = Arvioitu liikkeestä
motion-unclear = Liike epäselvä
motion-not-visible = Poissa näkyvistä
motion-at-shot = liitetyn laukauksen kohdalla
motion-at-kill = kuoleman hetkellä
motion-none = Analyysissä ei ole liikettä tälle tapolle.
legend-shot = Laukaus
legend-charge = Lataus
legend-kill = Tappo
legend-death = Kuolema
legend-switch = Aseen vaihto

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

clips-new = Uusi klippi
clips-select-event = Valitse ensin tapahtuma luettelosta tai aikajanalta.
clips-other-session = Klippi ei voi alkaa tästä, koska tallenteen aiemmalla istunnolla on samat ajat.
clips-add-window = Klippi tästä kiivaasta hetkestä
clip-default-name = Klippi { $number }
clip-name = Nimi
clip-length = Pituus { $duration }
clip-remove = Poista
clip-loop = Silmukka
clip-time-help = Sekunteina (96), minuutteina ja sekunteina (1:36) tai tunteina, minuutteina ja sekunteina (00:01:36.250)
clips-empty-hint = Klippejä ei vielä ole. Valitse tapahtuma ja paina Klippaa, tai tee klippi toistokohtaan painikkeella Uusi klippi.
clips-new-help = Kymmenen sekunnin klippi toistokohdasta alkaen.
clip-in = Alku
clip-out = Loppu
clip-set-in = Aseta alku toistokohtaan
clip-set-out = Aseta loppu toistokohtaan
clip-no-playhead = Ei toistokohtaa, josta ajan voisi ottaa.
clip-play = Toista klippi kerran
clip-show-on-timeline = Näytä aikajanalla
export-destination = Vaihda vientikansio
export-skipped = Ohitettu. { $reason }
export-earlier-range = Tämä vienti kattaa välin { $start } – { $end }, eli klipin välin ennen muokkausta.
scores-player = Pelaaja
scores-none = Tallenteessa ei ole pisteitä.
clip-effective = Vienti leikkaa { $start } – { $end } ({ $duration }) tallenteen ruutujen mukaan.
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
settings-viewer-game-none = Ei valittu
settings-viewer-game-hint = Kansio, jossa valve on. Katselin toistaa demot pelin kartoilla, malleilla ja äänillä tästä kansiosta.
settings-executable = Pelin ohjelmatiedosto
settings-executable-hint = Löytää Half-Life-kansion. Katselin lukee pelin tiedostot itse eikä koskaan käynnistä peliä.
settings-executable-not-set = Ei asetettu, joten katselimessa ei ole kuvaa eikä ääntä
settings-executable-optional = Ei asetettu, eikä tarvita, kun Half-Life-kansiossa on peli
settings-executable-missing = { $path } ei ole enää paikallaan. Katselin lukee yhä Half-Life-kansiota.
settings-game-override = Tämä ajo käyttää polkua { $path }, joka annettiin valitsimella --game. Täällä tehty valinta pätee seuraavasta ajosta.
settings-browse = Selaa…
dialog-executables = Ohjelmat
setup-title = Ota demokatselin käyttöön
setup-body = Kuva ja ääni tarvitsevat Half-Lifen pelitiedostot. Näytä Lambda Slicerille, mihin peli on asennettu, niin se lukee ne sieltä.
setup-locate-exe = Etsi hl.exe…
setup-locate-folder = Etsi Half-Life…
setup-settings = Asetukset
setup-still-works = Aikajana, tapahtumat ja klipit toimivat ilmankin.
setup-relocate = Kansiossa { $folder } ei ole enää Half-Lifea. Etsi se uudelleen.
setup-not-executable = { $name } ei ole { $executable }. Valitse pelin oma ohjelmatiedosto, ei pikakuvaketta.
setup-not-installed = Kansiossa { $folder } ei ole Half-Life-asennusta, jossa on valve-kansio.
tab-events = Tapahtumat
tab-clips = Klipit
tab-scoreboard = Pisteet
list-hide = Piilota demoluettelo
list-show = Näytä demoluettelo
demo-details = Tiedot
viewer-start = Takaisin alkuun
viewer-mute = Mykistä
viewer-unmute = Poista mykistys
viewer-picture = Kuvan ja katselimen asetukset
viewer-needs-files = Tarvitsee pelin tiedostot
viewer-no-sound-why = Ei ääntä: { $reason }
settings-tab-general = Yleiset
settings-section-folders = Kansiot
settings-folders-hint = Kansiot, joista kirjasto etsii demoja alikansioineen.
settings-clip-padding-hint = Kuinka paljon tapahtuman ympärille tehty klippi säilyttää ennen ja jälkeen sen.
settings-section-picture = Kuva
settings-section-sound = Ääni
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

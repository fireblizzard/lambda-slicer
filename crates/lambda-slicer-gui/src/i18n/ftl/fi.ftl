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
settings-workers = Yhtä aikaa analysoitavat demot
settings-workers-hint = Useamman demon analysointi yhtä aikaa valmistuu nopeammin mutta kuormittaa tietokonetta enemmän.
settings-cache = Kirjaston tallennuspaikka
settings-sort = Järjestys käynnistettäessä
settings-sort-hint = Miten demot lajitellaan aina, kun Lambda Slicer käynnistyy. Sarakkeen otsikon napsauttaminen lajittelee silti sen mukaan, kunnes suljet Lambda Slicerin.
settings-start-filters = Suodattimet käynnistettäessä
settings-start-filters-hint = Kirjasto käyttää näitä suodattimia aina, kun Lambda Slicer käynnistyy, joten lyhyet demot, joita et koskaan halua nähdä, pysyvät poissa tieltä. Voit silti muuttaa suodattimia, kunnes suljet Lambda Slicerin.
settings-hide-unplayed-maps = Jätä usean kartan demoista pois kartat, joilla ei ole tappoja ja joita nauhoitettiin alle 5 minuuttia
settings-folders = Kirjaston kansiot
settings-folders-empty = Kansioita ei ole vielä lisätty.
settings-remove = Poista

## Kirjasto

library-empty = Aloita lisäämällä kansio tai avaamalla demo.
library-no-matches = Mikään demo ei vastaa näitä suodattimia.
library-no-cache = Tällä tietokoneella ei ole paikkaa, johon kirjaston voisi tallentaa.
library-demos = { $count ->
    [one] { $count } demo
   *[other] { $count } demoa
}
library-percent = { $percent } %
library-demos-of = { $shown } / { $total ->
    [one] { $total } demo
   *[other] { $total } demoa
}
library-matches = { $count ->
    [one] { $count } ottelu
   *[other] { $count } ottelua
}
library-matches-of = { $shown } / { $total ->
    [one] { $total } ottelu
   *[other] { $total } ottelua
}
library-no-games = Missään demossa ei ole ottelua eikä viittä minuuttia pelaamista ilman ottelua.
library-no-games-filtered = Mikään ottelu ei vastaa näitä suodattimia.
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
path-copy = Kopioi tiedoston sijainti
path-show = Näytä kansiossa
path-show-failed = Kansiota ei voitu avata ({ $detail }). Kopioi tiedoston sijainti sen sijaan.

filter-search = Hae tiedostojen ja kansioiden nimistä
filter-map = Kartta
filter-player = Pelaaja
filter-weapon-any = Mikä tahansa ase
filter-min-minutes = Vähintään (minuuttia)
filter-min-kills = Vähintään (tappoa)
filter-players = Pelaajat
filter-teams = Joukkueet
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
kind-hltv = HLTV
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

scan-progress = Löydetty { $found }, valmiina { $done } (jo tuttuja { $cached }, analysoitu { $analysed }), luettu { $megabytes } Mt
scan-cancelling = Pysäytetään…
scan-cancelled = Pysäytetty, skannaamatta { $left }.
scan-finished = Valmis: { $done ->
    [one] { $done } demo
   *[other] { $done } demoa
}, analysoitu { $analysed }, aikaa { $seconds } s.
scan-problems = { $count ->
    [one] { $count } ongelma
   *[other] { $count } ongelmaa
}
problems-title = Skannauksen ongelmat
problem-changed = Tiedosto muuttui analyysin aikana, joten mitään ei tallennettu.
drop-hint = Pudota demoja tai kansioita lisätäksesi ne
drop-not-demo = Ei demo: { $names }

## Analyysiytimen viestit koodeittain

analysis-panicked = Analyysi pysähtyi, koska Lambda Slicerissa tapahtui vika: { $detail }
cache-cancelled = Analyysi peruttiin, joten mitään ei tallennettu.
cache-database = Tallennettua kirjastoa ei voi käyttää: { $detail }
cache-directory = Tallennetun kirjaston kansiota ei voi luoda: { $detail }
container-bad-magic = Tämä ei ole Half-Life-demo.
container-frame-bad-time = Tallenne on vioittunut osan { $entry } ruudussa { $frame }, jonka aika on rikki.
container-frame-missing-terminal = Tallenteen osa { $entry } ei pääty kunnolla.
container-frame-negative-length = Tallenne on vioittunut osan { $entry } ruudussa { $frame }.
container-frame-over-limit = Osan { $entry } ruutu { $frame } on sallittua suurempi.
container-frame-truncated = Tallenne katkeaa osan { $entry } ruutuun { $frame }.
container-frame-unknown-tag = Tallenne on vioittunut osan { $entry } ruudussa { $frame }, jossa on jotain, mitä Lambda Slicer ei tunnista.
container-io = Tiedostoa ei voi lukea: { $detail }
container-no-directory = Tallennetta ei koskaan viety loppuun, joten siinä ei ole sisällysluetteloa.
container-range = Tallenteen sisällysluettelo osoittaa tiedoston ulkopuolelle.
container-too-short = Tiedosto on liian lyhyt ollakseen demo.
coverage-hltv = HLTV-tallenne. Mikään pelaaja ei tallentanut sitä, joten tapahtumat seuraavat sitä, mitä HLTV katsoi.
coverage-mostly-spectating = Tallentaja oli katsojana { $percent } % toistosta, joten useimmat tapahtumat ovat muiden pelaajien, sellaisina kuin ne näkyivät heitä katsoessa.
coverage-no-local-events = Tallentajan omia toimintoja ei tallennettu tähän demoon, joten vain palvelimen lähettämät näkyvät.
coverage-singleplayer = Yksinpeli. Kaikki muu on hirviöitä tai kartan osia, eikä killfeediä ole.
coverage-unknown = Tallennetta ei saatu luettua niin pitkälle, että sen laji selviäisi.
coverage-visible-only = Peli saa muiden pelaajien toiminnot vain silloin, kun palvelin katsoo tallentajan näkevän heidät, joten heidän määränsä voivat olla pienempiä kuin mitä oikeasti tapahtui.
decode-failed = Tallenteen lukeminen pysähtyi kohdassa { $time } (ruutu { $frame }): { $detail }
export-active-effect = { $effect } on vielä käynnissä alussa, eikä klippi voi vielä ottaa sitä mukaan.
export-client-dropped-packets = Kohdassa { $time } peli heitti pois liian myöhään saapuneita tietoja, mitä klippi ei vielä osaa kopioida.
export-cross-segment = Kartta vaihtui tai pelaaja yhdisti uudelleen kohdassa { $time } klipin sisällä, eikä klippi voi jatkua sen yli.
export-destination-exists = { $path } on jo olemassa. Valitse toinen nimi.
export-empty-range = Klipissä ei ole mitään vietävää.
export-io = Kohdetta { $path } ei voi kirjoittaa: { $detail }
export-missing-delta-base = Tallenteesta puuttuu alkua edeltäviä tietoja, joita klippi tarvitsee (numero { $sequence }).
export-missing-parent = Kansiota { $path } ei ole.
export-range-beyond-end = Loppu { $end } on tallenteen lopun ({ $segment_end }) jälkeen.
export-same-as-source = Vienti korvaisi demon, josta se on otettu.
export-source-changed = { $path } muuttui viennin aikana.
export-source-decode-failed = Demoa ei voi lukea: { $detail }
export-source-failed-crosscheck = Tarkistuksessa se, mitä Lambda Slicer päätteli tästä demosta, ei vastannut itse demoa, joten sen klippeihin ei voi luottaa.
export-unknown-persistent-message = Viestiä { $name } ei vielä tueta klipeissä.
export-unproven-state = Kohdassa { $time } tallenne nojaa tietoihin, jotka piti arvata, joten klippiä ei voi tehdä tarkasti.
export-unrepresentable-time-field = Alun aikaa ei voi tallentaa tarkasti: { $detail }
export-unsupported = Ei tuettu: { $detail }
export-validation-failed = Viety klippi ei läpäissyt lopputarkistusta: { $detail }
io-not-found = Tiedostoa tai kansiota ei löytynyt.
io-other = { $detail }
io-permission-denied = Pääsy estetty.
io-unexpected-end = Tiedosto päättyi odottamatta.
protocol-unsupported = Tämä demo on pelin versiosta, jota ei tueta (protokolla { $demo }/{ $network }). Vain 5/47 ja 5/48 ovat tuettuja.
protocol-unsupported-network = Palvelin vaihtoi pelin versioon, jota ei tueta (protokolla { $network }).

## Demonäkymä

demo-close = Sulje
demo-loading = Analysoidaan { $name }…
show-shots = Laukaukset
show-actions = Muut toiminnot
show-kills = Tapot
show-switches = Aseen vaihdot
show-chat = Chatti
show-match = Ottelu
lane-unknown = Tuntematon pelaaja
lane-recorder = { $name } (tallentaja)
event-kill = Tappoi pelaajan { $victim }
event-switch = Aseen vaihto
event-copy = Kopioi
event-copy-kill = { $killer } tappoi pelaajan { $victim } aseella { $weapon }
event-copy-teamkill = { $killer } tappoi joukkuetoverinsa { $victim } aseella { $weapon }
event-copy-suicide = { $victim } tappoi itsensä aseella { $weapon }
event-copy-death = { $victim } kuoli, syynä { $weapon }
event-vote-called = Äänestys aloitettu: { $subject }
event-vote-cast = Äänestys { $subject }: { $change } ({ $tally })
event-vote-more-yes = { $count ->
    [one] yksi kyllä lisää
   *[other] { $count } kyllä lisää
}
event-vote-fewer-yes = { $count ->
    [one] yksi kyllä vähemmän
   *[other] { $count } kyllä vähemmän
}
event-vote-more-no = { $count ->
    [one] yksi ei lisää
   *[other] { $count } ei lisää
}
event-vote-fewer-no = { $count ->
    [one] yksi ei vähemmän
   *[other] { $count } ei vähemmän
}
event-vote-tally = { $yes } kyllä, { $no } ei, { $undecided } äänestämättä
event-vote-accepted = Äänestys { $subject } hyväksytty
event-vote-denied = Äänestys { $subject } hylätty
event-timeout-called = Aikalisä pyydetty
event-timeout-under-way = Aikalisä alkoi
event-timeout-over = Aikalisä päättyi
event-paused = Peli tauolla
event-resumed = Peli jatkuu
event-sudden-death = Sudden death alkoi
event-sudden-death-over = Sudden death päättyi
event-map-changed = Kartta vaihdettu: { $map }
event-spectating = Siirtyi katsojaksi
event-playing = Palasi pelaamaan
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
column-file-date = Tiedoston päiväys
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
viewer-position = Kohta demossa
viewer-skip-back = 10 sekuntia taaksepäin
viewer-skip-forward = 10 sekuntia eteenpäin
viewer-rate = { $rate }×
viewer-shortcut = Pikanäppäin: { $key }
key-space = Välilyönti
key-shift-left = Shift+Vasen
key-shift-right = Shift+Oikea
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
timeline-gestures = Napsauta merkkiä valitaksesi sen tapahtuman tai mitä tahansa muuta kohtaa siirtyäksesi sinne. Vedä yläreunan aikoja tai nykyistä hetkeä merkitsevää viivaa liikkuaksesi demossa. Vedä mistä tahansa muualta siirtääksesi näkymää. Vieritä zoomataksesi sisään ja ulos hiiren kohdalta ja kaksoisnapsauta nähdäksesi koko demon. Vedä klipin reunaa muuttaaksesi, mistä se alkaa tai mihin se päättyy, tai napsauta klippiä muokataksesi sitä.
timeline-density = Varjostetut palkit näyttävät, kuinka moni näytetyistä tapahtumista osuu kuhunkin kohtaan, verrattuna demon vilkkaimpaan kohtaan.
timeline-density-count = { $count ->
    [one] Yksi näytetty tapahtuma tässä
   *[other] { $count } näytettyä tapahtumaa tässä
}
timeline-session-count = Osa { $session }: { $count }
timeline-choose = Tapahtumat tässä kohdassa
legend-clip = Klippi
event-session = Osa { $session }
event-unreachable = Tämä tapahtuma tulee kartanvaihdon tai uudelleenyhdistämisen jälkeen, kun demon kello alkoi alusta, joten katselin ei voi siirtyä siihen.
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
help-midair-either = Tappaja, uhri tai molemmat olivat ilmassa. Tappaja tarkistetaan tappoon liitetyn laukauksen kohdalla, tai tapon kohdalla, jos sellaista ei ole, ja uhri kuoleman hetkellä. Pelkkää hyppyä alle 50 yksikön korkeudella maasta ei lasketa. Tähän tarvitaan valittu Half-Life-kansio, jotta tiedetään, missä maa on.
help-midair-both = Tappaja ja uhri olivat molemmat ilmassa, kumpikin omalla tarkistushetkellään.
help-midair-one = Täsmälleen toinen heistä oli ilmassa, ja toisen tiedetään olleen maassa.
speed-at-least = Vähintään
speed-unit = ups
speed-killer = Tappaja
speed-victim = Uhri
help-speed = Nopeus maan suuntaisesti ups-yksikköinä, kuten nopeusmittari sen näyttää, ilman liikettä ylös ja alas. Tappaja mitataan tappoon liitetyn laukauksen kohdalla, tai tapon kohdalla, jos sellaista ei ole, ja uhri kuoleman hetkellä. Muiden pelaajien nopeudet ovat arvioita.
wallgauss-weapon = Wallgauss koskee vain gauss-tappoja, ja asesuodattimessa on { $weapon }.
wallgauss-checking = Wallgauss-tarkistus… { $examined } / { $total } tappoa
wallgauss-needs-files = Wallgauss tarvitsee pelitiedostot tietääkseen, missä kartan seinät ovat.
wallgauss-failed = Wallgauss-tarkistus epäonnistui. { $reason }
help-wallgauss = Seinän läpi tehdyt gauss-tapot, löydetty seuraamalla jokaista laukausta kartan läpi samalla tavalla kuin palvelin. Tappo, jota ei voi ratkaista, esimerkiksi jos uhri oli tallenteen näkymän ulkopuolella, lasketaan tuntemattomaksi ja jätetään pois.
badge-wall = WG
wall-yes-exit = Wallgauss, päätelty laukauksesta: räjähdys kohdassa, jossa säde tuli ulos { $thickness } yksikön paksuisesta seinästä.
wall-yes-beyond = Wallgauss, päätelty laukauksesta: säde osui mentyään { $thickness } yksikön paksuisen seinän läpi.
wall-limits = Päätelty laukauksesta ja pelaajien sijainneista, koska palvelin ei tallenna sitä. Pelaaja on 32 yksikköä leveä ja lasketaan laatikoksi, ja uhri saattoi olla vartalon leveyden tai kaksi sivussa siitä, missä demo hänet näyttää.
wall-no-direct = Ei Wallgauss: säde olisi voinut osua uhriin ennen yhtään seinää.
wall-no-reflection = Ei Wallgauss: heijastuksen räjähdys olisi voinut osua uhriin ennen yhtään seinää.
wall-doubt-no-shot = Wallgauss ei tiedossa: tappajan laukaus ei ole tallenteessa.
wall-doubt-no-map = Wallgauss ei tiedossa: kartan tiedostoja ei voitu lukea.
wall-doubt-unseen = Wallgauss ei tiedossa: uhri oli tallenteen näkymän ulkopuolella.
wall-doubt-brush = Wallgauss ei tiedossa: säde osui oveen, hissiin tai muuhun kartan erikoisosaan, jota ei tarkisteta.
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
events-follow = Pidä luettelo nykyisessä hetkessä, kun demo toistuu. Luettelon vierittäminen poistaa tämän käytöstä.
clip-it = Klippaa
badge-headshot = PL
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = itsemurha
badge-teamkill = tiimitappo
badge-world = maailma
badge-clip = klippi
clip-of = Klippi demosta { $name }
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
filter-midair = Midair-tapot
event-headshot = päälaukaus
scores-headshots = PL

column-accuracy = Tarkkuus
column-players = Pelaajat
column-teams = Joukkueet
help-players-column = Pelissä olevat pelaajat, ei katsojia, laskettuna silloin, kun ottelu alkoi lähtölaskentansa jälkeen. Joukkuepelissä lasketaan vain joukkueessa olevat pelaajat. Demo näyttää suurimman määrän missä tahansa kokonaisessa ottelussa, ja demo, jossa ei ole kokonaista ottelua, suurimman määrän millä tahansa hetkellä.
help-teams-column = Joukkueet, joissa on pelaaja, laskettuna samoin kuin pelaajat. Pelissä ilman joukkueita niitä ei ole.
filter-min-accuracy = Tarkkuus vähintään (%)
scores-accuracy = Tark.
scores-kd = K/D
help-kd = Fragit kuolemaa kohden eli fragit jaettuna kuolemilla. Jos pelaaja ei kuollut kertaakaan, näytetään hänen fragiensa määrä.

help-headshots = Tapot osumalla päähän, tunnistettu uhrin kuolinanimaatiosta. Räjähdykset eivät lasketa. Vain tallentajan tapot näytetään, koska muut pelaajat tappavat usein tallennuksen näkymän ulkopuolella. Kaksintaistelussa näytetään myös vastustajan tapot, koska ne kaikki kohdistuvat tallentajaan.
help-headshots-column = Tallentajan tapot osumalla päähän, tunnistettu uhrin kuolinanimaatiosta. Räjähdykset eivät lasketa. Muiden pelaajien tappoja ei lasketa, koska ne tapahtuvat usein tallennuksen näkymän ulkopuolella.
help-date-column = Milloin ottelu tallennettiin. Päiväys luetaan tiedoston nimestä, kun nimessä on päiväys kuten 20230910_214553, niin kuin pelin itse tekemissä tallenteissa. Muuten se on tiedoston päiväys, jonka kopiointi tai lataus voi muuttaa.
help-file-date = Milloin tiedosto tallennettiin. Viedyn klipin kohdalla se on hetki, jolloin klippi vietiin.
help-accuracy = Osuneiden laukausten osuus, päätelty verestä, jonka osuma jättää tähtäyslinjalle, vähennettynä sattumanvaraisilla osumilla. Mukana ovat glock, python, MP5:n luodit, haulikko, tähtäimellä ammuttu varsijousi ja gauss-säteen suorat osumat. Räjähdykset ja gaussin heijastukset eivät jätä verta eivätkä lasketa. Näytetään vain tallentajalle ja vasta vähintään 50 laukauksen jälkeen.
help-headshot-filter = Demot, joissa on vähintään yksi tallennuksessa näkyvä päälaukaustappo, kenen tahansa.
help-midair-filter = Demot, joissa on vähintään yksi tappo, jonka hetkellä tappaja tai uhri oli ilmassa vähintään 50 yksikön korkeudella maasta. Ilman valittua Half-Life-kansiota myös pelkkä hyppy lasketaan.
help-accuracy-filter = Demot, joissa tallentaja osui vähintään tähän osuuteen laukauksistaan.

clips-new = Uusi klippi
clips-select-event = Valitse ensin tapahtuma luettelosta tai aikajanalta.
clips-other-session = Klippi ei voi alkaa tästä, koska demon kello alkoi alusta kartanvaihdon tai uudelleenyhdistämisen jälkeen, ja aiemmalla osalla on samat ajat.
clips-add-window = Klippi tästä kiivaasta hetkestä
clip-default-name = Klippi { $number }
clip-name = Nimi
clip-length = Pituus { $duration }
clip-remove = Poista
clip-loop = Silmukka
clip-time-help = Sekunteina (96), minuutteina ja sekunteina (1:36) tai tunteina, minuutteina ja sekunteina (00:01:36.250)
clips-empty-hint = Klippejä ei vielä ole. Valitse tapahtuma ja paina Klippaa, tai paina Uusi klippi aloittaaksesi klipin nykyisestä hetkestä.
clips-new-help = Kymmenen sekunnin klippi nykyisestä hetkestä alkaen.
clip-in = Alku
clip-out = Loppu
clip-set-in = Aloita klippi nykyisestä hetkestä
clip-set-out = Lopeta klippi nykyiseen hetkeen
clip-no-playhead = Katselin ei ole käytettävissä, joten nykyistä hetkeä ei voi käyttää.
clip-play = Toista klippi kerran
clip-show-on-timeline = Näytä aikajanalla
export-destination = Vaihda vientikansio
export-skipped = Ohitettu. { $reason }
export-earlier-range = Tämä vienti kattaa välin { $start } – { $end }, eli klipin sellaisena kuin se oli ennen kuin muutit sitä.
scores-player = Pelaaja
scores-none = Tallenteessa ei ole pisteitä.
clip-effective = Vienti kattaa { $start } – { $end } ({ $duration }), niin tarkasti kuin tallenne sallii.
settings-clip-padding = Klipin marginaali
settings-preroll = Sekuntia ennen tapahtumaa
settings-postroll = Sekuntia tapahtuman jälkeen
export-start-first-group = Klippi ei voi alkaa tallenteen aivan alusta. Aloita hieman myöhemmin.
export-start-during-loading = Alku osuu kartan lataukseen. Aloita hieman myöhemmin.

export-all = Vie kaikki klipit
export-one = Vie
export-cancel = Pysäytä vienti
export-folder = Viedyt tiedostot menevät kansioon { $folder }.
export-folder-beside = Viedyt tiedostot menevät demon viereen, kansioon { $folder }.
export-waiting = Odottaa vientiä…
export-running = Viedään… Koko demo luetaan, mikä voi kestää hetken.
export-cancelled = Ei viety, koska vienti pysäytettiin.
export-saved = Tallennettu: { $path }. Alkuperäiseen verratut hetket: { $count }.
export-playdemo = Pelin konsolissa:
export-copy-first = Kopioi { $name } modin kansioon, jossa on liblist.gam, ja sitten pelin konsolissa:
export-copy = Kopioi
settings-export-folder = Vientikansio
settings-export-choose = Valitse…
settings-file-names = Tiedostonimet
settings-clip-name-files = Käytä klipin nimeä tiedostonimenä
settings-demo-name-first = Aloita demon nimellä
settings-viewer = Demokatselin
settings-viewer-game = Half-Life-kansio
settings-viewer-game-none = Ei valittu
settings-viewer-game-hint = Kansio, jossa valve on. Katselin toistaa demot pelin kartoilla, malleilla ja äänillä tästä kansiosta.
settings-executable = Pelin ohjelma
settings-executable-hint = Löytää Half-Life-kansion. Katselin lukee pelin tiedostot itse eikä koskaan käynnistä peliä.
settings-executable-not-set = Ei asetettu, joten katselimessa ei ole kuvaa eikä ääntä
settings-executable-optional = Ei asetettu, eikä tarvita, kun Half-Life-kansiossa on peli
settings-executable-missing = { $path } ei ole enää paikallaan. Katselin lukee yhä Half-Life-kansiota.
settings-game-override = Tällä kertaa Lambda Slicer käyttää tiedostoa { $path }, koska se käynnistettiin --game-valinnalla. Täällä tehty valinta pätee seuraavasta käynnistyksestä.
settings-browse = Selaa…
dialog-executables = Ohjelmat
setup-title = Ota demokatselin käyttöön
setup-body = Kuva ja ääni tarvitsevat Half-Lifen pelitiedostot. Näytä Lambda Slicerille, mihin peli on asennettu, niin se lukee ne sieltä.
setup-locate-exe = Etsi hl.exe…
setup-locate-folder = Etsi Half-Life…
setup-settings = Asetukset
setup-still-works = Aikajana, tapahtumat ja klipit toimivat ilmankin.
setup-relocate = Kansiossa { $folder } ei ole enää Half-Lifea. Etsi se uudelleen.
setup-not-executable = { $name } ei ole { $executable }. Valitse pelin oma ohjelma, ei pikakuvaketta.
setup-not-installed = Kansiossa { $folder } ei ole Half-Life-asennusta, jossa on valve-kansio.
tab-events = Tapahtumat
tab-clips = Klipit
tab-scoreboard = Pisteet
tab-demos = Demot
tab-matches = Ottelut
filter-complete = Vain kokonaiset ottelut
help-complete-filter = Vain ottelut, jotka tallennettiin lähtölaskennasta loppuun asti.
help-status-column = Kuinka paljon ottelusta demo tallensi. Osoita rivin tilaa nähdäksesi, mitä se tarkoittaa.
help-match-players = Pelissä olevat pelaajat, ei katsojia, kun ottelu alkoi lähtölaskennan jälkeen, tai ensimmäisen tapon kohdalla, jos ottelu oli jo käynnissä tallennuksen alkaessa. Vapaassa pelissä suurin määrä yhtä aikaa.
help-match-teams = Joukkueet, joissa on pelaaja, laskettuna samoin kuin pelaajat. Pelissä ilman joukkueita niitä ei ole.
match-complete = Kokonainen
match-joined-late = Alkoi kesken
match-interrupted = Keskeytyi
match-cut-off = Katkesi
match-free-play = Vapaa peli
help-match-complete = Tallennettu lähtölaskennasta ottelun loppuun.
help-match-joined-late = Ottelu oli jo käynnissä tallennuksen alkaessa, ja se tallennettiin loppuun asti.
help-match-interrupted = Toinen ottelu alkoi, ottelu peruttiin tai kartta vaihtui ennen kuin ottelu päättyi.
help-match-cut-off = Tallenne päättyi ennen ottelua.
help-match-free-play = Viisi minuuttia tai enemmän pelaamista ilman käynnissä olevaa ottelua.
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
settings-tab-library = Kirjasto
settings-tab-clips = Klipit
settings-section-game = Half-Life
settings-section-view = Näkymä
settings-language-system = Tietokoneen kieli ({ $name })
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
settings-zoom-fov = Näkökenttä zoomatessa
settings-viewmodel = Näytä ase kädessä
settings-crosshair = Näytä tähtäin
settings-chat = Näytä chatti
settings-vitals = Näytä elämä, panssari, ammukset ja pickupit
settings-vitals-size = Elämän, panssarin, ammusten ja pickupien koko
settings-free-vitals = Näytä elämä, panssari, ammukset ja pickupit myös vapaalla kameralla
settings-scores = Näytä pisteet vasemmassa yläkulmassa
settings-score-rows = Pisterivejä enintään
settings-server-settings = Näytä serverin asetukset oikeassa yläkulmassa
settings-hud-messages = Näytä serverin tekstit, kuten bhop- ja KZ-ajastimet
settings-see-through = Näe pelaajat seinien läpi
settings-see-through-items = Myös muut mallit, kuten ammukset ja esineet
settings-see-through-hint = Seinien takana olevat pelaajat näkyvät siluetteina yläosan värissään, jonka joukkuepelit asettavat joukkueen mukaan. Näkyä voi vain se, mitä demo sisältää. Palvelin lähettää tallentajan lähellä olevat pelaajat, eivätkä palvelimet, jotka piilottavat pelaajat, joita tallentaja ei näe, jätä mitään näytettävää.
settings-section-decals = Decalit
settings-decals = Enintään
settings-decals-hint = Luodinreiät, veri ja muut pelin aikana jääneet jäljet. Vain uusimmat jäävät, ja vanhimmat katoavat, kun uusia syntyy. Kartan omat jäljet jäävät aina.
settings-restore = Palauta oletukset…
settings-restore-title = Palauta oletukset
settings-restore-list = Nämä asetukset palaavat sellaisiksi kuin ne olivat alussa. Half-Life-kansio ja kirjaston kansiot pysyvät ennallaan.
settings-restore-none = Kaikki asetukset ovat jo oletuksissaan.
settings-restore-columns = Kirjastossa näkyvät sarakkeet
settings-restore-layout = Avoimet paneelit ja niiden koot
settings-clear-library = Tyhjennä kirjasto…
settings-clear-library-title = Tyhjennä kirjasto
settings-clear-library-text = { $count ->
    [one] Lambda Slicer unohtaa, mitä se löysi { $count } demosta, ja skannaa kirjaston kansiot uudelleen, mikä voi kestää hetken. Demotiedostosi, klippisi ja asetuksesi pysyvät ennallaan.
   *[other] Lambda Slicer unohtaa, mitä se löysi { $count } demosta, ja skannaa kirjaston kansiot uudelleen, mikä voi kestää hetken. Demotiedostosi, klippisi ja asetuksesi pysyvät ennallaan.
}
viewer-zoom = Zoomaa lähemmäs
viewer-free-camera = Vapaa kamera
viewer-free-camera-help = Lennä vapaasti ympäri karttaa nauhoittaneesta pelaajasta riippumatta. W, A, S ja D liikuttavat, välilyönti nostaa ylös, Ctrl laskee alas ja Shift nopeuttaa. Pidä hiiren oikeaa painiketta pohjassa ja vedä katsoaksesi ympärillesi.
viewer-fullscreen = Koko näyttö
viewer-leave-fullscreen = Poistu koko näytön tilasta
viewer-see-through = Näe seinien läpi
viewer-fullbright = Valaise kaikki
viewer-fullbright-help = Näyttää koko kartan täysin valaistuna, joten pimeät nurkat ja niissä olevat pelaajat näkyvät helposti.
viewer-paused = Tauko
viewer-map-change = Serveri vaihtaa kartaksi { $map }
settings-export-beside = Kunkin demon viereen
settings-export-hint = Modin kansiossa, jossa on liblist.gam, kuten Half-Life\ag, klipit toistuvat yhdellä playdemo-komennolla.
settings-file-names-hint = Viety klippi nimetään demonsa sekä alku- ja loppuaikansa mukaan, ellet käytä klipin nimeä. Peli avaa vain tiedostoja, joiden nimessä on englannin aakkosten kirjaimia, numeroita, - ja ., joten kaikki muu muuttuu merkiksi _. Jos klipin nimessä ei ole yhtään näistä merkeistä, tiedosto nimetään sen sijaan aikojen mukaan.

clips-store-moved = Klippitiedostoa { $path } ei voitu lukea, joten se säilytettiin nimellä { $moved } ja klipit alkavat tyhjästä. { $detail }
clips-store-unreadable = Klippitiedostoa { $path } ei voitu lukea: { $detail }. Klippejä ei tallenneta ennen kuin Lambda Slicer suljetaan, joten mitään tiedoston sisällöstä ei katoa.
clips-save-failed = Klippejä ei voitu tallentaa kohteeseen { $path }: { $detail }
notice-dismiss = Sulje

# Polski. Wersja robocza bez korekty

app-title = Lambda Slicer

## Pasek narzędzi

toolbar-open = Otwórz demo…
toolbar-add-folder = Dodaj folder…
toolbar-rescan = Skanuj ponownie
toolbar-cancel = Anuluj
toolbar-settings = Ustawienia
toolbar-rescan-busy = Skanowanie już trwa.
toolbar-rescan-nothing = Dodaj folder lub otwórz demo, aby skanować.
toolbar-cancel-idle = Żadne skanowanie nie trwa.
dialog-demos = Dema Half-Life

## Ustawienia

settings-title = Ustawienia
settings-language = Język
settings-unreviewed = To tłumaczenie jest wersją roboczą, której nie sprawdził jeszcze żaden rodzimy użytkownik języka.
settings-workers = Ile dem analizować naraz
settings-workers-hint = Analizowanie większej liczby naraz kończy się szybciej, ale bardziej obciąża komputer.
settings-cache = Gdzie jest przechowywana biblioteka
settings-sort = Kolejność przy uruchomieniu
settings-sort-hint = Jak dema są sortowane przy każdym uruchomieniu Lambda Slicer. Kliknięcie nagłówka kolumny nadal sortuje według niej, dopóki nie zamkniesz Lambda Slicer.
settings-folders = Foldery biblioteki
settings-folders-empty = Nie dodano jeszcze folderów.
settings-remove = Usuń

## Biblioteka

library-empty = Dodaj folder lub otwórz demo, aby zacząć.
library-no-matches = Żadne demo nie pasuje do tych filtrów.
library-no-cache = Na tym komputerze nie ma gdzie przechowywać biblioteki.
library-demos = { $count ->
    [one] { $count } demo
    [few] { $count } dema
    [many] { $count } dem
   *[other] { $count } dema
}
library-percent = { $percent }%
library-demos-of = { $shown } z { $total ->
    [one] { $total } dema
    [few] { $total } dem
    [many] { $total } dem
   *[other] { $total } dema
}
library-total = { $count ->
    [one] { $count } demo w bibliotece
    [few] { $count } dema w bibliotece
    [many] { $count } dem w bibliotece
   *[other] { $count } dema w bibliotece
}
filters-title = Filtry
filters-find = Szukaj
filters-pov = Perspektywa
filters-quality = Jakość
filters-recorded = Nagrane
pov-any = Wszystkie
pov-playing = Gra
pov-spectating = Obserwacja
help-pov = „Gra” oznacza, że nagranie było w trybie obserwatora najwyżej przez połowę odtwarzania, a „Obserwacja” – ponad połowę.
columns-button = Kolumny
list-back = Wróć do tabeli biblioteki
sort-by = Sortuj według
sort-ascending = Rosnąco
sort-descending = Malejąco
score-more-sides = { $count ->
    [one] jeszcze { $count } strona
    [few] jeszcze { $count } strony
    [many] jeszcze { $count } stron
   *[other] jeszcze { $count } strony
}
intensity-help = Intensywność { $score }. Pasek porównuje ją z najwyższą w bibliotece, { $max }.
value-unknown = Nieznane
path-copy = Kopiuj lokalizację pliku
path-show = Pokaż w folderze
path-show-failed = Nie udało się otworzyć folderu ({ $detail }). Skopiuj lokalizację pliku.

filter-search = Szukaj w nazwach plików i folderów
filter-map = Mapa
filter-player = Gracz
filter-weapon-any = Dowolna broń
filter-min-minutes = Co najmniej (minuty)
filter-spectating-any = Gra lub obserwacja
filter-spectating-mostly = Głównie obserwacja
filter-spectating-playing = Głównie gra
filter-clear = Wyczyść filtry

column-status = Stan
column-name = Demo
column-map = Mapa
column-duration = Długość
column-gauss = Gauss
column-kills = Zabójstwa
column-action = Intensywność
column-spectating = Obserwacja

status-complete = Przeanalizowane
status-partial = Częściowo
status-failed = Błąd
status-unsupported = Nieobsługiwane
status-no-use = Bez użycia broni

kind-multiplayer-view = Widok gracza
kind-singleplayer = Gra jednoosobowa
kind-hltv = HLTV
kind-unknown = Nieznany rodzaj

details-recorder = Nagrał { $name }
details-shots = { $count ->
    [one] { $count } strzał
    [few] { $count } strzały
    [many] { $count } strzałów
   *[other] { $count } strzału
}
details-kills = { $count ->
    [one] { $count } zabójstwo
    [few] { $count } zabójstwa
    [many] { $count } zabójstw
   *[other] { $count } zabójstwa
}
details-action = Najwyższa intensywność { NUMBER($score, minimumFractionDigits: 2) }

## Skanowanie

scan-progress = Znaleziono { $found }, gotowe { $done } (już znane { $cached }, przeanalizowane { $analysed }), odczytano { $megabytes } MB
scan-cancelling = Zatrzymywanie…
scan-cancelled = Zatrzymano, do zeskanowania zostało { $left }.
scan-finished = Gotowe: dem { $done }, przeanalizowanych { $analysed }, w { $seconds } s.
scan-problems = { $count ->
    [one] { $count } problem
    [few] { $count } problemy
    [many] { $count } problemów
   *[other] { $count } problemu
}
problems-title = Problemy skanowania
problem-changed = Plik zmienił się podczas analizy, więc nic nie zapisano.
drop-hint = Upuść dema lub foldery, aby je dodać
drop-not-demo = To nie jest demo: { $names }

## Komunikaty rdzenia analizy według kodów

analysis-panicked = Analiza zatrzymała się z powodu usterki w Lambda Slicer: { $detail }
cache-cancelled = Analizę anulowano, więc nic nie zapisano.
cache-database = Nie można użyć zapisanej biblioteki: { $detail }
cache-directory = Nie można utworzyć folderu na zapisaną bibliotekę: { $detail }
container-bad-magic = To nie jest demo Half-Life.
container-frame-bad-time = Nagranie jest uszkodzone w klatce { $frame } części { $entry }, która ma zepsuty czas.
container-frame-missing-terminal = Część { $entry } nagrania nie kończy się poprawnie.
container-frame-negative-length = Nagranie jest uszkodzone w klatce { $frame } części { $entry }.
container-frame-over-limit = Klatka { $frame } części { $entry } jest większa niż dozwolone.
container-frame-truncated = Nagranie urywa się w klatce { $frame } części { $entry }.
container-frame-unknown-tag = Nagranie jest uszkodzone w klatce { $frame } części { $entry }, która zawiera coś, czego Lambda Slicer nie rozpoznaje.
container-io = Nie można odczytać pliku: { $detail }
container-no-directory = Nagranie nie zostało dokończone, więc nie ma spisu treści.
container-range = Spis treści nagrania wskazuje poza plik.
container-too-short = Plik jest za krótki, aby był demem.
coverage-hltv = Nagranie HLTV. Nie nagrywał go żaden gracz, więc zdarzenia podążają za tym, co oglądał HLTV.
coverage-mostly-spectating = Nagrywający obserwował przez { $percent }% odtwarzania, więc większość zdarzeń należy do innych graczy, tak jak było je widać podczas ich obserwowania.
coverage-no-local-events = Własne działania nagrywającego nie zostały zapisane w tym demie, więc widać tylko te, które wysłał serwer.
coverage-singleplayer = Gra jednoosobowa. Cała reszta to potwory lub części mapy i nie ma kill feedu.
coverage-unknown = Nagrania nie udało się odczytać na tyle, by ustalić jego rodzaj.
coverage-visible-only = Gra dostaje działania innych graczy tylko wtedy, gdy serwer uznaje, że nagrywający może ich widzieć, więc ich liczby mogą być niższe niż w rzeczywistości.
decode-failed = Odczyt nagrania zatrzymał się w { $time } (klatka { $frame }): { $detail }
export-active-effect = Na początku wciąż trwa { $effect } i klip nie potrafi go jeszcze przenieść.
export-client-dropped-packets = W { $time } gra wyrzuciła dane, które przyszły za późno, a klip nie potrafi jeszcze tego skopiować.
export-cross-segment = W { $time }, wewnątrz klipu, zmieniła się mapa albo gracz połączył się ponownie, a klip nie może przez to przechodzić.
export-destination-exists = { $path } już istnieje. Wybierz inną nazwę.
export-empty-range = Klip nie zawiera niczego do eksportu.
export-io = Nie można zapisać { $path }: { $detail }
export-missing-delta-base = W nagraniu brakuje danych sprzed początku, których klip potrzebuje (numer { $sequence }).
export-missing-parent = Folder { $path } nie istnieje.
export-range-beyond-end = Koniec { $end } wypada po końcu nagrania, { $segment_end }.
export-same-as-source = Eksport zastąpiłby demo, z którego pochodzi.
export-source-changed = { $path } zmienił się podczas eksportu.
export-source-decode-failed = Nie można odczytać dema: { $detail }
export-source-failed-crosscheck = Przy sprawdzeniu to, co Lambda Slicer ustalił z tego dema, nie zgadzało się z samym demem, więc klipom z niego nie można ufać.
export-unknown-persistent-message = { $name } przed początkiem ustawia coś, czego klip nie potrafi jeszcze przenieść.
export-unproven-state = W { $time } nagranie opiera się na danych, które trzeba było zgadnąć, więc klipu nie da się zrobić dokładnie.
export-unrepresentable-time-field = Czasu na początku nie da się zapisać dokładnie: { $detail }
export-unsupported = Nieobsługiwane: { $detail }
export-validation-failed = Wyeksportowany klip nie przeszedł końcowego sprawdzenia: { $detail }
io-not-found = Nie znaleziono pliku ani folderu.
io-other = { $detail }
io-permission-denied = Odmowa dostępu.
io-unexpected-end = Plik nieoczekiwanie się skończył.
protocol-unsupported = To demo pochodzi z nieobsługiwanej wersji gry (protokół { $demo }/{ $network }). Obsługiwana jest tylko 5/48.
protocol-unsupported-network = Serwer przełączył się na nieobsługiwaną wersję gry (protokół { $network }).

## Widok dema

demo-close = Zamknij
demo-loading = Analiza { $name }…
show-shots = Strzały
show-actions = Inne działania
show-kills = Zabójstwa
show-switches = Zmiany broni
lane-unknown = Nieznany gracz
lane-recorder = { $name } (nagrywający)
event-kill = Zabił { $victim }
event-switch = Zmiana broni
action-swing = Uderzenie
action-fire = Strzał
action-launch = Wystrzelenie
action-charge-activity = Ładowanie
action-charge-stop = Koniec ładowania
action-beam-start = Początek wiązki
action-beam-sustain = Wiązka
action-beam-stop = Koniec wiązki
action-placement-attempt = Próba umieszczenia
action-release-attempt = Próba wypuszczenia
mode-primary = podstawowy
mode-charged = naładowany
mode-zoomed = z lunetą
mode-unzoomed = bez lunety
mode-grenade = granat
mode-bullets = pociski
mode-single = pojedynczy
mode-double = podwójny
mode-rapid = szybki
demo-events = { $count ->
    [one] { $count } zdarzenie
    [few] { $count } zdarzenia
    [many] { $count } zdarzeń
   *[other] { $count } zdarzenia
}

column-date = Data
column-file-date = Data pliku
column-score = Wynik
filter-from = Od
filter-to = Do
date-from-name = Data z nazwy pliku
date-from-modified = Data ostatniej modyfikacji pliku, którą kopiowanie lub pobieranie może zmienić
timeline-best-action = Złota minuta
timeline-window = Nr { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Wyniki końcowe, przy ostatnim zabójstwie { $time }
scores-at = Wyniki o { $time }
scores-show-final = Pokaż wyniki końcowe
scores-frags = Fragi
scores-deaths = Zgony

viewer-play = Odtwórz
viewer-pause = Wstrzymaj
viewer-volume = Głośność
viewer-no-sound = Brak dźwięku
viewer-play-backward = Przewiń do tyłu
viewer-step-back = Poprzednia klatka
viewer-step-forward = Następna klatka
viewer-position = Pozycja w demie
viewer-skip-back = 10 sekund do tyłu
viewer-skip-forward = 10 sekund do przodu
viewer-rate = { $rate }×
viewer-shortcut = Skrót: { $key }
key-space = Spacja
key-shift-left = Shift+Lewo
key-shift-right = Shift+Prawo
viewer-map-failed = Nie udało się wczytać mapy { $map }
viewer-stopped = Odtwarzanie zatrzymało się przed końcem: { $reason }
viewer-missing = Pliki, których nie udało się odczytać: { $count }
event-previous = Poprzednie zdarzenie
event-next = Następne zdarzenie
event-play = Odtwórz zdarzenie
timeline-title = Oś czasu
timeline-lanes = Wiersze graczy
timeline-zoom-in = Przybliż
timeline-zoom-out = Oddal
timeline-fit-all = Dopasuj całe demo
timeline-gestures = Kliknij znacznik, aby wybrać jego zdarzenie, lub gdziekolwiek indziej, aby tam przejść. Przeciągnij po czasach u góry albo przeciągnij linię bieżącego momentu, aby poruszać się po demie. Przeciągnij gdziekolwiek indziej, aby przesunąć widok. Przewiń kółkiem, aby przybliżać i oddalać tam, gdzie jest myszka, i kliknij dwukrotnie, aby zobaczyć całe demo. Przeciągnij krawędź klipu, aby zmienić jego początek lub koniec, lub kliknij klip, aby go edytować.
timeline-density = Zacienione słupki pokazują, ile z pokazanych zdarzeń dzieje się w każdym miejscu, w porównaniu z najbardziej zatłoczonym miejscem dema.
timeline-density-count = { $count ->
    [one] Jedno pokazane zdarzenie tutaj
    [few] { $count } pokazane zdarzenia tutaj
    [many] { $count } pokazanych zdarzeń tutaj
   *[other] { $count } pokazanego zdarzenia tutaj
}
timeline-session-count = Część { $session }: { $count }
timeline-choose = Zdarzenia w tym miejscu
legend-clip = Klip
event-session = Część { $session }
event-unreachable = To zdarzenie jest po zmianie mapy albo ponownym połączeniu, gdzie zegar dema zaczął się od nowa, więc podgląd nie może do niego przejść.
event-hidden = Filtry ukrywają wybrane zdarzenie.
highlight-jump = Przejdź do highlightu…
highlight-help = Highlighty oceniają całe demo według akcji, niezależnie od tego, co pokazują filtry.
kills-with = Zabójstwa z
refine-midair = Midair
refine-speed = Prędkość
refine-wallgauss = Wallgauss
refine-clear = Wyczyść
refine-airborne = W powietrzu
refine-either = Dowolny
midair-both = Obaj
midair-one = Tylko jeden
help-midair-either = Zabójca, ofiara lub obaj byli w powietrzu. Zabójcę sprawdza się przy strzale powiązanym z zabójstwem, a bez takiego strzału przy zabójstwie, ofiarę zaś w chwili śmierci. Zwykły skok, mniej niż 50 jednostek nad podłożem, się nie liczy. Wymaga to wybrania folderu Half-Life, żeby wiedzieć, gdzie jest podłoże.
help-midair-both = Zabójca i ofiara byli obaj w powietrzu, każdy w chwili sprawdzenia.
help-midair-one = Dokładnie jeden z nich był w powietrzu, a o drugim wiadomo, że był na ziemi.
speed-at-least = Co najmniej
speed-unit = ups
speed-killer = Zabójca
speed-victim = Ofiara
help-speed = Prędkość po ziemi w ups, tak jak pokazuje ją speedometr, bez ruchu w górę i w dół. Zabójcę mierzy się przy strzale powiązanym z zabójstwem, a bez takiego strzału przy zabójstwie, ofiarę zaś w chwili śmierci. Prędkości innych graczy są szacunkowe.
wallgauss-weapon = Wallgauss obejmuje tylko zabójstwa z gaussa, a filtr broni pokazuje { $weapon }.
wallgauss-checking = Sprawdzanie Wallgauss… { $examined } z { $total } zabójstw
wallgauss-needs-files = Wallgauss potrzebuje plików gry, aby wiedzieć, gdzie są ściany mapy.
wallgauss-failed = Nie udało się sprawdzić Wallgauss. { $reason }
help-wallgauss = Zabójstwa z gaussa przez ścianę, znalezione przez śledzenie każdego strzału na mapie tak, jak robi to serwer. Zabójstwo, którego nie da się rozstrzygnąć, na przykład gdy ofiara była poza polem widzenia nagrania, liczy się jako nieznane i jest pomijane.
badge-wall = WG
wall-yes-exit = Wallgauss, ustalony na podstawie strzału: wybuch w miejscu, gdzie wiązka wyszła ze ściany o grubości { $thickness } jednostek.
wall-yes-beyond = Wallgauss, ustalony na podstawie strzału: wiązka trafiła po przejściu przez ścianę o grubości { $thickness } jednostek.
wall-limits = Ustalone na podstawie strzału i pozycji graczy, bo serwer tego nie zapisuje. Gracz ma 32 jednostki szerokości i liczy się jako pudełko, a ofiara mogła stać o jedną lub dwie szerokości ciała od miejsca, w którym pokazuje ją demo.
wall-no-direct = To nie Wallgauss: wiązka mogła trafić ofiarę przed jakąkolwiek ścianą.
wall-no-reflection = To nie Wallgauss: wybuch odbicia mógł trafić ofiarę przed jakąkolwiek ścianą.
wall-doubt-no-shot = Wallgauss nieznany: strzału zabójcy nie ma w nagraniu.
wall-doubt-no-map = Wallgauss nieznany: nie udało się odczytać plików mapy.
wall-doubt-unseen = Wallgauss nieznany: ofiara była poza polem widzenia nagrania.
wall-doubt-brush = Wallgauss nieznany: wiązka trafiła w drzwi, windę lub inną specjalną część mapy, której się nie sprawdza.
wall-doubt-ambiguous = Wallgauss nieznany: strzał mógł zabić zarówno przez ścianę, jak i bez niej.
wall-doubt-unexplained = Wallgauss nieznany: nie znaleziono drogi, którą strzał dotarł do ofiary.
wall-doubt-unconfirmed = Wallgauss nieznany: strzał mógł dosięgnąć ofiary tylko przez ścianę, ale nie na pewno.
wall-doubt-not-reached = Wallgauss nieznany: nagranie kończy się przed strzałem.
wall-pending = Wallgauss jeszcze niesprawdzony.
events-unknown = { $count ->
    [one] Dla jednego zabójstwa brakuje danych, by to ustalić
    [few] Dla { $count } zabójstw brakuje danych, by to ustalić
    [many] Dla { $count } zabójstw brakuje danych, by to ustalić
   *[other] Dla { $count } zabójstwa brakuje danych, by to ustalić
}
help-unknown = Ruch, którego nagranie nie mogło zobaczyć, liczy się jako nieznany, nigdy jako stanie na ziemi czy bezruch, więc te zabójstwa są pomijane.
clip-it = Zrób klip
badge-headshot = SG
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = samobójstwo
badge-teamkill = teamkill
badge-world = świat
badge-clip = klip
clip-of = Klip z { $name }
air-both = Obaj byli w powietrzu w chwili sprawdzenia.
air-killer = Zabójca był w powietrzu w chwili sprawdzenia.
air-victim = Ofiara była w powietrzu w chwili sprawdzenia.
air-other-unknown = Nie wiadomo, czy drugi też.
speed-horizontal = Prędkość pozioma, równolegle do ziemi.
motion-line = { $who }: { $air }, { $speed }. { $how }, { $when }.
motion-who-killer = Zabójca
motion-who-victim = Ofiara
motion-air-yes = w powietrzu
motion-air-no = na ziemi
motion-air-unknown = nie wiadomo, czy w powietrzu
motion-speed-unknown = prędkość nieznana
motion-exact = Zapisane przez grę
motion-trajectory = Oszacowane z ruchu
motion-unclear = Ruch niejasny
motion-not-visible = Poza widokiem
motion-at-shot = przy powiązanym strzale
motion-at-kill = w chwili śmierci
motion-none = Analiza nie ma ruchu dla tego zabójstwa.
legend-shot = Strzał
legend-charge = Ładowanie
legend-kill = Zabójstwo
legend-death = Zgon
legend-switch = Zmiana broni

column-headshots = Strzały w głowę
filter-headshots = Zabójstwa strzałem w głowę
filter-midair = Zabójstwa midair
event-headshot = w głowę
scores-headshots = SG

column-accuracy = Celność
filter-min-accuracy = Celność co najmniej (%)
scores-accuracy = Cel.

help-headshots = Zabójstwa trafieniem w głowę, rozpoznane po animacji śmierci ofiary. Wybuchy się nie liczą. Pokazywane są tylko zabójstwa nagrywającego, bo inni gracze często zabijają poza jego polem widzenia. W pojedynku pokazywane są też zabójstwa przeciwnika, bo wszystkie dotyczą nagrywającego.
help-headshots-column = Zabójstwa nagrywającego trafieniem w głowę, rozpoznane po animacji śmierci ofiary. Wybuchy się nie liczą. Zabójstwa innych graczy nie są liczone, bo często dzieją się poza polem widzenia nagrania.
help-date-column = Kiedy mecz został nagrany. Data jest brana z nazwy pliku, jeśli nazwa zawiera datę taką jak 20230910_214553, tak jak w automatycznych nagraniach gry. W przeciwnym razie to data pliku, którą kopiowanie lub pobieranie może zmienić.
help-file-date = Kiedy plik został zapisany. W przypadku wyeksportowanego klipu to moment eksportu.
help-accuracy = Odsetek strzałów, które trafiły, ustalony na podstawie krwi, jaką trafienie zostawia na linii celowania, po odjęciu przypadkowych dopasowań. Obejmuje glocka, pythona, pociski MP5, strzelbę, kuszę z lunetą i bezpośrednie trafienia wiązki gaussa. Wybuchy i odbicia gaussa nie zostawiają krwi i nie są liczone. Pokazywany tylko dla nagrywającego, po co najmniej 50 strzałach.
help-headshot-filter = Dema z co najmniej jednym zabójstwem w głowę widocznym w nagraniu, przez dowolnego gracza.
help-midair-filter = Dema z co najmniej jednym zabójstwem, przy którym zabójca lub ofiara byli w powietrzu, co najmniej 50 jednostek nad podłożem. Bez wybranego folderu Half-Life liczy się też zwykły skok.
help-accuracy-filter = Dema, w których nagrywający trafił co najmniej taki odsetek strzałów.

clips-new = Nowy klip
clips-select-event = Najpierw wybierz zdarzenie na liście lub na osi czasu.
clips-other-session = Klip nie może zacząć się tutaj, bo po zmianie mapy albo ponownym połączeniu zegar dema zaczął się od nowa, a wcześniejsza część ma te same czasy.
clips-add-window = Klip tego gorącego momentu
clip-default-name = Klip { $number }
clip-name = Nazwa
clip-length = Długość { $duration }
clip-remove = Usuń
clip-loop = Zapętl
clip-time-help = Sekundy (96), minuty i sekundy (1:36) albo godziny, minuty i sekundy (00:01:36.250)
clips-empty-hint = Nie ma jeszcze klipów. Wybierz zdarzenie i naciśnij Zrób klip albo naciśnij Nowy klip, aby zacząć klip od bieżącego momentu.
clips-new-help = Dziesięciosekundowy klip od bieżącego momentu.
clip-in = Początek
clip-out = Koniec
clip-set-in = Zacznij klip od bieżącego momentu
clip-set-out = Zakończ klip na bieżącym momencie
clip-no-playhead = Podgląd jest niedostępny, więc nie ma bieżącego momentu do użycia.
clip-play = Odtwórz klip raz
clip-show-on-timeline = Pokaż na osi czasu
export-destination = Zmień folder eksportu
export-skipped = Pominięto. { $reason }
export-earlier-range = Ten eksport obejmuje { $start } – { $end }, czyli klip sprzed twoich zmian.
scores-player = Gracz
scores-none = Nagranie nie ma wyników.
clip-effective = Eksport obejmie { $start } – { $end } ({ $duration }), tak dokładnie, jak pozwala nagranie.
settings-clip-padding = Zapas wokół klipu
settings-preroll = Sekundy przed zdarzeniem
settings-postroll = Sekundy po zdarzeniu
export-start-first-group = Klip nie może zaczynać się na samym początku nagrania. Zacznij odrobinę później.
export-start-during-loading = Początek wypada podczas wczytywania mapy. Zacznij odrobinę później.

export-all = Eksportuj wszystkie klipy
export-one = Eksportuj
export-cancel = Zatrzymaj eksport
export-folder = Eksport trafia do { $folder }.
export-folder-beside = Eksport trafia obok dema, do { $folder }.
export-waiting = Czeka na eksport…
export-running = Eksportowanie… Całe demo jest czytane, to może chwilę potrwać.
export-cancelled = Nie wyeksportowano, bo eksport zatrzymano.
export-saved = Zapisano { $path }. Momenty sprawdzone z oryginałem: { $count }.
export-playdemo = W konsoli gry:
export-copy-first = Skopiuj { $name } do folderu moda, tego z liblist.gam, a potem w konsoli gry:
export-copy = Kopiuj
settings-export-folder = Folder eksportu
settings-export-choose = Wybierz…
settings-file-names = Nazwy plików
settings-clip-name-files = Używaj nazwy klipu jako nazwy pliku
settings-demo-name-first = Zaczynaj od nazwy dema
settings-viewer = Podgląd dem
settings-viewer-game = Folder Half-Life
settings-viewer-game-none = Nie wybrano
settings-viewer-game-hint = Folder zawierający valve. Podgląd odtwarza dema z mapami, modelami i dźwiękami gry z tego folderu.
settings-executable = Program gry
settings-executable-hint = Pozwala znaleźć folder Half-Life. Podgląd sam czyta pliki gry i nigdy jej nie uruchamia.
settings-executable-not-set = Nie ustawiono, więc podgląd nie ma obrazu ani dźwięku
settings-executable-optional = Nie ustawiono i nie trzeba, dopóki folder Half-Life zawiera grę
settings-executable-missing = { $path } już nie istnieje. Podgląd nadal czyta folder Half-Life.
settings-game-override = Tym razem Lambda Slicer używa { $path }, bo został uruchomiony z --game. Wybór tutaj zadziała przy następnym uruchomieniu.
settings-browse = Przeglądaj…
dialog-executables = Programy
setup-title = Skonfiguruj podgląd dem
setup-body = Obraz i dźwięk wymagają plików gry Half-Life. Wskaż Lambda Slicer, gdzie gra jest zainstalowana, a będzie je stamtąd czytać.
setup-locate-exe = Znajdź hl.exe…
setup-locate-folder = Znajdź Half-Life…
setup-settings = Ustawienia
setup-still-works = Oś czasu, zdarzenia i klipy działają także bez nich.
setup-relocate = { $folder } nie zawiera już Half-Life. Znajdź go ponownie.
setup-not-executable = { $name } to nie { $executable }. Wybierz program samej gry, nie skrót.
setup-not-installed = { $folder } nie zawiera instalacji Half-Life z folderem valve.
tab-events = Zdarzenia
tab-clips = Klipy
tab-scoreboard = Tabela wyników
list-hide = Ukryj listę dem
list-show = Pokaż listę dem
demo-details = Szczegóły
viewer-start = Wróć na początek
viewer-mute = Wycisz
viewer-unmute = Włącz dźwięk
viewer-picture = Obraz i ustawienia podglądu
viewer-needs-files = Wymaga plików gry
viewer-no-sound-why = Brak dźwięku: { $reason }
settings-tab-general = Ogólne
settings-tab-library = Biblioteka
settings-tab-clips = Klipy
settings-section-game = Half-Life
settings-section-view = Widok
settings-language-system = Język komputera ({ $name })
settings-section-folders = Foldery
settings-folders-hint = Foldery, w których biblioteka szuka dem, razem z podfolderami.
settings-clip-padding-hint = Ile klip utworzony wokół zdarzenia zachowuje przed nim i po nim.
settings-section-picture = Obraz
settings-section-sound = Dźwięk
settings-gamma = Gamma
settings-brightness = Jasność
settings-texgamma = Gamma tekstur
settings-lightgamma = Gamma światła
settings-overbright = Prześwietlone oświetlenie
settings-gamma-hint = Ustawienia gry o tych samych nazwach (gamma, brightness, texgamma, lightgamma, gl_overbright). Ustaw je jak w grze, aby zobaczyć tę samą jasność.
settings-gamma-defaults = Wartości gry
settings-fov-recorded = Pole widzenia jak w nagraniu
settings-fov = Pole widzenia
settings-zoom-fov = Pole widzenia po przybliżeniu
settings-viewmodel = Pokazuj broń w ręku
settings-see-through = Widź graczy przez ściany
settings-see-through-items = Także inne modele, np. pociski i przedmioty
settings-see-through-hint = Gracze za ścianami są widoczni jako sylwetki w ich górnym kolorze, który gry drużynowe ustawiają według drużyny. Widać tylko to, co zawiera demo. Serwer wysyła graczy w pobliżu nagrywającego, a serwery ukrywające graczy, których nagrywający nie widzi, nie zostawiają nic do pokazania.
viewer-zoom = Przybliż widok
viewer-see-through = Widź przez ściany
settings-export-beside = Obok każdego dema
settings-export-hint = W folderze moda, tym z liblist.gam, na przykład Half-Life\ag, klipy odtwarza jedno polecenie playdemo.
settings-file-names-hint = Wyeksportowany klip dostaje nazwę od swojego dema oraz czasu początku i końca, chyba że używasz nazwy klipu. Gra otwiera tylko pliki, których nazwy składają się z liter alfabetu angielskiego, cyfr, - i ., więc wszystko inne zmienia się w _. Jeśli nazwa klipu nie ma żadnego z tych znaków, plik dostaje nazwę od czasów.

clips-store-moved = Nie udało się odczytać pliku klipów { $path }, więc zachowano go jako { $moved }, a lista klipów zaczyna się od nowa. { $detail }
clips-store-unreadable = Nie udało się odczytać pliku klipów { $path }: { $detail }. Dopóki Lambda Slicer nie zostanie zamknięty, klipy nie są zapisywane, więc nic w pliku nie przepadnie.
clips-save-failed = Nie udało się zapisać klipów w { $path }: { $detail }
notice-dismiss = Zamknij

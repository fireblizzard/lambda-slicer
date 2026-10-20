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
settings-workers = Wątki analizy
settings-workers-hint = Więcej wątków analizuje szybciej, ale bardziej obciąża procesor.
settings-cache = Pamięć podręczna biblioteki
settings-folders = Foldery biblioteki
settings-folders-empty = Nie dodano jeszcze folderów.
settings-remove = Usuń

## Biblioteka

library-empty = Dodaj folder lub otwórz demo, aby zacząć.
library-no-matches = Żadne demo nie pasuje do tych filtrów.
library-no-cache = W tym systemie nie ma gdzie przechowywać pamięci podręcznej biblioteki.
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
path-copy = Kopiuj ścieżkę
path-show = Pokaż w folderze
path-show-failed = Nie udało się otworzyć folderu ({ $detail }). Skopiuj ścieżkę.

filter-search = Szukaj w ścieżkach
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
kind-hltv = Proxy obserwatora
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

scan-progress = Znaleziono { $found }, gotowe { $done } (z pamięci { $cached }, przeanalizowane { $analysed }), odczytano { $megabytes } MB
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

analysis-panicked = Analiza zatrzymała się z powodu błędu wewnętrznego: { $detail }
cache-cancelled = Analizę anulowano, więc nic nie zapisano.
cache-database = Nie można użyć pamięci podręcznej biblioteki: { $detail }
cache-directory = Nie można utworzyć folderu pamięci podręcznej: { $detail }
container-bad-magic = To nie jest demo Half-Life.
container-frame-bad-time = Nagranie jest uszkodzone w klatce { $frame } części { $entry }, która ma nieprawidłowy czas.
container-frame-missing-terminal = Część { $entry } nagrania kończy się bez rekordu zamykającego.
container-frame-negative-length = Nagranie jest uszkodzone w klatce { $frame } części { $entry }.
container-frame-over-limit = Klatka { $frame } części { $entry } jest większa niż dozwolone.
container-frame-truncated = Nagranie urywa się w klatce { $frame } części { $entry }.
container-frame-unknown-tag = Nagranie jest uszkodzone w klatce { $frame } części { $entry }, która ma nieznany typ.
container-io = Nie można odczytać pliku: { $detail }
container-no-directory = Nagranie nie zostało dokończone, więc nie ma spisu treści.
container-range = Spis treści nagrania wskazuje poza plik.
container-too-short = Plik jest za krótki, aby był demem.
coverage-hltv = Nagranie proxy obserwatora. Nie ma nagrywającego gracza, a zdarzenia śledzą widok proxy.
coverage-mostly-spectating = Nagrywający obserwował przez { $percent }% odtwarzania, więc większość zdarzeń należy do innych graczy, tak jak pokazywał je obserwowany widok.
coverage-no-local-events = Brak zdarzeń lokalnych. Brakuje działań nagrywającego, poza tymi, które wysłał serwer.
coverage-singleplayer = Gra jednoosobowa. Inni uczestnicy to potwory lub obiekty mapy i nie ma listy zabójstw.
coverage-unknown = Nagrania nie udało się odczytać na tyle, by ustalić jego rodzaj.
coverage-visible-only = Zdarzenia innych graczy docierają do tego klienta tylko wtedy, gdy serwer uznaje ich za widocznych, więc ich liczby są dolnym ograniczeniem.
decode-failed = Dekodowanie zatrzymało się w { $time } (klatka { $frame }): { $detail }
export-active-effect = Na początku trwa { $effect } i nie da się go jeszcze przenieść.
export-client-dropped-packets = W { $time } gra odrzuciła spóźnione pakiety, czego cięcie jeszcze nie potrafi odtworzyć.
export-cross-segment = W { $time }, wewnątrz zakresu, zaczyna się nowa sesja.
export-destination-exists = { $path } już istnieje. Wybierz inną nazwę.
export-empty-range = Wybrany zakres nie zawiera niczego do eksportu.
export-io = Nie można zapisać { $path }: { $detail }
export-missing-delta-base = W nagraniu brakuje stanu potrzebnego przed początkiem (sekwencja { $sequence }).
export-missing-parent = Folder { $path } nie istnieje.
export-range-beyond-end = Koniec { $end } wypada po końcu nagrania, { $segment_end }.
export-same-as-source = Miejscem docelowym jest samo demo źródłowe.
export-source-changed = { $path } zmienił się podczas eksportu.
export-source-decode-failed = Nie można zdekodować dema źródłowego: { $detail }
export-source-failed-crosscheck = Demo źródłowe nie przeszło kontroli krzyżowej, więc odtworzonemu stanowi nie można ufać.
export-unknown-persistent-message = { $name } przed początkiem ustawia stan, którego cięcie jeszcze nie potrafi przenieść.
export-unproven-state = Stan w { $time } opiera się na wywnioskowanej bazie i nie jest udowodniony.
export-unrepresentable-time-field = Czasu na początku nie da się zapisać wiernie: { $detail }
export-unsupported = Nieobsługiwane: { $detail }
export-validation-failed = Wyeksportowany plik nie przeszedł weryfikacji: { $detail }
io-not-found = Nie znaleziono pliku ani folderu.
io-other = { $detail }
io-permission-denied = Odmowa dostępu.
io-unexpected-end = Plik nieoczekiwanie się skończył.
protocol-unsupported = Protokół dema { $demo }/{ $network } nie jest obsługiwany, tylko 5/48.
protocol-unsupported-network = Serwer przełączył się na nieobsługiwany protokół sieciowy { $network }.

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
viewer-rate = { $rate }×
viewer-shortcut = Skrót: { $key }
key-space = Spacja
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
timeline-gestures = Kliknij znacznik, aby wybrać jego zdarzenie, lub gdziekolwiek indziej, aby tam przejść. Przeciągnij linijkę lub głowicę odtwarzania, aby przewijać, a przeciągnij gdzie indziej, aby przesunąć widok. Przewiń kółkiem, aby przybliżyć wokół kursora, i kliknij dwukrotnie, aby zobaczyć całe demo. Przeciągnij krawędź klipu, aby ją przesunąć, lub kliknij klip, aby go edytować.
timeline-density = Zacienione słupki pokazują, ile z pokazanych zdarzeń przypada na każdy odcinek, w porównaniu z najbardziej zatłoczonym odcinkiem dema.
timeline-density-count = { $count ->
    [one] Jedno pokazane zdarzenie tutaj
    [few] { $count } pokazane zdarzenia tutaj
    [many] { $count } pokazanych zdarzeń tutaj
   *[other] { $count } pokazanego zdarzenia tutaj
}
timeline-session-count = Sesja { $session }: { $count }
timeline-choose = Zdarzenia w tym miejscu
legend-clip = Klip
event-session = Sesja { $session }
event-unreachable = To zdarzenie jest w późniejszej sesji, która używa tych samych czasów, więc podgląd nie może do niego przejść.
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
help-midair-either = Zabójca, ofiara lub obaj byli w powietrzu. Zabójcę sprawdza się przy strzale powiązanym z zabójstwem, a bez takiego strzału przy zabójstwie, ofiarę zaś w chwili śmierci.
help-midair-both = Zabójca i ofiara byli obaj w powietrzu, każdy w chwili sprawdzenia.
help-midair-one = Dokładnie jeden z nich był w powietrzu, a o drugim wiadomo, że był na ziemi.
speed-at-least = Co najmniej
speed-unit = ups
speed-killer = Zabójca
speed-victim = Ofiara
help-speed = Prędkość pozioma w jednostkach na sekundę, równolegle do ziemi. Zabójcę mierzy się przy strzale powiązanym z zabójstwem, a bez takiego strzału przy zabójstwie, ofiarę zaś w chwili śmierci. Prędkości innych graczy są szacunkowe.
wallgauss-pending = Ta wersja nie potrafi jeszcze rozpoznać, które zabójstwa z gaussa przeszły przez ścianę, więc żadne zabójstwo nie pasuje. Wyczyść zawężenie, aby znów zobaczyć wszystkie zabójstwa.
wallgauss-weapon = Wallgauss obejmuje tylko zabójstwa z gaussa, a filtr broni pokazuje { $weapon }.
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
event-headshot = w głowę
scores-headshots = SG

column-accuracy = Celność
filter-min-accuracy = Celność co najmniej (%)
scores-accuracy = Cel.

help-headshots = Zabójstwa trafieniem w głowę, rozpoznane po animacji śmierci ofiary. Wybuchy się nie liczą. Pokazywane są tylko zabójstwa nagrywającego, bo inni gracze często zabijają poza jego polem widzenia. W pojedynku pokazywane są też zabójstwa przeciwnika, bo wszystkie dotyczą nagrywającego.
help-headshots-column = Zabójstwa nagrywającego trafieniem w głowę, rozpoznane po animacji śmierci ofiary. Wybuchy się nie liczą. Zabójstwa innych graczy nie są liczone, bo często dzieją się poza polem widzenia nagrania.
help-accuracy = Odsetek strzałów, które trafiły, ustalony na podstawie krwi, jaką trafienie zostawia na linii celowania, po odjęciu przypadkowych dopasowań. Obejmuje glocka, pythona, pociski MP5, strzelbę, kuszę z lunetą i bezpośrednie trafienia wiązki gaussa. Wybuchy i odbicia gaussa nie zostawiają krwi i nie są liczone. Pokazywany tylko dla nagrywającego, po co najmniej 50 strzałach.
help-headshot-filter = Dema z co najmniej jednym zabójstwem w głowę widocznym w nagraniu, przez dowolnego gracza.
help-accuracy-filter = Dema, w których nagrywający trafił co najmniej taki odsetek strzałów.

clips-title = Klipy
clips-new = Nowy klip
clips-add-event = Klip { $start } – { $end }
clips-add-event-none = Klip wybranego zdarzenia
clips-select-event = Najpierw wybierz zdarzenie na liście lub na osi czasu.
clips-other-session = Klip nie może zacząć się tutaj, bo wcześniejsza sesja nagrania ma te same czasy.
clips-add-window = Klip tego gorącego momentu
clips-empty = Nie ma jeszcze klipów. Wybierz zdarzenie lub jeden z najgorętszych momentów i dodaj wokół niego klip. Ile klip obejmuje przed zdarzeniem i po nim, ustawisz w Ustawieniach. Nowy klip dodaje klip, którego początek i koniec wpisujesz sam.
clip-default-name = Klip { $number }
clip-name = Nazwa
clip-length = Długość { $duration }
clip-show = Pokaż
clip-remove = Usuń
clip-loop = Zapętl
clip-time-help = Sekundy (96), minuty i sekundy (1:36) albo godziny, minuty i sekundy (00:01:36.250)
clip-exact = Eksport zacznie się i skończy dokładnie tutaj.
clip-effective = Eksport wytnie { $start } – { $end } ({ $duration }), według klatek nagrania.
settings-clip-padding = Zapas wokół klipu
settings-preroll = Sekundy przed zdarzeniem
settings-postroll = Sekundy po zdarzeniu
export-start-first-group = Wycinek nie może zaczynać się od pierwszej klatki nagrania. Zacznij odrobinę później.
export-start-during-loading = Początek wypada podczas wczytywania mapy. Zacznij odrobinę później.

export-all = Eksportuj wszystkie klipy
export-one = Eksportuj
export-cancel = Zatrzymaj eksport
export-folder = Eksport trafia do { $folder }.
export-folder-beside = Eksport trafia obok dema, do { $folder }.
export-waiting = Czeka na eksport…
export-running = Eksportowanie… Całe demo jest czytane, to może chwilę potrwać.
export-cancelled = Nie wyeksportowano, bo eksport zatrzymano.
export-saved = Zapisano { $path }. Migawki sprawdzone z oryginałem: { $count }.
export-playdemo = W konsoli gry:
export-copy-first = Skopiuj { $name } do folderu moda, tego z liblist.gam, a potem w konsoli gry:
export-copy = Kopiuj
settings-export-folder = Folder eksportu
settings-export-choose = Wybierz…
settings-viewer = Podgląd dem
settings-viewer-game = Folder Half-Life
settings-viewer-game-none = Nie wybrano
settings-viewer-game-hint = Folder zawierający valve. Podgląd odtwarza dema z mapami, modelami i dźwiękami gry z tego folderu.
settings-executable = Plik wykonywalny gry
settings-executable-hint = Pozwala znaleźć folder Half-Life. Podgląd sam czyta pliki gry i nigdy jej nie uruchamia.
settings-executable-not-set = Nie ustawiono, więc podgląd nie ma obrazu ani dźwięku
settings-executable-optional = Nie ustawiono i nie trzeba, dopóki folder Half-Life zawiera grę
settings-executable-missing = { $path } już nie istnieje. Podgląd nadal czyta folder Half-Life.
settings-game-override = To uruchomienie używa { $path }, podanego przez --game. Wybór tutaj zadziała od następnego uruchomienia.
settings-browse = Przeglądaj…
dialog-executables = Programy
setup-title = Skonfiguruj podgląd dem
setup-body = Obraz i dźwięk wymagają plików gry Half-Life. Wskaż Lambda Slicer, gdzie gra jest zainstalowana, a będzie je stamtąd czytać.
setup-locate-exe = Znajdź hl.exe…
setup-locate-folder = Znajdź Half-Life…
setup-settings = Ustawienia
setup-still-works = Oś czasu, zdarzenia i klipy działają także bez nich.
setup-relocate = { $folder } nie zawiera już Half-Life. Znajdź go ponownie.
setup-not-executable = { $name } to nie { $executable }. Wybierz plik wykonywalny gry, nie skrót.
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
settings-viewmodel = Pokazuj broń w ręku
settings-see-through = Widź graczy przez ściany
settings-see-through-items = Także inne modele, np. pociski i przedmioty
settings-see-through-hint = Gracze za ścianami są widoczni jako sylwetki w ich górnym kolorze, który gry drużynowe ustawiają według drużyny. Widać tylko to, co zawiera demo. Serwer wysyła graczy w pobliżu nagrywającego, a serwery ukrywające graczy, których nagrywający nie widzi, nie zostawiają nic do pokazania.
viewer-see-through = Widź przez ściany
settings-export-beside = Obok każdego dema
settings-export-hint = W folderze moda, tym z liblist.gam, na przykład Half-Life\ag, klipy odtwarza jedno polecenie playdemo.

clips-store-moved = Nie udało się odczytać pliku klipów { $path }, więc zachowano go jako { $moved }, a lista klipów zaczyna się od nowa. { $detail }
clips-store-unreadable = Nie udało się odczytać pliku klipów { $path }: { $detail }. W tej sesji klipy nie są zapisywane, więc nic z niego nie przepadnie.
clips-save-failed = Nie udało się zapisać klipów w { $path }: { $detail }
notice-dismiss = Zamknij

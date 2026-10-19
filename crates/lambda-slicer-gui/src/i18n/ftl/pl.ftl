# Polski. Wersja robocza bez korekty

app-title = Lambda Slicer

## Pasek narzędzi

toolbar-open = Otwórz demo…
toolbar-add-folder = Dodaj folder…
toolbar-rescan = Skanuj ponownie
toolbar-cancel = Anuluj
toolbar-settings = Ustawienia
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
timeline-fit = Dopasuj
show-shots = Strzały
show-actions = Inne działania
show-kills = Zabójstwa
show-switches = Zmiany broni
lane-unknown = Nieznany gracz
lane-recorder = { $name } (nagrywający)
column-time = Czas
column-player = Gracz
column-weapon = Broń
column-event = Zdarzenie
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
timeline-windows = Najgorętsze momenty
timeline-window = Nr { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Wyniki końcowe, przy ostatnim zabójstwie { $time }
scores-at = Wyniki o { $time }
scores-show-final = Pokaż wyniki końcowe
scores-frags = Fragi
scores-deaths = Zgony

show-timeline = Pokaż oś czasu
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
legend-shot = Strzał
legend-charge = Ładowanie
legend-kill = Zabójstwo
legend-death = Zgon
legend-switch = Zmiana broni
timeline-help = Każdy wiersz to gracz. Przewiń, by przybliżyć, przeciągnij, by przesunąć, kliknij dwukrotnie, by zobaczyć całość, kliknij znacznik, by go wybrać.

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
timeline-clip-help = Klipy to jasnoniebieskie pasy. Przeciągnij krawędź pasa, aby ją przesunąć.
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

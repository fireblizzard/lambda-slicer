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
column-action = Akcja
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
details-action = Najlepszy wynik akcji { NUMBER($score, minimumFractionDigits: 2) }

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
export-missing-parent = Folder dla { $path } nie istnieje.
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

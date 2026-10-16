# Polski. Wersja robocza bez korekty

app-title = Lambda Slicer

toolbar-open = Otwórz demo…
toolbar-add-folder = Dodaj folder…
toolbar-rescan = Skanuj ponownie
toolbar-cancel = Anuluj
toolbar-settings = Ustawienia

settings-title = Ustawienia
settings-language = Język
settings-unreviewed = To tłumaczenie jest wersją roboczą, której nie sprawdził jeszcze żaden rodzimy użytkownik języka.
settings-workers = Wątki analizy
settings-workers-hint = Więcej wątków analizuje szybciej, ale bardziej obciąża procesor.
settings-cache = Pamięć podręczna biblioteki
settings-folders = Foldery biblioteki
settings-folders-empty = Nie dodano jeszcze folderów.
settings-remove = Usuń
settings-close = Zamknij

library-empty = Dodaj folder lub otwórz demo, aby zacząć.
library-demos = { $count ->
    [one] { $count } demo
    [few] { $count } dema
    [many] { $count } dem
   *[other] { $count } dema
}
library-kills = { $count ->
    [one] { $count } zabójstwo
    [few] { $count } zabójstwa
    [many] { $count } zabójstw
   *[other] { $count } zabójstwa
}
library-action = Wynik akcji { NUMBER($score, minimumFractionDigits: 2) }
library-spectating = Obserwacja { $percent }%

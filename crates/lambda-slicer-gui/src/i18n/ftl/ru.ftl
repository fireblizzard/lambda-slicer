# Русский. Черновик без проверки

app-title = Lambda Slicer

toolbar-open = Открыть демо…
toolbar-add-folder = Добавить папку…
toolbar-rescan = Пересканировать
toolbar-cancel = Отмена
toolbar-settings = Настройки

settings-title = Настройки
settings-language = Язык
settings-unreviewed = Это черновой перевод, его ещё не проверил носитель языка.
settings-workers = Потоки анализа
settings-workers-hint = Больше потоков анализируют быстрее, но сильнее нагружают процессор.
settings-cache = Кэш библиотеки
settings-folders = Папки библиотеки
settings-folders-empty = Папки ещё не добавлены.
settings-remove = Удалить
settings-close = Закрыть

library-empty = Добавьте папку или откройте демо, чтобы начать.
library-demos = { $count ->
    [one] { $count } демозапись
    [few] { $count } демозаписи
    [many] { $count } демозаписей
   *[other] { $count } демозаписи
}
library-kills = { $count ->
    [one] { $count } убийство
    [few] { $count } убийства
    [many] { $count } убийств
   *[other] { $count } убийства
}
library-action = Оценка экшена { NUMBER($score, minimumFractionDigits: 2) }
library-spectating = Наблюдение { $percent } %

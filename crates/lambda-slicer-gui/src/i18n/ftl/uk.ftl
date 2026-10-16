# Українська. Чернетка без перевірки

app-title = Lambda Slicer

toolbar-open = Відкрити демо…
toolbar-add-folder = Додати теку…
toolbar-rescan = Пересканувати
toolbar-cancel = Скасувати
toolbar-settings = Налаштування

settings-title = Налаштування
settings-language = Мова
settings-unreviewed = Це чернетка перекладу, яку ще не перевірив носій мови.
settings-workers = Потоки аналізу
settings-workers-hint = Більше потоків аналізують швидше, але більше навантажують процесор.
settings-cache = Кеш бібліотеки
settings-folders = Теки бібліотеки
settings-folders-empty = Теки ще не додано.
settings-remove = Вилучити
settings-close = Закрити

library-empty = Додайте теку або відкрийте демо, щоб почати.
library-demos = { $count ->
    [one] { $count } демозапис
    [few] { $count } демозаписи
    [many] { $count } демозаписів
   *[other] { $count } демозапису
}
library-kills = { $count ->
    [one] { $count } вбивство
    [few] { $count } вбивства
    [many] { $count } вбивств
   *[other] { $count } вбивства
}
library-action = Оцінка екшену { NUMBER($score, minimumFractionDigits: 2) }
library-spectating = Спостереження { $percent } %

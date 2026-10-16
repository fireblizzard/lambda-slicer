# Українська. Чернетка без перевірки

app-title = Lambda Slicer

## Панель інструментів

toolbar-open = Відкрити демо…
toolbar-add-folder = Додати теку…
toolbar-rescan = Пересканувати
toolbar-cancel = Скасувати
toolbar-settings = Налаштування
dialog-demos = Демозаписи Half-Life

## Налаштування

settings-title = Налаштування
settings-language = Мова
settings-unreviewed = Це чернетка перекладу, яку ще не перевірив носій мови.
settings-workers = Потоки аналізу
settings-workers-hint = Більше потоків аналізують швидше, але більше навантажують процесор.
settings-cache = Кеш бібліотеки
settings-folders = Теки бібліотеки
settings-folders-empty = Теки ще не додано.
settings-remove = Вилучити

## Бібліотека

library-empty = Додайте теку або відкрийте демо, щоб почати.
library-no-matches = Жоден демозапис не відповідає цим фільтрам.
library-no-cache = У цій системі ніде зберігати кеш бібліотеки.
library-demos = { $count ->
    [one] { $count } демозапис
    [few] { $count } демозаписи
    [many] { $count } демозаписів
   *[other] { $count } демозапису
}
library-percent = { $percent } %

filter-search = Пошук за шляхами
filter-map = Мапа
filter-player = Гравець
filter-weapon-any = Будь-яка зброя
filter-min-minutes = Не коротше (хвилин)
filter-spectating-any = Гра або спостереження
filter-spectating-mostly = Переважно спостереження
filter-spectating-playing = Переважно гра
filter-clear = Скинути фільтри

column-status = Стан
column-name = Демо
column-map = Мапа
column-duration = Тривалість
column-gauss = Гаусс
column-kills = Вбивства
column-action = Екшен
column-spectating = Спостереження

status-complete = Розібрано
status-partial = Частково
status-failed = Помилка
status-unsupported = Не підтримується
status-no-use = Без стрільби

kind-multiplayer-view = Вигляд гравця
kind-singleplayer = Одиночна гра
kind-hltv = Проксі спостерігача
kind-unknown = Невідомий тип

details-recorder = Записав { $name }
details-shots = { $count ->
    [one] { $count } постріл
    [few] { $count } постріли
    [many] { $count } пострілів
   *[other] { $count } пострілу
}
details-kills = { $count ->
    [one] { $count } вбивство
    [few] { $count } вбивства
    [many] { $count } вбивств
   *[other] { $count } вбивства
}
details-action = Найкраща оцінка екшену { NUMBER($score, minimumFractionDigits: 2) }

## Сканування

scan-progress = Знайдено { $found }, готово { $done } (з кешу { $cached }, розібрано { $analysed }), прочитано { $megabytes } МБ
scan-cancelling = Зупинка…
scan-cancelled = Зупинено, лишилося просканувати { $left }.
scan-finished = Готово: демозаписів { $done }, розібрано { $analysed }, за { $seconds } с.
scan-problems = { $count ->
    [one] { $count } проблема
    [few] { $count } проблеми
    [many] { $count } проблем
   *[other] { $count } проблеми
}
problems-title = Проблеми сканування
problem-changed = Файл змінився під час розбору, тому нічого не збережено.
drop-hint = Перетягніть сюди демо або теки, щоб додати їх
drop-not-demo = Не демозапис: { $names }

## Повідомлення ядра аналізу за кодами

analysis-panicked = Аналіз зупинено через внутрішню помилку: { $detail }
cache-cancelled = Аналіз скасовано, тому нічого не збережено.
cache-database = Неможливо використати кеш бібліотеки: { $detail }
cache-directory = Неможливо створити теку кешу: { $detail }
container-bad-magic = Це не демозапис Half-Life.
container-frame-bad-time = Запис пошкоджено на кадрі { $frame } частини { $entry }: у кадру неправильний час.
container-frame-missing-terminal = Частина { $entry } запису закінчується без завершального запису.
container-frame-negative-length = Запис пошкоджено на кадрі { $frame } частини { $entry }.
container-frame-over-limit = Кадр { $frame } частини { $entry } більший за допустимий.
container-frame-truncated = Запис обривається на кадрі { $frame } частини { $entry }.
container-frame-unknown-tag = Запис пошкоджено на кадрі { $frame } частини { $entry }: невідомий тип кадру.
container-io = Не вдається прочитати файл: { $detail }
container-no-directory = Запис не було завершено, тому він не має змісту.
container-range = Зміст запису вказує за межі файлу.
container-too-short = Файл закороткий для демозапису.
coverage-hltv = Запис проксі спостерігача. Гравця, що записує, немає, події стежать за виглядом проксі.
coverage-mostly-spectating = Автор запису спостерігав { $percent } % часу відтворення, тому більшість подій — це дії інших гравців так, як їх показував спостережуваний вигляд.
coverage-no-local-events = Немає локальних подій. Дій автора запису бракує, крім тих, що надіслав сервер.
coverage-singleplayer = Одиночна гра. Інші учасники — монстри або об'єкти мапи, стрічки вбивств немає.
coverage-unknown = Запис не вдалося розібрати настільки, щоб визначити його тип.
coverage-visible-only = Події інших гравців доходять до цього клієнта, лише поки сервер вважає їх видимими, тому їхня кількість — нижня межа.
decode-failed = Декодування зупинено на { $time } (кадр { $frame }): { $detail }
export-active-effect = На момент початку триває { $effect }, його поки не можна перенести.
export-client-dropped-packets = О { $time } гра відкинула запізнілі пакети, а нарізка поки не може це відтворити.
export-cross-segment = О { $time }, усередині діапазону, починається нова сесія.
export-destination-exists = { $path } уже існує. Виберіть іншу назву.
export-empty-range = У вибраному діапазоні нічого експортувати.
export-io = Не вдається записати { $path }: { $detail }
export-missing-delta-base = У записі бракує стану, потрібного до початку (послідовність { $sequence }).
export-missing-parent = Теки для { $path } не існує.
export-range-beyond-end = Кінець { $end } пізніший за кінець запису, { $segment_end }.
export-same-as-source = Місце призначення збігається з вихідним демозаписом.
export-source-changed = { $path } змінився під час експорту.
export-source-decode-failed = Вихідний демозапис не вдається декодувати: { $detail }
export-source-failed-crosscheck = Вихідний демозапис не пройшов перехресну перевірку, тому відновленому стану не можна довіряти.
export-unknown-persistent-message = { $name } до початку задає стан, який нарізка поки не може перенести.
export-unproven-state = Стан на { $time } спирається на виведену базу й не доведений.
export-unrepresentable-time-field = Час на початку не можна записати точно: { $detail }
export-unsupported = Не підтримується: { $detail }
export-validation-failed = Експортований файл не пройшов перевірку: { $detail }
io-not-found = Файл або теку не знайдено.
io-other = { $detail }
io-permission-denied = Доступ заборонено.
io-unexpected-end = Файл несподівано закінчився.
protocol-unsupported = Протокол демо { $demo }/{ $network } не підтримується, лише 5/48.
protocol-unsupported-network = Сервер перейшов на мережевий протокол { $network }, який не підтримується.

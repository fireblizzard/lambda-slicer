# Русский. Черновик без проверки

app-title = Lambda Slicer

## Панель инструментов

toolbar-open = Открыть демо…
toolbar-add-folder = Добавить папку…
toolbar-rescan = Пересканировать
toolbar-cancel = Отмена
toolbar-settings = Настройки
dialog-demos = Демозаписи Half-Life

## Настройки

settings-title = Настройки
settings-language = Язык
settings-unreviewed = Это черновой перевод, его ещё не проверил носитель языка.
settings-workers = Потоки анализа
settings-workers-hint = Больше потоков анализируют быстрее, но сильнее нагружают процессор.
settings-cache = Кэш библиотеки
settings-folders = Папки библиотеки
settings-folders-empty = Папки ещё не добавлены.
settings-remove = Удалить

## Библиотека

library-empty = Добавьте папку или откройте демо, чтобы начать.
library-no-matches = Нет демозаписей, подходящих под эти фильтры.
library-no-cache = В этой системе негде хранить кэш библиотеки.
library-demos = { $count ->
    [one] { $count } демозапись
    [few] { $count } демозаписи
    [many] { $count } демозаписей
   *[other] { $count } демозаписи
}
library-percent = { $percent } %

filter-search = Поиск по путям
filter-map = Карта
filter-player = Игрок
filter-weapon-any = Любое оружие
filter-min-minutes = Не короче (минут)
filter-spectating-any = Игра или наблюдение
filter-spectating-mostly = В основном наблюдение
filter-spectating-playing = В основном игра
filter-clear = Сбросить фильтры

column-status = Статус
column-name = Демо
column-map = Карта
column-duration = Длина
column-gauss = Гаусс
column-kills = Убийства
column-action = Накал
column-spectating = Наблюдение

status-complete = Разобрана
status-partial = Частично
status-failed = Ошибка
status-unsupported = Не поддерживается
status-no-use = Без стрельбы

kind-multiplayer-view = Вид игрока
kind-singleplayer = Одиночная игра
kind-hltv = Прокси наблюдателя
kind-unknown = Неизвестный тип

details-recorder = Записал { $name }
details-shots = { $count ->
    [one] { $count } выстрел
    [few] { $count } выстрела
    [many] { $count } выстрелов
   *[other] { $count } выстрела
}
details-kills = { $count ->
    [one] { $count } убийство
    [few] { $count } убийства
    [many] { $count } убийств
   *[other] { $count } убийства
}
details-action = Наивысший накал { NUMBER($score, minimumFractionDigits: 2) }

## Сканирование

scan-progress = Найдено { $found }, готово { $done } (из кэша { $cached }, разобрано { $analysed }), прочитано { $megabytes } МБ
scan-cancelling = Остановка…
scan-cancelled = Остановлено, осталось просканировать { $left }.
scan-finished = Готово: демозаписей { $done }, разобрано { $analysed }, за { $seconds } с.
scan-problems = { $count ->
    [one] { $count } проблема
    [few] { $count } проблемы
    [many] { $count } проблем
   *[other] { $count } проблемы
}
problems-title = Проблемы сканирования
problem-changed = Файл изменился во время разбора, поэтому ничего не сохранено.
drop-hint = Перетащите сюда демо или папки, чтобы добавить их
drop-not-demo = Не демозапись: { $names }

## Сообщения ядра анализа по кодам

analysis-panicked = Анализ остановлен из-за внутренней ошибки: { $detail }
cache-cancelled = Анализ отменён, поэтому ничего не сохранено.
cache-database = Невозможно использовать кэш библиотеки: { $detail }
cache-directory = Невозможно создать папку кэша: { $detail }
container-bad-magic = Это не демозапись Half-Life.
container-frame-bad-time = Запись повреждена на кадре { $frame } части { $entry }: у кадра неверное время.
container-frame-missing-terminal = Часть { $entry } записи заканчивается без завершающей записи.
container-frame-negative-length = Запись повреждена на кадре { $frame } части { $entry }.
container-frame-over-limit = Кадр { $frame } части { $entry } больше допустимого.
container-frame-truncated = Запись обрывается на кадре { $frame } части { $entry }.
container-frame-unknown-tag = Запись повреждена на кадре { $frame } части { $entry }: неизвестный тип кадра.
container-io = Не удаётся прочитать файл: { $detail }
container-no-directory = Запись не была завершена, поэтому у неё нет оглавления.
container-range = Оглавление записи указывает за пределы файла.
container-too-short = Файл слишком короткий для демозаписи.
coverage-hltv = Запись прокси наблюдателя. Записывающего игрока нет, события следуют за видом прокси.
coverage-mostly-spectating = Записывающий наблюдал { $percent } % времени воспроизведения, поэтому большинство событий — это действия других игроков так, как их показывал наблюдаемый вид.
coverage-no-local-events = Нет локальных событий. Действия записывающего отсутствуют, кроме тех, что прислал сервер.
coverage-singleplayer = Одиночная игра. Остальные участники — монстры или объекты карты, ленты убийств нет.
coverage-unknown = Запись не удалось разобрать настолько, чтобы определить её тип.
coverage-visible-only = События других игроков доходят до этого клиента, только пока сервер считает их видимыми, поэтому их количество — нижняя граница.
decode-failed = Декодирование остановлено на { $time } (кадр { $frame }): { $detail }
export-active-effect = В момент начала идёт { $effect }, его пока нельзя перенести.
export-client-dropped-packets = В { $time } игра отбросила опоздавшие пакеты, а нарезка пока не может это воспроизвести.
export-cross-segment = В { $time }, внутри диапазона, начинается новая сессия.
export-destination-exists = { $path } уже существует. Выберите другое имя.
export-empty-range = В выбранном диапазоне нечего экспортировать.
export-io = Не удаётся записать { $path }: { $detail }
export-missing-delta-base = В записи нет состояния, нужного до начала (последовательность { $sequence }).
export-missing-parent = Папка { $path } не существует.
export-range-beyond-end = Конец { $end } позже конца записи, { $segment_end }.
export-same-as-source = Место назначения совпадает с исходной демозаписью.
export-source-changed = { $path } изменился во время экспорта.
export-source-decode-failed = Исходную демозапись не удаётся декодировать: { $detail }
export-source-failed-crosscheck = Исходная демозапись не прошла перекрёстную проверку, поэтому восстановленному состоянию нельзя доверять.
export-unknown-persistent-message = { $name } до начала задаёт состояние, которое нарезка пока не может перенести.
export-unproven-state = Состояние на { $time } опирается на выведенную базу и не доказано.
export-unrepresentable-time-field = Время в начале нельзя записать точно: { $detail }
export-unsupported = Не поддерживается: { $detail }
export-validation-failed = Экспортированный файл не прошёл проверку: { $detail }
io-not-found = Файл или папка не найдены.
io-other = { $detail }
io-permission-denied = Доступ запрещён.
io-unexpected-end = Файл неожиданно закончился.
protocol-unsupported = Протокол демо { $demo }/{ $network } не поддерживается, только 5/48.
protocol-unsupported-network = Сервер перешёл на сетевой протокол { $network }, который не поддерживается.

## Просмотр демо

demo-close = Закрыть
demo-loading = Разбор { $name }…
timeline-fit = Вместить
show-shots = Выстрелы
show-actions = Другие действия
show-kills = Убийства
show-switches = Смена оружия
lane-unknown = Неизвестный игрок
lane-recorder = { $name } (записывал)
column-time = Время
column-player = Игрок
column-weapon = Оружие
column-event = Событие
event-kill = Убил { $victim }
event-switch = Смена оружия
action-swing = Удар
action-fire = Выстрел
action-launch = Запуск
action-charge-activity = Зарядка
action-charge-stop = Конец зарядки
action-beam-start = Начало луча
action-beam-sustain = Луч
action-beam-stop = Конец луча
action-placement-attempt = Попытка установки
action-release-attempt = Попытка выпуска
mode-primary = основной
mode-charged = заряженный
mode-zoomed = с прицелом
mode-unzoomed = без прицела
mode-grenade = граната
mode-bullets = пули
mode-single = одиночный
mode-double = двойной
mode-rapid = быстрый
demo-events = { $count ->
    [one] { $count } событие
    [few] { $count } события
    [many] { $count } событий
   *[other] { $count } события
}

column-date = Дата
column-score = Счёт
filter-from = С
filter-to = По
date-from-name = Дата из имени файла
date-from-modified = Дата последнего изменения файла, она может измениться при копировании или скачивании
timeline-best-action = Золотая минута
timeline-windows = Самые жаркие моменты
timeline-window = №{ $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Итоговый счёт на последнем убийстве { $time }
scores-at = Счёт на { $time }
scores-show-final = Показать итоговый счёт
scores-frags = Фраги
scores-deaths = Смерти

show-timeline = Показать шкалу времени
viewer-play = Воспроизвести
viewer-pause = Пауза
viewer-volume = Громкость
viewer-no-sound = Нет звука
viewer-play-backward = Перемотка назад
viewer-step-back = Предыдущий кадр
viewer-step-forward = Следующий кадр
viewer-rate = { $rate }×
viewer-shortcut = Клавиша: { $key }
key-space = Пробел
viewer-map-failed = Не удалось загрузить карту { $map }
viewer-stopped = Воспроизведение остановилось раньше времени: { $reason }
viewer-missing = Файлы, которые не удалось прочитать: { $count }
event-previous = Предыдущее событие
event-next = Следующее событие
event-play = Воспроизвести событие
legend-shot = Выстрел
legend-charge = Зарядка
legend-kill = Убийство
legend-death = Смерть
legend-switch = Смена оружия
timeline-help = Каждая строка — игрок. Колесо мыши масштабирует, перетаскивание сдвигает, двойной щелчок показывает всё, щелчок по метке выбирает её.

column-headshots = Хедшоты
filter-headshots = Убийства в голову
event-headshot = в голову
scores-headshots = ХШ

column-accuracy = Точность
filter-min-accuracy = Точность не ниже (%)
scores-accuracy = Точн.

help-headshots = Убийства попаданием в голову, определённые по анимации смерти жертвы. Взрывы не считаются. Показываются только убийства записывающего, потому что другие игроки часто убивают вне его поля зрения. В дуэли показываются и убийства соперника, ведь все они — убийства записывающего.
help-headshots-column = Убийства записывающего попаданием в голову, определённые по анимации смерти жертвы. Взрывы не считаются. Убийства других игроков не учитываются, потому что часто происходят вне поля зрения записи.
help-accuracy = Доля выстрелов, попавших в цель, по крови, которую попадание оставляет на линии прицела, за вычетом случайных совпадений. Учитываются glock, python, пули MP5, дробовик, арбалет с прицелом и прямые попадания луча гаусса. Взрывы и отражения гаусса не оставляют крови и не учитываются. Показывается только для записывающего и не меньше чем после 50 выстрелов.
help-headshot-filter = Демозаписи хотя бы с одним убийством в голову, которое видно в записи, любым игроком.
help-accuracy-filter = Демозаписи, где записывающий попал хотя бы такой долей своих выстрелов.

clips-title = Клипы
clips-new = Новый клип
clips-add-event = Клип { $start } – { $end }
clips-add-event-none = Клип выбранного события
clips-select-event = Сначала выберите событие в списке или на шкале времени.
clips-other-session = Клип не может начинаться здесь: у более ранней сессии записи те же отметки времени.
clips-add-window = Клип этого жаркого момента
clips-empty = Клипов пока нет. Выберите событие или один из самых жарких моментов и добавьте клип вокруг него. Сколько клип захватывает до и после события, задаётся в настройках. «Новый клип» добавляет клип, начало и конец которого вы вводите сами.
clip-default-name = Клип { $number }
clip-name = Название
clip-length = Длина { $duration }
clip-show = Показать
clip-remove = Удалить
clip-loop = Повтор
clip-time-help = Секунды (96), минуты и секунды (1:36) или часы, минуты и секунды (00:01:36.250)
clip-exact = Экспорт начнётся и закончится точно здесь.
clip-effective = Экспорт вырежет { $start } – { $end } ({ $duration }), по кадрам записи.
timeline-clip-help = Клипы — светло-голубые полосы. Перетащите край полосы, чтобы сдвинуть его.
settings-clip-padding = Запас вокруг клипа
settings-preroll = Секунд до события
settings-postroll = Секунд после события
export-start-first-group = Вырезка не может начинаться с самого первого кадра записи. Начните чуть позже.
export-start-during-loading = Начало приходится на загрузку карты. Начните чуть позже.

export-all = Экспортировать все клипы
export-one = Экспорт
export-cancel = Остановить экспорт
export-folder = Экспорт сохраняется в { $folder }.
export-folder-beside = Экспорт сохраняется рядом с демо, в { $folder }.
export-waiting = Ожидает экспорта…
export-running = Экспорт… Демо читается целиком, это может занять время.
export-cancelled = Не экспортирован: экспорт остановлен.
export-saved = Сохранено: { $path }. Снимков сверено с оригиналом: { $count }.
export-playdemo = В консоли игры:
export-copy-first = Скопируйте { $name } в папку мода, где лежит liblist.gam, затем в консоли игры:
export-copy = Копировать
settings-export-folder = Папка экспорта
settings-export-choose = Выбрать…
settings-viewer = Просмотр демозаписей
settings-viewer-game = Папка Half-Life
settings-viewer-game-none = Не выбрана. Выберите папку с valve, чтобы смотреть демозаписи в просмотре.
settings-gamma = Гамма
settings-brightness = Яркость
settings-texgamma = Гамма текстур
settings-lightgamma = Гамма света
settings-overbright = Пересвет
settings-gamma-hint = Одноимённые настройки игры (gamma, brightness, texgamma, lightgamma, gl_overbright). Задайте их как в игре, чтобы видеть ту же яркость.
settings-gamma-defaults = Значения игры
settings-fov-recorded = Поле зрения как в записи
settings-fov = Поле зрения
settings-viewmodel = Показывать оружие в руках
settings-see-through = Видеть игроков сквозь стены
settings-see-through-items = А также другие модели, например снаряды и предметы
settings-see-through-hint = Игроки за стенами видны силуэтами своего верхнего цвета, который командные игры задают по команде. Видно только то, что есть в демозаписи. Сервер отправляет игроков рядом с записывающим, а серверы, скрывающие игроков, которых записывающий не видит, не оставляют ничего для показа.
viewer-see-through = Видеть сквозь стены
settings-export-beside = Рядом с каждым демо
settings-export-hint = В папке мода, где лежит liblist.gam, например Half-Life\ag, клипы запускаются одной командой playdemo.

clips-store-moved = Не удалось прочитать файл клипов { $path }, поэтому он сохранён как { $moved }, а список клипов начат заново. { $detail }
clips-store-unreadable = Не удалось прочитать файл клипов { $path }: { $detail }. В этой сессии клипы не сохраняются, поэтому его содержимое не потеряется.
clips-save-failed = Не удалось сохранить клипы в { $path }: { $detail }
notice-dismiss = Закрыть

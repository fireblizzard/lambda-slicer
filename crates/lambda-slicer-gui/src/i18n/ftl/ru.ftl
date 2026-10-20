# Русский. Черновик без проверки

app-title = Lambda Slicer

## Панель инструментов

toolbar-open = Открыть демо…
toolbar-add-folder = Добавить папку…
toolbar-rescan = Пересканировать
toolbar-cancel = Отмена
toolbar-settings = Настройки
toolbar-rescan-busy = Сканирование уже идёт.
toolbar-rescan-nothing = Добавьте папку или откройте демо, чтобы сканировать.
toolbar-cancel-idle = Сканирование не идёт.
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
library-demos-of = { $shown } из { $total ->
    [one] { $total } демозаписи
    [few] { $total } демозаписей
    [many] { $total } демозаписей
   *[other] { $total } демозаписи
}
library-total = { $count ->
    [one] В библиотеке { $count } демозапись
    [few] В библиотеке { $count } демозаписи
    [many] В библиотеке { $count } демозаписей
   *[other] В библиотеке { $count } демозаписи
}
filters-title = Фильтры
filters-find = Поиск
filters-pov = Вид
filters-quality = Качество
filters-recorded = Дата записи
pov-any = Все
pov-playing = Игра
pov-spectating = Наблюдение
help-pov = «Игра» значит, что запись провела в наблюдении не больше половины воспроизведения, «Наблюдение» — больше половины.
columns-button = Столбцы
list-back = Назад к таблице библиотеки
sort-by = Сортировка
sort-ascending = По возрастанию
sort-descending = По убыванию
score-more-sides = { $count ->
    [one] ещё { $count } сторона
    [few] ещё { $count } стороны
    [many] ещё { $count } сторон
   *[other] ещё { $count } стороны
}
intensity-help = Накал { $score }. Полоса сравнивает его с самым высоким в библиотеке, { $max }.
value-unknown = Неизвестно
path-copy = Копировать путь
path-show = Показать в папке
path-show-failed = Не удалось открыть папку ({ $detail }). Скопируйте путь.

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
show-shots = Выстрелы
show-actions = Другие действия
show-kills = Убийства
show-switches = Смена оружия
lane-unknown = Неизвестный игрок
lane-recorder = { $name } (записывал)
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
timeline-window = №{ $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Итоговый счёт на последнем убийстве { $time }
scores-at = Счёт на { $time }
scores-show-final = Показать итоговый счёт
scores-frags = Фраги
scores-deaths = Смерти

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
timeline-title = Шкала времени
timeline-lanes = Строки игроков
timeline-zoom-in = Приблизить
timeline-zoom-out = Отдалить
timeline-fit-all = Вместить всю демозапись
timeline-gestures = Щёлкните по метке, чтобы выбрать её событие, или в любом другом месте, чтобы перейти туда. Перетаскивайте линейку или указатель воспроизведения для перемотки, а в другом месте — для сдвига. Колесо мыши масштабирует вокруг курсора, а двойной щелчок показывает всю демозапись. Перетащите край клипа, чтобы сдвинуть его, или щёлкните по клипу, чтобы изменить его.
timeline-density = Затенённые столбцы показывают, сколько показанных событий приходится на каждый отрезок, по сравнению с самым насыщенным отрезком демозаписи.
timeline-density-count = { $count ->
    [one] Здесь { $count } показанное событие
    [few] Здесь { $count } показанных события
    [many] Здесь { $count } показанных событий
   *[other] Здесь { $count } показанного события
}
timeline-session-count = Сессия { $session }: { $count }
timeline-choose = События в этой точке
legend-clip = Клип
event-session = Сессия { $session }
event-unreachable = Это событие в более поздней сессии, которая заново использует те же отметки времени, поэтому просмотр не может к нему перейти.
event-hidden = Выбранное событие скрыто фильтрами.
highlight-jump = Перейти к хайлайту…
highlight-help = Хайлайты ранжируют всю демозапись по накалу действия, что бы ни показывали фильтры.
kills-with = Убийства с
refine-midair = Midair
refine-speed = Скорость
refine-wallgauss = Wallgauss
refine-clear = Сбросить
refine-airborne = В воздухе
refine-either = Любой
midair-both = Оба
midair-one = Только один
help-midair-either = Убийца, жертва или оба были в воздухе. Убийцу проверяют в момент выстрела, связанного с убийством, а без него — в момент убийства, жертву — в момент смерти.
help-midair-both = Убийца и жертва оба были в воздухе, каждый в момент проверки.
help-midair-one = В воздухе был ровно один из них, а другой точно был на земле.
speed-at-least = Не менее
speed-unit = ups
speed-killer = Убийца
speed-victim = Жертва
help-speed = Горизонтальная скорость в юнитах в секунду, параллельно земле. Скорость убийцы измеряется в момент выстрела, связанного с убийством, а без него — в момент убийства, жертвы — в момент смерти. Скорости других игроков — оценки.
wallgauss-weapon = Wallgauss относится только к убийствам из гаусса, а фильтр оружия показывает { $weapon }.
wallgauss-checking = Проверка Wallgauss… { $examined } из { $total } убийств
wallgauss-needs-files = Для Wallgauss нужны файлы игры, чтобы трассировать выстрелы по карте.
wallgauss-failed = Не удалось проверить Wallgauss. { $reason }
help-wallgauss = Убийства из гаусса сквозь стену, определённые трассировкой каждого выстрела по карте так же, как это делает сервер. Убийство, которое нельзя определить, например если жертва была вне поля зрения записи, считается неизвестным и не учитывается.
badge-wall = WALL
wall-yes-exit = Wallgauss определён реконструкцией: взрыв там, где луч вышел из стены толщиной { $thickness } юнитов.
wall-yes-beyond = Wallgauss определён реконструкцией: луч после прохождения сквозь стену толщиной { $thickness } юнитов.
wall-limits = Восстановлено по выстрелу и снимку, а не записано сервером. Игроки трассируются как боксы, и жертва могла быть в нескольких десятках юнитов от места, где её показывает снимок.
wall-no-direct = Не Wallgauss: луч мог попасть в жертву раньше любой стены.
wall-no-reflection = Не Wallgauss: взрыв отражения мог попасть в жертву раньше любой стены.
wall-doubt-no-shot = Wallgauss неизвестен: выстрела убийцы нет в записи.
wall-doubt-no-map = Wallgauss неизвестен: не удалось прочитать файлы карты.
wall-doubt-unseen = Wallgauss неизвестен: жертва была вне поля зрения записи.
wall-doubt-brush = Wallgauss неизвестен: луч встретил дверь, платформу или другую браш-энтити, а они не трассируются.
wall-doubt-ambiguous = Wallgauss неизвестен: выстрел мог убить и сквозь стену, и без неё.
wall-doubt-unexplained = Wallgauss неизвестен: не найдено, как выстрел дошёл до жертвы.
wall-doubt-unconfirmed = Wallgauss неизвестен: выстрел мог достать жертву только сквозь стену, но это не точно.
wall-doubt-not-reached = Wallgauss неизвестен: запись заканчивается до выстрела.
wall-pending = Wallgauss ещё не проверен.
events-unknown = { $count ->
    [one] Для { $count } убийства не хватает данных, чтобы это определить
    [few] Для { $count } убийств не хватает данных, чтобы это определить
    [many] Для { $count } убийств не хватает данных, чтобы это определить
   *[other] Для { $count } убийства не хватает данных, чтобы это определить
}
help-unknown = Движение, которое запись не смогла увидеть, считается неизвестным, но никогда — нахождением на земле или неподвижностью, поэтому эти убийства не учитываются.
clip-it = В клип
badge-headshot = ХШ
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = суицид
badge-teamkill = тимкилл
badge-world = мир
air-both = Оба были в воздухе в момент проверки.
air-killer = Убийца был в воздухе в момент проверки.
air-victim = Жертва была в воздухе в момент проверки.
air-other-unknown = Был ли в воздухе другой, неизвестно.
speed-horizontal = Горизонтальная скорость, параллельно земле.
motion-line = { $who }: { $air }, { $speed }. { $how }, { $when }.
motion-who-killer = Убийца
motion-who-victim = Жертва
motion-air-yes = в воздухе
motion-air-no = на земле
motion-air-unknown = неизвестно, в воздухе ли
motion-speed-unknown = скорость неизвестна
motion-exact = Записано игрой
motion-trajectory = Оценено по движению
motion-unclear = Движение неясно
motion-not-visible = Вне поля зрения
motion-at-shot = в момент связанного выстрела
motion-at-kill = в момент смерти
motion-none = В анализе нет движения для этого убийства.
legend-shot = Выстрел
legend-charge = Зарядка
legend-kill = Убийство
legend-death = Смерть
legend-switch = Смена оружия

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

clips-new = Новый клип
clips-select-event = Сначала выберите событие в списке или на шкале времени.
clips-other-session = Клип не может начинаться здесь: у более ранней сессии записи те же отметки времени.
clips-add-window = Клип этого жаркого момента
clip-default-name = Клип { $number }
clip-name = Название
clip-length = Длина { $duration }
clip-remove = Удалить
clip-loop = Повтор
clip-time-help = Секунды (96), минуты и секунды (1:36) или часы, минуты и секунды (00:01:36.250)
clips-empty-hint = Клипов пока нет. Выберите событие и нажмите «В клип» или создайте клип на указателе воспроизведения кнопкой «Новый клип».
clips-new-help = Десятисекундный клип от указателя воспроизведения.
clip-in = Вход
clip-out = Выход
clip-set-in = Поставить начало на указатель воспроизведения
clip-set-out = Поставить конец на указатель воспроизведения
clip-no-playhead = Нет указателя воспроизведения, с которого можно взять время.
clip-play = Воспроизвести клип один раз
clip-show-on-timeline = Показать на шкале времени
export-destination = Изменить папку экспорта
export-skipped = Пропущено. { $reason }
export-earlier-range = Этот экспорт охватывает { $start } – { $end }, диапазон до изменения клипа.
scores-player = Игрок
scores-none = В записи нет счёта.
clip-effective = Экспорт вырежет { $start } – { $end } ({ $duration }), по кадрам записи.
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
settings-viewer-game-none = Не выбрана
settings-viewer-game-hint = Папка, в которой лежит valve. Просмотр воспроизводит демозаписи с картами, моделями и звуками игры из неё.
settings-executable = Исполняемый файл игры
settings-executable-hint = Помогает найти папку Half-Life. Просмотр сам читает файлы игры и никогда её не запускает.
settings-executable-not-set = Не задан, поэтому в просмотре нет ни картинки, ни звука
settings-executable-optional = Не задан и не нужен, пока в папке Half-Life есть игра
settings-executable-missing = { $path } больше нет. Просмотр по-прежнему читает папку Half-Life.
settings-game-override = Этот запуск использует { $path }, заданную через --game. Выбор здесь вступит в силу со следующего запуска.
settings-browse = Обзор…
dialog-executables = Программы
setup-title = Настройте просмотр демозаписей
setup-body = Для картинки и звука нужны файлы Half-Life. Укажите Lambda Slicer, где установлена игра, и он будет читать их оттуда.
setup-locate-exe = Найти hl.exe…
setup-locate-folder = Найти Half-Life…
setup-settings = Настройки
setup-still-works = Шкала времени, события и клипы работают и без них.
setup-relocate = В { $folder } больше нет Half-Life. Найдите её снова.
setup-not-executable = { $name } — не { $executable }. Выберите сам исполняемый файл игры, а не ярлык.
setup-not-installed = В { $folder } нет установки Half-Life с папкой valve.
tab-events = События
tab-clips = Клипы
tab-scoreboard = Счёт
list-hide = Скрыть список демозаписей
list-show = Показать список демозаписей
demo-details = Подробности
viewer-start = В начало
viewer-mute = Выключить звук
viewer-unmute = Включить звук
viewer-picture = Картинка и настройки просмотра
viewer-needs-files = Нужны файлы игры
viewer-no-sound-why = Нет звука: { $reason }
settings-tab-general = Общие
settings-section-folders = Папки
settings-folders-hint = Папки, в которых библиотека ищет демозаписи, включая вложенные.
settings-clip-padding-hint = Сколько клип, созданный вокруг события, захватывает до и после него.
settings-section-picture = Изображение
settings-section-sound = Звук
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

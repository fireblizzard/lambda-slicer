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
settings-workers = Демозаписей одновременно
settings-workers-hint = Если анализировать больше одновременно, всё закончится быстрее, но компьютер будет сильнее нагружен.
settings-cache = Где хранится библиотека
settings-sort = Порядок при запуске
settings-sort-hint = Как сортируются демозаписи при каждом запуске Lambda Slicer. Щелчок по заголовку столбца всё равно сортирует по нему, пока вы не закроете Lambda Slicer.
settings-folders = Папки библиотеки
settings-folders-empty = Папки ещё не добавлены.
settings-remove = Удалить

## Библиотека

library-empty = Добавьте папку или откройте демо, чтобы начать.
library-no-matches = Нет демозаписей, подходящих под эти фильтры.
library-no-cache = На этом компьютере негде хранить библиотеку.
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
path-copy = Копировать расположение файла
path-show = Показать в папке
path-show-failed = Не удалось открыть папку ({ $detail }). Скопируйте расположение файла.

filter-search = Поиск по именам файлов и папкам
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
kind-hltv = HLTV
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

scan-progress = Найдено { $found }, готово { $done } (уже известно { $cached }, разобрано { $analysed }), прочитано { $megabytes } МБ
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

analysis-panicked = Анализ остановлен из-за сбоя в Lambda Slicer: { $detail }
cache-cancelled = Анализ отменён, поэтому ничего не сохранено.
cache-database = Невозможно использовать сохранённую библиотеку: { $detail }
cache-directory = Невозможно создать папку для сохранённой библиотеки: { $detail }
container-bad-magic = Это не демозапись Half-Life.
container-frame-bad-time = Запись повреждена на кадре { $frame } части { $entry }: у кадра испорчено время.
container-frame-missing-terminal = Часть { $entry } записи заканчивается неправильно.
container-frame-negative-length = Запись повреждена на кадре { $frame } части { $entry }.
container-frame-over-limit = Кадр { $frame } части { $entry } больше допустимого.
container-frame-truncated = Запись обрывается на кадре { $frame } части { $entry }.
container-frame-unknown-tag = Запись повреждена на кадре { $frame } части { $entry }: в нём что-то, чего Lambda Slicer не знает.
container-io = Не удаётся прочитать файл: { $detail }
container-no-directory = Запись не была завершена, поэтому у неё нет оглавления.
container-range = Оглавление записи указывает за пределы файла.
container-too-short = Файл слишком короткий для демозаписи.
coverage-hltv = Запись HLTV. Её не записывал игрок, поэтому события следуют за тем, что смотрел HLTV.
coverage-mostly-spectating = Записывающий наблюдал { $percent } % времени воспроизведения, поэтому большинство событий — это действия других игроков, какими они были видны при наблюдении за ними.
coverage-no-local-events = Собственные действия записывающего не сохранились в этой демозаписи, поэтому видны только те, что прислал сервер.
coverage-singleplayer = Одиночная игра. Всё остальное — монстры или части карты, ленты убийств нет.
coverage-unknown = Запись не удалось прочитать настолько, чтобы понять, какого она типа.
coverage-visible-only = Игра получает действия других игроков, только пока сервер считает, что записывающий может их видеть, поэтому их число может быть меньше, чем было на самом деле.
decode-failed = Чтение записи остановлено на { $time } (кадр { $frame }): { $detail }
export-active-effect = В момент начала ещё идёт { $effect }, и клип пока не может его перенести.
export-client-dropped-packets = В { $time } игра выбросила данные, пришедшие слишком поздно, а клип пока не может это повторить.
export-cross-segment = В { $time }, внутри клипа, сменилась карта или игрок переподключился, а клип не может проходить через это место.
export-destination-exists = { $path } уже существует. Выберите другое имя.
export-empty-range = В клипе нечего экспортировать.
export-io = Не удаётся записать { $path }: { $detail }
export-missing-delta-base = В записи не хватает данных до начала, которые нужны клипу (номер { $sequence }).
export-missing-parent = Папка { $path } не существует.
export-range-beyond-end = Конец { $end } позже конца записи, { $segment_end }.
export-same-as-source = Экспорт заменил бы демозапись, из которой он сделан.
export-source-changed = { $path } изменился во время экспорта.
export-source-decode-failed = Демозапись не удаётся прочитать: { $detail }
export-source-failed-crosscheck = При проверке то, что Lambda Slicer вычислил по этой демозаписи, не совпало с самой демозаписью, поэтому клипам из неё нельзя доверять.
export-unknown-persistent-message = { $name } до начала задаёт то, что клип пока не может перенести.
export-unproven-state = В { $time } запись опирается на данные, которые пришлось угадать, поэтому клип нельзя сделать точно.
export-unrepresentable-time-field = Время в начале нельзя сохранить точно: { $detail }
export-unsupported = Не поддерживается: { $detail }
export-validation-failed = Экспортированный клип не прошёл финальную проверку: { $detail }
io-not-found = Файл или папка не найдены.
io-other = { $detail }
io-permission-denied = Доступ запрещён.
io-unexpected-end = Файл неожиданно закончился.
protocol-unsupported = Эта демозапись сделана в неподдерживаемой версии игры (протокол { $demo }/{ $network }). Поддерживается только 5/48.
protocol-unsupported-network = Сервер перешёл на неподдерживаемую версию игры (протокол { $network }).

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
column-file-date = Дата файла
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
viewer-skip-back = Назад на 10 секунд
viewer-skip-forward = Вперёд на 10 секунд
viewer-rate = { $rate }×
viewer-shortcut = Клавиша: { $key }
key-space = Пробел
key-shift-left = Shift+Влево
key-shift-right = Shift+Вправо
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
timeline-gestures = Щёлкните по метке, чтобы выбрать её событие, или в любом другом месте, чтобы перейти туда. Тяните по времени вверху или за линию текущего момента, чтобы двигаться по демозаписи. Тяните в любом другом месте, чтобы сдвинуть вид. Колесо мыши приближает и отдаляет там, где стоит курсор, а двойной щелчок показывает всю демозапись. Перетащите край клипа, чтобы изменить его начало или конец, или щёлкните по клипу, чтобы изменить его.
timeline-density = Затенённые столбцы показывают, сколько из показанных событий происходит в каждом месте, по сравнению с самым насыщенным местом демозаписи.
timeline-density-count = { $count ->
    [one] Здесь { $count } показанное событие
    [few] Здесь { $count } показанных события
    [many] Здесь { $count } показанных событий
   *[other] Здесь { $count } показанного события
}
timeline-session-count = Часть { $session }: { $count }
timeline-choose = События в этой точке
legend-clip = Клип
event-session = Часть { $session }
event-unreachable = Это событие идёт после смены карты или переподключения, где часы демозаписи начались заново, поэтому просмотр не может к нему перейти.
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
help-midair-either = Убийца, жертва или оба были в воздухе. Убийцу проверяют в момент выстрела, связанного с убийством, а без него — в момент убийства, жертву — в момент смерти. Обычный прыжок, ниже 50 юнитов над полом, не считается. Для этого нужно выбрать папку Half-Life, чтобы знать, где пол.
help-midair-both = Убийца и жертва оба были в воздухе, каждый в момент проверки.
help-midair-one = В воздухе был ровно один из них, а другой точно был на земле.
speed-at-least = Не менее
speed-unit = ups
speed-killer = Убийца
speed-victim = Жертва
help-speed = Скорость по земле в ups, как её показывает спидометр, без движения вверх и вниз. Скорость убийцы измеряется в момент выстрела, связанного с убийством, а без него — в момент убийства, жертвы — в момент смерти. Скорости других игроков — оценки.
wallgauss-weapon = Wallgauss относится только к убийствам из гаусса, а фильтр оружия показывает { $weapon }.
wallgauss-checking = Проверка Wallgauss… { $examined } из { $total } убийств
wallgauss-needs-files = Для Wallgauss нужны файлы игры, чтобы знать, где на карте стены.
wallgauss-failed = Не удалось проверить Wallgauss. { $reason }
help-wallgauss = Убийства из гаусса сквозь стену, найденные так: каждый выстрел прослеживается по карте так же, как это делает сервер. Убийство, которое нельзя определить, например если жертва была вне поля зрения записи, считается неизвестным и не учитывается.
badge-wall = WG
wall-yes-exit = Wallgauss, вычислено по выстрелу: взрыв там, где луч вышел из стены толщиной { $thickness } юнитов.
wall-yes-beyond = Wallgauss, вычислено по выстрелу: луч попал после того, как прошёл сквозь стену толщиной { $thickness } юнитов.
wall-limits = Вычислено по выстрелу и положению игроков, потому что сервер этого не записывает. Игрок шириной 32 юнита считается коробкой, а жертва могла стоять на одну-две ширины тела в стороне от того места, где её показывает демозапись.
wall-no-direct = Не Wallgauss: луч мог попасть в жертву раньше любой стены.
wall-no-reflection = Не Wallgauss: взрыв отражения мог попасть в жертву раньше любой стены.
wall-doubt-no-shot = Wallgauss неизвестен: выстрела убийцы нет в записи.
wall-doubt-no-map = Wallgauss неизвестен: не удалось прочитать файлы карты.
wall-doubt-unseen = Wallgauss неизвестен: жертва была вне поля зрения записи.
wall-doubt-brush = Wallgauss неизвестен: луч попал в дверь, лифт или другую особую часть карты, а они не проверяются.
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
badge-clip = клип
clip-of = Клип из { $name }
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
filter-midair = Midair-убийства
event-headshot = в голову
scores-headshots = ХШ

column-accuracy = Точность
filter-min-accuracy = Точность не ниже (%)
scores-accuracy = Точн.

help-headshots = Убийства попаданием в голову, определённые по анимации смерти жертвы. Взрывы не считаются. Показываются только убийства записывающего, потому что другие игроки часто убивают вне его поля зрения. В дуэли показываются и убийства соперника, ведь все они — убийства записывающего.
help-headshots-column = Убийства записывающего попаданием в голову, определённые по анимации смерти жертвы. Взрывы не считаются. Убийства других игроков не учитываются, потому что часто происходят вне поля зрения записи.
help-date-column = Когда был записан матч. Дата берётся из имени файла, если в нём есть дата вида 20230910_214553, как у записей, которые игра делает сама. Иначе это дата файла, которая может измениться при копировании или скачивании.
help-file-date = Когда файл был сохранён. Для экспортированного клипа — когда он был экспортирован.
help-accuracy = Доля выстрелов, попавших в цель, по крови, которую попадание оставляет на линии прицела, за вычетом случайных совпадений. Учитываются glock, python, пули MP5, дробовик, арбалет с прицелом и прямые попадания луча гаусса. Взрывы и отражения гаусса не оставляют крови и не учитываются. Показывается только для записывающего и не меньше чем после 50 выстрелов.
help-headshot-filter = Демозаписи хотя бы с одним убийством в голову, которое видно в записи, любым игроком.
help-midair-filter = Демозаписи хотя бы с одним убийством, при котором убийца или жертва были в воздухе, на высоте не меньше 50 юнитов над полом. Без выбранной папки Half-Life считается и обычный прыжок.
help-accuracy-filter = Демозаписи, где записывающий попал хотя бы такой долей своих выстрелов.

clips-new = Новый клип
clips-select-event = Сначала выберите событие в списке или на шкале времени.
clips-other-session = Клип не может начинаться здесь: после смены карты или переподключения часы демозаписи начались заново, и у более ранней части те же отметки времени.
clips-add-window = Клип этого жаркого момента
clip-default-name = Клип { $number }
clip-name = Название
clip-length = Длина { $duration }
clip-remove = Удалить
clip-loop = Повтор
clip-time-help = Секунды (96), минуты и секунды (1:36) или часы, минуты и секунды (00:01:36.250)
clips-empty-hint = Клипов пока нет. Выберите событие и нажмите «В клип» или нажмите «Новый клип», чтобы начать клип с текущего момента.
clips-new-help = Десятисекундный клип с текущего момента.
clip-in = Начало
clip-out = Конец
clip-set-in = Начать клип с текущего момента
clip-set-out = Закончить клип на текущем моменте
clip-no-playhead = Просмотр недоступен, поэтому нет текущего момента, который можно взять.
clip-play = Воспроизвести клип один раз
clip-show-on-timeline = Показать на шкале времени
export-destination = Изменить папку экспорта
export-skipped = Пропущено. { $reason }
export-earlier-range = Этот экспорт охватывает { $start } – { $end }, клип в том виде, каким он был до ваших изменений.
scores-player = Игрок
scores-none = В записи нет счёта.
clip-effective = Экспорт пойдёт { $start } – { $end } ({ $duration }), настолько точно, насколько позволяет запись.
settings-clip-padding = Запас вокруг клипа
settings-preroll = Секунд до события
settings-postroll = Секунд после события
export-start-first-group = Клип не может начинаться с самого начала записи. Начните чуть позже.
export-start-during-loading = Начало приходится на загрузку карты. Начните чуть позже.

export-all = Экспортировать все клипы
export-one = Экспорт
export-cancel = Остановить экспорт
export-folder = Экспорт сохраняется в { $folder }.
export-folder-beside = Экспорт сохраняется рядом с демо, в { $folder }.
export-waiting = Ожидает экспорта…
export-running = Экспорт… Демо читается целиком, это может занять время.
export-cancelled = Не экспортирован: экспорт остановлен.
export-saved = Сохранено: { $path }. Моментов сверено с оригиналом: { $count }.
export-playdemo = В консоли игры:
export-copy-first = Скопируйте { $name } в папку мода, где лежит liblist.gam, затем в консоли игры:
export-copy = Копировать
settings-export-folder = Папка экспорта
settings-export-choose = Выбрать…
settings-file-names = Имена файлов
settings-clip-name-files = Использовать название клипа как имя файла
settings-demo-name-first = Начинать с названия демо
settings-viewer = Просмотр демозаписей
settings-viewer-game = Папка Half-Life
settings-viewer-game-none = Не выбрана
settings-viewer-game-hint = Папка, в которой лежит valve. Просмотр воспроизводит демозаписи с картами, моделями и звуками игры из неё.
settings-executable = Программа игры
settings-executable-hint = Помогает найти папку Half-Life. Просмотр сам читает файлы игры и никогда её не запускает.
settings-executable-not-set = Не задан, поэтому в просмотре нет ни картинки, ни звука
settings-executable-optional = Не задан и не нужен, пока в папке Half-Life есть игра
settings-executable-missing = { $path } больше нет. Просмотр по-прежнему читает папку Half-Life.
settings-game-override = Сейчас Lambda Slicer использует { $path }, потому что был запущен с --game. Выбор здесь вступит в силу при следующем запуске.
settings-browse = Обзор…
dialog-executables = Программы
setup-title = Настройте просмотр демозаписей
setup-body = Для картинки и звука нужны файлы Half-Life. Укажите Lambda Slicer, где установлена игра, и он будет читать их оттуда.
setup-locate-exe = Найти hl.exe…
setup-locate-folder = Найти Half-Life…
setup-settings = Настройки
setup-still-works = Шкала времени, события и клипы работают и без них.
setup-relocate = В { $folder } больше нет Half-Life. Найдите её снова.
setup-not-executable = { $name } — не { $executable }. Выберите саму программу игры, а не ярлык.
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
settings-tab-library = Библиотека
settings-tab-clips = Клипы
settings-section-game = Half-Life
settings-section-view = Вид
settings-language-system = Язык компьютера ({ $name })
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
settings-zoom-fov = Поле зрения при приближении
settings-viewmodel = Показывать оружие в руках
settings-see-through = Видеть игроков сквозь стены
settings-see-through-items = А также другие модели, например снаряды и предметы
settings-see-through-hint = Игроки за стенами видны силуэтами своего верхнего цвета, который командные игры задают по команде. Видно только то, что есть в демозаписи. Сервер отправляет игроков рядом с записывающим, а серверы, скрывающие игроков, которых записывающий не видит, не оставляют ничего для показа.
viewer-zoom = Приблизить вид
viewer-see-through = Видеть сквозь стены
settings-export-beside = Рядом с каждым демо
settings-export-hint = В папке мода, где лежит liblist.gam, например Half-Life\ag, клипы запускаются одной командой playdemo.
settings-file-names-hint = Экспортированный клип получает имя по своему демо и времени начала и конца, если вы не используете название клипа. Игра открывает только файлы, имена которых состоят из латинских букв, цифр, - и ., поэтому всё остальное заменяется на _. Если в названии клипа нет ни одного из этих символов, файл называется по времени.

clips-store-moved = Не удалось прочитать файл клипов { $path }, поэтому он сохранён как { $moved }, а список клипов начат заново. { $detail }
clips-store-unreadable = Не удалось прочитать файл клипов { $path }: { $detail }. Пока Lambda Slicer не закрыт, клипы не сохраняются, поэтому ничего в файле не потеряется.
clips-save-failed = Не удалось сохранить клипы в { $path }: { $detail }
notice-dismiss = Закрыть

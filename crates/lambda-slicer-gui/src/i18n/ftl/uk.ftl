# Українська. Чернетка без перевірки

app-title = Lambda Slicer

## Панель інструментів

toolbar-open = Відкрити демо…
toolbar-add-folder = Додати теку…
toolbar-rescan = Пересканувати
toolbar-cancel = Скасувати
toolbar-settings = Налаштування
toolbar-rescan-busy = Сканування вже триває.
toolbar-rescan-nothing = Додайте теку або відкрийте демо, щоб сканувати.
toolbar-cancel-idle = Сканування не триває.
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
library-demos-of = { $shown } з { $total ->
    [one] { $total } демозапису
    [few] { $total } демозаписів
    [many] { $total } демозаписів
   *[other] { $total } демозапису
}
library-total = { $count ->
    [one] У бібліотеці { $count } демозапис
    [few] У бібліотеці { $count } демозаписи
    [many] У бібліотеці { $count } демозаписів
   *[other] У бібліотеці { $count } демозапису
}
filters-title = Фільтри
filters-find = Пошук
filters-pov = Вигляд
filters-quality = Якість
filters-recorded = Дата запису
pov-any = Усі
pov-playing = Гра
pov-spectating = Спостереження
help-pov = «Гра» означає, що запис провів у спостереженні не більше половини відтворення, «Спостереження» — більше половини.
columns-button = Стовпці
list-back = Назад до таблиці бібліотеки
sort-by = Сортування
sort-ascending = За зростанням
sort-descending = За спаданням
score-more-sides = { $count ->
    [one] ще { $count } сторона
    [few] ще { $count } сторони
    [many] ще { $count } сторін
   *[other] ще { $count } сторони
}
intensity-help = Напруга { $score }. Смуга порівнює її з найвищою в бібліотеці, { $max }.
value-unknown = Невідомо
path-copy = Копіювати шлях
path-show = Показати в теці
path-show-failed = Не вдалося відкрити теку ({ $detail }). Скопіюйте шлях.

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
column-action = Напруга
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
details-action = Найвища напруга { NUMBER($score, minimumFractionDigits: 2) }

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
export-missing-parent = Теки { $path } не існує.
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

## Перегляд демо

demo-close = Закрити
demo-loading = Розбір { $name }…
show-shots = Постріли
show-actions = Інші дії
show-kills = Вбивства
show-switches = Зміна зброї
lane-unknown = Невідомий гравець
lane-recorder = { $name } (записував)
event-kill = Убив { $victim }
event-switch = Зміна зброї
action-swing = Удар
action-fire = Постріл
action-launch = Запуск
action-charge-activity = Заряджання
action-charge-stop = Кінець заряду
action-beam-start = Початок променя
action-beam-sustain = Промінь
action-beam-stop = Кінець променя
action-placement-attempt = Спроба встановлення
action-release-attempt = Спроба випуску
mode-primary = основний
mode-charged = заряджений
mode-zoomed = з прицілом
mode-unzoomed = без прицілу
mode-grenade = граната
mode-bullets = кулі
mode-single = одиночний
mode-double = подвійний
mode-rapid = швидкий
demo-events = { $count ->
    [one] { $count } подія
    [few] { $count } події
    [many] { $count } подій
   *[other] { $count } події
}

column-date = Дата
column-score = Рахунок
filter-from = З
filter-to = По
date-from-name = Дата з назви файлу
date-from-modified = Дата останньої зміни файлу, вона може змінитися під час копіювання чи завантаження
timeline-best-action = Золота хвилина
timeline-window = №{ $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Підсумковий рахунок на останньому вбивстві { $time }
scores-at = Рахунок на { $time }
scores-show-final = Показати підсумковий рахунок
scores-frags = Фраги
scores-deaths = Смерті

viewer-play = Відтворити
viewer-pause = Пауза
viewer-volume = Гучність
viewer-no-sound = Немає звуку
viewer-play-backward = Перемотати назад
viewer-step-back = Попередній кадр
viewer-step-forward = Наступний кадр
viewer-rate = { $rate }×
viewer-shortcut = Клавіша: { $key }
key-space = Пробіл
viewer-map-failed = Не вдалося завантажити мапу { $map }
viewer-stopped = Відтворення зупинилося раніше часу: { $reason }
viewer-missing = Файли, які не вдалося прочитати: { $count }
event-previous = Попередня подія
event-next = Наступна подія
event-play = Відтворити подію
timeline-title = Шкала часу
timeline-lanes = Рядки гравців
timeline-zoom-in = Наблизити
timeline-zoom-out = Віддалити
timeline-fit-all = Умістити весь демозапис
timeline-gestures = Клацніть мітку, щоб вибрати її подію, або будь-де інде, щоб перейти туди. Перетягуйте лінійку або курсор відтворення для перемотування, а деінде — для зсуву. Коліщатко масштабує навколо вказівника, а подвійне клацання показує весь демозапис. Перетягніть край кліпу, щоб зсунути його, або клацніть кліп, щоб змінити його.
timeline-density = Затінені стовпчики показують, скільки показаних подій припадає на кожен відрізок, порівняно з найнасиченішим відрізком демозапису.
timeline-density-count = { $count ->
    [one] Тут { $count } показана подія
    [few] Тут { $count } показані події
    [many] Тут { $count } показаних подій
   *[other] Тут { $count } показаної події
}
timeline-session-count = Сесія { $session }: { $count }
timeline-choose = Події в цій точці
legend-clip = Кліп
event-session = Сесія { $session }
event-unreachable = Ця подія в пізнішій сесії, яка знову використовує ті самі позначки часу, тому перегляд не може до неї перейти.
event-hidden = Вибрану подію приховано фільтрами.
highlight-jump = Перейти до хайлайту…
highlight-help = Хайлайти ранжують увесь демозапис за напругою дії, хай що показують фільтри.
kills-with = Вбивства з
refine-midair = Midair
refine-speed = Швидкість
refine-wallgauss = Wallgauss
refine-clear = Скинути
refine-airborne = У повітрі
refine-either = Будь-хто
midair-both = Обидва
midair-one = Лише один
help-midair-either = Вбивця, жертва або обидва були в повітрі. Вбивцю перевіряють у момент пострілу, пов’язаного з вбивством, а без нього — у момент вбивства, жертву — у момент смерті.
help-midair-both = Вбивця і жертва обидва були в повітрі, кожен у момент перевірки.
help-midair-one = У повітрі був рівно один із них, а інший напевно був на землі.
speed-at-least = Не менше
speed-unit = ups
speed-killer = Вбивця
speed-victim = Жертва
help-speed = Горизонтальна швидкість у юнітах за секунду, паралельно землі. Швидкість вбивці вимірюється в момент пострілу, пов’язаного з вбивством, а без нього — у момент вбивства, жертви — у момент смерті. Швидкості інших гравців — оцінки.
wallgauss-weapon = Wallgauss стосується лише вбивств із гаусса, а фільтр зброї показує { $weapon }.
wallgauss-checking = Перевірка Wallgauss… { $examined } з { $total } вбивств
wallgauss-needs-files = Для Wallgauss потрібні файли гри, щоб трасувати постріли по мапі.
wallgauss-failed = Не вдалося перевірити Wallgauss. { $reason }
help-wallgauss = Вбивства з гаусса крізь стіну, визначені трасуванням кожного пострілу по мапі так само, як це робить сервер. Вбивство, яке не вдається визначити, наприклад якщо жертва була поза полем зору запису, вважається невідомим і не враховується.
badge-wall = WALL
wall-yes-exit = Wallgauss визначено реконструкцією: вибух там, де промінь вийшов зі стіни завтовшки { $thickness } юнітів.
wall-yes-beyond = Wallgauss визначено реконструкцією: промінь після проходження крізь стіну завтовшки { $thickness } юнітів.
wall-limits = Відновлено за пострілом і знімком, а не записано сервером. Гравці трасуються як бокси, і жертва могла бути за кілька десятків юнітів від місця, де її показує знімок.
wall-no-direct = Не Wallgauss: промінь міг влучити в жертву раніше за будь-яку стіну.
wall-no-reflection = Не Wallgauss: вибух відбиття міг влучити в жертву раніше за будь-яку стіну.
wall-doubt-no-shot = Wallgauss невідомий: пострілу вбивці немає в записі.
wall-doubt-no-map = Wallgauss невідомий: не вдалося прочитати файли мапи.
wall-doubt-unseen = Wallgauss невідомий: жертва була поза полем зору запису.
wall-doubt-brush = Wallgauss невідомий: промінь натрапив на двері, платформу або іншу браш-ентіті, а вони не трасуються.
wall-doubt-ambiguous = Wallgauss невідомий: постріл міг убити і крізь стіну, і без неї.
wall-doubt-unexplained = Wallgauss невідомий: не знайдено, як постріл дістався жертви.
wall-doubt-unconfirmed = Wallgauss невідомий: постріл міг дістати жертву лише крізь стіну, але це не напевно.
wall-doubt-not-reached = Wallgauss невідомий: запис закінчується до пострілу.
wall-pending = Wallgauss ще не перевірено.
events-unknown = { $count ->
    [one] Для { $count } вбивства бракує даних, щоб це визначити
    [few] Для { $count } вбивств бракує даних, щоб це визначити
    [many] Для { $count } вбивств бракує даних, щоб це визначити
   *[other] Для { $count } вбивства бракує даних, щоб це визначити
}
help-unknown = Рух, якого запис не зміг побачити, вважається невідомим, але ніколи — перебуванням на землі чи нерухомістю, тому ці вбивства не враховуються.
clip-it = У кліп
badge-headshot = ХШ
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = суїцид
badge-teamkill = тімкіл
badge-world = світ
air-both = Обидва були в повітрі в момент перевірки.
air-killer = Вбивця був у повітрі в момент перевірки.
air-victim = Жертва була в повітрі в момент перевірки.
air-other-unknown = Чи був у повітрі інший, невідомо.
speed-horizontal = Горизонтальна швидкість, паралельно землі.
motion-line = { $who }: { $air }, { $speed }. { $how }, { $when }.
motion-who-killer = Вбивця
motion-who-victim = Жертва
motion-air-yes = у повітрі
motion-air-no = на землі
motion-air-unknown = невідомо, чи в повітрі
motion-speed-unknown = швидкість невідома
motion-exact = Записано грою
motion-trajectory = Оцінено за рухом
motion-unclear = Рух неясний
motion-not-visible = Поза полем зору
motion-at-shot = у момент пов’язаного пострілу
motion-at-kill = у момент смерті
motion-none = В аналізі немає руху для цього вбивства.
legend-shot = Постріл
legend-charge = Заряджання
legend-kill = Вбивство
legend-death = Смерть
legend-switch = Зміна зброї

column-headshots = Хедшоти
filter-headshots = Вбивства в голову
event-headshot = в голову
scores-headshots = ХШ

column-accuracy = Влучність
filter-min-accuracy = Влучність не нижче (%)
scores-accuracy = Влуч.

help-headshots = Вбивства влучанням у голову, визначені за анімацією смерті жертви. Вибухи не враховуються. Показуються лише вбивства автора запису, бо інші гравці часто вбивають поза його полем зору. У дуелі показуються й вбивства суперника, адже всі вони — вбивства автора запису.
help-headshots-column = Вбивства автора запису влучанням у голову, визначені за анімацією смерті жертви. Вибухи не враховуються. Вбивства інших гравців не враховуються, бо часто стаються поза полем зору запису.
help-accuracy = Частка пострілів, що влучили, за кров’ю, яку влучання залишає на лінії прицілу, за винятком випадкових збігів. Враховуються glock, python, кулі MP5, дробовик, арбалет із прицілом і прямі влучання променя гаусса. Вибухи та відбиття гаусса не залишають крові й не враховуються. Показується лише для автора запису і щонайменше після 50 пострілів.
help-headshot-filter = Демозаписи щонайменше з одним вбивством у голову, яке видно в записі, будь-яким гравцем.
help-accuracy-filter = Демозаписи, де автор запису влучив щонайменше такою часткою пострілів.

clips-title = Кліпи
clips-new = Новий кліп
clips-add-event = Кліп { $start } – { $end }
clips-add-event-none = Кліп вибраної події
clips-select-event = Спершу виберіть подію у списку або на шкалі часу.
clips-other-session = Кліп не може починатися тут: у ранішої сесії запису ті самі позначки часу.
clips-add-window = Кліп цього гарячого моменту
clips-empty = Кліпів поки немає. Виберіть подію або один із найгарячіших моментів і додайте кліп навколо нього. Скільки кліп захоплює до та після події, задається в налаштуваннях. «Новий кліп» додає кліп, початок і кінець якого ви вводите самі.
clip-default-name = Кліп { $number }
clip-name = Назва
clip-length = Довжина { $duration }
clip-show = Показати
clip-remove = Видалити
clip-loop = Повтор
clip-time-help = Секунди (96), хвилини й секунди (1:36) або години, хвилини й секунди (00:01:36.250)
clip-exact = Експорт почнеться й закінчиться точно тут.
clip-effective = Експорт виріже { $start } – { $end } ({ $duration }), за кадрами запису.
settings-clip-padding = Запас навколо кліпу
settings-preroll = Секунд до події
settings-postroll = Секунд після події
export-start-first-group = Вирізка не може починатися з самого першого кадру запису. Почніть трохи пізніше.
export-start-during-loading = Початок припадає на завантаження мапи. Почніть трохи пізніше.

export-all = Експортувати всі кліпи
export-one = Експорт
export-cancel = Зупинити експорт
export-folder = Експорт зберігається в { $folder }.
export-folder-beside = Експорт зберігається поруч із демо, в { $folder }.
export-waiting = Очікує експорту…
export-running = Експорт… Демо читається повністю, це може тривати.
export-cancelled = Не експортовано: експорт зупинено.
export-saved = Збережено: { $path }. Знімків звірено з оригіналом: { $count }.
export-playdemo = У консолі гри:
export-copy-first = Скопіюйте { $name } у папку мода, де лежить liblist.gam, потім у консолі гри:
export-copy = Копіювати
settings-export-folder = Папка експорту
settings-export-choose = Вибрати…
settings-viewer = Перегляд демозаписів
settings-viewer-game = Тека Half-Life
settings-viewer-game-none = Не вибрано
settings-viewer-game-hint = Тека, у якій лежить valve. Перегляд відтворює демозаписи з мапами, моделями та звуками гри з неї.
settings-executable = Виконуваний файл гри
settings-executable-hint = Допомагає знайти теку Half-Life. Перегляд сам читає файли гри й ніколи її не запускає.
settings-executable-not-set = Не задано, тому в перегляді немає ні зображення, ні звуку
settings-executable-optional = Не задано й не потрібно, поки в теці Half-Life є гра
settings-executable-missing = { $path } більше немає. Перегляд і далі читає теку Half-Life.
settings-game-override = Цей запуск використовує { $path }, задану через --game. Вибір тут діятиме з наступного запуску.
settings-browse = Огляд…
dialog-executables = Програми
setup-title = Налаштуйте перегляд демозаписів
setup-body = Для зображення й звуку потрібні файли Half-Life. Вкажіть Lambda Slicer, де встановлено гру, і він читатиме їх звідти.
setup-locate-exe = Знайти hl.exe…
setup-locate-folder = Знайти Half-Life…
setup-settings = Налаштування
setup-still-works = Шкала часу, події та кліпи працюють і без них.
setup-relocate = У { $folder } більше немає Half-Life. Знайдіть її знову.
setup-not-executable = { $name } — не { $executable }. Виберіть сам виконуваний файл гри, а не ярлик.
setup-not-installed = У { $folder } немає встановлення Half-Life з текою valve.
tab-events = Події
tab-clips = Кліпи
tab-scoreboard = Рахунок
list-hide = Сховати список демозаписів
list-show = Показати список демозаписів
demo-details = Подробиці
viewer-start = На початок
viewer-mute = Вимкнути звук
viewer-unmute = Увімкнути звук
viewer-picture = Зображення й налаштування перегляду
viewer-needs-files = Потрібні файли гри
viewer-no-sound-why = Немає звуку: { $reason }
settings-tab-general = Загальні
settings-section-folders = Теки
settings-folders-hint = Теки, у яких бібліотека шукає демозаписи, разом із вкладеними.
settings-clip-padding-hint = Скільки кліп, створений навколо події, захоплює до і після неї.
settings-section-picture = Зображення
settings-section-sound = Звук
settings-gamma = Гама
settings-brightness = Яскравість
settings-texgamma = Гама текстур
settings-lightgamma = Гама світла
settings-overbright = Пересвіт
settings-gamma-hint = Однойменні налаштування гри (gamma, brightness, texgamma, lightgamma, gl_overbright). Задайте їх як у грі, щоб бачити ту саму яскравість.
settings-gamma-defaults = Значення гри
settings-fov-recorded = Поле зору як у записі
settings-fov = Поле зору
settings-viewmodel = Показувати зброю в руках
settings-see-through = Бачити гравців крізь стіни
settings-see-through-items = А також інші моделі, як-от снаряди й предмети
settings-see-through-hint = Гравці за стінами видно силуетами їхнього верхнього кольору, який командні ігри задають за командою. Видно лише те, що є в демозаписі. Сервер надсилає гравців поруч із тим, хто записує, а сервери, що приховують гравців, яких той не бачить, не залишають нічого для показу.
viewer-see-through = Бачити крізь стіни
settings-export-beside = Поруч із кожним демо
settings-export-hint = У папці мода, де лежить liblist.gam, наприклад Half-Life\ag, кліпи запускаються однією командою playdemo.

clips-store-moved = Не вдалося прочитати файл кліпів { $path }, тому його збережено як { $moved }, а список кліпів почато заново. { $detail }
clips-store-unreadable = Не вдалося прочитати файл кліпів { $path }: { $detail }. У цій сесії кліпи не зберігаються, тож його вміст не загубиться.
clips-save-failed = Не вдалося зберегти кліпи в { $path }: { $detail }
notice-dismiss = Закрити

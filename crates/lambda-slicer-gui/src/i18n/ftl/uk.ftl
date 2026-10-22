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
settings-workers = Демозаписів одночасно
settings-workers-hint = Якщо аналізувати більше одночасно, усе закінчиться швидше, але комп’ютер буде більше навантажений.
settings-cache = Де зберігається бібліотека
settings-sort = Порядок під час запуску
settings-sort-hint = Як сортуються демозаписи під час кожного запуску Lambda Slicer. Клацання на заголовок стовпця все одно сортує за ним, доки ви не закриєте Lambda Slicer.
settings-start-filters = Фільтри під час запуску
settings-start-filters-hint = Під час кожного запуску Lambda Slicer у бібліотеці ввімкнено ці фільтри, щоб короткі демозаписи, які ви не хочете бачити, не заважали. Фільтри все одно можна змінювати, доки ви не закриєте Lambda Slicer.
settings-folders = Теки бібліотеки
settings-folders-empty = Теки ще не додано.
settings-remove = Вилучити

## Бібліотека

library-empty = Додайте теку або відкрийте демо, щоб почати.
library-no-matches = Жоден демозапис не відповідає цим фільтрам.
library-no-cache = На цьому комп’ютері ніде зберігати бібліотеку.
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
path-copy = Копіювати розташування файлу
path-show = Показати в теці
path-show-failed = Не вдалося відкрити теку ({ $detail }). Скопіюйте розташування файлу.

filter-search = Пошук за назвами файлів і тек
filter-map = Мапа
filter-player = Гравець
filter-weapon-any = Будь-яка зброя
filter-min-minutes = Не коротше (хвилин)
filter-min-kills = Не менше (вбивств)
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
kind-hltv = HLTV
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

scan-progress = Знайдено { $found }, готово { $done } (уже відомо { $cached }, розібрано { $analysed }), прочитано { $megabytes } МБ
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

analysis-panicked = Аналіз зупинено через збій у Lambda Slicer: { $detail }
cache-cancelled = Аналіз скасовано, тому нічого не збережено.
cache-database = Неможливо використати збережену бібліотеку: { $detail }
cache-directory = Неможливо створити теку для збереженої бібліотеки: { $detail }
container-bad-magic = Це не демозапис Half-Life.
container-frame-bad-time = Запис пошкоджено на кадрі { $frame } частини { $entry }: у кадру зіпсований час.
container-frame-missing-terminal = Частина { $entry } запису закінчується неправильно.
container-frame-negative-length = Запис пошкоджено на кадрі { $frame } частини { $entry }.
container-frame-over-limit = Кадр { $frame } частини { $entry } більший за допустимий.
container-frame-truncated = Запис обривається на кадрі { $frame } частини { $entry }.
container-frame-unknown-tag = Запис пошкоджено на кадрі { $frame } частини { $entry }: у ньому щось, чого Lambda Slicer не знає.
container-io = Не вдається прочитати файл: { $detail }
container-no-directory = Запис не було завершено, тому він не має змісту.
container-range = Зміст запису вказує за межі файлу.
container-too-short = Файл закороткий для демозапису.
coverage-hltv = Запис HLTV. Його не записував гравець, тому події стежать за тим, що дивився HLTV.
coverage-mostly-spectating = Автор запису спостерігав { $percent } % часу відтворення, тому більшість подій — це дії інших гравців, якими їх було видно під час спостереження за ними.
coverage-no-local-events = Власні дії автора запису не збереглися в цьому демозаписі, тому видно лише ті, що надіслав сервер.
coverage-singleplayer = Одиночна гра. Усе інше — монстри або частини мапи, стрічки вбивств немає.
coverage-unknown = Запис не вдалося прочитати настільки, щоб зрозуміти, якого він типу.
coverage-visible-only = Гра отримує дії інших гравців, лише поки сервер вважає, що автор запису може їх бачити, тому їхня кількість може бути меншою, ніж було насправді.
decode-failed = Читання запису зупинено на { $time } (кадр { $frame }): { $detail }
export-active-effect = На момент початку ще триває { $effect }, і кліп поки не може його перенести.
export-client-dropped-packets = О { $time } гра викинула дані, що надійшли надто пізно, а кліп поки не може це повторити.
export-cross-segment = О { $time }, усередині кліпу, змінилася мапа або гравець перепідключився, а кліп не може проходити через це місце.
export-destination-exists = { $path } уже існує. Виберіть іншу назву.
export-empty-range = У кліпі нічого експортувати.
export-io = Не вдається записати { $path }: { $detail }
export-missing-delta-base = У записі бракує даних до початку, потрібних кліпу (номер { $sequence }).
export-missing-parent = Теки { $path } не існує.
export-range-beyond-end = Кінець { $end } пізніший за кінець запису, { $segment_end }.
export-same-as-source = Експорт замінив би демозапис, з якого його зроблено.
export-source-changed = { $path } змінився під час експорту.
export-source-decode-failed = Демозапис не вдається прочитати: { $detail }
export-source-failed-crosscheck = Під час перевірки те, що Lambda Slicer обчислив за цим демозаписом, не збіглося з самим демозаписом, тому кліпам із нього не можна довіряти.
export-unknown-persistent-message = { $name } до початку задає те, що кліп поки не може перенести.
export-unproven-state = О { $time } запис спирається на дані, які довелося вгадати, тому кліп не можна зробити точно.
export-unrepresentable-time-field = Час на початку не можна зберегти точно: { $detail }
export-unsupported = Не підтримується: { $detail }
export-validation-failed = Експортований кліп не пройшов фінальну перевірку: { $detail }
io-not-found = Файл або теку не знайдено.
io-other = { $detail }
io-permission-denied = Доступ заборонено.
io-unexpected-end = Файл несподівано закінчився.
protocol-unsupported = Цей демозапис зроблено в непідтримуваній версії гри (протокол { $demo }/{ $network }). Підтримується лише 5/48.
protocol-unsupported-network = Сервер перейшов на непідтримувану версію гри (протокол { $network }).

## Перегляд демо

demo-close = Закрити
demo-loading = Розбір { $name }…
show-shots = Постріли
show-actions = Інші дії
show-kills = Вбивства
show-switches = Зміна зброї
show-chat = Чат
show-match = Матч
lane-unknown = Невідомий гравець
lane-recorder = { $name } (записував)
event-kill = Убив { $victim }
event-switch = Зміна зброї
event-vote-called = Запущено голосування «{ $subject }»
event-vote-cast = Голосування «{ $subject }»: { $change } ({ $tally })
event-vote-more-yes = { $count ->
    [one] на { $count } голос «за» більше
    [few] на { $count } голоси «за» більше
    [many] на { $count } голосів «за» більше
   *[other] на { $count } голосу «за» більше
}
event-vote-fewer-yes = { $count ->
    [one] на { $count } голос «за» менше
    [few] на { $count } голоси «за» менше
    [many] на { $count } голосів «за» менше
   *[other] на { $count } голосу «за» менше
}
event-vote-more-no = { $count ->
    [one] на { $count } голос «проти» більше
    [few] на { $count } голоси «проти» більше
    [many] на { $count } голосів «проти» більше
   *[other] на { $count } голосу «проти» більше
}
event-vote-fewer-no = { $count ->
    [one] на { $count } голос «проти» менше
    [few] на { $count } голоси «проти» менше
    [many] на { $count } голосів «проти» менше
   *[other] на { $count } голосу «проти» менше
}
event-vote-tally = { $yes } за, { $no } проти, { $undecided } не голосували
event-vote-accepted = Голосування «{ $subject }» прийнято
event-vote-denied = Голосування «{ $subject }» відхилено
event-timeout-called = Запитано таймаут
event-timeout-under-way = Таймаут почався
event-timeout-over = Таймаут закінчився
event-paused = Гру призупинено
event-resumed = Гру відновлено
event-sudden-death = Почалася раптова смерть
event-sudden-death-over = Раптова смерть закінчилася
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
column-file-date = Дата файлу
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
viewer-position = Позиція в демозаписі
viewer-skip-back = Назад на 10 секунд
viewer-skip-forward = Вперед на 10 секунд
viewer-rate = { $rate }×
viewer-shortcut = Клавіша: { $key }
key-space = Пробіл
key-shift-left = Shift+Вліво
key-shift-right = Shift+Вправо
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
timeline-gestures = Клацніть мітку, щоб вибрати її подію, або будь-де інде, щоб перейти туди. Тягніть по часу вгорі або за лінію поточного моменту, щоб рухатися демозаписом. Тягніть будь-де інде, щоб зсунути вигляд. Коліщатко наближає й віддаляє там, де стоїть вказівник, а подвійне клацання показує весь демозапис. Перетягніть край кліпу, щоб змінити його початок або кінець, або клацніть кліп, щоб змінити його.
timeline-density = Затінені стовпчики показують, скільки з показаних подій відбувається в кожному місці, порівняно з найнасиченішим місцем демозапису.
timeline-density-count = { $count ->
    [one] Тут { $count } показана подія
    [few] Тут { $count } показані події
    [many] Тут { $count } показаних подій
   *[other] Тут { $count } показаної події
}
timeline-session-count = Частина { $session }: { $count }
timeline-choose = Події в цій точці
legend-clip = Кліп
event-session = Частина { $session }
event-unreachable = Ця подія йде після зміни мапи або перепідключення, де годинник демозапису почався заново, тому перегляд не може до неї перейти.
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
help-midair-either = Вбивця, жертва або обидва були в повітрі. Вбивцю перевіряють у момент пострілу, пов’язаного з вбивством, а без нього — у момент вбивства, жертву — у момент смерті. Звичайний стрибок, нижче 50 юнітів над підлогою, не зараховується. Для цього треба вибрати теку Half-Life, щоб знати, де підлога.
help-midair-both = Вбивця і жертва обидва були в повітрі, кожен у момент перевірки.
help-midair-one = У повітрі був рівно один із них, а інший напевно був на землі.
speed-at-least = Не менше
speed-unit = ups
speed-killer = Вбивця
speed-victim = Жертва
help-speed = Швидкість по землі в ups, як її показує спідометр, без руху вгору й униз. Швидкість вбивці вимірюється в момент пострілу, пов’язаного з вбивством, а без нього — у момент вбивства, жертви — у момент смерті. Швидкості інших гравців — оцінки.
wallgauss-weapon = Wallgauss стосується лише вбивств із гаусса, а фільтр зброї показує { $weapon }.
wallgauss-checking = Перевірка Wallgauss… { $examined } з { $total } вбивств
wallgauss-needs-files = Для Wallgauss потрібні файли гри, щоб знати, де на мапі стіни.
wallgauss-failed = Не вдалося перевірити Wallgauss. { $reason }
help-wallgauss = Вбивства з гаусса крізь стіну, знайдені так: кожен постріл простежується по мапі так само, як це робить сервер. Вбивство, яке не вдається визначити, наприклад якщо жертва була поза полем зору запису, вважається невідомим і не враховується.
badge-wall = WG
wall-yes-exit = Wallgauss, обчислено за пострілом: вибух там, де промінь вийшов зі стіни завтовшки { $thickness } юнітів.
wall-yes-beyond = Wallgauss, обчислено за пострілом: промінь влучив після того, як пройшов крізь стіну завтовшки { $thickness } юнітів.
wall-limits = Обчислено за пострілом і положенням гравців, бо сервер цього не записує. Гравець завширшки 32 юніти вважається коробкою, а жертва могла стояти на одну-дві ширини тіла осторонь від місця, де її показує демозапис.
wall-no-direct = Не Wallgauss: промінь міг влучити в жертву раніше за будь-яку стіну.
wall-no-reflection = Не Wallgauss: вибух відбиття міг влучити в жертву раніше за будь-яку стіну.
wall-doubt-no-shot = Wallgauss невідомий: пострілу вбивці немає в записі.
wall-doubt-no-map = Wallgauss невідомий: не вдалося прочитати файли мапи.
wall-doubt-unseen = Wallgauss невідомий: жертва була поза полем зору запису.
wall-doubt-brush = Wallgauss невідомий: промінь влучив у двері, ліфт або іншу особливу частину мапи, а вони не перевіряються.
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
events-follow = Тримати список на поточному моменті під час відтворення демозапису. Прокручування списку це вимикає.
clip-it = У кліп
badge-headshot = ХШ
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = суїцид
badge-teamkill = тімкіл
badge-world = світ
badge-clip = кліп
clip-of = Кліп з { $name }
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
filter-midair = Midair-вбивства
event-headshot = в голову
scores-headshots = ХШ

column-accuracy = Влучність
filter-min-accuracy = Влучність не нижче (%)
scores-accuracy = Влуч.

help-headshots = Вбивства влучанням у голову, визначені за анімацією смерті жертви. Вибухи не враховуються. Показуються лише вбивства автора запису, бо інші гравці часто вбивають поза його полем зору. У дуелі показуються й вбивства суперника, адже всі вони — вбивства автора запису.
help-headshots-column = Вбивства автора запису влучанням у голову, визначені за анімацією смерті жертви. Вибухи не враховуються. Вбивства інших гравців не враховуються, бо часто стаються поза полем зору запису.
help-date-column = Коли було записано матч. Дата береться з назви файлу, якщо в ній є дата на кшталт 20230910_214553, як у записах, які гра робить сама. Інакше це дата файлу, яка може змінитися під час копіювання чи завантаження.
help-file-date = Коли файл було збережено. Для експортованого кліпу — коли його було експортовано.
help-accuracy = Частка пострілів, що влучили, за кров’ю, яку влучання залишає на лінії прицілу, за винятком випадкових збігів. Враховуються glock, python, кулі MP5, дробовик, арбалет із прицілом і прямі влучання променя гаусса. Вибухи та відбиття гаусса не залишають крові й не враховуються. Показується лише для автора запису і щонайменше після 50 пострілів.
help-headshot-filter = Демозаписи щонайменше з одним вбивством у голову, яке видно в записі, будь-яким гравцем.
help-midair-filter = Демозаписи щонайменше з одним вбивством, під час якого вбивця або жертва були в повітрі, на висоті щонайменше 50 юнітів над підлогою. Без вибраної теки Half-Life зараховується й звичайний стрибок.
help-accuracy-filter = Демозаписи, де автор запису влучив щонайменше такою часткою пострілів.

clips-new = Новий кліп
clips-select-event = Спершу виберіть подію у списку або на шкалі часу.
clips-other-session = Кліп не може починатися тут: після зміни мапи або перепідключення годинник демозапису почався заново, і рання частина має ті самі позначки часу.
clips-add-window = Кліп цього гарячого моменту
clip-default-name = Кліп { $number }
clip-name = Назва
clip-length = Довжина { $duration }
clip-remove = Видалити
clip-loop = Повтор
clip-time-help = Секунди (96), хвилини й секунди (1:36) або години, хвилини й секунди (00:01:36.250)
clips-empty-hint = Кліпів поки немає. Виберіть подію й натисніть «У кліп» або натисніть «Новий кліп», щоб почати кліп з поточного моменту.
clips-new-help = Десятисекундний кліп з поточного моменту.
clip-in = Початок
clip-out = Кінець
clip-set-in = Почати кліп з поточного моменту
clip-set-out = Закінчити кліп на поточному моменті
clip-no-playhead = Перегляд недоступний, тому немає поточного моменту, який можна взяти.
clip-play = Відтворити кліп один раз
clip-show-on-timeline = Показати на шкалі часу
export-destination = Змінити папку експорту
export-skipped = Пропущено. { $reason }
export-earlier-range = Цей експорт охоплює { $start } – { $end }, кліп таким, яким він був до ваших змін.
scores-player = Гравець
scores-none = У записі немає рахунку.
clip-effective = Експорт іде { $start } – { $end } ({ $duration }), настільки точно, наскільки дозволяє запис.
settings-clip-padding = Запас навколо кліпу
settings-preroll = Секунд до події
settings-postroll = Секунд після події
export-start-first-group = Кліп не може починатися з самого початку запису. Почніть трохи пізніше.
export-start-during-loading = Початок припадає на завантаження мапи. Почніть трохи пізніше.

export-all = Експортувати всі кліпи
export-one = Експорт
export-cancel = Зупинити експорт
export-folder = Експорт зберігається в { $folder }.
export-folder-beside = Експорт зберігається поруч із демо, в { $folder }.
export-waiting = Очікує експорту…
export-running = Експорт… Демо читається повністю, це може тривати.
export-cancelled = Не експортовано: експорт зупинено.
export-saved = Збережено: { $path }. Моментів звірено з оригіналом: { $count }.
export-playdemo = У консолі гри:
export-copy-first = Скопіюйте { $name } у папку мода, де лежить liblist.gam, потім у консолі гри:
export-copy = Копіювати
settings-export-folder = Папка експорту
settings-export-choose = Вибрати…
settings-file-names = Назви файлів
settings-clip-name-files = Використовувати назву кліпу як назву файлу
settings-demo-name-first = Починати з назви демо
settings-viewer = Перегляд демозаписів
settings-viewer-game = Тека Half-Life
settings-viewer-game-none = Не вибрано
settings-viewer-game-hint = Тека, у якій лежить valve. Перегляд відтворює демозаписи з мапами, моделями та звуками гри з неї.
settings-executable = Програма гри
settings-executable-hint = Допомагає знайти теку Half-Life. Перегляд сам читає файли гри й ніколи її не запускає.
settings-executable-not-set = Не задано, тому в перегляді немає ні зображення, ні звуку
settings-executable-optional = Не задано й не потрібно, поки в теці Half-Life є гра
settings-executable-missing = { $path } більше немає. Перегляд і далі читає теку Half-Life.
settings-game-override = Зараз Lambda Slicer використовує { $path }, бо його запущено з --game. Вибір тут діятиме з наступного запуску.
settings-browse = Огляд…
dialog-executables = Програми
setup-title = Налаштуйте перегляд демозаписів
setup-body = Для зображення й звуку потрібні файли Half-Life. Вкажіть Lambda Slicer, де встановлено гру, і він читатиме їх звідти.
setup-locate-exe = Знайти hl.exe…
setup-locate-folder = Знайти Half-Life…
setup-settings = Налаштування
setup-still-works = Шкала часу, події та кліпи працюють і без них.
setup-relocate = У { $folder } більше немає Half-Life. Знайдіть її знову.
setup-not-executable = { $name } — не { $executable }. Виберіть саму програму гри, а не ярлик.
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
settings-tab-library = Бібліотека
settings-tab-clips = Кліпи
settings-section-game = Half-Life
settings-section-view = Вигляд
settings-language-system = Мова комп’ютера ({ $name })
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
settings-zoom-fov = Поле зору під час наближення
settings-viewmodel = Показувати зброю в руках
settings-crosshair = Показувати приціл
settings-chat = Показувати чат
settings-vitals = Показувати здоров’я, броню та набої
settings-vitals-size = Розмір здоров’я, броні та набоїв
settings-scores = Показувати рахунок ліворуч угорі
settings-score-rows = Максимум рядків у рахунку
settings-see-through = Бачити гравців крізь стіни
settings-see-through-items = А також інші моделі, як-от снаряди й предмети
settings-see-through-hint = Гравці за стінами видно силуетами їхнього верхнього кольору, який командні ігри задають за командою. Видно лише те, що є в демозаписі. Сервер надсилає гравців поруч із тим, хто записує, а сервери, що приховують гравців, яких той не бачить, не залишають нічого для показу.
settings-section-decals = Декалі
settings-decals = Максимум
settings-decals-hint = Діри від куль, кров та інші сліди, залишені під час гри. Лишаються тільки найновіші, а найстаріші зникають, коли виникають нові. Сліди, що є частиною мапи, лишаються завжди.
settings-restore = Скинути налаштування…
settings-restore-title = Скинути налаштування
settings-restore-list = Ці налаштування повернуться до того, якими були спочатку. Тека Half-Life і теки бібліотеки лишаться як є.
settings-restore-none = Усі налаштування вже стоять за замовчуванням.
settings-restore-columns = Стовпці, показані в бібліотеці
settings-restore-layout = Відкриті панелі та їхні розміри
settings-clear-library = Очистити бібліотеку…
settings-clear-library-title = Очистити бібліотеку
settings-clear-library-text = { $count ->
    [one] Lambda Slicer забуде, що знайшов у { $count } демозаписі, і знову просканує теки бібліотеки, це може зайняти час. Ваші файли демозаписів, кліпи та налаштування лишаться як є.
    [few] Lambda Slicer забуде, що знайшов у { $count } демозаписах, і знову просканує теки бібліотеки, це може зайняти час. Ваші файли демозаписів, кліпи та налаштування лишаться як є.
    [many] Lambda Slicer забуде, що знайшов у { $count } демозаписах, і знову просканує теки бібліотеки, це може зайняти час. Ваші файли демозаписів, кліпи та налаштування лишаться як є.
   *[other] Lambda Slicer забуде, що знайшов у { $count } демозапису, і знову просканує теки бібліотеки, це може зайняти час. Ваші файли демозаписів, кліпи та налаштування лишаться як є.
}
viewer-zoom = Наблизити зображення
viewer-see-through = Бачити крізь стіни
viewer-paused = Пауза
viewer-map-change = Сервер змінює мапу на { $map }
settings-export-beside = Поруч із кожним демо
settings-export-hint = У папці мода, де лежить liblist.gam, наприклад Half-Life\ag, кліпи запускаються однією командою playdemo.
settings-file-names-hint = Експортований кліп отримує назву за своїм демо та часом початку й кінця, якщо ви не використовуєте назву кліпу. Гра відкриває лише файли, назви яких складаються з латинських літер, цифр, - і ., тому все інше замінюється на _. Якщо в назві кліпу немає жодного з цих символів, файл називається за часом.

clips-store-moved = Не вдалося прочитати файл кліпів { $path }, тому його збережено як { $moved }, а список кліпів почато заново. { $detail }
clips-store-unreadable = Не вдалося прочитати файл кліпів { $path }: { $detail }. Поки Lambda Slicer не закрито, кліпи не зберігаються, тож нічого у файлі не загубиться.
clips-save-failed = Не вдалося зберегти кліпи в { $path }: { $detail }
notice-dismiss = Закрити

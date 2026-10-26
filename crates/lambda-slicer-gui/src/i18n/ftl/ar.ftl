# العربية. مسودة لم تُراجع

app-title = Lambda Slicer

## شريط الأدوات

toolbar-open = فتح تسجيل…
toolbar-add-folder = إضافة مجلد…
toolbar-rescan = إعادة الفحص
toolbar-cancel = إلغاء
toolbar-settings = الإعدادات
toolbar-rescan-busy = الفحص جارٍ بالفعل.
toolbar-rescan-nothing = أضف مجلدًا أو افتح تسجيلًا لفحصه.
toolbar-cancel-idle = لا يجري أي فحص.
dialog-demos = تسجيلات Half-Life

## الإعدادات

settings-title = الإعدادات
settings-language = اللغة
settings-unreviewed = هذه الترجمة مسودة لم يراجعها أي متحدث أصلي بعد.
settings-workers = عدد التسجيلات التي تُحلَّل معًا
settings-workers-hint = تحليل المزيد معًا ينتهي أسرع، لكنه يُجهد الكمبيوتر أكثر.
settings-cache = مكان حفظ المكتبة
settings-start-tab = التبويب عند التشغيل
settings-start-tab-hint = تبويب المكتبة الذي يظهر في كل مرة يُشغَّل فيها Lambda Slicer. النقر على تبويب آخر يظل يعرضه حتى تُغلق Lambda Slicer.
settings-sort = الترتيب عند التشغيل
settings-sort-hint = كيف تُرتَّب التسجيلات في كل مرة يُشغَّل فيها Lambda Slicer. النقر على عنوان عمود يظل يرتّب حسبه حتى تُغلق Lambda Slicer.
settings-sprees = سلاسل القتل
settings-sprees-hint = آخر قتلة للاعب قبل موته تظهر عليها شارة بعدد القتلات التي حققها على التوالي، إذا كان عددها هذا الرقم على الأقل. الموت بأي طريقة ينهي السلسلة، وكذلك انتهاء الخريطة أو خروج اللاعب. قتل الزملاء لا يُحتسب.
settings-spree-least = على الأقل (قتلات)
settings-start-filters =المرشحات عند التشغيل
settings-start-filters-hint = تبدأ المكتبة بهذه المرشحات في كل مرة يُشغَّل فيها Lambda Slicer، فتبقى التسجيلات القصيرة التي لا تريد رؤيتها بعيدًا عن طريقك. يظل بإمكانك تغيير المرشحات حتى تُغلق Lambda Slicer.
settings-hide-unplayed-maps = في التسجيلات ذات الخرائط المتعددة، إخفاء الخرائط التي لا قتلات فيها وسُجِّلت أقل من 5 دقائق
settings-folders = مجلدات المكتبة
settings-folders-empty = لم تُضف أي مجلدات بعد.
settings-remove = إزالة

## المكتبة

library-empty = أضف مجلدًا أو افتح تسجيلًا للبدء.
library-no-matches = لا توجد تسجيلات تطابق هذه المرشحات.
library-no-cache = لا يوجد مكان على هذا الكمبيوتر لحفظ المكتبة.
library-demos = { $count ->
    [zero] لا توجد تسجيلات
    [one] تسجيل واحد
    [two] تسجيلان
    [few] { $count } تسجيلات
    [many] { $count } تسجيلًا
   *[other] { $count } تسجيل
}
library-percent = { $percent }٪
library-demos-of = { $shown } من { $total ->
    [zero] { $total } تسجيل
    [one] تسجيل واحد
    [two] تسجيلين
    [few] { $total } تسجيلات
    [many] { $total } تسجيلًا
   *[other] { $total } تسجيل
}
library-matches = { $count ->
    [zero] لا توجد مباريات
    [one] مباراة واحدة
    [two] مباراتان
    [few] { $count } مباريات
    [many] { $count } مباراةً
   *[other] { $count } مباراة
}
library-matches-of = { $shown } من { $total ->
    [zero] { $total } مباراة
    [one] مباراة واحدة
    [two] مباراتين
    [few] { $total } مباريات
    [many] { $total } مباراةً
   *[other] { $total } مباراة
}
library-no-games = لا يحتوي أي تسجيل على مباراة، ولا على خمس دقائق من اللعب من دون مباراة.
library-no-games-filtered = لا توجد مباريات تطابق هذه المرشحات.
library-total = { $count ->
    [zero] لا توجد تسجيلات في المكتبة
    [one] تسجيل واحد في المكتبة
    [two] تسجيلان في المكتبة
    [few] { $count } تسجيلات في المكتبة
    [many] { $count } تسجيلًا في المكتبة
   *[other] { $count } تسجيل في المكتبة
}
filters-title = المرشحات
filters-find = البحث
filters-pov = زاوية الرؤية
filters-quality = الجودة
filters-recorded = تاريخ التسجيل
pov-playing = لعب
pov-spectating = مشاهدة
help-pov = «لعب» يعني أن التسجيل قضى نصف مدة تشغيله أو أقل في المشاهدة، و«مشاهدة» يعني أكثر من النصف.
columns-button = الأعمدة
list-back = العودة إلى جدول المكتبة
sort-by = الترتيب حسب
sort-ascending = تصاعدي
sort-descending = تنازلي
score-more-sides = { $count ->
    [zero] لا أطراف أخرى
    [one] طرف آخر
    [two] طرفان آخران
    [few] { $count } أطراف أخرى
    [many] { $count } طرفًا آخر
   *[other] { $count } طرف آخر
}
intensity-help = الحماس { $score }. يقارنه الشريط بأعلى قيمة في المكتبة، { $max }.
value-unknown = غير معروف
path-copy = نسخ موقع الملف
path-show = إظهار في المجلد
path-show-failed = تعذر فتح المجلد ({ $detail }). انسخ موقع الملف بدلًا من ذلك.

filter-search = البحث في أسماء الملفات والمجلدات
filter-map = الخريطة
filter-player = اللاعب
help-player-filter = اكتب جزءًا من اسم للعثور على التسجيلات التي استخدمه فيها أي لاعب. اختر لاعبًا من القائمة للعثور على التسجيلات التي لعب فيها بأي اسم، إذ يُميَّز بـ SteamID الخاص به. يجب أن يكون كل اللاعبين المختارين قد لعبوا، لا أن يكونوا شاهدوا فقط. البوتات وبعض التسجيلات القديمة ليس لها SteamID، لذا لا يمكن العثور على هؤلاء اللاعبين إلا بالاسم.
player-remove = إزالة من المرشح
player-also = أيضًا { $names }
player-demos = { $count ->
    [zero] لا توجد تسجيلات
    [one] تسجيل واحد
    [two] تسجيلان
    [few] { $count } تسجيلات
    [many] { $count } تسجيلًا
   *[other] { $count } تسجيل
}
filter-weapon-any = أي سلاح
filter-min-minutes = على الأقل (دقائق)
filter-min-kills = على الأقل (قتلات)
filter-min-spree = سلسلة على الأقل (قتلات)
filter-players = اللاعبون
filter-teams = الفرق
filter-bots = البوتات
bots-any = أي عدد
bots-few = قليلة
bots-none = بلا بوتات
filter-spectating-any = لعب أو مشاهدة
filter-spectating-mostly = مشاهدة في الغالب
filter-spectating-playing = لعب في الغالب
filter-clear = مسح المرشحات

column-status = الحالة
column-name = التسجيل
column-map = الخريطة
column-duration = المدة
column-gauss = غاوس
column-kills = القتلات
column-action = الحماس
column-spectating = المشاهدة

status-complete = مُحلَّل
status-partial = جزئي
status-failed = فشل
status-unsupported = غير مدعوم
status-no-use = بلا استخدام للأسلحة

kind-multiplayer-view = منظور اللاعب
kind-singleplayer = لاعب واحد
kind-hltv = HLTV
kind-unknown = نوع غير معروف

details-recorder = سجّله { $name }
details-shots = { $count ->
    [zero] لا طلقات
    [one] طلقة واحدة
    [two] طلقتان
    [few] { $count } طلقات
    [many] { $count } طلقة
   *[other] { $count } طلقة
}
details-kills = { $count ->
    [zero] لا توجد عمليات قتل
    [one] عملية قتل واحدة
    [two] عمليتا قتل
    [few] { $count } عمليات قتل
    [many] { $count } عملية قتل
   *[other] { $count } عملية قتل
}
details-action = أعلى حماس { NUMBER($score, minimumFractionDigits: 2) }

## الفحص

scan-progress = عُثر على { $found }، اكتمل { $done } ({ $cached } معروفة من قبل، { $analysed } حُلّلت)، قُرئ { $megabytes } ميغابايت
scan-cancelling = جارٍ الإيقاف…
scan-cancelled = توقف، وبقي { $left } للفحص.
scan-finished = اكتمل: { $done ->
    [zero] لا تسجيلات
    [one] تسجيل واحد
    [two] تسجيلان
    [few] { $done } تسجيلات
    [many] { $done } تسجيلًا
   *[other] { $done } تسجيل
}، حُلّل { $analysed }، خلال { $seconds } ث.
scan-problems = { $count ->
    [zero] لا مشكلات
    [one] مشكلة واحدة
    [two] مشكلتان
    [few] { $count } مشكلات
    [many] { $count } مشكلة
   *[other] { $count } مشكلة
}
problems-title = مشكلات الفحص
problem-changed = تغيّر الملف أثناء تحليله، فلم يُحفظ شيء.
drop-hint = أفلت التسجيلات أو المجلدات لإضافتها
drop-not-demo = ليس تسجيلًا: { $names }

## رسائل نواة التحليل حسب الرمز

analysis-panicked = توقف التحليل بسبب عطل في Lambda Slicer: { $detail }
cache-cancelled = أُلغي التحليل، فلم يُحفظ شيء.
cache-database = تعذّر استخدام المكتبة المحفوظة: { $detail }
cache-directory = تعذّر إنشاء مجلد المكتبة المحفوظة: { $detail }
container-bad-magic = هذا ليس تسجيل Half-Life.
container-frame-bad-time = التسجيل تالف عند الإطار { $frame } من الجزء { $entry }، وزمنه معطوب.
container-frame-missing-terminal = لا ينتهي الجزء { $entry } من التسجيل كما ينبغي.
container-frame-negative-length = التسجيل تالف عند الإطار { $frame } من الجزء { $entry }.
container-frame-over-limit = الإطار { $frame } من الجزء { $entry } أكبر من المسموح.
container-frame-truncated = ينقطع التسجيل عند الإطار { $frame } من الجزء { $entry }.
container-frame-unknown-tag = التسجيل تالف عند الإطار { $frame } من الجزء { $entry }، وفيه شيء لا يتعرّف عليه Lambda Slicer.
container-io = تعذّرت قراءة الملف: { $detail }
container-no-directory = لم يُكمَل التسجيل قط، لذا ليس له فهرس.
container-range = يشير فهرس التسجيل إلى خارج الملف.
container-too-short = الملف أقصر من أن يكون تسجيلًا.
coverage-hltv = تسجيل HLTV. لم يسجّله لاعب، لذا تتبع الأحداث ما كان HLTV يشاهده.
coverage-mostly-spectating = كان المسجِّل مشاهدًا خلال { $percent }٪ من التشغيل، لذا فمعظم الأحداث للاعبين آخرين، كما ظهرت أثناء مشاهدتهم.
coverage-no-local-events = لم تُحفظ أفعال المسجِّل نفسه في هذا التسجيل، لذا لا يظهر منها إلا ما أرسله الخادم.
coverage-singleplayer = لاعب واحد. كل ما عدا اللاعب وحوش أو أجزاء من الخريطة، ولا توجد قائمة قتلات.
coverage-unknown = تعذّرت قراءة التسجيل بما يكفي لمعرفة نوعه.
coverage-visible-only = لا تصل أفعال اللاعبين الآخرين إلى اللعبة إلا حين يرى الخادم أن المسجِّل يستطيع رؤيتهم، لذا قد تكون أعدادها أقل مما حدث فعلًا.
decode-failed = توقفت قراءة التسجيل عند { $time } (الإطار { $frame }): { $detail }
export-active-effect = { $effect } ما زال جاريًا عند البداية، ولا يستطيع المقطع نقله بعد.
export-client-dropped-packets = عند { $time } تخلّصت اللعبة من بيانات وصلت متأخرة، وهذا ما لا يستطيع المقطع نسخه بعد.
export-clock-starts-over = بدأت ساعة التسجيل من جديد عند { $time }، بعد تغيير الخريطة أو إعادة الاتصال داخل المقطع، ولا يمكن للمقطع أن يتخطى ذلك.
export-destination-exists = { $path } موجود بالفعل. اختر اسمًا آخر.
export-empty-range = لا يحتوي المقطع على ما يُصدَّر.
export-io = تعذّرت كتابة { $path }: { $detail }
export-missing-delta-base = ينقص التسجيلَ بياناتٌ من قبل البداية يحتاجها المقطع (الرقم { $sequence }).
export-missing-parent = المجلد { $path } غير موجود.
export-range-beyond-end = النهاية { $end } بعد نهاية التسجيل { $segment_end }.
export-same-as-source = سيحلّ التصدير محلّ التسجيل الذي أُخذ منه.
export-source-changed = تغيّر { $path } أثناء التصدير.
export-source-decode-failed = تعذّرت قراءة التسجيل: { $detail }
export-source-failed-crosscheck = عند الفحص، لم يطابق ما استنتجه Lambda Slicer من هذا التسجيل التسجيلَ نفسه، لذا لا يمكن الوثوق بالمقاطع المأخوذة منه.
export-unknown-persistent-message = رسالة { $name } غير مدعومة في المقاطع بعد.
export-unproven-state = عند { $time } يعتمد التسجيل على بيانات كان لا بد من تخمينها، لذا لا يمكن صنع المقطع بدقة.
export-unrepresentable-time-field = لا يمكن حفظ زمن عند البداية بدقة: { $detail }
export-unsupported = غير مدعوم: { $detail }
export-validation-failed = لم يجتز المقطع المُصدَّر فحصه الأخير: { $detail }
io-not-found = لم يُعثر على الملف أو المجلد.
io-other = { $detail }
io-permission-denied = تم رفض الوصول.
io-unexpected-end = انتهى الملف على نحو غير متوقع.
protocol-unsupported = هذا التسجيل من إصدار للعبة غير مدعوم (البروتوكول { $demo }/{ $network }). المدعوم 5/47 و5/48 فقط.
protocol-unsupported-network = انتقل الخادم إلى إصدار للعبة غير مدعوم (البروتوكول { $network }).

## عرض التسجيل

demo-close = إغلاق
demo-loading = جارٍ تحليل { $name }…
show-shots = الطلقات
show-actions = أفعال أخرى
show-kills = القتلات
show-switches = تبديل الأسلحة
show-chat = الدردشة
show-match = المباراة
lane-unknown = لاعب غير معروف
lane-recorder = { $name } (المسجِّل)
event-kill = قتل { $victim }
event-switch = تبديل السلاح
event-copy = نسخ
event-copy-kill = { $killer } قتل { $victim } بسلاح { $weapon }
event-copy-teamkill = { $killer } قتل زميله في الفريق { $victim } بسلاح { $weapon }
event-copy-suicide = { $victim } قتل نفسه بسلاح { $weapon }
event-copy-death = { $victim } مات بسبب { $weapon }
event-vote-called = بدأ تصويت على { $subject }
event-vote-cast = تصويت على { $subject }: { $change } ({ $tally })
event-vote-more-yes = { $count ->
    [zero] لا أصوات إضافية بنعم
    [one] صوت إضافي بنعم
    [two] صوتان إضافيان بنعم
    [few] { $count } أصوات إضافية بنعم
    [many] { $count } صوتًا إضافيًا بنعم
   *[other] { $count } صوت إضافي بنعم
}
event-vote-fewer-yes = { $count ->
    [zero] لا أصوات أقل بنعم
    [one] صوت أقل بنعم
    [two] صوتان أقل بنعم
    [few] { $count } أصوات أقل بنعم
    [many] { $count } صوتًا أقل بنعم
   *[other] { $count } صوت أقل بنعم
}
event-vote-more-no = { $count ->
    [zero] لا أصوات إضافية بلا
    [one] صوت إضافي بلا
    [two] صوتان إضافيان بلا
    [few] { $count } أصوات إضافية بلا
    [many] { $count } صوتًا إضافيًا بلا
   *[other] { $count } صوت إضافي بلا
}
event-vote-fewer-no = { $count ->
    [zero] لا أصوات أقل بلا
    [one] صوت أقل بلا
    [two] صوتان أقل بلا
    [few] { $count } أصوات أقل بلا
    [many] { $count } صوتًا أقل بلا
   *[other] { $count } صوت أقل بلا
}
event-vote-tally = { $yes } نعم، { $no } لا، { $undecided } لم يصوّتوا
event-vote-accepted = قُبل التصويت على { $subject }
event-vote-denied = رُفض التصويت على { $subject }
event-timeout-called = طُلب وقت مستقطع
event-timeout-under-way = بدأ الوقت المستقطع
event-timeout-over = انتهى الوقت المستقطع
event-paused = توقفت اللعبة مؤقتًا
event-resumed = استؤنفت اللعبة
event-sudden-death = بدأ الموت المفاجئ
event-sudden-death-over = انتهى الموت المفاجئ
event-map-changed = تغيّرت الخريطة إلى { $map }
event-spectating = بدء المشاهدة
event-playing = انتهاء المشاهدة
action-swing = ضربة
action-fire = إطلاق
action-launch = قذف
action-charge-activity = شحن
action-charge-stop = نهاية الشحن
action-beam-start = بداية الشعاع
action-beam-sustain = شعاع
action-beam-stop = نهاية الشعاع
action-placement-attempt = محاولة وضع
action-release-attempt = محاولة إطلاق
mode-primary = أساسي
mode-charged = مشحون
mode-zoomed = مع التكبير
mode-unzoomed = دون تكبير
mode-grenade = قنبلة
mode-bullets = رصاص
mode-single = مفرد
mode-double = مزدوج
mode-rapid = سريع

column-date = التاريخ
column-file-date = تاريخ الملف
column-score = النتيجة
filter-from = من
filter-to = إلى
date-from-name = التاريخ مأخوذ من اسم الملف
date-from-modified = تاريخ آخر تعديل للملف، وقد يتغير عند النسخ أو التنزيل
timeline-best-action = الدقيقة الذهبية
timeline-window = رقم { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = النتائج النهائية عند آخر قتل { $time }
scores-at = النتائج عند { $time }
scores-show-final = عرض النتائج النهائية
scores-spectators = المشاهدون
scores-follow = انقر لتشاهد بعينيه
pane-fill = منحه اللوحة الجانبية كاملة
pane-share = تقاسم اللوحة الجانبية مناصفة
scores-frags = القتلات
scores-deaths = الوفيات

viewer-play = تشغيل
viewer-pause = إيقاف مؤقت
viewer-volume = مستوى الصوت
viewer-no-sound = لا صوت
viewer-play-backward = ترجيع
viewer-step-back = الإطار السابق
viewer-step-forward = الإطار التالي
viewer-position = الموضع في التسجيل
viewer-skip-back = الرجوع 10 ثوانٍ
viewer-skip-forward = التقدّم 10 ثوانٍ
viewer-rate = { $rate }×
viewer-slower = أبطأ
viewer-faster = أسرع
viewer-rate-normal = العودة إلى السرعة العادية
viewer-shortcut = الاختصار: { $key }
key-space = مسافة
key-ctrl = Ctrl
key-shift = Shift
key-alt = Alt
key-left = يسار
key-right = يمين
key-up = أعلى
key-down = أسفل
key-home = Home
key-end = End
key-page-up = Page Up
key-page-down = Page Down
key-enter = Enter
key-backspace = Backspace
key-delete = Delete
key-insert = Insert
key-tab = Tab
key-mouse-3 = زر الفأرة 3
key-mouse-4 = زر الفأرة 4
key-mouse-5 = زر الفأرة 5
key-wheel-up = العجلة لأعلى
key-wheel-down = العجلة لأسفل
viewer-map-failed = تعذّر تحميل الخريطة { $map }
viewer-stopped = توقّف التشغيل مبكرًا: { $reason }
viewer-missing = ملفات تعذّرت قراءتها: { $count }
event-previous = الحدث السابق
event-next = الحدث التالي
event-details = تفاصيل القتل
event-play = تشغيل الحدث
timeline-title = الخط الزمني
timeline-lanes = صفوف اللاعبين
timeline-zoom-in = تكبير
timeline-zoom-out = تصغير
timeline-fit-all = ملاءمة التسجيل كله
timeline-gestures = انقر على علامة لتحديد حدثها، أو في أي مكان آخر للانتقال إليه. اسحب على الأوقات في الأعلى، أو اسحب الخط الذي يحدد اللحظة الحالية، للتنقل في التسجيل. اسحب في أي مكان آخر لتحريك العرض. مرّر العجلة للتكبير والتصغير عند موضع الفأرة، وانقر مرتين لرؤية التسجيل كله. اسحب طرف المقطع لتغيير بدايته أو نهايته، أو انقر على المقطع لتحريره.
timeline-density = تبيّن الأشرطة المظللة عدد الأحداث المعروضة التي تقع عند كل نقطة، مقارنةً بأكثر نقاط التسجيل ازدحامًا.
timeline-density-count = { $count ->
    [zero] لا أحداث معروضة هنا
    [one] حدث معروض واحد هنا
    [two] حدثان معروضان هنا
    [few] { $count } أحداث معروضة هنا
    [many] { $count } حدثًا معروضًا هنا
   *[other] { $count } حدث معروض هنا
}
timeline-session-count = الجزء { $session }: { $count }
timeline-choose = الأحداث عند هذه النقطة
legend-clip = مقطع
event-session = الجزء { $session }
event-unreachable = هذا الحدث يأتي بعد تغيير الخريطة أو إعادة الاتصال، حيث بدأت ساعة التسجيل من جديد، لذا لا يستطيع العارض الانتقال إليه.
event-hidden = الحدث المحدد مخفي بسبب عوامل التصفية.
highlight-jump = الانتقال إلى لقطة بارزة…
highlight-help = ترتّب اللقطات البارزة التسجيل كله حسب الإثارة، أيًّا كان ما تعرضه المرشحات.
kills-with = قتلات فيها
refine-midair = في الهواء
refine-speed = السرعة
refine-wallgauss = Wallgauss
refine-multikill = قتل متعدد
refine-clear = مسح
refine-airborne = من في الهواء
refine-either = أيٌّ منهما
midair-both = كلاهما
midair-one = واحد فقط
help-midair-either = كان القاتل أو الضحية أو كلاهما في الهواء. يُفحص القاتل عند الطلقة المرتبطة بالقتل، أو عند القتل إن لم توجد، وتُفحص الضحية عند الموت. لا يُحسب من يقفز فقط، على ارتفاع أقل من 50 وحدة فوق الأرضية. يتطلب هذا اختيار مجلد Half-Life لمعرفة مكان الأرضية.
help-midair-both = كان القاتل والضحية كلاهما في الهواء، كلٌّ عند فحصه.
help-midair-one = كان أحدهما بالضبط في الهواء، ومن المعروف أن الآخر كان على الأرض.
speed-at-least = على الأقل
speed-unit = ups
speed-killer = القاتل
speed-victim = الضحية
help-multikill = عدة لاعبين قُتلوا بطلقة واحدة، مثل انفجار يصيبهم جميعًا أو شعاع غاوس يخترقهم. تُظهر ذلك كل قتلة من تلك الطلقة.
help-speed = السرعة على الأرض بوحدة ups، كما يعرضها عدّاد السرعة، دون احتساب الحركة صعودًا وهبوطًا. تُقاس سرعة القاتل عند الطلقة المرتبطة بالقتل، أو عند القتل إن لم توجد، وسرعة الضحية عند الموت. سرعات اللاعبين الآخرين تقديرية.
wallgauss-weapon = يشمل Wallgauss قتلات الغاوس فقط، ومرشح السلاح يعرض { $weapon }.
wallgauss-checking = جارٍ فحص Wallgauss… القتلات: { $examined } من { $total }
wallgauss-needs-files = يحتاج Wallgauss إلى ملفات اللعبة ليعرف أين جدران الخريطة.
wallgauss-failed = تعذّر فحص Wallgauss. { $reason }
help-wallgauss = قتلات الغاوس عبر جدار، تُكتشف باتّباع كل طلقة عبر الخريطة كما يفعل الخادم. القتلة التي لا يمكن حسمها، كأن تكون الضحية خارج مجال رؤية التسجيل، تُعدّ مجهولة وتُستبعد.
badge-wall = WG
wall-yes-exit = Wallgauss، استُنتج من الطلقة: الانفجار حيث خرج الشعاع من جدار بسُمك { $thickness } وحدة.
wall-yes-beyond = Wallgauss، استُنتج من الطلقة: أصاب الشعاع بعد اختراقه جدارًا بسُمك { $thickness } وحدة.
wall-limits = استُنتج من الطلقة ومن مواضع اللاعبين، لأن الخادم لا يسجّله. عرض اللاعب 32 وحدة ويُحسب كصندوق، وربما كانت الضحية على بُعد عرض جسم أو اثنين من موضعها في التسجيل.
wall-no-direct = ليس Wallgauss: كان يمكن للشعاع أن يصيب الضحية قبل أي جدار.
wall-no-reflection = ليس Wallgauss: كان يمكن لانفجار انعكاس أن يصيب الضحية قبل أي جدار.
wall-doubt-no-shot = Wallgauss مجهول: طلقة القاتل ليست في التسجيل.
wall-doubt-no-map = Wallgauss مجهول: تعذّرت قراءة ملفات الخريطة.
wall-doubt-unseen = Wallgauss مجهول: كانت الضحية خارج مجال رؤية التسجيل.
wall-doubt-brush = Wallgauss مجهول: أصاب الشعاع بابًا أو مصعدًا أو جزءًا خاصًا آخر من الخريطة، وهذه لا تُفحص.
wall-doubt-ambiguous = Wallgauss مجهول: كان يمكن للطلقة أن تقتل عبر جدار وبدونه أيضًا.
wall-doubt-unexplained = Wallgauss مجهول: لم يُعثر على طريقة وصلت بها الطلقة إلى الضحية.
wall-doubt-unconfirmed = Wallgauss مجهول: لم يكن للطلقة أن تصل إلى الضحية إلا عبر جدار، لكن ذلك غير مؤكد.
wall-doubt-not-reached = Wallgauss مجهول: يتوقف التسجيل قبل الطلقة.
wall-pending = لم يُفحص Wallgauss بعد.
events-unknown = { $count ->
    [zero] لا توجد عمليات قتل تنقصها البيانات اللازمة للحكم
    [one] عملية قتل واحدة تنقصها البيانات اللازمة للحكم
    [two] عمليتا قتل تنقصهما البيانات اللازمة للحكم
    [few] { $count } عمليات قتل تنقصها البيانات اللازمة للحكم
    [many] { $count } عملية قتل تنقصها البيانات اللازمة للحكم
   *[other] { $count } عملية قتل تنقصها البيانات اللازمة للحكم
}
help-unknown = الحركة التي لم يستطع التسجيل رؤيتها تُعدّ مجهولة، ولا تُعدّ أبدًا وقوفًا على الأرض أو ثباتًا، لذا تُستبعد هذه القتلات.
events-filters = المرشحات
events-filters-on = المرشحات ({ $count })
events-kinds-none = لا توجد أحداث
events-follow = أبقِ القائمة على اللحظة الحالية أثناء تشغيل التسجيل. تمرير القائمة يوقف ذلك.
clip-it = أنشئ مقطعًا
badge-headshot = HS
badge-double-kill = قتل مزدوج
badge-triple-kill = قتل ثلاثي
badge-quadra-kill = قتل رباعي
badge-penta-kill = قتل خماسي
badge-multikill = { $count } قتلات
badge-spree = ×{ $count }
badge-spree-help = { $count ->
    [zero] { $count } قتلة متتالية دون أن يموت
    [one] قتلة واحدة دون أن يموت
    [two] قتلتان متتاليتان دون أن يموت
    [few] { $count } قتلات متتالية دون أن يموت
    [many] { $count } قتلة متتالية دون أن يموت
   *[other] { $count } قتلة متتالية دون أن يموت
}
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = انتحار
badge-teamkill = قتل زميل
badge-world = العالم
badge-clip = مقطع
clip-of = مقطع من { $name }
air-both = كان كلاهما في الهواء عند الفحص.
air-killer = كان القاتل في الهواء عند الفحص.
air-victim = كانت الضحية في الهواء عند الفحص.
air-other-unknown = ولا يُعرف إن كان الآخر كذلك.
speed-horizontal = السرعة الأفقية، موازيةً للأرض.
motion-line = { $who }: { $air }، { $speed }. { $how }، { $when }.
motion-who-killer = القاتل
motion-who-victim = الضحية
motion-air-yes = في الهواء
motion-air-no = على الأرض
motion-air-unknown = لا يُعرف إن كان في الهواء
motion-speed-unknown = السرعة مجهولة
motion-exact = سجّلتها اللعبة
motion-trajectory = مقدّرة من الحركة
motion-unclear = الحركة غير واضحة
motion-not-visible = خارج مجال الرؤية
motion-at-shot = عند الطلقة المرتبطة
motion-at-kill = عند الموت
motion-none = لا يملك التحليل بيانات حركة لهذه القتلة.
legend-shot = طلقة
legend-charge = شحن
legend-kill = قتل
legend-death = موت
legend-switch = تبديل السلاح

column-headshots = طلقات الرأس
column-multikill = قتل متعدد
column-spree = سلسلة
filter-headshots = قتلات بطلقة في الرأس
filter-midair = قتلات في الهواء
event-headshot = طلقة في الرأس
scores-headshots = رأس

column-accuracy = الدقة
column-players = اللاعبون
column-teams = الفرق
help-players-column = اللاعبون في اللعبة، لا المشاهدون، يُعدّون حين تبدأ مباراة بعد عدّها التنازلي. في لعبة الفرق لا يُحتسب إلا اللاعبون المنضمون إلى فريق. يعرض التسجيل أكبر عدد في أي مباراة كاملة، والتسجيل الذي لا مباراة كاملة فيه يعرض أكبر عدد في أي لحظة.
help-teams-column = الفرق التي فيها لاعب واحد على الأقل، تُعدّ كما يُعدّ اللاعبون. اللعبة التي بلا فرق ليس فيها أي فريق.
filter-min-accuracy = الدقة لا تقل عن (٪)
scores-accuracy = الدقة
scores-kd = K/D
scores-ping = البينغ
help-kd = القتلات لكل وفاة، أي القتلات مقسومة على الوفيات. اللاعب الذي لم يمت أبدًا تظهر قتلاته.
help-ping-loss = بينغ كل لاعب بالمللي ثانية، ثم فقد الحزم، كما أرسلهما الخادم آخر مرة. لا يرسلهما الخادم إلا ما دام المسجِّل يضغط على مفتاح لوحة النتائج، لذا تظهر مكانهما شَرطة حتى ذلك الحين.

help-headshots = قتلات بإصابة في الرأس، تُعرف من حركة موت الضحية. لا تُحتسب الانفجارات. تظهر قتلات المسجِّل فقط، لأن اللاعبين الآخرين كثيرًا ما يقتلون خارج مجال رؤية التسجيل. في المبارزة تظهر قتلات الخصم أيضًا، لأنها كلها على المسجِّل.
help-headshots-column = قتلات المسجِّل بإصابة في الرأس، تُعرف من حركة موت الضحية. لا تُحتسب الانفجارات. لا تُحتسب قتلات اللاعبين الآخرين، لأنها كثيرًا ما تقع خارج مجال رؤية التسجيل.
help-multikill-column = أكبر عدد من اللاعبين قتلهم أي لاعب بطلقة واحدة في التسجيل، مثل 3 في القتل الثلاثي.
help-spree-column = أطول سلسلة قتل في التسجيل، لأي لاعب: أكبر عدد من القتلات حققها لاعب على التوالي دون أن يموت.
help-date-column = وقت تسجيل المباراة. يُؤخذ من اسم الملف حين يحتوي الاسم على تاريخ مثل 20230910_214553، كما في التسجيلات التي تصنعها اللعبة تلقائيًا. وإلا فهو تاريخ الملف، وقد يتغير عند النسخ أو التنزيل.
help-file-date = وقت حفظ الملف. وفي المقطع المُصدَّر، هو وقت تصديره.
help-accuracy = نسبة الطلقات التي أصابت، بحسب الدم الذي تتركه الإصابة على خط التصويب، بعد طرح التطابقات العرضية. تشمل الغلوك والبايثون ورصاص MP5 والبندقية والقوس مع التكبير والإصابات المباشرة لشعاع الغاوس. الانفجارات وانعكاسات الغاوس لا تترك دمًا ولا تُحتسب. تظهر للمسجِّل فقط، بعد 50 طلقة على الأقل.
help-headshot-filter = تسجيلات فيها قتل واحد على الأقل بإصابة في الرأس ظهر في التسجيل، من أي لاعب.
help-midair-filter = تسجيلات فيها قتل واحد على الأقل كان فيه القاتل أو الضحية في الهواء، على ارتفاع 50 وحدة على الأقل فوق الأرضية. إن لم يُختر مجلد Half-Life، تُحسب القفزة العادية أيضًا.
help-spree-filter = تسجيلات قتل فيها لاعب هذا العدد على الأقل على التوالي دون أن يموت. الموت بأي طريقة ينهي السلسلة، وكذلك انتهاء الخريطة أو خروج اللاعب. قتل الزملاء لا يُحتسب.
help-accuracy-filter = تسجيلات أصاب فيها المسجِّل هذه النسبة من طلقاته على الأقل.
help-bots-filter = «قليلة» يُبقي فقط التسجيلات والمباريات التي فيها بوتات أقل من اللاعبين الحقيقيين، فتُستبعد مباراة 1v1 ضد بوت، لكن لا يُستبعد سيرفر مزدحم فيه بضعة بوتات تملأ الأماكن. «بلا بوتات» يُبقي فقط ما ليس فيه أي بوت. البوت لاعب تعدّه اللعبة بوتًا، أو لاعب بلا مفتاح CD.

clips-new = مقطع جديد
clips-select-event = اختر حدثًا من القائمة أو من الخط الزمني أولًا.
clips-other-session = لا يمكن أن يبدأ مقطع هنا، لأن ساعة التسجيل بدأت من جديد عند تغيير الخريطة أو إعادة الاتصال قبل ذلك مباشرة.
clips-add-window = مقطع لهذه اللحظة الحماسية
clip-default-name = مقطع { $number }
clip-name = الاسم
clip-length = المدة { $duration }
clip-remove = إزالة
clip-loop = تكرار
clip-time-help = ثوانٍ (96)، أو دقائق وثوانٍ (1:36)، أو ساعات ودقائق وثوانٍ (00:01:36.250)
clips-empty-hint = لا توجد مقاطع بعد. اختر حدثًا واضغط «أنشئ مقطعًا»، أو اضغط «مقطع جديد» لبدء مقطع من اللحظة الحالية.
clips-new-help = مقطع مدته عشر ثوانٍ يبدأ من اللحظة الحالية.
clip-in = البداية
clip-out = النهاية
clip-set-in = ابدأ المقطع من اللحظة الحالية
clip-set-out = أنهِ المقطع عند اللحظة الحالية
clip-no-playhead = العارض غير متاح، لذا لا توجد لحظة حالية لاستخدامها.
clip-play = تشغيل المقطع مرة واحدة
clip-unplayable = هذا المقطع يأتي بعد تغيير الخريطة أو إعادة الاتصال، حيث بدأت ساعة التسجيل من جديد، لذا لا يستطيع العارض تشغيله. ومع ذلك فإن تصديره يقتطع اللحظة الصحيحة.
clip-show-on-timeline = عرض على الخط الزمني
export-destination = تغيير مجلد التصدير
export-skipped = تم التخطي. { $reason }
export-earlier-range = هذا التصدير للفترة { $start } – { $end }، أي المقطع كما كان قبل أن تغيّره.
scores-player = اللاعب
scores-none = لا توجد نتائج في التسجيل.
clip-effective = يمتد التصدير { $start } – { $end } ({ $duration })، بأقرب ما يسمح به التسجيل.
settings-clip-padding = هامش المقاطع
settings-preroll = ثوانٍ قبل الحدث
settings-postroll = ثوانٍ بعد الحدث
export-start-first-group = لا يمكن أن يبدأ المقطع من بداية التسجيل تمامًا. ابدأ بعدها بقليل.
export-stopped = أُوقف التصدير، فلم يُحفظ أي مقطع.
export-start-during-loading = البداية أثناء تحميل اللعبة لخريطة. ابدأ بعدها بقليل.

export-all = تصدير كل المقاطع
export-one = تصدير
export-cancel = إيقاف التصدير
export-folder = تُحفظ الملفات المصدَّرة في { $folder }.
export-folder-beside = تُحفظ الملفات المصدَّرة بجانب التسجيل، في { $folder }.
export-waiting = في انتظار التصدير…
export-running = جارٍ التصدير… يُقرأ التسجيل كله، وقد يستغرق ذلك بعض الوقت.
export-searching = لا يمكن أن يبدأ المقطع من هذه النقطة بالضبط. جارٍ البحث عن بداية قريبة تصلح…
export-nearest-earlier = يمكن أن يبدأ عند { $time } بدلًا من ذلك، أي قبل { $seconds } ث.
export-nearest-later = يمكن أن يبدأ عند { $time } بدلًا من ذلك، أي بعد { $seconds } ث.
export-start-there = ابدأ من هناك وصدِّر
export-cancelled = لم يُصدَّر، لأن التصدير أُوقف.
export-saved = حُفظ في { $path }. لحظات طوبقت مع الأصل: { $count }.
export-playdemo = في وحدة تحكم اللعبة:
export-copy-first = انسخ { $name } إلى مجلد التعديل الذي فيه liblist.gam، ثم في وحدة تحكم اللعبة:
export-copy = نسخ
settings-export-folder = مجلد التصدير
settings-export-choose = اختيار…
settings-file-names = أسماء الملفات
settings-clip-name-files = استخدام اسم المقطع اسمًا للملف
settings-demo-name-first = البدء باسم التسجيل
settings-viewer = عارض التسجيلات
settings-viewer-game = مجلد Half-Life
settings-viewer-game-none = لم يُختر بعد
settings-viewer-game-hint = المجلد الذي يحتوي valve. يشغّل العارض التسجيلات بخرائط اللعبة ونماذجها وأصواتها الموجودة فيه.
settings-game-override = هذه المرة يستخدم Lambda Slicer { $path }، لأنه شُغّل بـ ‎--game. يسري الاختيار هنا من المرة القادمة التي يُشغَّل فيها.
settings-more-games = مجلدات Half-Life إضافية
settings-more-games-hint = أماكن تثبيت أخرى لـ Half-Life، مثل مجلد فيه خرائط كثيرة. حين لا تكون خريطة أو نموذج أو سبرايت أو صوت في مجلد Half-Life أعلاه، يبحث العارض في هذه المجلدات من الأعلى إلى الأسفل. تغييرها يجعل المكتبة تفحص تسجيلاتها من جديد بحثًا عن القتلات في الهواء، مع الخرائط التي تضيفها.
settings-more-games-empty = لم تُضف أي مجلدات بعد.
settings-move-up = نقل لأعلى
settings-move-down = نقل لأسفل
settings-game-content = محتوى اللعبة
settings-game-content-hint = تبدأ هذه الخيارات كما هي في لعبتك. تغييرها هنا يغيّر العارض فقط، ولا يغيّر اللعبة أبدًا. إذا شُغّل أكثر من خيار، يُتّبع ترتيب اللعبة: محتوى الإضافات أولًا، ثم المحتوى المترجم، ثم نماذج HD، ثم ملفات اللعبة نفسها.
settings-hd-models = استخدام نماذج HD
settings-hd-models-hint = يبحث أولًا في مجلدات _hd، مثل valve_hd، عن النماذج عالية الجودة. يبدأ مثل خيار اللعبة "Use high quality models".
settings-addon-content = استخدام محتوى الإضافات المخصص
settings-addon-content-hint = يبحث أولًا في مجلدات _addon، مثل valve_addon، عن المحتوى الذي أضفته إلى اللعبة. يبدأ مثل خيار اللعبة "Allow custom addon content".
settings-localized-content = استخدام المحتوى المترجم
settings-localized-content-hint = يبحث أولًا في مجلدات اللغة التي يشغّل بها Steam لعبة Half-Life، مثل valve_spanish للإسبانية. تستخدمها اللعبة دائمًا، وليس للإنجليزية أي منها.
settings-steam-language = يشغّل Steam لعبة Half-Life بهذه اللغة: { $language }.
settings-steam-english = يشغّل Steam لعبة Half-Life بالإنجليزية، وليس لها مجلدات خاصة بها.
dialog-executables = البرامج
setup-title = إعداد عارض التسجيلات
setup-body = تحتاج الصورة والصوت إلى ملفات Half-Life. أرِ Lambda Slicer مكان تثبيت اللعبة ليقرأها من هناك.
setup-locate-exe = تحديد hl.exe…
setup-locate-folder = تحديد Half-Life…
setup-settings = الإعدادات
setup-still-works = يبقى الخط الزمني والأحداث والمقاطع تعمل من دونها.
setup-relocate = لم يعد { $folder } يحتوي على Half-Life. حدده من جديد.
setup-not-executable = { $name } ليس { $executable }. اختر برنامج اللعبة نفسه لا اختصارًا.
setup-not-installed = لا يحتوي { $folder } على تثبيت Half-Life فيه مجلد valve.
tab-events = الأحداث
tab-clips = المقاطع
tab-scoreboard = النتائج
tab-demos = التسجيلات
tab-matches = المباريات
filter-status = الحالة
library-all = الكل
library-favourites = المفضلة
library-no-favourites = لا توجد تسجيلات مفضلة بعد. انقر بزر الفأرة الأيمن على تسجيل واختر «إضافة إلى المفضلة».
menu-favourite-add = إضافة إلى المفضلة
menu-favourite-remove = إزالة من المفضلة
favourite-star = مفضّل
library-hidden = المخفية
library-no-hidden = لا توجد تسجيلات مخفية. انقر بزر الفأرة الأيمن على تسجيل واختر «إخفاء» لإبعاده عن القائمة.
menu-hide = إخفاء
menu-unhide = إظهار
filter-status-all = الكل
help-status-column = كم سجّل التسجيل من المباراة. أشر إلى حالة صف لترى معناها.
help-match-players = اللاعبون في اللعبة، لا المشاهدون، عند بدء المباراة بعد العد التنازلي، أو عند أول قتل إذا كانت المباراة جارية حين بدأ التسجيل. في اللعب الحر، أكبر عدد في الوقت نفسه.
help-match-teams = الفرق التي فيها لاعب واحد على الأقل، تُعدّ كما يُعدّ اللاعبون. اللعبة التي لا فرق فيها ليس لها أي فريق.
help-match-multikill = أكبر عدد من اللاعبين قتلهم أي لاعب بطلقة واحدة في المباراة، مثل 3 في القتل الثلاثي.
help-match-spree = أطول سلسلة قتل انتهت في المباراة، لأي لاعب: أكبر عدد من القتلات حققها لاعب على التوالي دون أن يموت.
match-complete = كاملة
match-joined-late = بدأت قبل التسجيل
match-interrupted = توقفت
match-cut-off = مقطوعة
match-free-play = لعب حر
help-match-complete = سُجّلت من العد التنازلي حتى نهاية المباراة.
help-match-joined-late = كانت المباراة جارية حين بدأ التسجيل، وسُجّلت حتى نهايتها.
help-match-interrupted = بدأت مباراة أخرى، أو أُلغيت المباراة، أو تغيرت الخريطة قبل أن تنتهي المباراة.
help-match-cut-off = انتهى التسجيل قبل أن تنتهي المباراة.
help-match-free-play = خمس دقائق أو أكثر من اللعب من دون مباراة جارية.
list-hide = إخفاء قائمة التسجيلات
list-show = إظهار قائمة التسجيلات
demo-details = التفاصيل
viewer-start = العودة إلى البداية
viewer-mute = كتم الصوت
viewer-unmute = إلغاء الكتم
viewer-picture = إعدادات الصورة والعارض
viewer-needs-files = يحتاج إلى ملفات اللعبة
viewer-no-sound-why = لا صوت: { $reason }
settings-tab-general = عام
settings-tab-library = المكتبة
settings-tab-clips = المقاطع
settings-tab-shortcuts = الاختصارات
settings-shortcuts-hint = انقر على مفتاح لتغييره، ثم اضغط المفتاح الجديد أو زر الفأرة، أو أدر عجلة الفأرة والمؤشر فوقه، أو اضغط Escape لإلغاء التغيير. انقر بالزر الأيمن على مفتاح لمسحه. وإذا كان المفتاح لإجراء آخر فإنه ينتقل إلى هنا. تعمل اختصارات عجلة الفأرة فوق صورة التسجيل.
settings-shortcuts-defaults = المفاتيح الافتراضية
settings-shortcut-press = اضغط مفتاحًا…
settings-shortcut-change = انقر للتغيير، وانقر بالزر الأيمن للمسح
settings-shortcut-moved = أُخذ { $key } من «{ $action }».
shortcut-none = بلا مفتاح
shortcuts-anywhere = في أي مكان
shortcuts-demo = التسجيل
shortcuts-playback = التشغيل
shortcuts-view = العرض
shortcuts-players = بعيني مَن تشاهد
shortcuts-free-camera = الكاميرا الحرة
shortcut-open-demo = فتح تسجيل
shortcut-settings = فتح الإعدادات أو إغلاقها
shortcut-back = الرجوع بين التسجيلات والمباريات التي فتحتها
shortcut-forward = التقدّم بين التسجيلات والمباريات التي فتحتها
shortcut-previous-event = الحدث السابق
shortcut-next-event = الحدث التالي
shortcut-clip-start = جعل بداية المقطع المختار الآن
shortcut-clip-end = جعل نهاية المقطع المختار الآن
shortcut-play-pause = تشغيل أو إيقاف مؤقت
shortcut-rewind = ترجيع
shortcut-pause = إيقاف مؤقت
shortcut-seek-back = الرجوع 5 ثوانٍ
shortcut-seek-forward = التقدّم 5 ثوانٍ
shortcut-skip-back = الرجوع 10 ثوانٍ
shortcut-skip-forward = التقدّم 10 ثوانٍ
shortcut-frame-back = الإطار السابق
shortcut-frame-forward = الإطار التالي
shortcut-slower = أبطأ
shortcut-faster = أسرع
shortcut-start = العودة إلى البداية
shortcut-fullscreen = ملء الشاشة
shortcut-zoom = تقريب الصورة أو إبعادها
shortcut-see-through = رؤية اللاعبين خلف الجدران
shortcut-fullbright = إضاءة كل شيء
shortcut-names = أسماء اللاعبين
shortcut-free-camera = الكاميرا الحرة
shortcut-recorder = المشهد بعيني المسجِّل
shortcut-follow = المشهد بعيني أفضل لاعب في الخريطة رقم { $place }
shortcut-previous-player = المشهد بعيني اللاعب السابق
shortcut-next-player = المشهد بعيني اللاعب التالي
shortcut-fly-forward = للأمام
shortcut-fly-back = للخلف
shortcut-fly-left = يسار
shortcut-fly-right = يمين
shortcut-fly-up = للأعلى
shortcut-fly-down = للأسفل
shortcut-fly-fast = أسرع
settings-section-game = Half-Life
settings-section-view = المنظور
settings-language-system = لغة الكمبيوتر ({ $name })
settings-section-folders = المجلدات
settings-folders-hint = المجلدات التي تبحث فيها المكتبة عن التسجيلات، مع مجلداتها الفرعية.
settings-clip-padding-hint = مقدار ما يحتفظ به المقطع المصنوع حول حدث قبله وبعده.
settings-section-picture = الصورة
settings-section-sound = الصوت
settings-gamma = غاما
settings-brightness = السطوع
settings-texgamma = غاما الخامات
settings-lightgamma = غاما الإضاءة
settings-overbright = إضاءة مضاعفة
settings-gamma-hint = إعدادات اللعبة بالأسماء نفسها (gamma, brightness, texgamma, lightgamma, gl_overbright). اضبطها كما في اللعبة لترى السطوع نفسه.
settings-gamma-defaults = قيم اللعبة الافتراضية
settings-fov-recorded = مجال الرؤية كما سُجّل
settings-fov = مجال الرؤية
settings-zoom-fov = مجال الرؤية عند التقريب
settings-viewmodel = إظهار السلاح في اليد
settings-weapon-bob = تمايل السلاح (cl_bob)
settings-weapon-bob-recorded = تمايل السلاح كما سُجّل
settings-crosshair = إظهار علامة التصويب
settings-chat = إظهار الدردشة
settings-vitals = إظهار الصحة والدرع والذخيرة والأغراض الملتقطة
settings-vitals-size = حجم الصحة والدرع والذخيرة والأغراض الملتقطة
settings-free-vitals = إظهار الصحة والدرع والذخيرة والأغراض الملتقطة مع الكاميرا الحرة أيضًا
settings-scores = إظهار النتائج أعلى اليسار
settings-score-rows = أقصى عدد من صفوف النتائج
settings-scoreboard = إظهار لوحة النتائج ما دام المسجِّل يضغط على مفتاحها، وفي نهاية الخريطة
settings-scoreboard-off = معطّلة
settings-scoreboard-fullscreen = في ملء الشاشة فقط
settings-scoreboard-always = مفعّلة
settings-server-settings = إظهار إعدادات السيرفر أعلى اليمين
settings-text-size = حجم الدردشة وقائمة القتلات وإعدادات السيرفر
settings-hud-color = لون HUD
settings-hud-color-hint = لون الصحة والدرع والذخيرة والأغراض الملتقطة وقائمة الأسلحة والسرعات و FPS المسجِّل، ومؤقت AG والتصويتات وإعدادات السيرفر فيه، مثل hud_color في OpenAG. تبقى الصحة المنخفضة والأسلحة التي نفدت ذخيرتها باللون الأحمر، ويحتفظ نص HUD الخاص بالسيرفر بلونه.
settings-text-color = لون النص
settings-text-color-default = لون اللعبة
settings-text-color-hint = لون رسائل الدردشة، والأسطر التي لم يرسلها أي لاعب، والموقع وزوايا النظر، مثل con_color في اللعبة. تبقى أسماء اللاعبين بلون فريقهم.
settings-hud-messages = إظهار النصوص من السيرفر، مثل مؤقتات bhop و KZ
settings-see-through = رؤية اللاعبين عبر الجدران
settings-see-through-items = ونماذج أخرى أيضًا، كالمقذوفات والأغراض
settings-player-names = إظهار أسماء اللاعبين فوقهم
settings-show-position = إظهار الموقع وزوايا النظر
settings-speedometer = إظهار عدّاد السرعة
settings-jump-speed = إظهار سرعة آخر قفزة
settings-jump-speed-hint = سرعة المسجِّل لحظة مغادرته الأرض آخر مرة، وتظهر تحت عدّاد السرعة حتى قفزته التالية.
settings-recorder-fps = إظهار FPS المسجِّل
settings-recorder-fps-hint = عدد الإطارات في الثانية (FPS) التي كانت اللعبة تعمل بها عند المسجِّل، في أعلى اليسار، بمتوسط كل نصف ثانية. ليست سرعة رسم هذا التطبيق لها.
settings-speed-axes = السرعة المحتسبة
settings-speed-horizontal = أفقية
settings-speed-vertical = عمودية
settings-speed-both = كلتاهما (3D)
viewer-jump-speed = قفزة { $speed }
settings-show-position-hint = المكان الذي تُرى منه الصورة بوحدات اللعبة، والجهة التي تنظر إليها بالدرجات: لأعلى أو لأسفل (pitch) وبالدوران حولك (yaw). عند تشغيل الكاميرا الحرة تكون هذه قيم الكاميرا الحرة. مفيد لتخبر أحدًا بالضبط أين ينظر.
viewer-position-line = الموقع { $x } { $y } { $z }
viewer-fps = { $fps } FPS
viewer-angles-line = الزوايا { $pitch } { $yaw }
settings-player-names-hint =يظهر اسم كل لاعب في مجال الرؤية فوق رأسه بلون فريقه، ومن الكاميرا الحرة أيضًا. لا تظهر أسماء اللاعبين خلف الجدران إلا أثناء رؤية اللاعبين عبر الجدران.
settings-see-through-hint = يظهر اللاعبون خلف الجدران كظلال بلون الجزء العلوي، الذي تحدده الألعاب الجماعية حسب الفريق. لا يظهر إلا ما يحويه التسجيل. يرسل الخادم اللاعبين القريبين من المسجِّل، والخوادم التي تخفي اللاعبين الذين لا يراهم المسجِّل لا تترك شيئًا ليظهر.
settings-section-decals = الآثار
settings-decals = الحد الأقصى المحفوظ
settings-decals-hint = ثقوب الرصاص والدم وغيرها من الآثار التي تُترك أثناء اللعب. يبقى الأحدث فقط، ويختفي الأقدم كلما ظهرت آثار جديدة. الآثار التي هي جزء من الخريطة تبقى دائمًا.
settings-restore = استعادة الإعدادات الافتراضية…
settings-restore-title = استعادة الإعدادات الافتراضية
settings-restore-list = ستعود هذه الإعدادات إلى ما كانت عليه في البداية. يبقى مجلد Half-Life ومجلدات المكتبة كما هي.
settings-restore-none = كل الإعدادات على قيمها الافتراضية أصلًا.
settings-restore-columns = الأعمدة الظاهرة في المكتبة
settings-restore-layout = اللوحات المفتوحة وأحجامها
settings-clear-library = تفريغ المكتبة…
settings-clear-library-title = تفريغ المكتبة
settings-clear-library-text = { $count ->
    [zero] ينسى Lambda Slicer ما وجده في { $count } تسجيل ويعيد فحص مجلدات المكتبة، وقد يستغرق ذلك بعض الوقت. تبقى ملفات تسجيلاتك ومقاطعك وإعداداتك كما هي.
    [one] ينسى Lambda Slicer ما وجده في { $count } تسجيل ويعيد فحص مجلدات المكتبة، وقد يستغرق ذلك بعض الوقت. تبقى ملفات تسجيلاتك ومقاطعك وإعداداتك كما هي.
    [two] ينسى Lambda Slicer ما وجده في { $count } تسجيلين ويعيد فحص مجلدات المكتبة، وقد يستغرق ذلك بعض الوقت. تبقى ملفات تسجيلاتك ومقاطعك وإعداداتك كما هي.
    [few] ينسى Lambda Slicer ما وجده في { $count } تسجيلات ويعيد فحص مجلدات المكتبة، وقد يستغرق ذلك بعض الوقت. تبقى ملفات تسجيلاتك ومقاطعك وإعداداتك كما هي.
    [many] ينسى Lambda Slicer ما وجده في { $count } تسجيلًا ويعيد فحص مجلدات المكتبة، وقد يستغرق ذلك بعض الوقت. تبقى ملفات تسجيلاتك ومقاطعك وإعداداتك كما هي.
   *[other] ينسى Lambda Slicer ما وجده في { $count } تسجيل ويعيد فحص مجلدات المكتبة، وقد يستغرق ذلك بعض الوقت. تبقى ملفات تسجيلاتك ومقاطعك وإعداداتك كما هي.
}
viewer-zoom = تقريب الصورة
viewer-free-camera = الكاميرا الحرة
viewer-free-camera-help = تجوّل في الخريطة بحرية بعيدًا عن اللاعب الذي سجّل المقطع. { $forward } و{ $left } و{ $back } و{ $right } للحركة، و{ $up } للصعود، و{ $down } للنزول، و{ $fast } للإسراع. اضغط مع الاستمرار على زر الفأرة الأيمن واسحب للنظر حولك.
viewer-fullscreen = ملء الشاشة
viewer-leave-fullscreen = إنهاء ملء الشاشة
viewer-see-through = الرؤية خلف الجدران
viewer-player-names = أسماء اللاعبين
viewer-fullbright = إضاءة كل شيء
viewer-fullbright-help = تعرض الخريطة كلها بإضاءة كاملة، فتسهل رؤية الزوايا المظلمة واللاعبين الموجودين فيها.
viewer-paused = متوقف مؤقتًا
viewer-map-change = السيرفر يغيّر الخريطة إلى { $map }
viewer-pov-recorder = { $name } (المسجِّل)
viewer-pov-recorder-unnamed = المسجِّل
viewer-pov-help = بعيني مَن تشاهد: المسجِّل أو أفضل لاعبي هذه الخريطة حسب النقاط، وبجانب كلٍّ منهم مفتاحه. اللاعب السابق: { $previous }. اللاعب التالي: { $next }.
viewer-scoreboard-ping-loss = البينغ/الفقد
viewer-scoreboard-bot = BOT
viewer-scoreboard-players = { $count ->
    [zero] ({ $count } لاعب)
    [one] (لاعب واحد)
    [two] (لاعبان)
    [few] ({ $count } لاعبين)
    [many] ({ $count } لاعبًا)
   *[other] ({ $count } لاعب)
}
viewer-pov-unseen = التسجيل لا يُظهر { $name } هنا، لذا ترى المشهد بعيني المسجِّل
settings-events-leave-free = مغادرة الكاميرا الحرة عند اختيار حدث
settings-events-leave-free-hint = عملية القتل التي تختارها تُعرض بعيني القاتل أو الضحية أو المسجِّل. عند إطفاء هذا الخيار تبقى الكاميرا الحرة في مكانها، وعند إيقافها تنتقل إلى ذلك المشهد.
settings-particles = إظهار الجسيمات
settings-particles-hint = الشرر والشظايا المتطايرة من الانفجارات والغبار حيث تصيب الرصاصات.
settings-export-beside = بجانب كل تسجيل
settings-export-hint = داخل مجلد التعديل الذي فيه liblist.gam مثل Half-Life\ag، تُشغَّل المقاطع بأمر playdemo واحد.
settings-file-names-hint = يُسمّى المقطع المصدَّر باسم تسجيله ووقتي بدايته ونهايته، ما لم تستخدم اسم المقطع. لا تفتح اللعبة إلا الملفات التي تتكوّن أسماؤها من حروف إنجليزية وأرقام و - و .، لذا يتحوّل كل ما عداها إلى _. وإن لم يحتوِ اسم المقطع على أيٍّ منها، يُسمّى الملف بأوقاته بدلًا من ذلك.

clips-store-moved = تعذّرت قراءة ملف المقاطع { $path }، لذا حُفظ باسم { $moved } وتبدأ المقاطع فارغة. { $detail }
clips-store-unreadable = تعذّرت قراءة ملف المقاطع { $path }: { $detail }. لن تُحفظ المقاطع حتى يُغلق Lambda Slicer، فلا يضيع شيء مما في الملف.
clips-save-failed = تعذّر حفظ المقاطع في { $path }: { $detail }
notice-dismiss = إغلاق

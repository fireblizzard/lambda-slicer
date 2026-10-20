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
settings-workers = عمليات التحليل
settings-workers-hint = المزيد من العمليات يحلل أسرع لكنه يستهلك وقتًا أكثر من المعالج.
settings-cache = ذاكرة المكتبة المؤقتة
settings-folders = مجلدات المكتبة
settings-folders-empty = لم تُضف أي مجلدات بعد.
settings-remove = إزالة

## المكتبة

library-empty = أضف مجلدًا أو افتح تسجيلًا للبدء.
library-no-matches = لا توجد تسجيلات تطابق هذه المرشحات.
library-no-cache = لا يوجد مكان لحفظ ذاكرة المكتبة المؤقتة على هذا النظام.
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
pov-any = الكل
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
path-copy = نسخ المسار
path-show = إظهار في المجلد
path-show-failed = تعذر فتح المجلد ({ $detail }). انسخ المسار بدلًا من ذلك.

filter-search = البحث في المسارات
filter-map = الخريطة
filter-player = اللاعب
filter-weapon-any = أي سلاح
filter-min-minutes = على الأقل (دقائق)
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
kind-hltv = وكيل مشاهدة
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

scan-progress = عُثر على { $found }، اكتمل { $done } ({ $cached } من الذاكرة المؤقتة، { $analysed } حُلّلت)، قُرئ { $megabytes } ميغابايت
scan-cancelling = جارٍ الإيقاف…
scan-cancelled = توقف، وبقي { $left } للفحص.
scan-finished = اكتمل: { $done } تسجيلًا، حُلّل منها { $analysed }، خلال { $seconds } ث.
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

analysis-panicked = توقف التحليل بسبب خطأ داخلي: { $detail }
cache-cancelled = أُلغي التحليل، فلم يُحفظ شيء.
cache-database = تعذّر استخدام ذاكرة المكتبة المؤقتة: { $detail }
cache-directory = تعذّر إنشاء مجلد الذاكرة المؤقتة: { $detail }
container-bad-magic = هذا ليس تسجيل Half-Life.
container-frame-bad-time = التسجيل تالف عند الإطار { $frame } من الجزء { $entry }، وزمنه غير صالح.
container-frame-missing-terminal = ينتهي الجزء { $entry } من التسجيل دون سجل الإغلاق.
container-frame-negative-length = التسجيل تالف عند الإطار { $frame } من الجزء { $entry }.
container-frame-over-limit = الإطار { $frame } من الجزء { $entry } أكبر من المسموح.
container-frame-truncated = ينقطع التسجيل عند الإطار { $frame } من الجزء { $entry }.
container-frame-unknown-tag = التسجيل تالف عند الإطار { $frame } من الجزء { $entry }، ونوعه غير معروف.
container-io = تعذّرت قراءة الملف: { $detail }
container-no-directory = لم يُكمَل التسجيل قط، لذا ليس له فهرس.
container-range = يشير فهرس التسجيل إلى خارج الملف.
container-too-short = الملف أقصر من أن يكون تسجيلًا.
coverage-hltv = تسجيل وكيل مشاهدة. لا يوجد لاعب يسجّل، والأحداث تتبع منظور الوكيل.
coverage-mostly-spectating = كان المسجِّل مشاهدًا خلال { $percent }٪ من التشغيل، لذا فمعظم الأحداث للاعبين آخرين كما عرضها المنظور المُشاهَد.
coverage-no-local-events = لا أحداث محلية. أفعال المسجِّل نفسه مفقودة إلا ما أرسله الخادم.
coverage-singleplayer = لاعب واحد. الأطراف الأخرى وحوش أو عناصر من الخريطة، ولا يوجد سجل للقتلات.
coverage-unknown = لم يُفكَّ ترميز التسجيل بما يكفي لمعرفة نوعه.
coverage-visible-only = لا تصل أحداث اللاعبين الآخرين إلى هذا العميل إلا ما دام الخادم يعدّهم مرئيين، لذا فأعدادها حدود دنيا.
decode-failed = توقف فك الترميز عند { $time } (الإطار { $frame }): { $detail }
export-active-effect = { $effect } جارٍ عند البداية ولا يمكن نقله بعد.
export-client-dropped-packets = أسقطت اللعبة حزمًا متأخرة عند { $time }، وهو ما لا يستطيع القص إعادة إنتاجه بعد.
export-cross-segment = تبدأ جلسة جديدة عند { $time } داخل النطاق.
export-destination-exists = { $path } موجود بالفعل. اختر اسمًا آخر.
export-empty-range = لا يحتوي النطاق المحدد على ما يُصدَّر.
export-io = تعذّرت كتابة { $path }: { $detail }
export-missing-delta-base = ينقص التسجيلَ حالةٌ لازمة قبل البداية (التسلسل { $sequence }).
export-missing-parent = المجلد { $path } غير موجود.
export-range-beyond-end = النهاية { $end } بعد نهاية التسجيل { $segment_end }.
export-same-as-source = الوجهة هي التسجيل المصدر نفسه.
export-source-changed = تغيّر { $path } أثناء التصدير.
export-source-decode-failed = تعذّر فك ترميز التسجيل المصدر: { $detail }
export-source-failed-crosscheck = لم يجتز التسجيل المصدر فحصه المتقاطع، لذا لا يُوثق بحالته المعاد بناؤها.
export-unknown-persistent-message = { $name } قبل البداية يضبط حالة لا يستطيع القص نقلها بعد.
export-unproven-state = الحالة عند { $time } تستند إلى أساس مستنتج وليست مثبتة.
export-unrepresentable-time-field = لا يمكن كتابة زمن عند البداية بدقة: { $detail }
export-unsupported = غير مدعوم: { $detail }
export-validation-failed = لم يجتز الملف المُصدَّر التحقق: { $detail }
io-not-found = لم يُعثر على الملف أو المجلد.
io-other = { $detail }
io-permission-denied = تم رفض الوصول.
io-unexpected-end = انتهى الملف على نحو غير متوقع.
protocol-unsupported = بروتوكول التسجيل { $demo }/{ $network } غير مدعوم، المدعوم 5/48 فقط.
protocol-unsupported-network = انتقل الخادم إلى بروتوكول الشبكة { $network } غير المدعوم.

## عرض التسجيل

demo-close = إغلاق
demo-loading = جارٍ تحليل { $name }…
show-shots = الطلقات
show-actions = أفعال أخرى
show-kills = القتلات
show-switches = تبديل الأسلحة
lane-unknown = لاعب غير معروف
lane-recorder = { $name } (المسجِّل)
event-kill = قتل { $victim }
event-switch = تبديل السلاح
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
demo-events = { $count ->
    [zero] لا أحداث
    [one] حدث واحد
    [two] حدثان
    [few] { $count } أحداث
    [many] { $count } حدثًا
   *[other] { $count } حدث
}

column-date = التاريخ
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
scores-frags = القتلات
scores-deaths = الوفيات

viewer-play = تشغيل
viewer-pause = إيقاف مؤقت
viewer-volume = مستوى الصوت
viewer-no-sound = لا صوت
viewer-play-backward = ترجيع
viewer-step-back = الإطار السابق
viewer-step-forward = الإطار التالي
viewer-rate = { $rate }×
viewer-shortcut = الاختصار: { $key }
key-space = مسافة
viewer-map-failed = تعذّر تحميل الخريطة { $map }
viewer-stopped = توقّف التشغيل مبكرًا: { $reason }
viewer-missing = ملفات تعذّرت قراءتها: { $count }
event-previous = الحدث السابق
event-next = الحدث التالي
event-play = تشغيل الحدث
timeline-title = الخط الزمني
timeline-lanes = صفوف اللاعبين
timeline-zoom-in = تكبير
timeline-zoom-out = تصغير
timeline-fit-all = ملاءمة التسجيل كله
timeline-gestures = انقر على علامة لتحديد حدثها، أو في أي مكان آخر للانتقال إليه. اسحب المسطرة أو رأس التشغيل للتنقل، واسحب في مكان آخر للتحريك. مرّر العجلة للتكبير حول المؤشر، وانقر مرتين لرؤية التسجيل كله. اسحب طرف المقطع لتحريكه، أو انقر على المقطع لتحريره.
timeline-density = تبيّن الأشرطة المظللة عدد الأحداث المعروضة في كل فترة، مقارنةً بأكثر فترات التسجيل ازدحامًا.
timeline-density-count = { $count ->
    [zero] لا أحداث معروضة هنا
    [one] حدث معروض واحد هنا
    [two] حدثان معروضان هنا
    [few] { $count } أحداث معروضة هنا
    [many] { $count } حدثًا معروضًا هنا
   *[other] { $count } حدث معروض هنا
}
timeline-session-count = الجلسة { $session }: { $count }
timeline-choose = الأحداث عند هذه النقطة
legend-clip = مقطع
event-session = الجلسة { $session }
event-unreachable = هذا الحدث في جلسة لاحقة تعيد استخدام الأوقات نفسها، لذا لا يستطيع العارض الانتقال إليه.
event-hidden = الحدث المحدد مخفي بسبب عوامل التصفية.
highlight-jump = الانتقال إلى لقطة بارزة…
highlight-help = ترتّب اللقطات البارزة التسجيل كله حسب الإثارة، أيًّا كان ما تعرضه المرشحات.
kills-with = قتلات فيها
refine-midair = في الهواء
refine-speed = السرعة
refine-wallgauss = Wallgauss
refine-clear = مسح
refine-airborne = من في الهواء
refine-either = أيٌّ منهما
midair-both = كلاهما
midair-one = واحد فقط
help-midair-either = كان القاتل أو الضحية أو كلاهما في الهواء. يُفحص القاتل عند الطلقة المرتبطة بالقتل، أو عند القتل إن لم توجد، وتُفحص الضحية عند الموت.
help-midair-both = كان القاتل والضحية كلاهما في الهواء، كلٌّ عند فحصه.
help-midair-one = كان أحدهما بالضبط في الهواء، ومن المعروف أن الآخر كان على الأرض.
speed-at-least = على الأقل
speed-unit = ups
speed-killer = القاتل
speed-victim = الضحية
help-speed = السرعة الأفقية بالوحدات في الثانية، موازيةً للأرض. تُقاس سرعة القاتل عند الطلقة المرتبطة بالقتل، أو عند القتل إن لم توجد، وسرعة الضحية عند الموت. سرعات اللاعبين الآخرين تقديرية.
wallgauss-weapon = يشمل Wallgauss قتلات الغاوس فقط، ومرشح السلاح يعرض { $weapon }.
wallgauss-checking = جارٍ فحص Wallgauss… القتلات: { $examined } من { $total }
wallgauss-needs-files = يحتاج Wallgauss إلى ملفات اللعبة لتتبّع الطلقات عبر الخريطة.
wallgauss-failed = تعذّر فحص Wallgauss. { $reason }
help-wallgauss = قتلات الغاوس عبر جدار، تُحدَّد بتتبّع كل طلقة عبر الخريطة كما يفعل الخادم. القتلة التي لا يمكن حسمها، كأن تكون الضحية خارج مجال رؤية التسجيل، تُعدّ مجهولة وتُستبعد.
badge-wall = WALL
wall-yes-exit = حُدِّد Wallgauss بإعادة البناء: الانفجار حيث خرج الشعاع من جدار بسُمك { $thickness } وحدة.
wall-yes-beyond = حُدِّد Wallgauss بإعادة البناء: الشعاع بعد اختراقه جدارًا بسُمك { $thickness } وحدة.
wall-limits = أُعيد بناؤه من الطلقة واللقطة، ولم يسجّله الخادم. يُتتبَّع اللاعبون كصناديق، وربما كانت الضحية على بُعد بضع عشرات من الوحدات من موضعها في اللقطة.
wall-no-direct = ليس Wallgauss: كان يمكن للشعاع أن يصيب الضحية قبل أي جدار.
wall-no-reflection = ليس Wallgauss: كان يمكن لانفجار انعكاس أن يصيب الضحية قبل أي جدار.
wall-doubt-no-shot = Wallgauss مجهول: طلقة القاتل ليست في التسجيل.
wall-doubt-no-map = Wallgauss مجهول: تعذّرت قراءة ملفات الخريطة.
wall-doubt-unseen = Wallgauss مجهول: كانت الضحية خارج مجال رؤية التسجيل.
wall-doubt-brush = Wallgauss مجهول: صادف الشعاع بابًا أو منصة أو كيان brush آخر، وهذه لا تُتتبَّع.
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
clip-it = أنشئ مقطعًا
badge-headshot = HS
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = انتحار
badge-teamkill = قتل زميل
badge-world = العالم
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
filter-headshots = قتلات بطلقة في الرأس
event-headshot = طلقة في الرأس
scores-headshots = رأس

column-accuracy = الدقة
filter-min-accuracy = الدقة لا تقل عن (٪)
scores-accuracy = الدقة

help-headshots = قتلات بإصابة في الرأس، تُعرف من حركة موت الضحية. لا تُحتسب الانفجارات. تظهر قتلات المسجِّل فقط، لأن اللاعبين الآخرين كثيرًا ما يقتلون خارج مجال رؤية التسجيل. في المبارزة تظهر قتلات الخصم أيضًا، لأنها كلها على المسجِّل.
help-headshots-column = قتلات المسجِّل بإصابة في الرأس، تُعرف من حركة موت الضحية. لا تُحتسب الانفجارات. لا تُحتسب قتلات اللاعبين الآخرين، لأنها كثيرًا ما تقع خارج مجال رؤية التسجيل.
help-accuracy = نسبة الطلقات التي أصابت، بحسب الدم الذي تتركه الإصابة على خط التصويب، بعد طرح التطابقات العرضية. تشمل الغلوك والبايثون ورصاص MP5 والبندقية والقوس مع التكبير والإصابات المباشرة لشعاع الغاوس. الانفجارات وانعكاسات الغاوس لا تترك دمًا ولا تُحتسب. تظهر للمسجِّل فقط، بعد 50 طلقة على الأقل.
help-headshot-filter = تسجيلات فيها قتل واحد على الأقل بإصابة في الرأس ظهر في التسجيل، من أي لاعب.
help-accuracy-filter = تسجيلات أصاب فيها المسجِّل هذه النسبة من طلقاته على الأقل.

clips-new = مقطع جديد
clips-select-event = اختر حدثًا من القائمة أو من الخط الزمني أولًا.
clips-other-session = لا يمكن أن يبدأ مقطع هنا، لأن جلسة سابقة من التسجيل لها الأوقات نفسها.
clips-add-window = مقطع لهذه اللحظة الحماسية
clip-default-name = مقطع { $number }
clip-name = الاسم
clip-length = المدة { $duration }
clip-remove = إزالة
clip-loop = تكرار
clip-time-help = ثوانٍ (96)، أو دقائق وثوانٍ (1:36)، أو ساعات ودقائق وثوانٍ (00:01:36.250)
clips-empty-hint = لا توجد مقاطع بعد. اختر حدثًا واضغط «أنشئ مقطعًا»، أو أنشئ مقطعًا عند رأس التشغيل بزر «مقطع جديد».
clips-new-help = مقطع مدته عشر ثوانٍ يبدأ من رأس التشغيل.
clip-in = دخول
clip-out = خروج
clip-set-in = ضبط البداية على رأس التشغيل
clip-set-out = ضبط النهاية على رأس التشغيل
clip-no-playhead = لا يوجد رأس تشغيل يؤخذ منه الوقت.
clip-play = تشغيل المقطع مرة واحدة
clip-show-on-timeline = عرض على الخط الزمني
export-destination = تغيير مجلد التصدير
export-skipped = تم التخطي. { $reason }
export-earlier-range = هذا التصدير للنطاق { $start } – { $end }، أي النطاق قبل تعديل المقطع.
scores-player = اللاعب
scores-none = لا توجد نتائج في التسجيل.
clip-effective = يقطع التصدير { $start } – { $end } ({ $duration })، حسب إطارات التسجيل.
settings-clip-padding = هامش المقاطع
settings-preroll = ثوانٍ قبل الحدث
settings-postroll = ثوانٍ بعد الحدث
export-start-first-group = لا يمكن أن يبدأ القطع من أول إطار في التسجيل. ابدأ بعده بقليل.
export-start-during-loading = البداية أثناء تحميل اللعبة لخريطة. ابدأ بعدها بقليل.

export-all = تصدير كل المقاطع
export-one = تصدير
export-cancel = إيقاف التصدير
export-folder = تُحفظ الملفات المصدَّرة في { $folder }.
export-folder-beside = تُحفظ الملفات المصدَّرة بجانب التسجيل، في { $folder }.
export-waiting = في انتظار التصدير…
export-running = جارٍ التصدير… يُقرأ التسجيل كله، وقد يستغرق ذلك بعض الوقت.
export-cancelled = لم يُصدَّر، لأن التصدير أُوقف.
export-saved = حُفظ في { $path }. لقطات طوبقت مع الأصل: { $count }.
export-playdemo = في وحدة تحكم اللعبة:
export-copy-first = انسخ { $name } إلى مجلد التعديل الذي فيه liblist.gam، ثم في وحدة تحكم اللعبة:
export-copy = نسخ
settings-export-folder = مجلد التصدير
settings-export-choose = اختيار…
settings-viewer = عارض التسجيلات
settings-viewer-game = مجلد Half-Life
settings-viewer-game-none = لم يُختر بعد
settings-viewer-game-hint = المجلد الذي يحتوي valve. يشغّل العارض التسجيلات بخرائط اللعبة ونماذجها وأصواتها الموجودة فيه.
settings-executable = ملف اللعبة التنفيذي
settings-executable-hint = يحدد مجلد Half-Life. يقرأ العارض ملفات اللعبة بنفسه ولا يشغّل اللعبة أبدًا.
settings-executable-not-set = غير محدد، لذا لا صورة ولا صوت في العارض
settings-executable-optional = غير محدد، ولا حاجة إليه ما دام مجلد Half-Life يحتوي على اللعبة
settings-executable-missing = { $path } لم يعد موجودًا. ما زال العارض يقرأ مجلد Half-Life.
settings-game-override = هذا التشغيل يستخدم { $path } المعطى بـ ‎--game. يسري الاختيار هنا من التشغيل التالي.
settings-browse = استعراض…
dialog-executables = البرامج
setup-title = إعداد عارض التسجيلات
setup-body = تحتاج الصورة والصوت إلى ملفات Half-Life. أرِ Lambda Slicer مكان تثبيت اللعبة ليقرأها من هناك.
setup-locate-exe = تحديد hl.exe…
setup-locate-folder = تحديد Half-Life…
setup-settings = الإعدادات
setup-still-works = يبقى الخط الزمني والأحداث والمقاطع تعمل من دونها.
setup-relocate = لم يعد { $folder } يحتوي على Half-Life. حدده من جديد.
setup-not-executable = { $name } ليس { $executable }. اختر ملف اللعبة التنفيذي نفسه لا اختصارًا.
setup-not-installed = لا يحتوي { $folder } على تثبيت Half-Life فيه مجلد valve.
tab-events = الأحداث
tab-clips = المقاطع
tab-scoreboard = النتائج
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
settings-viewmodel = إظهار السلاح في اليد
settings-see-through = رؤية اللاعبين عبر الجدران
settings-see-through-items = ونماذج أخرى أيضًا، كالمقذوفات والأغراض
settings-see-through-hint = يظهر اللاعبون خلف الجدران كظلال بلون الجزء العلوي، الذي تحدده الألعاب الجماعية حسب الفريق. لا يظهر إلا ما يحويه التسجيل. يرسل الخادم اللاعبين القريبين من المسجِّل، والخوادم التي تخفي اللاعبين الذين لا يراهم المسجِّل لا تترك شيئًا ليظهر.
viewer-see-through = الرؤية خلف الجدران
settings-export-beside = بجانب كل تسجيل
settings-export-hint = داخل مجلد التعديل الذي فيه liblist.gam مثل Half-Life\ag، تُشغَّل المقاطع بأمر playdemo واحد.

clips-store-moved = تعذّرت قراءة ملف المقاطع { $path }، لذا حُفظ باسم { $moved } وتبدأ المقاطع فارغة. { $detail }
clips-store-unreadable = تعذّرت قراءة ملف المقاطع { $path }: { $detail }. لا تُحفظ المقاطع في هذه الجلسة، فلا يضيع شيء مما فيه.
clips-save-failed = تعذّر حفظ المقاطع في { $path }: { $detail }
notice-dismiss = إغلاق

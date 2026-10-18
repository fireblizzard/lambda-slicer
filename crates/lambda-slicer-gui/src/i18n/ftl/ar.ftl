# العربية. مسودة لم تُراجع

app-title = Lambda Slicer

## شريط الأدوات

toolbar-open = فتح تسجيل…
toolbar-add-folder = إضافة مجلد…
toolbar-rescan = إعادة الفحص
toolbar-cancel = إلغاء
toolbar-settings = الإعدادات
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
timeline-fit = ملاءمة
show-shots = الطلقات
show-actions = أفعال أخرى
show-kills = القتلات
show-switches = تبديل الأسلحة
lane-unknown = لاعب غير معروف
lane-recorder = { $name } (المسجِّل)
column-time = الوقت
column-player = اللاعب
column-weapon = السلاح
column-event = الحدث
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
timeline-windows = أكثر اللحظات حماسًا
timeline-window = رقم { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = النتائج النهائية عند آخر قتل { $time }
scores-at = النتائج عند { $time }
scores-show-final = عرض النتائج النهائية
scores-frags = القتلات
scores-deaths = الوفيات

show-timeline = إظهار الخط الزمني
viewer-play = تشغيل
viewer-pause = إيقاف مؤقت
viewer-volume = مستوى الصوت
legend-shot = طلقة
legend-charge = شحن
legend-kill = قتل
legend-death = موت
legend-switch = تبديل السلاح
timeline-help = كل صف لاعب. مرّر العجلة للتكبير، واسحب للتحريك، وانقر مرتين لرؤية الكل، وانقر على علامة لتحديدها.

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

clips-title = المقاطع
clips-new = مقطع جديد
clips-add-event = مقطع { $start } – { $end }
clips-add-event-none = مقطع للحدث المحدد
clips-select-event = اختر حدثًا من القائمة أو من الخط الزمني أولًا.
clips-other-session = لا يمكن أن يبدأ مقطع هنا، لأن جلسة سابقة من التسجيل لها الأوقات نفسها.
clips-add-window = مقطع لهذه اللحظة الحماسية
clips-empty = لا توجد مقاطع بعد. اختر حدثًا أو إحدى أكثر اللحظات حماسًا، ثم أضف مقطعًا حوله. يُضبط ما يحتفظ به المقطع قبل الحدث وبعده من الإعدادات. «مقطع جديد» يضيف مقطعًا تكتب بدايته ونهايته بنفسك.
clip-default-name = مقطع { $number }
clip-name = الاسم
clip-length = المدة { $duration }
clip-show = عرض
clip-remove = إزالة
clip-time-help = ثوانٍ (96)، أو دقائق وثوانٍ (1:36)، أو ساعات ودقائق وثوانٍ (00:01:36.250)
clip-exact = يبدأ التصدير وينتهي هنا تمامًا.
clip-effective = يقطع التصدير { $start } – { $end } ({ $duration })، حسب إطارات التسجيل.
timeline-clip-help = المقاطع هي الأشرطة الزرقاء الفاتحة. اسحب طرف الشريط لتحريكه.
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
settings-export-beside = بجانب كل تسجيل
settings-export-hint = داخل مجلد التعديل الذي فيه liblist.gam مثل Half-Life\ag، تُشغَّل المقاطع بأمر playdemo واحد.

clips-store-moved = تعذّرت قراءة ملف المقاطع { $path }، لذا حُفظ باسم { $moved } وتبدأ المقاطع فارغة. { $detail }
clips-store-unreadable = تعذّرت قراءة ملف المقاطع { $path }: { $detail }. لا تُحفظ المقاطع في هذه الجلسة، فلا يضيع شيء مما فيه.
clips-save-failed = تعذّر حفظ المقاطع في { $path }: { $detail }
notice-dismiss = إغلاق

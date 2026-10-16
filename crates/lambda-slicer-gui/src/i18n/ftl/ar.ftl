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
column-action = الإثارة
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
details-action = أفضل نقاط إثارة { NUMBER($score, minimumFractionDigits: 2) }

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
export-missing-parent = مجلد { $path } غير موجود.
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

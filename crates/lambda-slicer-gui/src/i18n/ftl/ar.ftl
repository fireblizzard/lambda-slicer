# العربية. مسودة لم تُراجع

app-title = Lambda Slicer

toolbar-open = فتح تسجيل…
toolbar-add-folder = إضافة مجلد…
toolbar-rescan = إعادة الفحص
toolbar-cancel = إلغاء
toolbar-settings = الإعدادات

settings-title = الإعدادات
settings-language = اللغة
settings-unreviewed = هذه الترجمة مسودة لم يراجعها أي متحدث أصلي بعد.
settings-workers = عمليات التحليل
settings-workers-hint = المزيد من العمليات يحلل أسرع لكنه يستهلك وقتًا أكثر من المعالج.
settings-cache = ذاكرة المكتبة المؤقتة
settings-folders = مجلدات المكتبة
settings-folders-empty = لم تُضف أي مجلدات بعد.
settings-remove = إزالة
settings-close = إغلاق

library-empty = أضف مجلدًا أو افتح تسجيلًا للبدء.
library-demos = { $count ->
    [zero] لا توجد تسجيلات
    [one] تسجيل واحد
    [two] تسجيلان
    [few] { $count } تسجيلات
    [many] { $count } تسجيلًا
   *[other] { $count } تسجيل
}
library-kills = { $count ->
    [zero] لا توجد عمليات قتل
    [one] عملية قتل واحدة
    [two] عمليتا قتل
    [few] { $count } عمليات قتل
    [many] { $count } عملية قتل
   *[other] { $count } عملية قتل
}
library-action = نقاط الإثارة { NUMBER($score, minimumFractionDigits: 2) }
library-spectating = المشاهدة { $percent }٪

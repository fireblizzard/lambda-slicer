# 简体中文。未经审校的草稿

app-title = Lambda Slicer

toolbar-open = 打开录像…
toolbar-add-folder = 添加文件夹…
toolbar-rescan = 重新扫描
toolbar-cancel = 取消
toolbar-settings = 设置

settings-title = 设置
settings-language = 语言
settings-unreviewed = 此翻译为草稿，尚未经母语人士审校。
settings-workers = 分析线程
settings-workers-hint = 线程越多分析越快，但会占用更多处理器时间。
settings-cache = 库缓存
settings-folders = 库文件夹
settings-folders-empty = 尚未添加文件夹。
settings-remove = 移除
settings-close = 关闭

library-empty = 添加文件夹或打开录像即可开始。
library-demos = { $count ->
   *[other] { $count } 个录像
}
library-kills = { $count ->
   *[other] { $count } 次击杀
}
library-action = 精彩度 { NUMBER($score, minimumFractionDigits: 2) }
library-spectating = 观战 { $percent }%

# 简体中文。未经审校的草稿

app-title = Lambda Slicer

## 工具栏

toolbar-open = 打开录像…
toolbar-add-folder = 添加文件夹…
toolbar-rescan = 重新扫描
toolbar-cancel = 取消
toolbar-settings = 设置
dialog-demos = Half-Life 录像

## 设置

settings-title = 设置
settings-language = 语言
settings-unreviewed = 此翻译为草稿，尚未经母语人士审校。
settings-workers = 分析线程
settings-workers-hint = 线程越多分析越快，但会占用更多处理器时间。
settings-cache = 库缓存
settings-folders = 库文件夹
settings-folders-empty = 尚未添加文件夹。
settings-remove = 移除

## 录像库

library-empty = 添加文件夹或打开录像即可开始。
library-no-matches = 没有符合这些筛选条件的录像。
library-no-cache = 此系统上没有可存放库缓存的位置。
library-demos = { $count ->
   *[other] { $count } 个录像
}
library-percent = { $percent }%

filter-search = 搜索路径
filter-map = 地图
filter-player = 玩家
filter-weapon-any = 任意武器
filter-min-minutes = 至少（分钟）
filter-spectating-any = 游戏或观战
filter-spectating-mostly = 主要在观战
filter-spectating-playing = 主要在游戏
filter-clear = 清除筛选

column-status = 状态
column-name = 录像
column-map = 地图
column-duration = 时长
column-gauss = 高斯
column-kills = 击杀
column-action = 激烈度
column-spectating = 观战

status-complete = 已分析
status-partial = 部分
status-failed = 失败
status-unsupported = 不支持
status-no-use = 无武器使用

kind-multiplayer-view = 玩家视角
kind-singleplayer = 单人游戏
kind-hltv = 观战代理
kind-unknown = 未知类型

details-recorder = 录制者 { $name }
details-shots = { $count ->
   *[other] { $count } 次射击
}
details-kills = { $count ->
   *[other] { $count } 次击杀
}
details-action = 最高激烈度 { NUMBER($score, minimumFractionDigits: 2) }

## 扫描

scan-progress = 已找到 { $found }，已完成 { $done }（缓存 { $cached }，已分析 { $analysed }），已读取 { $megabytes } MB
scan-cancelling = 正在停止…
scan-cancelled = 已停止，还有 { $left } 个未扫描。
scan-finished = 完成：{ $done } 个录像，已分析 { $analysed } 个，用时 { $seconds } 秒。
scan-problems = { $count ->
   *[other] { $count } 个问题
}
problems-title = 扫描问题
problem-changed = 文件在分析时发生了变化，因此未保存任何内容。
drop-hint = 拖放录像或文件夹以添加
drop-not-demo = 不是录像：{ $names }

## 分析核心的消息（按代码）

analysis-panicked = 分析因内部错误而停止：{ $detail }
cache-cancelled = 分析已取消，因此未保存任何内容。
cache-database = 无法使用库缓存：{ $detail }
cache-directory = 无法创建缓存文件夹：{ $detail }
container-bad-magic = 这不是 Half-Life 录像。
container-frame-bad-time = 录像在第 { $entry } 部分的第 { $frame } 帧损坏，该帧时间无效。
container-frame-missing-terminal = 录像第 { $entry } 部分缺少结束记录。
container-frame-negative-length = 录像在第 { $entry } 部分的第 { $frame } 帧损坏。
container-frame-over-limit = 第 { $entry } 部分的第 { $frame } 帧超出允许的大小。
container-frame-truncated = 录像在第 { $entry } 部分的第 { $frame } 帧处中断。
container-frame-unknown-tag = 录像在第 { $entry } 部分的第 { $frame } 帧损坏，该帧类型未知。
container-io = 无法读取文件：{ $detail }
container-no-directory = 录像从未完成，因此没有目录。
container-range = 录像目录指向文件之外。
container-too-short = 文件太短，不可能是录像。
coverage-hltv = 观战代理录像。没有录制玩家，事件跟随代理的视角。
coverage-mostly-spectating = 录制者在 { $percent }% 的回放时间里处于观战，因此大多数事件属于其他玩家，即被观看视角所呈现的样子。
coverage-no-local-events = 没有本地事件。除服务器发送的之外，录制者自己的动作缺失。
coverage-singleplayer = 单人游戏。其他角色是怪物或地图实体，没有击杀信息。
coverage-unknown = 录像解码进度不足，无法判断其类型。
coverage-visible-only = 其他玩家的事件只在服务器认为他们可见时才会发送到此客户端，因此其数量是下限。
decode-failed = 解码在 { $time } 停止（第 { $frame } 帧）：{ $detail }
export-active-effect = { $effect } 在开始时正在进行，暂时无法延续。
export-client-dropped-packets = 游戏在 { $time } 丢弃了迟到的数据包，剪辑暂时无法重现这一点。
export-cross-segment = 在范围内的 { $time } 开始了一个新会话。
export-destination-exists = { $path } 已存在。请换一个名称。
export-empty-range = 所选范围内没有可导出的内容。
export-io = 无法写入 { $path }：{ $detail }
export-missing-delta-base = 录像缺少开始前所需的状态（序号 { $sequence }）。
export-missing-parent = { $path } 所在的文件夹不存在。
export-range-beyond-end = 结束点 { $end } 超出了录像结尾 { $segment_end }。
export-same-as-source = 目标就是源录像本身。
export-source-changed = { $path } 在导出期间发生了变化。
export-source-decode-failed = 无法解码源录像：{ $detail }
export-source-failed-crosscheck = 源录像未通过交叉校验，因此其重建状态不可信。
export-unknown-persistent-message = 开始前的 { $name } 设置了剪辑暂时无法延续的状态。
export-unproven-state = { $time } 的状态基于推断的基准，未经证实。
export-unrepresentable-time-field = 开始处的时间无法如实写入：{ $detail }
export-unsupported = 不支持：{ $detail }
export-validation-failed = 导出的文件未通过校验：{ $detail }
io-not-found = 找不到文件或文件夹。
io-other = { $detail }
io-permission-denied = 访问被拒绝。
io-unexpected-end = 文件意外结束。
protocol-unsupported = 不支持录像协议 { $demo }/{ $network }，仅支持 5/48。
protocol-unsupported-network = 服务器切换到了不受支持的网络协议 { $network }。

## 录像视图

demo-close = 关闭
demo-loading = 正在分析 { $name }…
timeline-fit = 适应
show-shots = 射击
show-actions = 其他动作
show-kills = 击杀
show-switches = 切换武器
lane-unknown = 未知玩家
lane-recorder = { $name }（录制者）
column-time = 时间
column-player = 玩家
column-weapon = 武器
column-event = 事件
event-kill = 击杀了 { $victim }
event-switch = 切换武器
action-swing = 挥击
action-fire = 开火
action-launch = 发射
action-charge-activity = 蓄力
action-charge-stop = 蓄力结束
action-beam-start = 光束开始
action-beam-sustain = 光束
action-beam-stop = 光束结束
action-placement-attempt = 放置尝试
action-release-attempt = 释放尝试
mode-primary = 主要
mode-charged = 蓄力
mode-zoomed = 开镜
mode-unzoomed = 不开镜
mode-grenade = 榴弹
mode-bullets = 子弹
mode-single = 单发
mode-double = 双发
mode-rapid = 速射
demo-events = { $count ->
   *[other] { $count } 个事件
}

column-date = 日期
column-score = 比分
filter-from = 从
filter-to = 到
date-from-name = 日期取自文件名
date-from-modified = 日期为文件最后修改时间，复制或下载可能会改变它
timeline-best-action = 黄金一分钟
timeline-windows = 最激烈的时刻
timeline-window = 第 { $rank } 名 { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = 最终比分（最后一次击杀 { $time }）
scores-at = { $time } 时的比分
scores-show-final = 显示最终比分
scores-frags = 击杀
scores-deaths = 死亡

show-timeline = 显示时间线
legend-shot = 射击
legend-charge = 蓄力
legend-kill = 击杀
legend-death = 死亡
legend-switch = 切换武器
timeline-help = 每一行是一名玩家。滚动以缩放，拖动以移动，双击查看全部，点击标记以选中。

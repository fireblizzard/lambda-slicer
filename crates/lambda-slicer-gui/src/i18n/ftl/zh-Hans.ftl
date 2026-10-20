# 简体中文。未经审校的草稿

app-title = Lambda Slicer

## 工具栏

toolbar-open = 打开录像…
toolbar-add-folder = 添加文件夹…
toolbar-rescan = 重新扫描
toolbar-cancel = 取消
toolbar-settings = 设置
toolbar-rescan-busy = 已在扫描。
toolbar-rescan-nothing = 先添加文件夹或打开录像再扫描。
toolbar-cancel-idle = 当前没有扫描。
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
library-demos-of = { $shown } / { $total } 个录像
library-total = 库中共 { $count } 个录像
filters-title = 筛选
filters-find = 查找
filters-pov = 视角
filters-quality = 质量
filters-recorded = 录制日期
pov-any = 全部
pov-playing = 游戏
pov-spectating = 观战
help-pov = “游戏”指录像观战时间不超过回放的一半，“观战”指超过一半。
columns-button = 列
list-back = 返回录像库表格
sort-by = 排序方式
sort-ascending = 升序
sort-descending = 降序
score-more-sides = 另有 { $count } 方
intensity-help = 激烈度 { $score }。进度条与库中最高值 { $max } 比较。
value-unknown = 未知
path-copy = 复制路径
path-show = 在文件夹中显示
path-show-failed = 无法打开文件夹（{ $detail }）。请改为复制路径。

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
export-missing-parent = 文件夹 { $path } 不存在。
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
show-shots = 射击
show-actions = 其他动作
show-kills = 击杀
show-switches = 切换武器
lane-unknown = 未知玩家
lane-recorder = { $name }（录制者）
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
timeline-window = 第 { $rank } 名 { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = 最终比分（最后一次击杀 { $time }）
scores-at = { $time } 时的比分
scores-show-final = 显示最终比分
scores-frags = 击杀
scores-deaths = 死亡

viewer-play = 播放
viewer-pause = 暂停
viewer-volume = 音量
viewer-no-sound = 无声音
viewer-play-backward = 倒带
viewer-step-back = 上一帧
viewer-step-forward = 下一帧
viewer-rate = { $rate }×
viewer-shortcut = 快捷键：{ $key }
key-space = 空格
viewer-map-failed = 无法加载地图 { $map }
viewer-stopped = 播放提前停止：{ $reason }
viewer-missing = 无法读取的文件：{ $count }
event-previous = 上一个事件
event-next = 下一个事件
event-play = 播放事件
timeline-title = 时间线
timeline-lanes = 玩家分行
timeline-zoom-in = 放大
timeline-zoom-out = 缩小
timeline-fit-all = 显示整个录像
timeline-gestures = 点击标记可选中其事件，点击其他位置则跳转到那里。拖动标尺或播放头可快速浏览，在其他位置拖动可平移。滚动可围绕指针缩放，双击可查看整个录像。拖动片段的边缘可移动它，点击片段可编辑它。
timeline-density = 阴影条显示每一段中有多少已显示的事件，以录像中最密集的一段为基准。
timeline-density-count = { $count ->
   *[other] 此处有 { $count } 个已显示事件
}
timeline-session-count = 会话 { $session }：{ $count }
timeline-choose = 此处的事件
legend-clip = 片段
event-session = 会话 { $session }
event-unreachable = 此事件位于一个重复使用相同时间的较晚会话中，因此查看器无法跳转到它。
event-hidden = 所选事件已被筛选条件隐藏。
highlight-jump = 跳转到高光…
highlight-help = 高光按精彩程度为整个录像排序，与筛选显示的内容无关。
kills-with = 击杀条件
refine-midair = 空中
refine-speed = 速度
refine-wallgauss = 穿墙高斯
refine-clear = 清除
refine-airborne = 谁在空中
refine-either = 任一方
midair-both = 双方
midair-one = 仅一方
help-midair-either = 击杀者、受害者或双方在空中。击杀者在与击杀关联的射击时检查，没有关联射击则在击杀时检查；受害者在死亡时检查。
help-midair-both = 击杀者和受害者在各自检查时都在空中。
help-midair-one = 恰好其中一方在空中，且已知另一方在地面上。
speed-at-least = 至少
speed-unit = ups
speed-killer = 击杀者
speed-victim = 受害者
help-speed = 沿地面方向的水平速度，单位为每秒单位数。击杀者在与击杀关联的射击时测量，没有关联射击则在击杀时测量；受害者在死亡时测量。其他玩家的速度为估算值。
wallgauss-weapon = 穿墙高斯只涵盖高斯击杀，而武器筛选显示的是 { $weapon }。
wallgauss-checking = 正在检查穿墙高斯… { $examined } / { $total } 个击杀
wallgauss-needs-files = 穿墙高斯需要游戏文件，才能在地图中追踪射击。
wallgauss-failed = 无法检查穿墙高斯。{ $reason }
help-wallgauss = 穿过墙壁的高斯击杀，按服务器的方式在地图中追踪每次射击得出。无法判定的击杀（例如受害者在录像视野之外）计为未知并被排除。
badge-wall = 穿墙
wall-yes-exit = 经重建判定为穿墙高斯：光束穿出 { $thickness } 单位厚的墙时产生的爆炸。
wall-yes-beyond = 经重建判定为穿墙高斯：穿过 { $thickness } 单位厚的墙之后的光束。
wall-limits = 根据射击和快照重建，并非服务器记录。玩家按方盒追踪，受害者的实际位置可能与快照显示的相差几十个单位。
wall-no-direct = 不是穿墙高斯：光束可能在碰到任何墙之前就击中了受害者。
wall-no-reflection = 不是穿墙高斯：反射的爆炸可能在碰到任何墙之前就击中了受害者。
wall-doubt-no-shot = 穿墙高斯未知：录像中没有击杀者的射击。
wall-doubt-no-map = 穿墙高斯未知：无法读取地图文件。
wall-doubt-unseen = 穿墙高斯未知：受害者在录像视野之外。
wall-doubt-brush = 穿墙高斯未知：光束碰到了门、平台或其他 brush 实体，这些不会被追踪。
wall-doubt-ambiguous = 穿墙高斯未知：这次射击既可能穿墙击杀，也可能不穿墙击杀。
wall-doubt-unexplained = 穿墙高斯未知：找不到射击击中受害者的路径。
wall-doubt-unconfirmed = 穿墙高斯未知：射击只可能穿墙击中受害者，但无法确定。
wall-doubt-not-reached = 穿墙高斯未知：录像在射击之前就结束了。
wall-pending = 尚未检查穿墙高斯。
events-unknown = { $count ->
   *[other] { $count } 次击杀缺少判断所需的数据
}
help-unknown = 录像无法看到的移动计为未知，绝不计为在地面或静止，因此这些击杀被排除。
clip-it = 剪辑
badge-headshot = 爆头
badge-air = 空中
badge-air-both = 空中 ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = 自杀
badge-teamkill = 误杀队友
badge-world = 环境
air-both = 检查时双方都在空中。
air-killer = 检查时击杀者在空中。
air-victim = 检查时受害者在空中。
air-other-unknown = 另一方是否在空中未知。
speed-horizontal = 沿地面方向的水平速度。
motion-line = { $who }：{ $when }{ $air }，{ $speed }。{ $how }。
motion-who-killer = 击杀者
motion-who-victim = 受害者
motion-air-yes = 在空中
motion-air-no = 在地面
motion-air-unknown = 不确定是否在空中
motion-speed-unknown = 速度未知
motion-exact = 由游戏记录
motion-trajectory = 根据移动估算
motion-unclear = 移动不明
motion-not-visible = 不在视野内
motion-at-shot = 关联射击时
motion-at-kill = 死亡时
motion-none = 分析中没有此次击杀的移动数据。
legend-shot = 射击
legend-charge = 蓄力
legend-kill = 击杀
legend-death = 死亡
legend-switch = 切换武器

column-headshots = 爆头
filter-headshots = 爆头击杀
event-headshot = 爆头
scores-headshots = 爆头

column-accuracy = 命中率
filter-min-accuracy = 命中率至少（%）
scores-accuracy = 命中率

help-headshots = 爆头击杀，根据受害者的死亡动作判断。爆炸不计入。只显示录制者的爆头，因为其他玩家常在录像视野之外击杀。在一对一中也显示对手的爆头，因为对手的击杀都是针对录制者。
help-headshots-column = 录制者的爆头击杀，根据受害者的死亡动作判断。爆炸不计入。其他玩家的击杀不计入，因为它们常发生在录像视野之外。
help-accuracy = 命中射击所占比例，根据命中在瞄准线上留下的血迹判断，并扣除偶然重合。包括格洛克、蟒蛇、MP5 子弹、霰弹枪、开镜十字弩和高斯光束的直接命中。爆炸和高斯反射不留血迹，不计入。只显示录制者的命中率，且至少 50 次射击后才显示。
help-headshot-filter = 至少有一次录像中可见的爆头击杀的录像，任何玩家均可。
help-accuracy-filter = 录制者命中率至少达到该比例的录像。

clips-title = 片段
clips-new = 新建片段
clips-add-event = 片段 { $start } – { $end }
clips-add-event-none = 为所选事件建片段
clips-select-event = 请先在列表或时间轴上选择一个事件。
clips-other-session = 片段不能从这里开始，因为录像中较早的会话有相同的时间。
clips-add-window = 为这个激烈时刻建片段
clips-empty = 还没有片段。选择一个事件或一个最激烈的时刻，然后在其周围添加片段。片段在事件前后保留多长时间可在设置中调整。“新建片段”会添加一个由你自己输入起点和终点的片段。
clip-default-name = 片段 { $number }
clip-name = 名称
clip-length = 时长 { $duration }
clip-show = 显示
clip-remove = 移除
clip-loop = 循环
clip-time-help = 秒（96）、分和秒（1:36），或时、分和秒（00:01:36.250）
clip-exact = 导出将正好在此开始和结束。
clip-effective = 按录像的帧，导出将剪切 { $start } – { $end }（{ $duration }）。
settings-clip-padding = 片段余量
settings-preroll = 事件前的秒数
settings-postroll = 事件后的秒数
export-start-first-group = 剪切不能从录像的第一帧开始。请稍后一点开始。
export-start-during-loading = 起点位于游戏仍在加载地图时。请稍后一点开始。

export-all = 导出全部片段
export-one = 导出
export-cancel = 停止导出
export-folder = 导出文件保存到 { $folder }。
export-folder-beside = 导出文件保存在录像旁边，即 { $folder }。
export-waiting = 等待导出…
export-running = 正在导出…这需要读取整个录像，可能需要一段时间。
export-cancelled = 未导出，因为导出已停止。
export-saved = 已保存到 { $path }。与原录像核对过的快照：{ $count }。
export-playdemo = 在游戏控制台中：
export-copy-first = 把 { $name } 复制到包含 liblist.gam 的 mod 文件夹，然后在游戏控制台中：
export-copy = 复制
settings-export-folder = 导出文件夹
settings-export-choose = 选择…
settings-viewer = 录像查看器
settings-viewer-game = Half-Life 文件夹
settings-viewer-game-none = 尚未选择
settings-viewer-game-hint = 包含 valve 的文件夹。查看器用其中的游戏地图、模型和声音播放录像。
settings-executable = 游戏可执行文件
settings-executable-hint = 用来找到 Half-Life 文件夹。查看器自己读取游戏文件，从不启动游戏。
settings-executable-not-set = 未设置，所以查看器没有画面和声音
settings-executable-optional = 未设置，只要 Half-Life 文件夹里有游戏就不需要
settings-executable-missing = { $path } 已不存在。查看器仍读取 Half-Life 文件夹。
settings-game-override = 本次运行使用 --game 指定的 { $path }。这里的选择从下次运行起生效。
settings-browse = 浏览…
dialog-executables = 程序
setup-title = 设置录像查看器
setup-body = 画面和声音需要 Half-Life 的游戏文件。告诉 Lambda Slicer 游戏装在哪里，它会从那里读取。
setup-locate-exe = 找到 hl.exe…
setup-locate-folder = 找到 Half-Life…
setup-settings = 设置
setup-still-works = 时间线、事件和片段不需要它们也能用。
setup-relocate = { $folder } 中已没有 Half-Life。请重新找到它。
setup-not-executable = { $name } 不是 { $executable }。请选择游戏本身的可执行文件，而不是快捷方式。
setup-not-installed = { $folder } 中没有带 valve 文件夹的 Half-Life 安装。
tab-events = 事件
tab-clips = 片段
tab-scoreboard = 记分板
list-hide = 隐藏录像列表
list-show = 显示录像列表
demo-details = 详情
viewer-start = 回到开头
viewer-mute = 静音
viewer-unmute = 取消静音
viewer-picture = 画面与查看器设置
viewer-needs-files = 需要游戏文件
viewer-no-sound-why = 没有声音：{ $reason }
settings-tab-general = 常规
settings-section-folders = 文件夹
settings-folders-hint = 库在其中查找录像的文件夹，包括子文件夹。
settings-clip-padding-hint = 围绕事件创建的片段在事件前后保留多少时间。
settings-section-picture = 画面
settings-section-sound = 声音
settings-gamma = 伽马
settings-brightness = 亮度
settings-texgamma = 纹理伽马
settings-lightgamma = 光照伽马
settings-overbright = 过亮光照
settings-gamma-hint = 游戏中同名的设置（gamma, brightness, texgamma, lightgamma, gl_overbright）。按游戏中的值设置，即可看到相同的亮度。
settings-gamma-defaults = 游戏默认值
settings-fov-recorded = 按录像的视野
settings-fov = 视野
settings-viewmodel = 显示手中的武器
settings-see-through = 透过墙壁看到玩家
settings-see-through-items = 也包括其他模型，如投射物和物品
settings-see-through-hint = 墙后的玩家以其上衣颜色的剪影显示，团队游戏会按队伍设置该颜色。只能显示录像中包含的内容。服务器只发送录制者附近的玩家，隐藏录制者看不到的玩家的服务器不会留下任何可显示的内容。
viewer-see-through = 透视墙壁
settings-export-beside = 每个录像旁边
settings-export-hint = 在包含 liblist.gam 的 mod 文件夹中（例如 Half-Life\ag），片段只需一条 playdemo 命令即可播放。

clips-store-moved = 无法读取片段文件 { $path }，因此已将其保留为 { $moved }，片段从空开始。{ $detail }
clips-store-unreadable = 无法读取片段文件 { $path }：{ $detail }。本次会话不保存片段，因此其中的内容不会丢失。
clips-save-failed = 无法将片段保存到 { $path }：{ $detail }
notice-dismiss = 关闭

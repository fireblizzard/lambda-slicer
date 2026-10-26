# Português do Brasil. Rascunho não revisado

app-title = Lambda Slicer

## Barra de ferramentas

toolbar-open = Abrir demo…
toolbar-add-folder = Adicionar pasta…
toolbar-rescan = Escanear de novo
toolbar-cancel = Cancelar
toolbar-settings = Configurações
toolbar-rescan-busy = Já há um escaneamento em andamento.
toolbar-rescan-nothing = Adicione uma pasta ou abra uma demo para escanear.
toolbar-cancel-idle = Nenhum escaneamento em andamento.
dialog-demos = Demos de Half-Life

## Configurações

settings-title = Configurações
settings-language = Idioma
settings-unreviewed = Esta tradução é um rascunho que nenhum falante nativo revisou ainda.
settings-workers = Demos analisadas ao mesmo tempo
settings-workers-hint = Analisar mais ao mesmo tempo termina antes, mas faz o computador trabalhar mais.
settings-cache = Onde a biblioteca fica guardada
settings-start-tab = Aba ao abrir
settings-start-tab-hint = A aba da biblioteca mostrada cada vez que o Lambda Slicer abre. Clicar em outra aba ainda mostra essa aba, até você fechar o Lambda Slicer.
settings-sort = Ordem ao abrir
settings-sort-hint = Como as demos são ordenadas cada vez que o Lambda Slicer abre. Clicar no título de uma coluna ainda ordena por ela, até você fechar o Lambda Slicer.
settings-sprees = Sequências de abates
settings-sprees-hint = O último abate de um jogador antes de morrer mostra um selo com quantos abates ele fez seguidos, quando são pelo menos esses. Morrer de qualquer jeito encerra a sequência, assim como o fim do mapa ou a saída do jogador. Teamkills não contam.
settings-spree-least = Pelo menos (abates)
settings-start-filters =Filtros ao abrir
settings-start-filters-hint = A biblioteca começa com estes filtros cada vez que o Lambda Slicer abre, para que demos curtas que você nunca quer ver não atrapalhem. Você ainda pode mudar os filtros até fechar o Lambda Slicer.
settings-hide-unplayed-maps = Em demos com vários mapas, esconder os mapas sem abates e com menos de 5 minutos gravados
settings-folders = Pastas da biblioteca
settings-folders-empty = Nenhuma pasta adicionada ainda.
settings-remove = Remover

## Biblioteca

library-empty = Adicione uma pasta ou abra uma demo para começar.
library-no-matches = Nenhuma demo corresponde a estes filtros.
library-no-cache = Não há nenhum lugar neste computador para guardar a biblioteca.
library-demos = { $count ->
    [one] { $count } demo
    [many] { $count } de demos
   *[other] { $count } demos
}
library-percent = { $percent }%
library-demos-of = { $shown } de { $total ->
    [one] { $total } demo
    [many] { $total } de demos
   *[other] { $total } demos
}
library-matches = { $count ->
    [one] { $count } partida
    [many] { $count } de partidas
   *[other] { $count } partidas
}
library-matches-of = { $shown } de { $total ->
    [one] { $total } partida
    [many] { $total } de partidas
   *[other] { $total } partidas
}
library-no-games = Nenhuma das demos tem uma partida, nem cinco minutos de jogo sem partida.
library-no-games-filtered = Nenhuma partida corresponde a estes filtros.
library-total = { $count ->
    [one] { $count } demo na biblioteca
    [many] { $count } de demos na biblioteca
   *[other] { $count } demos na biblioteca
}
filters-title = Filtros
filters-find = Buscar
filters-pov = Ponto de vista
filters-quality = Qualidade
filters-kills = Abates
filters-recorded = Gravada
pov-playing = Jogando
pov-spectating = Espectando
help-pov = “Jogando” quer dizer que a gravação passou no máximo metade da reprodução espectando, e “Espectando”, mais da metade.
columns-button = Colunas
list-back = Voltar à tabela da biblioteca
sort-by = Ordenar por
sort-ascending = Crescente
sort-descending = Decrescente
score-more-sides = { $count ->
    [one] mais { $count } lado
    [many] mais { $count } de lados
   *[other] mais { $count } lados
}
intensity-help = Intensidade { $score }. A barra a compara com a maior da biblioteca, { $max }.
value-unknown = Desconhecido
path-copy = Copiar local do arquivo
path-show = Mostrar na pasta
path-show-failed = Não foi possível abrir a pasta ({ $detail }). Copie o local do arquivo.

filter-search = Buscar nomes de arquivos e pastas
filter-map = Mapa
filter-player = Jogador
help-player-filter = Digite parte de um nome para encontrar demos em que alguém o usou. Escolha um jogador da lista para encontrar as demos em que ele jogou com qualquer nome, reconhecido pelo SteamID. Todos os jogadores escolhidos precisam ter jogado, não só assistido. Bots e algumas demos antigas não têm SteamID, então esses jogadores só podem ser encontrados pelo nome.
player-remove = Tirar do filtro
player-also = também { $names }
player-demos = { $count ->
    [one] { $count } demo
    [many] { $count } de demos
   *[other] { $count } demos
}
filter-weapon-any = Qualquer arma
filter-min-minutes = Pelo menos (minutos)
filter-min-kills = Pelo menos (abates)
filter-min-spree = Sequência de pelo menos (abates)
filter-players = Jogadores
filter-teams = Equipes
filter-bots = Bots
bots-any = Qualquer
bots-few = Poucos
bots-none = Nenhum
filter-spectating-any = Jogando ou espectando
filter-spectating-mostly = Quase só espectando
filter-spectating-playing = Quase só jogando
filter-clear = Limpar filtros

column-status = Estado
column-name = Demo
column-map = Mapa
column-duration = Duração
column-gauss = Gauss
column-kills = Abates
column-action = Intensidade
column-spectating = Espectando

status-complete = Analisada
status-partial = Parcial
status-failed = Falhou
status-unsupported = Não suportada
status-no-use = Sem uso de armas

kind-multiplayer-view = Visão do jogador
kind-singleplayer = Um jogador
kind-hltv = HLTV
kind-unknown = Tipo desconhecido

details-recorder = Gravada por { $name }
details-shots = { $count ->
    [one] { $count } disparo
    [many] { $count } de disparos
   *[other] { $count } disparos
}
details-kills = { $count ->
    [one] { $count } abate
    [many] { $count } de abates
   *[other] { $count } abates
}
details-action = Maior intensidade { NUMBER($score, minimumFractionDigits: 2) }

## Escaneamento

scan-progress = Encontradas { $found }, prontas { $done } ({ $cached } já conhecidas, { $analysed } analisadas), { $megabytes } MB lidos
scan-cancelling = Parando…
scan-cancelled = Parado com { $left } por escanear.
scan-finished = Pronto: { $done ->
    [one] { $done } demo
    [many] { $done } de demos
   *[other] { $done } demos
}, { $analysed ->
    [one] { $analysed } analisada
   *[other] { $analysed } analisadas
}, em { $seconds } s.
scan-problems = { $count ->
    [one] { $count } problema
    [many] { $count } de problemas
   *[other] { $count } problemas
}
problems-title = Problemas do escaneamento
problem-changed = O arquivo mudou durante a análise, então nada foi salvo.
drop-hint = Solte demos ou pastas para adicioná-las
drop-not-demo = Não é uma demo: { $names }

## Mensagens do núcleo de análise, por código

analysis-panicked = A análise parou por causa de uma falha no Lambda Slicer: { $detail }
cache-cancelled = A análise foi cancelada, então nada foi salvo.
cache-database = Não é possível usar a biblioteca salva: { $detail }
cache-directory = Não é possível criar a pasta da biblioteca salva: { $detail }
container-bad-magic = Isto não é uma demo de Half-Life.
container-frame-bad-time = A gravação está danificada no quadro { $frame } da parte { $entry }, que tem um tempo errado.
container-frame-missing-terminal = A parte { $entry } da gravação não termina direito.
container-frame-negative-length = A gravação está danificada no quadro { $frame } da parte { $entry }.
container-frame-over-limit = O quadro { $frame } da parte { $entry } é maior do que o permitido.
container-frame-truncated = A gravação é cortada no quadro { $frame } da parte { $entry }.
container-frame-unknown-tag = A gravação está danificada no quadro { $frame } da parte { $entry }, que tem algo que o Lambda Slicer não reconhece.
container-io = Não é possível ler o arquivo: { $detail }
container-no-directory = A gravação nunca foi finalizada, então não tem índice.
container-range = O índice da gravação aponta para fora do arquivo.
container-too-short = O arquivo é curto demais para ser uma demo.
coverage-hltv = Gravação de HLTV. Nenhum jogador gravou, então os eventos seguem o que a HLTV estava assistindo.
coverage-mostly-spectating = Quem gravou ficou espectando em { $percent }% da reprodução, então a maioria dos eventos é de outros jogadores, como foram vistos enquanto eram espectados.
coverage-no-local-events = As ações de quem gravou não foram salvas nesta demo, então só aparecem as que o servidor enviou.
coverage-singleplayer = Um jogador. Todo o resto são monstros ou partes do mapa, e não há kill feed.
coverage-unknown = Não deu para ler a gravação o suficiente para saber de que tipo ela é.
coverage-visible-only = O jogo só recebe as ações dos outros jogadores enquanto o servidor acha que quem gravou consegue vê-los, então as contagens deles podem ser menores do que o que realmente aconteceu.
decode-failed = A leitura da gravação parou em { $time } (quadro { $frame }): { $detail }
export-active-effect = { $effect } ainda está acontecendo no início, e um clipe ainda não consegue levar isso junto.
export-client-dropped-packets = Em { $time } o jogo jogou fora dados que chegaram atrasados, algo que um clipe ainda não consegue copiar.
export-clock-starts-over = O relógio da demo recomeçou em { $time }, depois de uma troca de mapa ou reconexão dentro do clipe, e um clipe não pode passar por isso.
export-destination-exists = { $path } já existe. Escolha outro nome.
export-empty-range = O clipe não tem nada para exportar.
export-io = Não é possível gravar { $path }: { $detail }
export-missing-delta-base = Faltam na gravação dados de antes do início que o clipe precisa (número { $sequence }).
export-missing-parent = A pasta { $path } não existe.
export-range-beyond-end = O fim { $end } passa do fim da gravação, { $segment_end }.
export-same-as-source = A exportação substituiria a própria demo de onde ela vem.
export-source-changed = { $path } mudou durante a exportação.
export-source-decode-failed = Não é possível ler a demo: { $detail }
export-source-failed-crosscheck = Na conferência, o que o Lambda Slicer deduziu desta demo não bateu com a própria demo, então não dá para confiar nos clipes dela.
export-unknown-persistent-message = A mensagem { $name } ainda não é compatível com clipes.
export-unproven-state = Em { $time } a gravação depende de dados que tiveram que ser adivinhados, então o clipe não pode ser feito com exatidão.
export-unrepresentable-time-field = Um tempo no início não pode ser salvo com exatidão: { $detail }
export-unsupported = Não suportado: { $detail }
export-validation-failed = O clipe exportado não passou na conferência final: { $detail }
io-not-found = O arquivo ou a pasta não foi encontrado.
io-other = { $detail }
io-permission-denied = Acesso negado.
io-unexpected-end = O arquivo terminou inesperadamente.
protocol-unsupported = Esta demo é de uma versão do jogo que não é suportada (protocolo { $demo }/{ $network }). Só a 5/47 e a 5/48 são.
protocol-unsupported-network = O servidor mudou para uma versão do jogo que não é suportada (protocolo { $network }).

## Visão da demo

demo-close = Fechar
demo-loading = Analisando { $name }…
show-shots = Disparos
show-actions = Outras ações
show-kills = Abates
show-switches = Trocas de arma
show-chat = Chat
show-match = Partida
lane-unknown = Jogador desconhecido
lane-recorder = { $name } (gravou)
event-kill = Abateu { $victim }
event-switch = Troca de arma
event-copy = Copiar
event-copy-kill = { $killer } matou { $victim } com { $weapon }
event-copy-teamkill = { $killer } matou { $victim }, do próprio time, com { $weapon }
event-copy-suicide = { $victim } se matou com { $weapon }
event-copy-death = { $victim } morreu por { $weapon }
event-vote-called = Votação de { $subject } iniciada
event-vote-cast = Votação de { $subject }: { $change } ({ $tally })
event-vote-more-yes = { $count ->
    [one] mais um sim
    [many] mais { $count } sim
   *[other] mais { $count } sim
}
event-vote-fewer-yes = { $count ->
    [one] um sim a menos
    [many] { $count } sim a menos
   *[other] { $count } sim a menos
}
event-vote-more-no = { $count ->
    [one] mais um não
    [many] mais { $count } não
   *[other] mais { $count } não
}
event-vote-fewer-no = { $count ->
    [one] um não a menos
    [many] { $count } não a menos
   *[other] { $count } não a menos
}
event-vote-tally = { $yes } sim, { $no } não, { $undecided } sem votar
event-vote-accepted = Votação de { $subject } aprovada
event-vote-denied = Votação de { $subject } recusada
event-timeout-called = Timeout pedido
event-timeout-under-way = Timeout iniciado
event-timeout-over = Fim do timeout
event-paused = Jogo pausado
event-resumed = Jogo retomado
event-sudden-death = Morte súbita iniciada
event-sudden-death-over = Fim da morte súbita
event-map-changed = Mapa trocado para { $map }
event-spectating = Começou a espectar
event-playing = Parou de espectar
action-swing = Golpe
action-fire = Disparo
action-launch = Lançamento
action-charge-activity = Carregando
action-charge-stop = Fim da carga
action-beam-start = Início do raio
action-beam-sustain = Raio
action-beam-stop = Fim do raio
action-placement-attempt = Tentativa de colocação
action-release-attempt = Tentativa de liberação
mode-primary = primário
mode-charged = carregado
mode-zoomed = com zoom
mode-unzoomed = sem zoom
mode-grenade = granada
mode-bullets = balas
mode-single = simples
mode-double = duplo
mode-rapid = rápido

column-date = Data
column-file-date = Data do arquivo
column-score = Placar
filter-from = De
filter-to = Até
date-from-name = Data tirada do nome do arquivo
date-from-modified = Data da última modificação do arquivo, que copiar ou baixar pode alterar
timeline-best-action = Minuto de ouro
timeline-window = Nº { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Placar final, no último abate { $time }
scores-at = Placar em { $time }
scores-show-final = Ver placar final
scores-spectators = Espectadores
scores-follow = Clique para assistir pelos olhos dele
pane-fill = Dar o painel lateral inteiro para esta parte
pane-share = Dividir o painel lateral meio a meio
scores-frags = Abates
scores-deaths = Mortes

viewer-play = Reproduzir
viewer-pause = Pausar
viewer-volume = Volume
viewer-no-sound = Sem som
viewer-play-backward = Rebobinar
viewer-step-back = Frame anterior
viewer-step-forward = Próximo frame
viewer-position = Posição na demo
viewer-skip-back = Voltar 10 segundos
viewer-skip-forward = Avançar 10 segundos
viewer-rate = { $rate }×
viewer-slower = Mais lento
viewer-faster = Mais rápido
viewer-rate-normal = Voltar à velocidade normal
viewer-shortcut = Atalho: { $key }
key-space = Espaço
key-ctrl = Ctrl
key-shift = Shift
key-alt = Alt
key-left = Esquerda
key-right = Direita
key-up = Cima
key-down = Baixo
key-home = Home
key-end = End
key-page-up = Page Up
key-page-down = Page Down
key-enter = Enter
key-backspace = Backspace
key-delete = Delete
key-insert = Insert
key-tab = Tab
key-mouse-3 = Mouse 3
key-mouse-4 = Mouse 4
key-mouse-5 = Mouse 5
key-wheel-up = Rodinha para cima
key-wheel-down = Rodinha para baixo
viewer-map-failed = Não foi possível carregar o mapa { $map }
viewer-stopped = A reprodução parou antes do fim: { $reason }
viewer-missing = Arquivos que não puderam ser lidos: { $count }
event-previous = Evento anterior
event-next = Próximo evento
event-details = Detalhes do abate
event-play = Reproduzir evento
timeline-title = Linha do tempo
timeline-lanes = Linhas por jogador
timeline-zoom-in = Aproximar
timeline-zoom-out = Afastar
timeline-fit-all = Ajustar a demo inteira
timeline-gestures = Clique numa marca para selecionar o evento dela, ou em qualquer outro lugar para ir até lá. Arraste sobre os tempos no topo, ou arraste a linha que marca o momento atual, para andar pela demo. Arraste em qualquer outro lugar para deslizar a visão. Role para dar zoom e tirar o zoom onde está o mouse, e clique duas vezes para ver a demo inteira. Arraste a borda de um clipe para mudar onde ele começa ou termina, ou clique no clipe para editá-lo.
timeline-density = As barras sombreadas mostram quantos dos eventos exibidos acontecem em cada ponto, comparado com o ponto mais movimentado da demo.
timeline-density-count = { $count ->
    [one] Um evento exibido aqui
    [many] { $count } de eventos exibidos aqui
   *[other] { $count } eventos exibidos aqui
}
timeline-session-count = Parte { $session }: { $count }
timeline-choose = Eventos neste ponto
legend-clip = Clipe
event-session = Parte { $session }
event-unreachable = Este evento vem depois de uma troca de mapa ou reconexão, onde o relógio da demo recomeçou, então o visualizador não consegue ir até ele.
event-hidden = Os filtros ocultam o evento selecionado.
highlight-jump = Ir para um highlight…
highlight-help = Os highlights classificam a demo inteira pela ação, seja o que for que os filtros mostrem.
kills-with = Abates com
refine-midair = Midair
refine-speed = Velocidade
refine-wallgauss = Wallgauss
refine-multikill = Multikill
refine-clear = Limpar
refine-airborne = No ar
refine-either = Qualquer um
midair-both = Ambos
midair-one = Só um
help-midair-either = O matador, a vítima ou ambos estavam no ar. O matador é verificado no disparo ligado ao abate, ou no abate se não houver um, e a vítima na morte. Um simples pulo, a menos de 50 unidades acima do chão, não conta. É preciso ter escolhido a pasta do Half-Life para saber onde fica o chão.
help-midair-both = O matador e a vítima estavam ambos no ar, cada um ao ser verificado.
help-midair-one = Exatamente um dos dois estava no ar, e sabe-se que o outro estava no chão.
speed-at-least = Pelo menos
speed-unit = ups
speed-killer = Matador
speed-victim = Vítima
help-multikill = Vários jogadores mortos com um só disparo, como uma explosão que atinge todos eles ou um raio do gauss que os atravessa. Todos os abates desse disparo mostram isso.
help-speed = Velocidade pelo chão em ups, como um velocímetro mostra, sem contar o movimento para cima e para baixo. O matador é medido no disparo ligado ao abate, ou no abate se não houver um, e a vítima na morte. As velocidades dos outros jogadores são estimativas.
wallgauss-weapon = Wallgauss só abrange abates com o gauss, e o filtro de arma mostra { $weapon }.
wallgauss-checking = Verificando Wallgauss… { $examined } de { $total } abates
wallgauss-needs-files = Wallgauss precisa dos arquivos do jogo para saber onde ficam as paredes do mapa.
wallgauss-failed = Não foi possível verificar Wallgauss. { $reason }
help-wallgauss = Abates com o gauss através de uma parede, encontrados seguindo cada disparo pelo mapa do jeito que o servidor faz. Um abate que não dá para decidir, como um em que a vítima estava fora da visão da gravação, conta como desconhecido e fica de fora.
badge-wall = WG
wall-yes-exit = Wallgauss, deduzido do disparo: a explosão onde o raio saiu de uma parede de { $thickness } unidades de espessura.
wall-yes-beyond = Wallgauss, deduzido do disparo: o raio acertou depois de atravessar uma parede de { $thickness } unidades de espessura.
wall-limits = Deduzido do disparo e de onde os jogadores estavam, já que o servidor não registra isso. Um jogador tem 32 unidades de largura e conta como uma caixa, e a vítima pode ter estado a um ou dois corpos de distância de onde a demo a mostra.
wall-no-direct = Não é Wallgauss: o raio poderia ter acertado a vítima antes de qualquer parede.
wall-no-reflection = Não é Wallgauss: a explosão de um reflexo poderia ter acertado a vítima antes de qualquer parede.
wall-doubt-no-shot = Wallgauss desconhecido: o disparo do matador não está na gravação.
wall-doubt-no-map = Wallgauss desconhecido: não foi possível ler os arquivos do mapa.
wall-doubt-unseen = Wallgauss desconhecido: a vítima estava fora da visão da gravação.
wall-doubt-brush = Wallgauss desconhecido: o raio acertou uma porta, um elevador ou outra parte especial do mapa, que não é verificada.
wall-doubt-ambiguous = Wallgauss desconhecido: o disparo poderia ter matado tanto através de uma parede quanto sem ela.
wall-doubt-unexplained = Wallgauss desconhecido: não foi encontrado nenhum caminho pelo qual o disparo chegou à vítima.
wall-doubt-unconfirmed = Wallgauss desconhecido: o disparo só poderia ter chegado à vítima através de uma parede, mas não com certeza.
wall-doubt-not-reached = Wallgauss desconhecido: a gravação termina antes do disparo.
wall-pending = Wallgauss ainda não verificado.
events-unknown = { $count ->
    [one] Um abate não tem dados para dizer
    [many] { $count } de abates não têm dados para dizer
   *[other] { $count } abates não têm dados para dizer
}
help-unknown = O movimento que a gravação não conseguiu ver conta como desconhecido, nunca como no chão ou parado, então esses abates ficam de fora.
events-filters = Filtros
events-filters-on = Filtros ({ $count })
events-kinds-none = Nenhum evento
events-follow = Manter a lista no momento atual enquanto a demo é reproduzida. Rolar a lista desativa isso.
clip-it = Clipar
badge-headshot = HS
badge-double-kill = double kill
badge-triple-kill = triple kill
badge-quadra-kill = quadra kill
badge-penta-kill = penta kill
badge-multikill = { $count } abates
badge-spree = ×{ $count }
badge-spree-help = { $count ->
    [one] { $count } abate seguido sem morrer
    [many] { $count } de abates seguidos sem morrer
   *[other] { $count } abates seguidos sem morrer
}
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = suicídio
badge-teamkill = teamkill
badge-world = mundo
badge-clip = clipe
clip-of = Clipe de { $name }
air-both = Ambos estavam no ar ao serem verificados.
air-killer = O matador estava no ar ao ser verificado.
air-victim = A vítima estava no ar ao ser verificada.
air-other-unknown = Não se sabe se o outro também estava.
speed-horizontal = Velocidade horizontal, paralela ao chão.
motion-line = { $who }: { $air }, { $speed }. { $how }, { $when }.
motion-who-killer = Matador
motion-who-victim = Vítima
motion-air-yes = no ar
motion-air-no = no chão
motion-air-unknown = não se sabe se no ar
motion-speed-unknown = velocidade desconhecida
motion-exact = Registrado pelo jogo
motion-trajectory = Estimado pelo movimento
motion-unclear = Movimento incerto
motion-not-visible = Fora de vista
motion-at-shot = no disparo ligado
motion-at-kill = na morte
motion-none = A análise não tem movimento para este abate.
legend-shot = Disparo
legend-charge = Carregando
legend-kill = Abate
legend-death = Morte
legend-switch = Troca de arma

column-headshots = Headshots
column-multikill = Multikill
column-spree = Sequência
filter-headshots = Abates com headshot
filter-midair = Abates em midair
event-headshot = headshot
scores-headshots = HS

column-accuracy = Precisão
column-players = Jogadores
column-teams = Equipes
help-players-column = Jogadores no jogo, sem contar espectadores, contados quando uma partida começou depois da contagem regressiva. Num jogo em equipe, só contam os jogadores que estão numa equipe. Uma demo mostra o maior número em qualquer partida completa, e uma demo sem nenhuma partida completa, o maior número em qualquer momento.
help-teams-column = Equipes com pelo menos um jogador, contadas como os jogadores. Um jogo sem equipes não tem nenhuma.
filter-min-accuracy = Precisão mínima (%)
scores-accuracy = Prec.
scores-kd = K/D
scores-ping = Ping
help-kd = Abates por morte: os abates divididos pelas mortes. Se o jogador nunca morreu, aparecem os abates dele.
help-ping-loss = O ping de cada jogador em milissegundos e, depois, a perda de pacotes, como o servidor enviou por último. Ele só envia esses dados enquanto quem gravou segura a tecla do placar, então até lá aparece um traço.

help-headshots = Abates por headshot, lidos da animação de morte da vítima. Explosões não contam. Só aparecem os headshots de quem gravou, porque os outros jogadores costumam matar fora da visão da gravação. Num duelo aparecem também os do adversário, já que todos os abates dele são em quem gravou.
help-headshots-column = Headshots de quem gravou, lidos da animação de morte da vítima. Explosões não contam. Os dos outros jogadores não contam, porque costumam acontecer fora da visão da gravação.
help-multikill-column = O maior número de jogadores que alguém matou com um só tiro na demo, como 3 em um triple kill.
help-spree-column = A maior sequência de abates da demo, de qualquer jogador: o máximo de abates que um jogador fez seguidos sem morrer.
help-date-column = Quando a partida foi gravada. Vem do nome do arquivo quando o nome tem uma data como 20230910_214553, como nas gravações automáticas do jogo. Senão, é a data do arquivo, que copiar ou baixar pode alterar.
help-file-date = Quando o arquivo foi salvo. Num clipe exportado, é quando ele foi exportado.
help-accuracy = A parte dos disparos que acertaram, pelo sangue que um acerto deixa na linha de mira, descontando coincidências ao acaso. Vale para a glock, a python, as balas da MP5, a escopeta, a besta com zoom e os acertos diretos do raio gauss. Explosões e reflexos do gauss não deixam sangue e não contam. Aparece só para quem gravou, depois de pelo menos 50 disparos.
help-headshot-filter = Demos com pelo menos um headshot que a gravação viu, de qualquer jogador.
help-midair-filter = Demos com pelo menos um abate em que o matador ou a vítima estava no ar, a 50 unidades ou mais acima do chão. Sem a pasta do Half-Life escolhida, um simples pulo também conta.
help-spree-filter = Demos em que um jogador fez pelo menos esse número de abates seguidos sem morrer. Morrer de qualquer jeito encerra a sequência, assim como o fim do mapa ou a saída do jogador. Teamkills não contam.
help-accuracy-filter = Demos em que quem gravou acertou pelo menos esta parte dos disparos.
help-bots-filter = Poucos mantém só demos e partidas com menos bots do que jogadores de verdade, então um 1v1 contra um bot fica de fora, mas não um servidor cheio com alguns bots completando. Nenhum mantém só as que não têm nenhum bot. Um bot é um jogador que o jogo marca como bot, ou um sem CD key.

clips-new = Novo clipe
clips-select-event = Primeiro escolha um evento na lista ou na linha do tempo.
clips-other-session = Um clipe não pode começar aqui, porque o relógio da demo recomeçou numa troca de mapa ou reconexão logo antes.
clips-add-window = Criar clipe deste momento intenso
clip-default-name = Clipe { $number }
clip-name = Nome
clip-length = Duração { $duration }
clip-remove = Remover
clip-loop = Repetir
clip-time-help = Segundos (96), minutos e segundos (1:36) ou horas, minutos e segundos (00:01:36.250)
clips-empty-hint = Ainda não há clipes. Escolha um evento e aperte Clipar, ou aperte Novo clipe para começar um no momento atual.
clips-new-help = Um clipe de dez segundos a partir do momento atual.
clip-in = Início
clip-out = Fim
clip-set-in = Começar o clipe no momento atual
clip-set-out = Terminar o clipe no momento atual
clip-no-playhead = O visualizador não está disponível, então não há um momento atual para usar.
clip-play = Reproduzir o clipe uma vez
clip-unplayable = Este clipe vem depois de uma troca de mapa ou reconexão, onde o relógio da demo recomeçou, então o visualizador não consegue reproduzi-lo. Exportar ainda corta o momento certo.
clip-show-on-timeline = Mostrar na linha do tempo
export-destination = Mudar a pasta de exportação
export-skipped = Pulado. { $reason }
export-earlier-range = Esta exportação é de { $start } – { $end }, o clipe como estava antes de você mudá-lo.
scores-player = Jogador
scores-none = A gravação não tem placar.
clip-effective = A exportação cobre { $start } – { $end } ({ $duration }), o mais perto que a gravação permite.
settings-clip-padding = Margem dos clipes
settings-preroll = Segundos antes do evento
settings-postroll = Segundos depois do evento
export-start-first-group = Um clipe não pode começar bem no início da gravação. Comece um pouco depois.
export-stopped = A exportação foi parada, então nenhum clipe foi salvo.
export-start-during-loading = O início cai enquanto o jogo ainda carrega um mapa. Comece um pouco depois.

export-all = Exportar todos os clipes
export-one = Exportar
export-cancel = Parar a exportação
export-folder = As exportações vão para { $folder }.
export-folder-beside = As exportações ficam junto da demo, em { $folder }.
export-waiting = Aguardando para exportar…
export-running = Exportando… Isso lê a demo inteira e pode demorar.
export-searching = O clipe não pode começar exatamente aí. Procurando um início próximo que funcione…
export-nearest-earlier = Ele pode começar em { $time } em vez disso, { $seconds } s antes.
export-nearest-later = Ele pode começar em { $time } em vez disso, { $seconds } s depois.
export-start-there = Começar aí e exportar
export-cancelled = Não exportado, porque a exportação foi parada.
export-saved = Salvo em { $path }. Momentos conferidos com o original: { $count }.
export-playdemo = No console do jogo:
export-copy-first = Copie { $name } para a pasta do seu mod, a que tem liblist.gam, e depois no console do jogo:
export-copy = Copiar
settings-export-folder = Pasta de exportação
settings-export-choose = Escolher…
settings-file-names = Nomes de arquivo
settings-clip-name-files = Usar o nome do clipe no arquivo
settings-demo-name-first = Começar com o nome da demo
settings-viewer = Visualizador de demos
settings-viewer-game = Pasta do Half-Life
settings-viewer-game-none = Não escolhida
settings-viewer-game-hint = A pasta que contém valve. O visualizador reproduz as demos com os mapas, modelos e sons do jogo que estão nela.
settings-game-override = Desta vez o Lambda Slicer usa { $path }, porque foi aberto com --game. Uma escolha feita aqui vale a partir da próxima vez que ele abrir.
settings-more-games = Mais pastas do Half-Life
settings-more-games-hint = Outros lugares onde o Half-Life está instalado, como um com muitos mapas. Quando um mapa, modelo, sprite ou som não está na pasta do Half-Life acima, o visualizador procura nestas pastas, de cima para baixo. Mudá-las faz a biblioteca verificar suas demos de novo em busca de abates em midair, com os mapas que elas trazem.
settings-more-games-empty = Nenhuma pasta adicionada ainda.
settings-move-up = Mover para cima
settings-move-down = Mover para baixo
settings-game-content = Conteúdo do jogo
settings-game-content-hint = Estas opções começam como estão no seu jogo. Mudá-las aqui só muda o visualizador, nunca o jogo. Com mais de uma ligada, vale a ordem do jogo: primeiro o conteúdo de addons, depois o conteúdo localizado, depois os modelos HD e por fim os arquivos do próprio jogo.
settings-hd-models = Usar modelos HD
settings-hd-models-hint = Procura primeiro nas pastas _hd, como valve_hd, os modelos de alta qualidade. Começa como a opção "Use high quality models" do jogo.
settings-addon-content = Usar conteúdo personalizado de addons
settings-addon-content-hint = Procura primeiro nas pastas _addon, como valve_addon, o conteúdo que você adicionou ao jogo. Começa como a opção "Allow custom addon content" do jogo.
settings-localized-content = Usar conteúdo localizado
settings-localized-content-hint = Procura primeiro nas pastas do idioma em que a Steam abre o Half-Life, como valve_spanish para espanhol. O jogo sempre as usa, e o inglês não tem nenhuma.
settings-steam-language = A Steam abre o Half-Life neste idioma: { $language }.
settings-steam-english = A Steam abre o Half-Life em inglês, que não tem pastas próprias.
dialog-executables = Programas
setup-title = Configure o visualizador de demos
setup-body = Imagem e som precisam dos arquivos do Half-Life. Mostre ao Lambda Slicer onde o jogo está instalado e ele vai lê-los de lá.
setup-locate-exe = Localizar hl.exe…
setup-locate-folder = Localizar Half-Life…
setup-settings = Configurações
setup-still-works = A linha do tempo, os eventos e os clipes continuam funcionando sem eles.
setup-relocate = { $folder } não tem mais o Half-Life. Localize de novo.
setup-not-executable = { $name } não é { $executable }. Escolha o programa do próprio jogo, não um atalho.
setup-not-installed = { $folder } não tem uma instalação do Half-Life com a pasta valve.
tab-events = Eventos
tab-clips = Clipes
tab-scoreboard = Placar
tab-demos = Demos
tab-matches = Partidas
filter-status = Estado
library-all = Todas
library-favourites = Favoritas
library-no-favourites = Ainda não há demos favoritas. Clique com o botão direito em uma demo e escolha “Adicionar aos favoritos”.
menu-favourite-add = Adicionar aos favoritos
menu-favourite-remove = Remover dos favoritos
favourite-star = Favorita
library-hidden = Ocultas
library-no-hidden = Não há demos ocultas. Clique com o botão direito em uma demo e escolha “Ocultar” para tirá-la da lista.
menu-hide = Ocultar
menu-unhide = Mostrar
filter-status-all = Todos
help-status-column = Quanto da partida a demo gravou. Passe o mouse sobre o estado de uma linha para ver o que ele quer dizer.
help-match-players = Jogadores no jogo, sem contar espectadores, quando a partida começou depois da contagem regressiva, ou no primeiro abate se ela já estava em andamento quando a gravação começou. No jogo livre, o maior número ao mesmo tempo.
help-match-teams = Equipes com pelo menos um jogador, contadas como os jogadores. Um jogo sem equipes não tem nenhuma.
help-match-multikill = O maior número de jogadores que alguém matou com um só tiro na partida, como 3 em um triple kill.
help-match-spree = A maior sequência de abates encerrada na partida, de qualquer jogador: o máximo de abates que um jogador fez seguidos sem morrer.
match-complete = Completa
match-joined-late = Já em andamento
match-interrupted = Interrompida
match-cut-off = Cortada
match-free-play = Jogo livre
help-match-complete = Gravada da contagem regressiva até o fim da partida.
help-match-joined-late = A partida já estava em andamento quando a gravação começou, e foi gravada até o fim.
help-match-interrupted = Outra partida começou, a partida foi cancelada ou o mapa mudou antes de a partida acabar.
help-match-cut-off = A gravação acabou antes da partida.
help-match-free-play = Cinco minutos ou mais de jogo sem nenhuma partida em andamento.
list-hide = Esconder a lista de demos
list-show = Mostrar a lista de demos
demo-details = Detalhes
viewer-start = Voltar ao início
viewer-mute = Silenciar
viewer-unmute = Ativar som
viewer-picture = Imagem e opções do visualizador
viewer-needs-files = Precisa dos arquivos do jogo
viewer-no-sound-why = Sem som: { $reason }
settings-tab-general = Geral
settings-tab-library = Biblioteca
settings-tab-clips = Clipes
settings-tab-hud = HUD
settings-search = Buscar nas configurações
settings-search-none = Nenhuma configuração encontrada.
settings-tab-shortcuts = Atalhos
settings-shortcuts-hint = Clique numa tecla para mudá-la e depois aperte a nova tecla ou botão do mouse, gire a rodinha do mouse com o ponteiro sobre ela, ou aperte Esc para desistir. Clique com o botão direito numa tecla para apagá-la. Uma tecla que outra ação já usa passa para cá. Os atalhos da rodinha do mouse funcionam sobre a imagem da demo.
settings-shortcuts-defaults = Teclas padrão
settings-shortcut-press = Aperte uma tecla…
settings-shortcut-change = Clique para mudar, clique com o botão direito para apagar
settings-shortcut-moved = { $key } foi tirada de “{ $action }”.
shortcut-none = sem tecla
shortcuts-anywhere = Em qualquer lugar
shortcuts-demo = Demo
shortcuts-playback = Reprodução
shortcuts-view = Visão
shortcuts-players = Ponto de vista
shortcuts-free-camera = Câmera livre
shortcut-open-demo = Abrir uma demo
shortcut-settings = Abrir ou fechar as configurações
shortcut-back = Voltar pelas demos e partidas que você abriu
shortcut-forward = Avançar pelas demos e partidas que você abriu
shortcut-previous-event = Evento anterior
shortcut-next-event = Próximo evento
shortcut-clip-start = Começar o clipe escolhido no momento atual
shortcut-clip-end = Terminar o clipe escolhido no momento atual
shortcut-play-pause = Reproduzir ou pausar
shortcut-rewind = Rebobinar
shortcut-pause = Pausar
shortcut-seek-back = Voltar 5 segundos
shortcut-seek-forward = Avançar 5 segundos
shortcut-skip-back = Voltar 10 segundos
shortcut-skip-forward = Avançar 10 segundos
shortcut-frame-back = Frame anterior
shortcut-frame-forward = Próximo frame
shortcut-slower = Mais lento
shortcut-faster = Mais rápido
shortcut-start = Voltar ao início
shortcut-fullscreen = Tela cheia
shortcut-zoom = Dar ou tirar zoom
shortcut-see-through = Ver jogadores através das paredes
shortcut-fullbright = Iluminar tudo
shortcut-names = Nomes dos jogadores
shortcut-free-camera = Câmera livre
shortcut-recorder = Visão de quem gravou
shortcut-follow = Visão do { $place }º melhor jogador do mapa
shortcut-previous-player = Visão do jogador anterior
shortcut-next-player = Visão do próximo jogador
shortcut-fly-forward = Para frente
shortcut-fly-back = Para trás
shortcut-fly-left = Para a esquerda
shortcut-fly-right = Para a direita
shortcut-fly-up = Para cima
shortcut-fly-down = Para baixo
shortcut-fly-fast = Mais rápido
settings-section-game = Half-Life
settings-section-view = Visão
settings-language-system = Idioma do computador ({ $name })
settings-section-folders = Pastas
settings-folders-hint = As pastas em que a biblioteca procura demos, incluindo subpastas.
settings-clip-padding-hint = Quanto um clipe criado em torno de um evento guarda antes e depois dele.
settings-section-picture = Imagem
settings-section-sound = Som
settings-section-list = Lista da biblioteca
settings-section-effects = Efeitos
settings-section-vitals = Vida e munição
settings-section-scores = Placar
settings-section-chat = Chat
settings-section-speeds = Velocidade e FPS
settings-section-server = Do servidor
settings-section-colours = Cores e tamanho do texto
settings-gamma = Gama
settings-brightness = Brilho
settings-texgamma = Gama das texturas
settings-lightgamma = Gama da luz
settings-overbright = Iluminação estourada
settings-gamma-hint = As configurações do jogo com o mesmo nome (gamma, brightness, texgamma, lightgamma, gl_overbright). Ajuste-as como no jogo para ver o mesmo brilho.
settings-gamma-defaults = Valores do jogo
settings-fov-recorded = Campo de visão gravado
settings-fov = Campo de visão
settings-zoom-fov = Campo de visão com zoom
settings-viewmodel = Mostrar a arma na mão
settings-weapon-bob = Balanço da arma (cl_bob)
settings-weapon-bob-recorded = Balanço da arma como gravado
settings-crosshair = Mostrar a mira
settings-chat = Mostrar o chat
settings-chat-places = Mostrar mensagens que dizem um local
settings-chat-places-hint = Mensagens de equipe vindas de binds que dizem o local de quem enviou com %l ou %L. Com isto desligado, elas ficam de fora do chat como se nunca tivessem sido enviadas.
settings-chat-long-jump = Mostrar mensagens que dizem "got LJ" ou "no LJ"
settings-chat-long-jump-hint = Mensagens de equipe vindas de binds que dizem se quem enviou tem o long jump. Com isto desligado, elas ficam de fora do chat como se nunca tivessem sido enviadas.
settings-vitals = Mostrar vida, armadura, munição e itens coletados
settings-vitals-size = Tamanho da vida, armadura, munição e itens coletados
settings-free-vitals = Mostrar vida, armadura, munição e itens coletados também na câmera livre
settings-scores = Mostrar o placar no canto superior esquerdo
settings-score-rows = Máximo de linhas do placar
settings-scoreboard = Mostrar o placar enquanto quem gravou segura a tecla dele, e no fim do mapa
settings-scoreboard-off = Desligado
settings-scoreboard-fullscreen = Só em tela cheia
settings-scoreboard-always = Ligado
settings-server-settings = Mostrar as configurações do servidor no canto superior direito
settings-text-size = Tamanho do chat, do kill feed e das configurações do servidor
settings-hud-color = Cor do HUD
settings-hud-color-hint = A cor da vida, armadura, munição, itens coletados, menu de armas, velocidades e FPS de quem gravou, e do cronômetro, das votações e das configurações do servidor do AG, como o hud_color no OpenAG. Vida baixa e armas sem munição continuam vermelhas, e o texto do HUD do próprio servidor mantém a cor dele.
settings-text-color = Cor do texto
settings-text-color-default = Cor do jogo
settings-text-color-hint = A cor das mensagens do chat, das linhas que nenhum jogador enviou e da posição e dos ângulos de visão, como o con_color no jogo. Os nomes dos jogadores mantêm a cor da equipe.
settings-hud-messages = Mostrar textos do servidor, como os cronômetros de bhop e KZ
settings-see-through = Ver jogadores através das paredes
settings-see-through-items = Também outros modelos, como projéteis e itens
settings-player-names = Mostrar o nome dos jogadores acima deles
settings-show-position = Mostrar posição e ângulos de visão
settings-speedometer = Mostrar o velocímetro
settings-jump-speed = Mostrar a velocidade do último pulo
settings-jump-speed-hint = A velocidade de quem gravou ao sair do chão pela última vez, mostrada abaixo do velocímetro até o próximo pulo.
settings-recorder-fps = Mostrar o FPS de quem gravou
settings-recorder-fps-hint = Os quadros por segundo em que o jogo rodava para quem gravou a demo, no canto superior esquerdo, em média a cada meio segundo. Não é a velocidade com que este app desenha.
settings-speed-axes = Medir a velocidade
settings-speed-horizontal = Horizontal
settings-speed-vertical = Vertical
settings-speed-both = Ambas (3D)
viewer-jump-speed = Pulo { $speed }
settings-show-position-hint = De onde a imagem é vista, em unidades do jogo, e para onde ela olha, em graus para cima ou para baixo (pitch) e em volta (yaw). Com a câmera livre ligada, são os da câmera livre. Útil para dizer a alguém exatamente para onde olhar.
viewer-position-line = Posição { $x } { $y } { $z }
viewer-fps = { $fps } FPS
viewer-angles-line = Ângulos { $pitch } { $yaw }
settings-player-names-hint =Cada jogador à vista tem o nome acima da cabeça, na cor da sua equipe, também na câmera livre. Jogadores atrás de paredes só mostram o deles enquanto os jogadores são vistos através das paredes.
settings-see-through-hint = Jogadores atrás de paredes aparecem como silhuetas na cor de cima, que os jogos em equipe definem por equipe. Só aparece o que a demo contém. O servidor envia os jogadores perto de quem gravou, e servidores que escondem os jogadores que quem gravou não pode ver não deixam nada para mostrar.
settings-section-decals = Decals
settings-decals = Máximo
settings-decals-hint = Buracos de bala, sangue e outras marcas deixadas durante a partida. Só as mais novas ficam, e as mais antigas somem quando surgem novas. As marcas que fazem parte do mapa sempre ficam.
settings-restore = Restaurar padrões…
settings-restore-title = Restaurar padrões
settings-restore-list = Estas configurações voltam a ser como eram no começo. A pasta do Half-Life e as pastas da biblioteca continuam como estão.
settings-restore-none = Todas as configurações já estão no padrão.
settings-restore-columns = Colunas mostradas na biblioteca
settings-restore-layout = Painéis abertos e seus tamanhos
settings-clear-library = Limpar biblioteca…
settings-clear-library-title = Limpar biblioteca
settings-clear-library-text = { $count ->
    [one] O Lambda Slicer esquece o que encontrou em { $count } demo e escaneia de novo as pastas da biblioteca, o que pode demorar um pouco. Seus arquivos de demos, clipes e configurações continuam como estão.
    [many] O Lambda Slicer esquece o que encontrou em { $count } de demos e escaneia de novo as pastas da biblioteca, o que pode demorar um pouco. Seus arquivos de demos, clipes e configurações continuam como estão.
   *[other] O Lambda Slicer esquece o que encontrou em { $count } demos e escaneia de novo as pastas da biblioteca, o que pode demorar um pouco. Seus arquivos de demos, clipes e configurações continuam como estão.
}
viewer-zoom = Dar zoom
viewer-free-camera = Câmera livre
viewer-free-camera-help = Voe pelo mapa sem seguir o jogador que gravou a demo. { $forward }, { $left }, { $back } e { $right } movem, { $up } sobe, { $down } desce e { $fast } acelera. Segure o botão direito do mouse e arraste para olhar em volta.
viewer-fullscreen = Tela cheia
viewer-leave-fullscreen = Sair da tela cheia
viewer-see-through = Ver através das paredes
viewer-player-names = Nomes dos jogadores
viewer-fullbright = Iluminar tudo
viewer-fullbright-help = Mostra o mapa inteiro totalmente iluminado, para que os cantos escuros e os jogadores que estão neles fiquem fáceis de ver.
viewer-paused = Pausado
viewer-map-change = O servidor está trocando o mapa para { $map }
viewer-pov-recorder = { $name } (gravou)
viewer-pov-recorder-unnamed = Quem gravou
viewer-pov-help = Pelos olhos de quem assistir: quem gravou ou os melhores jogadores deste mapa por pontuação, cada um com sua tecla ao lado. Jogador anterior: { $previous }. Próximo jogador: { $next }.
viewer-scoreboard-ping-loss = Ping/perda
viewer-scoreboard-bot = BOT
viewer-scoreboard-players = { $count ->
    [one] ({ $count } jogador)
    [many] ({ $count } de jogadores)
   *[other] ({ $count } jogadores)
}
viewer-pov-unseen = A demo não mostra { $name } aqui, então você está vendo a visão de quem gravou
settings-events-leave-free = Sair da câmera livre ao escolher um evento
settings-events-leave-free-hint = Ao escolher um abate, ele aparece pelos olhos do matador, da vítima ou de quem gravou. Com isto desligado, a câmera livre fica onde está, e desligá-la leva a essa visão.
settings-particles = Mostrar partículas
settings-particles-hint = Faíscas, os pedaços que voam das explosões e a poeira onde as balas acertam.
settings-export-beside = Junto de cada demo
settings-export-hint = Dentro da pasta do seu mod, a que tem liblist.gam como Half-Life\ag, os clipes tocam com um só comando playdemo.
settings-file-names-hint = Um clipe exportado recebe o nome da sua demo e dos seus tempos de início e fim, a menos que você use o nome do clipe. O jogo só consegue abrir arquivos com nomes feitos de letras do alfabeto inglês, números, - e ., então todo o resto vira _. Se o nome do clipe não tiver nenhum desses caracteres, o arquivo recebe o nome pelos tempos.

clips-store-moved = Não foi possível ler o arquivo de clipes { $path }, então ele foi guardado como { $moved } e os clipes começam vazios. { $detail }
clips-store-unreadable = Não foi possível ler o arquivo de clipes { $path }: { $detail }. Até o Lambda Slicer ser fechado, os clipes não são salvos, então nada do arquivo se perde.
clips-save-failed = Não foi possível salvar os clipes em { $path }: { $detail }
notice-dismiss = Fechar

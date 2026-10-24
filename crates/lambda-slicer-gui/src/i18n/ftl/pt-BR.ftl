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
settings-sort = Ordem ao abrir
settings-sort-hint = Como as demos são ordenadas cada vez que o Lambda Slicer abre. Clicar no título de uma coluna ainda ordena por ela, até você fechar o Lambda Slicer.
settings-start-filters = Filtros ao abrir
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
library-total = { $count ->
    [one] { $count } demo na biblioteca
    [many] { $count } de demos na biblioteca
   *[other] { $count } demos na biblioteca
}
filters-title = Filtros
filters-find = Buscar
filters-pov = Ponto de vista
filters-quality = Qualidade
filters-recorded = Gravada
pov-any = Todas
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
filter-weapon-any = Qualquer arma
filter-min-minutes = Pelo menos (minutos)
filter-min-kills = Pelo menos (abates)
filter-players = Jogadores
filter-teams = Equipes
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
export-cross-segment = O mapa mudou ou o jogador reconectou em { $time }, dentro do clipe, e um clipe não pode passar por isso.
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
demo-events = { $count ->
    [one] { $count } evento
    [many] { $count } de eventos
   *[other] { $count } eventos
}

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
viewer-shortcut = Atalho: { $key }
key-space = Espaço
key-shift-left = Shift+Esquerda
key-shift-right = Shift+Direita
viewer-map-failed = Não foi possível carregar o mapa { $map }
viewer-stopped = A reprodução parou antes do fim: { $reason }
viewer-missing = Arquivos que não puderam ser lidos: { $count }
event-previous = Evento anterior
event-next = Próximo evento
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
events-follow = Manter a lista no momento atual enquanto a demo é reproduzida. Rolar a lista desativa isso.
clip-it = Clipar
badge-headshot = HS
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
filter-headshots = Abates com headshot
filter-midair = Abates em midair
event-headshot = headshot
scores-headshots = HS

column-accuracy = Precisão
column-players = Jogadores
column-teams = Equipes
help-players-column = Jogadores no jogo, sem contar espectadores, contados quando a partida começou depois da contagem regressiva, ou no primeiro abate se a partida já estava em andamento quando a gravação começou. Num jogo em equipe, só contam os jogadores que estão numa equipe. Uma demo com várias partidas mostra o maior número, e uma demo sem partida, o maior número em qualquer momento.
help-teams-column = Equipes com pelo menos um jogador, contadas como os jogadores. Um jogo sem equipes não tem nenhuma.
filter-min-accuracy = Precisão mínima (%)
scores-accuracy = Prec.

help-headshots = Abates por headshot, lidos da animação de morte da vítima. Explosões não contam. Só aparecem os headshots de quem gravou, porque os outros jogadores costumam matar fora da visão da gravação. Num duelo aparecem também os do adversário, já que todos os abates dele são em quem gravou.
help-headshots-column = Headshots de quem gravou, lidos da animação de morte da vítima. Explosões não contam. Os dos outros jogadores não contam, porque costumam acontecer fora da visão da gravação.
help-date-column = Quando a partida foi gravada. Vem do nome do arquivo quando o nome tem uma data como 20230910_214553, como nas gravações automáticas do jogo. Senão, é a data do arquivo, que copiar ou baixar pode alterar.
help-file-date = Quando o arquivo foi salvo. Num clipe exportado, é quando ele foi exportado.
help-accuracy = A parte dos disparos que acertaram, pelo sangue que um acerto deixa na linha de mira, descontando coincidências ao acaso. Vale para a glock, a python, as balas da MP5, a escopeta, a besta com zoom e os acertos diretos do raio gauss. Explosões e reflexos do gauss não deixam sangue e não contam. Aparece só para quem gravou, depois de pelo menos 50 disparos.
help-headshot-filter = Demos com pelo menos um headshot que a gravação viu, de qualquer jogador.
help-midair-filter = Demos com pelo menos um abate em que o matador ou a vítima estava no ar, a 50 unidades ou mais acima do chão. Sem a pasta do Half-Life escolhida, um simples pulo também conta.
help-accuracy-filter = Demos em que quem gravou acertou pelo menos esta parte dos disparos.

clips-new = Novo clipe
clips-select-event = Primeiro escolha um evento na lista ou na linha do tempo.
clips-other-session = Um clipe não pode começar aqui, porque o relógio da demo recomeçou depois de uma troca de mapa ou reconexão, e uma parte anterior tem os mesmos tempos.
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
export-start-during-loading = O início cai enquanto o jogo ainda carrega um mapa. Comece um pouco depois.

export-all = Exportar todos os clipes
export-one = Exportar
export-cancel = Parar a exportação
export-folder = As exportações vão para { $folder }.
export-folder-beside = As exportações ficam junto da demo, em { $folder }.
export-waiting = Aguardando para exportar…
export-running = Exportando… Isso lê a demo inteira e pode demorar.
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
settings-executable = Programa do jogo
settings-executable-hint = Serve para achar a pasta do Half-Life. O visualizador lê os arquivos do jogo sozinho e nunca abre o jogo.
settings-executable-not-set = Não definido, então o visualizador fica sem imagem e sem som
settings-executable-optional = Não definido, e não faz falta enquanto a pasta do Half-Life tiver o jogo
settings-executable-missing = { $path } não está mais lá. O visualizador continua lendo a pasta do Half-Life.
settings-game-override = Desta vez o Lambda Slicer usa { $path }, porque foi aberto com --game. Uma escolha feita aqui vale a partir da próxima vez que ele abrir.
settings-browse = Procurar…
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
settings-section-game = Half-Life
settings-section-view = Visão
settings-language-system = Idioma do computador ({ $name })
settings-section-folders = Pastas
settings-folders-hint = As pastas em que a biblioteca procura demos, incluindo subpastas.
settings-clip-padding-hint = Quanto um clipe criado em torno de um evento guarda antes e depois dele.
settings-section-picture = Imagem
settings-section-sound = Som
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
settings-crosshair = Mostrar a mira
settings-chat = Mostrar o chat
settings-vitals = Mostrar vida, armadura, munição e itens coletados
settings-vitals-size = Tamanho da vida, armadura, munição e itens coletados
settings-scores = Mostrar o placar no canto superior esquerdo
settings-score-rows = Máximo de linhas do placar
settings-server-settings = Mostrar as configurações do servidor no canto superior direito
settings-hud-messages = Mostrar textos do servidor, como os cronômetros de bhop e KZ
settings-see-through = Ver jogadores através das paredes
settings-see-through-items = Também outros modelos, como projéteis e itens
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
viewer-fullscreen = Tela cheia
viewer-leave-fullscreen = Sair da tela cheia
viewer-see-through = Ver através das paredes
viewer-paused = Pausado
viewer-map-change = O servidor está trocando o mapa para { $map }
settings-export-beside = Junto de cada demo
settings-export-hint = Dentro da pasta do seu mod, a que tem liblist.gam como Half-Life\ag, os clipes tocam com um só comando playdemo.
settings-file-names-hint = Um clipe exportado recebe o nome da sua demo e dos seus tempos de início e fim, a menos que você use o nome do clipe. O jogo só consegue abrir arquivos com nomes feitos de letras do alfabeto inglês, números, - e ., então todo o resto vira _. Se o nome do clipe não tiver nenhum desses caracteres, o arquivo recebe o nome pelos tempos.

clips-store-moved = Não foi possível ler o arquivo de clipes { $path }, então ele foi guardado como { $moved } e os clipes começam vazios. { $detail }
clips-store-unreadable = Não foi possível ler o arquivo de clipes { $path }: { $detail }. Até o Lambda Slicer ser fechado, os clipes não são salvos, então nada do arquivo se perde.
clips-save-failed = Não foi possível salvar os clipes em { $path }: { $detail }
notice-dismiss = Fechar

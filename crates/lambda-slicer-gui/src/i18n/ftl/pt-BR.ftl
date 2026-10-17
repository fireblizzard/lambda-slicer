# Português do Brasil. Rascunho não revisado

app-title = Lambda Slicer

## Barra de ferramentas

toolbar-open = Abrir demo…
toolbar-add-folder = Adicionar pasta…
toolbar-rescan = Escanear de novo
toolbar-cancel = Cancelar
toolbar-settings = Configurações
dialog-demos = Demos de Half-Life

## Configurações

settings-title = Configurações
settings-language = Idioma
settings-unreviewed = Esta tradução é um rascunho que nenhum falante nativo revisou ainda.
settings-workers = Processos de análise
settings-workers-hint = Mais processos analisam mais rápido, mas usam mais tempo do processador.
settings-cache = Cache da biblioteca
settings-folders = Pastas da biblioteca
settings-folders-empty = Nenhuma pasta adicionada ainda.
settings-remove = Remover

## Biblioteca

library-empty = Adicione uma pasta ou abra uma demo para começar.
library-no-matches = Nenhuma demo corresponde a estes filtros.
library-no-cache = Não há onde guardar o cache da biblioteca neste sistema.
library-demos = { $count ->
    [one] { $count } demo
    [many] { $count } de demos
   *[other] { $count } demos
}
library-percent = { $percent }%

filter-search = Buscar caminhos
filter-map = Mapa
filter-player = Jogador
filter-weapon-any = Qualquer arma
filter-min-minutes = Pelo menos (minutos)
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
kind-hltv = Proxy de espectador
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

scan-progress = Encontradas { $found }, prontas { $done } ({ $cached } no cache, { $analysed } analisadas), { $megabytes } MB lidos
scan-cancelling = Parando…
scan-cancelled = Parado com { $left } por escanear.
scan-finished = Pronto: { $done } demos, { $analysed } analisadas, em { $seconds } s.
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

analysis-panicked = A análise parou por um erro interno: { $detail }
cache-cancelled = A análise foi cancelada, então nada foi salvo.
cache-database = Não é possível usar o cache da biblioteca: { $detail }
cache-directory = Não é possível criar a pasta do cache: { $detail }
container-bad-magic = Isto não é uma demo de Half-Life.
container-frame-bad-time = A gravação está danificada no quadro { $frame } da parte { $entry }, que tem um tempo inválido.
container-frame-missing-terminal = A parte { $entry } da gravação termina sem o registro de fechamento.
container-frame-negative-length = A gravação está danificada no quadro { $frame } da parte { $entry }.
container-frame-over-limit = O quadro { $frame } da parte { $entry } é maior do que o permitido.
container-frame-truncated = A gravação é cortada no quadro { $frame } da parte { $entry }.
container-frame-unknown-tag = A gravação está danificada no quadro { $frame } da parte { $entry }, que é de um tipo desconhecido.
container-io = Não é possível ler o arquivo: { $detail }
container-no-directory = A gravação nunca foi finalizada, então não tem índice.
container-range = O índice da gravação aponta para fora do arquivo.
container-too-short = O arquivo é curto demais para ser uma demo.
coverage-hltv = Gravação de proxy de espectador. Não há jogador gravando e os eventos seguem a visão do proxy.
coverage-mostly-spectating = Quem gravou ficou espectando em { $percent }% da reprodução, então a maioria dos eventos é de outros jogadores, como a visão observada os mostrou.
coverage-no-local-events = Não há eventos locais. As ações de quem gravou faltam, exceto as que o servidor enviou.
coverage-singleplayer = Um jogador. Os outros atores são monstros ou entidades do mapa, e não há registro de mortes.
coverage-unknown = A gravação não foi decodificada o suficiente para saber de que tipo é.
coverage-visible-only = Os eventos de outros jogadores só chegam a este cliente enquanto o servidor os considera visíveis, então as contagens são mínimas.
decode-failed = A decodificação parou em { $time } (quadro { $frame }): { $detail }
export-active-effect = { $effect } está em andamento no início e ainda não pode ser levado adiante.
export-client-dropped-packets = O jogo descartou pacotes atrasados em { $time }, algo que um corte ainda não consegue reproduzir.
export-cross-segment = Uma nova sessão começa em { $time }, dentro do intervalo.
export-destination-exists = { $path } já existe. Escolha outro nome.
export-empty-range = O intervalo selecionado não tem nada para exportar.
export-io = Não é possível gravar { $path }: { $detail }
export-missing-delta-base = Falta à gravação um estado necessário antes do início (sequência { $sequence }).
export-missing-parent = A pasta { $path } não existe.
export-range-beyond-end = O fim { $end } passa do fim da gravação, { $segment_end }.
export-same-as-source = O destino é a própria demo de origem.
export-source-changed = { $path } mudou durante a exportação.
export-source-decode-failed = A demo de origem não pode ser decodificada: { $detail }
export-source-failed-crosscheck = A demo de origem não passou na verificação cruzada, então o estado reconstruído não é confiável.
export-unknown-persistent-message = { $name } antes do início define um estado que um corte ainda não consegue levar adiante.
export-unproven-state = O estado em { $time } depende de uma base inferida e não está comprovado.
export-unrepresentable-time-field = Um tempo no início não pode ser gravado com fidelidade: { $detail }
export-unsupported = Não suportado: { $detail }
export-validation-failed = O arquivo exportado não passou na validação: { $detail }
io-not-found = O arquivo ou a pasta não foi encontrado.
io-other = { $detail }
io-permission-denied = Acesso negado.
io-unexpected-end = O arquivo terminou inesperadamente.
protocol-unsupported = O protocolo de demo { $demo }/{ $network } não é suportado, só o 5/48.
protocol-unsupported-network = O servidor mudou para o protocolo de rede { $network }, que não é suportado.

## Visão da demo

demo-close = Fechar
demo-loading = Analisando { $name }…
timeline-fit = Ajustar
show-shots = Disparos
show-actions = Outras ações
show-kills = Abates
show-switches = Trocas de arma
lane-unknown = Jogador desconhecido
lane-recorder = { $name } (gravou)
column-time = Tempo
column-player = Jogador
column-weapon = Arma
column-event = Evento
event-kill = Abateu { $victim }
event-switch = Troca de arma
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
column-score = Placar
filter-from = De
filter-to = Até
date-from-name = Data tirada do nome do arquivo
date-from-modified = Data da última modificação do arquivo, que copiar ou baixar pode alterar
timeline-best-action = Minuto de ouro
timeline-windows = Momentos mais intensos
timeline-window = Nº { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Placar final, no último abate { $time }
scores-at = Placar em { $time }
scores-show-final = Ver placar final
scores-frags = Abates
scores-deaths = Mortes

show-timeline = Mostrar linha do tempo
legend-shot = Disparo
legend-charge = Carregando
legend-kill = Abate
legend-death = Morte
legend-switch = Troca de arma
timeline-help = Cada linha é um jogador. Role para aproximar, arraste para mover, clique duas vezes para ver tudo e clique numa marca para selecioná-la.

column-headshots = Headshots
filter-headshots = Abates com headshot
event-headshot = headshot
scores-headshots = HS

column-accuracy = Precisão
filter-min-accuracy = Precisão mínima (%)
scores-accuracy = Prec.

help-headshots = Abates por headshot, lidos da animação de morte da vítima. Explosões não contam. Só aparecem os headshots de quem gravou, porque os outros jogadores costumam matar fora da visão da gravação. Num duelo aparecem também os do adversário, já que todos os abates dele são em quem gravou.
help-headshots-column = Headshots de quem gravou, lidos da animação de morte da vítima. Explosões não contam. Os dos outros jogadores não contam, porque costumam acontecer fora da visão da gravação.
help-accuracy = A parte dos disparos que acertaram, pelo sangue que um acerto deixa na linha de mira, descontando coincidências ao acaso. Vale para a glock, a python, as balas da MP5, a escopeta, a besta com zoom e os acertos diretos do raio gauss. Explosões e reflexos do gauss não deixam sangue e não contam. Aparece só para quem gravou, depois de pelo menos 50 disparos.
help-headshot-filter = Demos com pelo menos um headshot que a gravação viu, de qualquer jogador.
help-accuracy-filter = Demos em que quem gravou acertou pelo menos esta parte dos disparos.

clips-title = Clipes
clips-add-event = Criar clipe { $start } – { $end }
clips-add-event-none = Criar clipe do evento escolhido
clips-select-event = Primeiro escolha um evento na lista ou na linha do tempo.
clips-other-session = Por enquanto só a primeira sessão da gravação pode ser exportada.
clips-add-window = Criar clipe deste momento intenso
clips-empty = Ainda não há clipes. Escolha um evento ou um dos momentos mais intensos e crie um clipe em volta dele. Quanto um clipe guarda antes e depois de um evento é ajustado em Configurações.
clip-default-name = Clipe { $number }
clip-name = Nome
clip-length = Duração { $duration }
clip-show = Mostrar
clip-remove = Remover
clip-time-help = Segundos (96), minutos e segundos (1:36) ou horas, minutos e segundos (00:01:36.250)
clip-exact = A exportação começa e termina exatamente aqui.
clip-effective = A exportação corta { $start } – { $end } ({ $duration }), onde caem os quadros da gravação.
timeline-clip-help = Os clipes são as faixas azul-claras. Arraste a borda de uma faixa para movê-la.
settings-clip-padding = Margem dos clipes
settings-preroll = Segundos antes do evento
settings-postroll = Segundos depois do evento
export-start-first-group = Um corte não pode começar no primeiro quadro da gravação. Comece um pouco depois.
export-start-later-session = O início está numa sessão posterior da gravação, onde um corte ainda não pode começar.

export-all = Exportar todos os clipes
export-one = Exportar
export-cancel = Parar a exportação
export-folder = As exportações vão para { $folder }.
export-folder-beside = As exportações ficam junto da demo, em { $folder }.
export-waiting = Aguardando para exportar…
export-running = Exportando… Isso lê a demo inteira e pode demorar.
export-cancelled = Não exportado, porque a exportação foi parada.
export-saved = Salvo em { $path }. Snapshots conferidos com o original: { $count }.
export-playdemo = No console do jogo:
export-copy-first = Copie { $name } para a pasta do seu mod, a que tem liblist.gam, e depois no console do jogo:
export-copy = Copiar
settings-export-folder = Pasta de exportação
settings-export-choose = Escolher…
settings-export-beside = Junto de cada demo
settings-export-hint = Dentro da pasta do seu mod, a que tem liblist.gam como Half-Life\ag, os clipes tocam com um só comando playdemo.

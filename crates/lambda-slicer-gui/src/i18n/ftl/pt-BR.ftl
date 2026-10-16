# Português do Brasil. Rascunho não revisado

app-title = Lambda Slicer

toolbar-open = Abrir demo…
toolbar-add-folder = Adicionar pasta…
toolbar-rescan = Escanear de novo
toolbar-cancel = Cancelar
toolbar-settings = Configurações

settings-title = Configurações
settings-language = Idioma
settings-unreviewed = Esta tradução é um rascunho que nenhum falante nativo revisou ainda.
settings-workers = Processos de análise
settings-workers-hint = Mais processos analisam mais rápido, mas usam mais tempo do processador.
settings-cache = Cache da biblioteca
settings-folders = Pastas da biblioteca
settings-folders-empty = Nenhuma pasta adicionada ainda.
settings-remove = Remover
settings-close = Fechar

library-empty = Adicione uma pasta ou abra uma demo para começar.
library-demos = { $count ->
    [one] { $count } demo
    [many] { $count } de demos
   *[other] { $count } demos
}
library-kills = { $count ->
    [one] { $count } abate
    [many] { $count } de abates
   *[other] { $count } abates
}
library-action = Pontuação de ação { NUMBER($score, minimumFractionDigits: 2) }
library-spectating = Espectando { $percent }%

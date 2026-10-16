# Español de Latinoamérica. Borrador sin revisar

app-title = Lambda Slicer

toolbar-open = Abrir demo…
toolbar-add-folder = Agregar carpeta…
toolbar-rescan = Volver a escanear
toolbar-cancel = Cancelar
toolbar-settings = Configuración

settings-title = Configuración
settings-language = Idioma
settings-unreviewed = Esta traducción es un borrador que todavía no revisó ningún hablante nativo.
settings-workers = Procesos de análisis
settings-workers-hint = Más procesos analizan más rápido, pero usan más tiempo del procesador.
settings-cache = Caché de la biblioteca
settings-folders = Carpetas de la biblioteca
settings-folders-empty = Todavía no agregaste carpetas.
settings-remove = Quitar
settings-close = Cerrar

library-empty = Agrega una carpeta o abre una demo para empezar.
library-demos = { $count ->
    [one] { $count } demo
    [many] { $count } de demos
   *[other] { $count } demos
}
library-kills = { $count ->
    [one] { $count } baja
    [many] { $count } de bajas
   *[other] { $count } bajas
}
library-action = Puntaje de acción { NUMBER($score, minimumFractionDigits: 2) }
library-spectating = Espectador { $percent } %

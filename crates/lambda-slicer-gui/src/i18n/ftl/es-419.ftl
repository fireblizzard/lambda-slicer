# Español de Latinoamérica. Borrador sin revisar

app-title = Lambda Slicer

## Barra de herramientas

toolbar-open = Abrir demo…
toolbar-add-folder = Agregar carpeta…
toolbar-rescan = Volver a escanear
toolbar-cancel = Cancelar
toolbar-settings = Configuración
dialog-demos = Demos de Half-Life

## Configuración

settings-title = Configuración
settings-language = Idioma
settings-unreviewed = Esta traducción es un borrador que todavía no revisó ningún hablante nativo.
settings-workers = Procesos de análisis
settings-workers-hint = Más procesos analizan más rápido, pero usan más tiempo del procesador.
settings-cache = Caché de la biblioteca
settings-folders = Carpetas de la biblioteca
settings-folders-empty = Todavía no agregaste carpetas.
settings-remove = Quitar

## Biblioteca

library-empty = Agrega una carpeta o abre una demo para empezar.
library-no-matches = Ninguna demo coincide con estos filtros.
library-no-cache = No hay dónde guardar la caché de la biblioteca en este sistema.
library-demos = { $count ->
    [one] { $count } demo
    [many] { $count } de demos
   *[other] { $count } demos
}
library-percent = { $percent } %

filter-search = Buscar rutas
filter-map = Mapa
filter-player = Jugador
filter-weapon-any = Cualquier arma
filter-min-minutes = Al menos (minutos)
filter-spectating-any = Jugando o de espectador
filter-spectating-mostly = Casi todo de espectador
filter-spectating-playing = Casi todo jugando
filter-clear = Quitar filtros

column-status = Estado
column-name = Demo
column-map = Mapa
column-duration = Duración
column-gauss = Gauss
column-kills = Bajas
column-action = Intensidad
column-spectating = Espectador

status-complete = Analizada
status-partial = Parcial
status-failed = Falló
status-unsupported = No compatible
status-no-use = Sin uso de armas

kind-multiplayer-view = Vista del jugador
kind-singleplayer = Un jugador
kind-hltv = Proxy de espectador
kind-unknown = Tipo desconocido

details-recorder = Grabada por { $name }
details-shots = { $count ->
    [one] { $count } disparo
    [many] { $count } de disparos
   *[other] { $count } disparos
}
details-kills = { $count ->
    [one] { $count } baja
    [many] { $count } de bajas
   *[other] { $count } bajas
}
details-action = Mayor intensidad { NUMBER($score, minimumFractionDigits: 2) }

## Escaneo

scan-progress = Encontradas { $found }, listas { $done } ({ $cached } en caché, { $analysed } analizadas), { $megabytes } MB leídos
scan-cancelling = Deteniendo…
scan-cancelled = Detenido con { $left } por escanear.
scan-finished = Listo: { $done } demos, { $analysed } analizadas, en { $seconds } s.
scan-problems = { $count ->
    [one] { $count } problema
    [many] { $count } de problemas
   *[other] { $count } problemas
}
problems-title = Problemas del escaneo
problem-changed = El archivo cambió mientras se analizaba, así que no se guardó nada.
drop-hint = Suelta demos o carpetas para agregarlas
drop-not-demo = No es una demo: { $names }

## Mensajes del núcleo de análisis, por código

analysis-panicked = El análisis se detuvo por un error interno: { $detail }
cache-cancelled = El análisis se canceló, así que no se guardó nada.
cache-database = No se puede usar la caché de la biblioteca: { $detail }
cache-directory = No se puede crear la carpeta de la caché: { $detail }
container-bad-magic = Esto no es una demo de Half-Life.
container-frame-bad-time = La grabación está dañada en el fotograma { $frame } de la parte { $entry }, que tiene un tiempo no válido.
container-frame-missing-terminal = La parte { $entry } de la grabación termina sin su registro de cierre.
container-frame-negative-length = La grabación está dañada en el fotograma { $frame } de la parte { $entry }.
container-frame-over-limit = El fotograma { $frame } de la parte { $entry } es más grande de lo permitido.
container-frame-truncated = La grabación se corta en el fotograma { $frame } de la parte { $entry }.
container-frame-unknown-tag = La grabación está dañada en el fotograma { $frame } de la parte { $entry }, que es de un tipo desconocido.
container-io = No se puede leer el archivo: { $detail }
container-no-directory = La grabación nunca se terminó, así que no tiene índice.
container-range = El índice de la grabación apunta fuera del archivo.
container-too-short = El archivo es demasiado corto para ser una demo.
coverage-hltv = Grabación de un proxy de espectador. No hay jugador que grabe y los eventos siguen la vista del proxy.
coverage-mostly-spectating = Quien grabó estuvo de espectador el { $percent } % de la reproducción, así que la mayoría de los eventos son de otros jugadores según los mostró la vista observada.
coverage-no-local-events = No hay eventos locales. Faltan las acciones de quien grabó, salvo las que envió el servidor.
coverage-singleplayer = Un jugador. Los demás actores son monstruos o entidades del mapa, y no hay registro de muertes.
coverage-unknown = La grabación no se decodificó lo suficiente para saber de qué tipo es.
coverage-visible-only = Los eventos de otros jugadores solo llegan a este cliente mientras el servidor los considera visibles, así que sus conteos son mínimos.
decode-failed = La decodificación se detuvo en { $time } (fotograma { $frame }): { $detail }
export-active-effect = { $effect } está en curso al inicio y todavía no se puede trasladar.
export-client-dropped-packets = El juego descartó paquetes atrasados en { $time }, algo que un corte todavía no puede reproducir.
export-cross-segment = Una nueva sesión empieza en { $time }, dentro del rango.
export-destination-exists = { $path } ya existe. Elige otro nombre.
export-empty-range = El rango seleccionado no contiene nada para exportar.
export-io = No se puede escribir { $path }: { $detail }
export-missing-delta-base = A la grabación le falta un estado necesario antes del inicio (secuencia { $sequence }).
export-missing-parent = La carpeta de { $path } no existe.
export-range-beyond-end = El final { $end } está después del final de la grabación, { $segment_end }.
export-same-as-source = El destino es la misma demo de origen.
export-source-changed = { $path } cambió durante la exportación.
export-source-decode-failed = La demo de origen no se puede decodificar: { $detail }
export-source-failed-crosscheck = La demo de origen no pasó su verificación cruzada, así que su estado reconstruido no es confiable.
export-unknown-persistent-message = { $name } antes del inicio fija un estado que un corte todavía no puede trasladar.
export-unproven-state = El estado en { $time } se basa en una base inferida y no está comprobado.
export-unrepresentable-time-field = Un tiempo al inicio no se puede escribir con fidelidad: { $detail }
export-unsupported = No compatible: { $detail }
export-validation-failed = El archivo exportado no pasó la validación: { $detail }
io-not-found = No se encontró el archivo o la carpeta.
io-other = { $detail }
io-permission-denied = Acceso denegado.
io-unexpected-end = El archivo terminó de forma inesperada.
protocol-unsupported = El protocolo de demo { $demo }/{ $network } no es compatible, solo 5/48.
protocol-unsupported-network = El servidor cambió al protocolo de red { $network }, que no es compatible.

## Vista de la demo

demo-close = Cerrar
demo-loading = Analizando { $name }…
timeline-fit = Ajustar
show-shots = Disparos
show-actions = Otras acciones
show-kills = Bajas
show-switches = Cambios de arma
lane-unknown = Jugador desconocido
lane-recorder = { $name } (grabó)
column-time = Tiempo
column-player = Jugador
column-weapon = Arma
column-event = Evento
event-kill = Mató a { $victim }
event-switch = Cambio de arma
action-swing = Golpe
action-fire = Disparo
action-launch = Lanzamiento
action-charge-activity = Cargando
action-charge-stop = Fin de carga
action-beam-start = Inicio del rayo
action-beam-sustain = Rayo
action-beam-stop = Fin del rayo
action-placement-attempt = Intento de colocación
action-release-attempt = Intento de liberación
mode-primary = primario
mode-charged = cargado
mode-zoomed = con zoom
mode-unzoomed = sin zoom
mode-grenade = granada
mode-bullets = balas
mode-single = simple
mode-double = doble
mode-rapid = rápido
demo-events = { $count ->
    [one] { $count } evento
    [many] { $count } de eventos
   *[other] { $count } eventos
}

column-date = Fecha
column-score = Marcador
filter-from = Desde
filter-to = Hasta
date-from-name = Fecha tomada del nombre del archivo
date-from-modified = Fecha de la última modificación del archivo, que puede cambiar al copiarlo o descargarlo
timeline-best-action = Minuto de oro
timeline-windows = Momentos más intensos
timeline-window = N.º { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Puntajes finales, en la última baja { $time }
scores-at = Puntajes a las { $time }
scores-show-final = Ver puntajes finales
scores-frags = Bajas
scores-deaths = Muertes

show-timeline = Mostrar línea de tiempo
legend-shot = Disparo
legend-charge = Cargando
legend-kill = Baja
legend-death = Muerte
legend-switch = Cambio de arma
timeline-help = Cada fila es un jugador. Usa la rueda para acercar, arrastra para moverte, haz doble clic para ver todo y clic en una marca para seleccionarla.

column-headshots = Headshots
filter-headshots = Bajas con headshot
event-headshot = headshot
scores-headshots = HS

column-accuracy = Precisión
filter-min-accuracy = Precisión mínima (%)
scores-accuracy = Prec.

help-headshots = Bajas por headshot, según la animación de muerte de la víctima. Las explosiones no cuentan. Solo se muestran los headshots de quien grabó, porque los demás jugadores suelen matar fuera de la vista de la grabación. En un duelo también se muestran los del rival, ya que todas sus bajas son sobre quien grabó.
help-headshots-column = Headshots de quien grabó, según la animación de muerte de la víctima. Las explosiones no cuentan. Los de los demás jugadores no se cuentan, porque suelen ocurrir fuera de la vista de la grabación.
help-accuracy = La parte de los disparos que impactaron, según la sangre que deja un impacto en la línea de tiro, descontando las coincidencias casuales. Incluye la glock, la python, las balas de la MP5, la escopeta, la ballesta con zoom y los impactos directos del rayo de la gauss. Las explosiones y los rebotes de la gauss no dejan sangre y no cuentan. Solo se muestra para quien grabó, después de al menos 50 disparos.
help-headshot-filter = Demos con al menos un headshot que la grabación vio, de cualquier jugador.
help-accuracy-filter = Demos en las que quien grabó acertó al menos esta parte de sus disparos.

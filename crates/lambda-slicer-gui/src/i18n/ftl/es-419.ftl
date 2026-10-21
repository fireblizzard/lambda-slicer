# Español de Latinoamérica. Borrador sin revisar

app-title = Lambda Slicer

## Barra de herramientas

toolbar-open = Abrir demo…
toolbar-add-folder = Agregar carpeta…
toolbar-rescan = Volver a escanear
toolbar-cancel = Cancelar
toolbar-settings = Configuración
toolbar-rescan-busy = Ya hay un escaneo en curso.
toolbar-rescan-nothing = Agrega una carpeta o abre una demo para escanear.
toolbar-cancel-idle = No hay ningún escaneo en curso.
dialog-demos = Demos de Half-Life

## Configuración

settings-title = Configuración
settings-language = Idioma
settings-unreviewed = Esta traducción es un borrador que todavía no revisó ningún hablante nativo.
settings-workers = Demos analizadas a la vez
settings-workers-hint = Analizar más a la vez termina antes, pero hace trabajar más a la computadora.
settings-cache = Dónde se guarda la biblioteca
settings-sort = Orden al abrir
settings-sort-hint = Cómo se ordenan las demos cada vez que se abre Lambda Slicer. Si haces clic en el encabezado de una columna, se ordena por ella hasta que cierres Lambda Slicer.
settings-folders = Carpetas de la biblioteca
settings-folders-empty = Todavía no agregaste carpetas.
settings-remove = Quitar

## Biblioteca

library-empty = Agrega una carpeta o abre una demo para empezar.
library-no-matches = Ninguna demo coincide con estos filtros.
library-no-cache = No hay ningún lugar en esta computadora para guardar la biblioteca.
library-demos = { $count ->
    [one] { $count } demo
    [many] { $count } de demos
   *[other] { $count } demos
}
library-percent = { $percent } %
library-demos-of = { $shown } de { $total ->
    [one] { $total } demo
    [many] { $total } de demos
   *[other] { $total } demos
}
library-total = { $count ->
    [one] { $count } demo en la biblioteca
    [many] { $count } de demos en la biblioteca
   *[other] { $count } demos en la biblioteca
}
filters-title = Filtros
filters-find = Buscar
filters-pov = Perspectiva
filters-quality = Calidad
filters-recorded = Grabada
pov-any = Todas
pov-playing = Jugando
pov-spectating = Espectador
help-pov = «Jugando» quiere decir que la grabación estuvo de espectador como mucho la mitad de la reproducción, y «Espectador», más de la mitad.
columns-button = Columnas
list-back = Volver a la tabla de la biblioteca
sort-by = Ordenar por
sort-ascending = Ascendente
sort-descending = Descendente
score-more-sides = { $count ->
    [one] { $count } bando más
    [many] { $count } de bandos más
   *[other] { $count } bandos más
}
intensity-help = Intensidad { $score }. La barra la compara con la más alta de la biblioteca, { $max }.
value-unknown = Se desconoce
path-copy = Copiar ubicación del archivo
path-show = Mostrar en la carpeta
path-show-failed = No se pudo abrir la carpeta ({ $detail }). Copia la ubicación del archivo en su lugar.

filter-search = Buscar nombres de archivo y carpetas
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
kind-hltv = HLTV
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

scan-progress = Encontradas { $found }, listas { $done } ({ $cached } ya conocidas, { $analysed } analizadas), { $megabytes } MB leídos
scan-cancelling = Deteniendo…
scan-cancelled = Detenido con { $left } por escanear.
scan-finished = Listo: { $done ->
    [one] { $done } demo
    [many] { $done } de demos
   *[other] { $done } demos
}, { $analysed ->
    [one] { $analysed } analizada
   *[other] { $analysed } analizadas
}, en { $seconds } s.
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

analysis-panicked = El análisis se detuvo por una falla de Lambda Slicer: { $detail }
cache-cancelled = El análisis se canceló, así que no se guardó nada.
cache-database = No se puede usar la biblioteca guardada: { $detail }
cache-directory = No se puede crear la carpeta para la biblioteca guardada: { $detail }
container-bad-magic = Esto no es una demo de Half-Life.
container-frame-bad-time = La grabación está dañada en el fotograma { $frame } de la parte { $entry }, que tiene un tiempo incorrecto.
container-frame-missing-terminal = La parte { $entry } de la grabación no termina como debería.
container-frame-negative-length = La grabación está dañada en el fotograma { $frame } de la parte { $entry }.
container-frame-over-limit = El fotograma { $frame } de la parte { $entry } es más grande de lo permitido.
container-frame-truncated = La grabación se corta en el fotograma { $frame } de la parte { $entry }.
container-frame-unknown-tag = La grabación está dañada en el fotograma { $frame } de la parte { $entry }, que contiene algo que Lambda Slicer no reconoce.
container-io = No se puede leer el archivo: { $detail }
container-no-directory = La grabación nunca se terminó, así que no tiene índice.
container-range = El índice de la grabación apunta fuera del archivo.
container-too-short = El archivo es demasiado corto para ser una demo.
coverage-hltv = Grabación de HLTV. Ningún jugador la grabó, así que los eventos siguen lo que HLTV estaba mirando.
coverage-mostly-spectating = Quien grabó estuvo de espectador el { $percent } % de la reproducción, así que la mayoría de los eventos son de otros jugadores, tal como se vieron mientras los miraba como espectador.
coverage-no-local-events = Las acciones de quien grabó no se guardaron en esta demo, así que solo aparecen las que envió el servidor.
coverage-singleplayer = Un jugador. Todo lo demás son monstruos o partes del mapa, y no hay kill feed.
coverage-unknown = No se pudo leer la grabación lo suficiente para saber de qué tipo es.
coverage-visible-only = El juego solo recibe las acciones de otros jugadores mientras el servidor cree que quien grabó puede verlos, así que sus conteos pueden ser menores que lo que pasó en realidad.
decode-failed = La lectura de la grabación se detuvo en { $time } (fotograma { $frame }): { $detail }
export-active-effect = { $effect } sigue en curso al inicio, y un clip todavía no puede conservarlo.
export-client-dropped-packets = En { $time } el juego descartó datos que llegaron tarde, algo que un clip todavía no puede copiar.
export-cross-segment = El mapa cambió o el jugador se reconectó en { $time }, dentro del clip, y un clip no puede pasar por encima de eso.
export-destination-exists = { $path } ya existe. Elige otro nombre.
export-empty-range = El clip no contiene nada para exportar.
export-io = No se puede escribir { $path }: { $detail }
export-missing-delta-base = A la grabación le faltan datos de antes del inicio que el clip necesita (número { $sequence }).
export-missing-parent = La carpeta { $path } no existe.
export-range-beyond-end = El final { $end } está después del final de la grabación, { $segment_end }.
export-same-as-source = La exportación reemplazaría la demo de la que sale.
export-source-changed = { $path } cambió durante la exportación.
export-source-decode-failed = No se puede leer la demo: { $detail }
export-source-failed-crosscheck = Al revisarla, lo que Lambda Slicer dedujo de esta demo no coincidió con la demo misma, así que no se puede confiar en los clips que salgan de ella.
export-unknown-persistent-message = { $name } antes del inicio fija algo que un clip todavía no puede conservar.
export-unproven-state = En { $time } la grabación depende de datos que hubo que adivinar, así que el clip no se puede hacer con exactitud.
export-unrepresentable-time-field = Un tiempo al inicio no se puede guardar con exactitud: { $detail }
export-unsupported = No compatible: { $detail }
export-validation-failed = El clip exportado no pasó su revisión final: { $detail }
io-not-found = No se encontró el archivo o la carpeta.
io-other = { $detail }
io-permission-denied = Acceso denegado.
io-unexpected-end = El archivo terminó de forma inesperada.
protocol-unsupported = Esta demo es de una versión del juego que no es compatible (protocolo { $demo }/{ $network }). Solo lo es la 5/48.
protocol-unsupported-network = El servidor cambió a una versión del juego que no es compatible (protocolo { $network }).

## Vista de la demo

demo-close = Cerrar
demo-loading = Analizando { $name }…
show-shots = Disparos
show-actions = Otras acciones
show-kills = Bajas
show-switches = Cambios de arma
lane-unknown = Jugador desconocido
lane-recorder = { $name } (grabó)
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
column-file-date = Fecha del archivo
column-score = Marcador
filter-from = Desde
filter-to = Hasta
date-from-name = Fecha tomada del nombre del archivo
date-from-modified = Fecha de la última modificación del archivo, que puede cambiar al copiarlo o descargarlo
timeline-best-action = Minuto de oro
timeline-window = N.º { $rank } { NUMBER($score, minimumFractionDigits: 2) } · { $start }–{ $end } · { $sides }

scores-final = Puntajes finales, en la última baja { $time }
scores-at = Puntajes a las { $time }
scores-show-final = Ver puntajes finales
scores-frags = Bajas
scores-deaths = Muertes

viewer-play = Reproducir
viewer-pause = Pausar
viewer-volume = Volumen
viewer-no-sound = Sin sonido
viewer-play-backward = Rebobinar
viewer-step-back = Frame anterior
viewer-step-forward = Siguiente frame
viewer-position = Posición en la demo
viewer-skip-back = Retroceder 10 segundos
viewer-skip-forward = Avanzar 10 segundos
viewer-rate = { $rate }×
viewer-shortcut = Atajo: { $key }
key-space = Espacio
key-shift-left = Shift+Izquierda
key-shift-right = Shift+Derecha
viewer-map-failed = No se pudo cargar el mapa { $map }
viewer-stopped = La reproducción se detuvo antes de tiempo: { $reason }
viewer-missing = Archivos que no se pudieron leer: { $count }
event-previous = Evento anterior
event-next = Evento siguiente
event-play = Reproducir evento
timeline-title = Línea de tiempo
timeline-lanes = Filas por jugador
timeline-zoom-in = Acercar
timeline-zoom-out = Alejar
timeline-fit-all = Ajustar a toda la demo
timeline-gestures = Haz clic en una marca para seleccionar su evento, o en cualquier otro lugar para ir ahí. Arrastra sobre los tiempos de arriba, o arrastra la línea que marca el momento actual, para moverte por la demo. Arrastra en cualquier otro lugar para deslizar la vista. Usa la rueda para acercar y alejar donde está el mouse, y haz doble clic para ver toda la demo. Arrastra el borde de un clip para cambiar dónde empieza o termina, o haz clic en el clip para editarlo.
timeline-density = Las barras sombreadas muestran cuántos de los eventos mostrados pasan en cada punto, comparado con el punto más movido de la demo.
timeline-density-count = { $count ->
    [one] Un evento mostrado aquí
    [many] { $count } de eventos mostrados aquí
   *[other] { $count } eventos mostrados aquí
}
timeline-session-count = Parte { $session }: { $count }
timeline-choose = Eventos en este punto
legend-clip = Clip
event-session = Parte { $session }
event-unreachable = Este evento viene después de un cambio de mapa o una reconexión, donde el reloj de la demo volvió a empezar, así que el visor no puede ir a él.
event-hidden = Los filtros ocultan el evento seleccionado.
highlight-jump = Ir a una jugada destacada…
highlight-help = Las jugadas destacadas ordenan toda la demo por su acción, sin importar lo que muestren los filtros.
kills-with = Bajas con
refine-midair = Midair
refine-speed = Velocidad
refine-wallgauss = Wallgauss
refine-clear = Quitar
refine-airborne = En el aire
refine-either = Cualquiera
midair-both = Ambos
midair-one = Solo uno
help-midair-either = El asesino, la víctima o ambos estaban en el aire. Al asesino se lo revisa en el disparo vinculado a la baja, o en la baja si no hay uno, y a la víctima en la muerte. Alguien que solo salta, a menos de 50 unidades sobre el suelo, no cuenta. Requiere elegir la carpeta de Half-Life para saber dónde está el suelo.
help-midair-both = El asesino y la víctima estaban ambos en el aire, cada uno al revisarlo.
help-midair-one = Exactamente uno de los dos estaba en el aire, y se sabe que el otro estaba en el suelo.
speed-at-least = Al menos
speed-unit = ups
speed-killer = Asesino
speed-victim = Víctima
help-speed = Velocidad sobre el suelo en ups, como la muestra un velocímetro, sin contar el movimiento hacia arriba y hacia abajo. Al asesino se lo mide en el disparo vinculado a la baja, o en la baja si no hay uno, y a la víctima en la muerte. Las velocidades de otros jugadores son estimaciones.
wallgauss-weapon = Wallgauss solo abarca bajas con la gauss, y el filtro de arma muestra { $weapon }.
wallgauss-checking = Comprobando Wallgauss… { $examined } de { $total } bajas
wallgauss-needs-files = Wallgauss necesita los archivos del juego para saber dónde están las paredes del mapa.
wallgauss-failed = No se pudo comprobar Wallgauss. { $reason }
help-wallgauss = Bajas con la gauss a través de una pared, encontradas siguiendo cada disparo por el mapa como lo hace el servidor. Una baja que no se puede decidir, como una cuya víctima estaba fuera de la vista de la grabación, cuenta como desconocida y queda fuera.
badge-wall = WG
wall-yes-exit = Wallgauss, deducido del disparo: la explosión donde el rayo salió de una pared de { $thickness } unidades de grosor.
wall-yes-beyond = Wallgauss, deducido del disparo: el rayo pegó después de atravesar una pared de { $thickness } unidades de grosor.
wall-limits = Deducido del disparo y de dónde estaban los jugadores, ya que el servidor no lo registra. Un jugador mide 32 unidades de ancho y cuenta como una caja, y la víctima pudo estar a uno o dos cuerpos de donde la muestra la demo.
wall-no-direct = No es Wallgauss: el rayo pudo darle a la víctima antes de cualquier pared.
wall-no-reflection = No es Wallgauss: la explosión de un rebote pudo darle a la víctima antes de cualquier pared.
wall-doubt-no-shot = Wallgauss desconocido: el disparo del asesino no está en la grabación.
wall-doubt-no-map = Wallgauss desconocido: no se pudieron leer los archivos del mapa.
wall-doubt-unseen = Wallgauss desconocido: la víctima estaba fuera de la vista de la grabación.
wall-doubt-brush = Wallgauss desconocido: el rayo dio con una puerta, un elevador u otra parte especial del mapa, que no se revisa.
wall-doubt-ambiguous = Wallgauss desconocido: el disparo pudo matar tanto a través de una pared como sin ella.
wall-doubt-unexplained = Wallgauss desconocido: no se encontró cómo el disparo llegó a la víctima.
wall-doubt-unconfirmed = Wallgauss desconocido: el disparo solo pudo llegar a la víctima a través de una pared, pero no es seguro.
wall-doubt-not-reached = Wallgauss desconocido: la grabación termina antes del disparo.
wall-pending = Wallgauss aún no comprobado.
events-unknown = { $count ->
    [one] A una baja le faltan datos para saberlo
    [many] A { $count } de bajas les faltan datos para saberlo
   *[other] A { $count } bajas les faltan datos para saberlo
}
help-unknown = El movimiento que la grabación no pudo ver cuenta como desconocido, nunca como en el suelo o quieto, así que estas bajas quedan fuera.
events-follow = Mantener la lista en el momento actual mientras se reproduce la demo. Si haces scroll en la lista, se desactiva.
clip-it = Clipear
badge-headshot = HS
badge-air = AIR
badge-air-both = AIR ×2
speed-ups = { $speed } ups
speed-ups-unknown = — ups
badge-suicide = suicidio
badge-teamkill = teamkill
badge-world = mundo
badge-clip = clip
clip-of = Clip de { $name }
air-both = Ambos estaban en el aire al revisarlos.
air-killer = El asesino estaba en el aire al revisarlo.
air-victim = La víctima estaba en el aire al revisarla.
air-other-unknown = No se sabe si el otro también.
speed-horizontal = Velocidad horizontal, paralela al suelo.
motion-line = { $who }: { $air }, { $speed }. { $how }, { $when }.
motion-who-killer = Asesino
motion-who-victim = Víctima
motion-air-yes = en el aire
motion-air-no = en el suelo
motion-air-unknown = no se sabe si en el aire
motion-speed-unknown = velocidad desconocida
motion-exact = Registrado por el juego
motion-trajectory = Estimado por el movimiento
motion-unclear = Movimiento poco claro
motion-not-visible = Fuera de vista
motion-at-shot = en el disparo vinculado
motion-at-kill = en la muerte
motion-none = El análisis no tiene movimiento para esta baja.
legend-shot = Disparo
legend-charge = Cargando
legend-kill = Baja
legend-death = Muerte
legend-switch = Cambio de arma

column-headshots = Headshots
filter-headshots = Bajas con headshot
filter-midair = Bajas en midair
event-headshot = headshot
scores-headshots = HS

column-accuracy = Precisión
filter-min-accuracy = Precisión mínima (%)
scores-accuracy = Prec.

help-headshots = Bajas por headshot, según la animación de muerte de la víctima. Las explosiones no cuentan. Solo se muestran los headshots de quien grabó, porque los demás jugadores suelen matar fuera de la vista de la grabación. En un duelo también se muestran los del rival, ya que todas sus bajas son sobre quien grabó.
help-headshots-column = Headshots de quien grabó, según la animación de muerte de la víctima. Las explosiones no cuentan. Los de los demás jugadores no se cuentan, porque suelen ocurrir fuera de la vista de la grabación.
help-date-column = Cuándo se grabó la partida. Se toma del nombre del archivo cuando el nombre tiene una fecha como 20230910_214553, como las grabaciones automáticas del juego. Si no, es la fecha del archivo, que puede cambiar al copiarlo o descargarlo.
help-file-date = Cuándo se guardó el archivo. En un clip exportado, es cuándo se exportó.
help-accuracy = La parte de los disparos que impactaron, según la sangre que deja un impacto en la línea de tiro, descontando las coincidencias casuales. Incluye la glock, la python, las balas de la MP5, la escopeta, la ballesta con zoom y los impactos directos del rayo de la gauss. Las explosiones y los rebotes de la gauss no dejan sangre y no cuentan. Solo se muestra para quien grabó, después de al menos 50 disparos.
help-headshot-filter = Demos con al menos un headshot que la grabación vio, de cualquier jugador.
help-midair-filter = Demos con al menos una baja en la que el asesino o la víctima estaba en el aire, a 50 unidades o más sobre el suelo. Sin la carpeta de Half-Life elegida, también cuenta un salto normal.
help-accuracy-filter = Demos en las que quien grabó acertó al menos esta parte de sus disparos.

clips-new = Nuevo clip
clips-select-event = Primero elige un evento en la lista o en la línea de tiempo.
clips-other-session = Un clip no puede empezar aquí, porque el reloj de la demo volvió a empezar después de un cambio de mapa o una reconexión, y una parte anterior tiene los mismos tiempos.
clips-add-window = Crear clip de este momento intenso
clip-default-name = Clip { $number }
clip-name = Nombre
clip-length = Duración { $duration }
clip-remove = Quitar
clip-loop = Repetir
clip-time-help = Segundos (96), minutos y segundos (1:36) u horas, minutos y segundos (00:01:36.250)
clips-empty-hint = Todavía no hay clips. Elige un evento y presiona Clipear, o presiona Nuevo clip para empezar uno en el momento actual.
clips-new-help = Un clip de diez segundos desde el momento actual.
clip-in = Inicio
clip-out = Fin
clip-set-in = Empezar el clip en el momento actual
clip-set-out = Terminar el clip en el momento actual
clip-no-playhead = El visor no está disponible, así que no hay un momento actual que usar.
clip-play = Reproducir el clip una vez
clip-show-on-timeline = Mostrar en la línea de tiempo
export-destination = Cambiar la carpeta de exportación
export-skipped = Omitido. { $reason }
export-earlier-range = Esta exportación es de { $start } – { $end }, el clip como estaba antes de que lo cambiaras.
scores-player = Jugador
scores-none = La grabación no tiene puntajes.
clip-effective = La exportación abarca { $start } – { $end } ({ $duration }), lo más cerca que permite la grabación.
settings-clip-padding = Margen de los clips
settings-preroll = Segundos antes del evento
settings-postroll = Segundos después del evento
export-start-first-group = Un clip no puede empezar justo al principio de la grabación. Empieza un poco después.
export-start-during-loading = El inicio cae mientras el juego todavía carga un mapa. Empieza un poco después.

export-all = Exportar todos los clips
export-one = Exportar
export-cancel = Detener la exportación
export-folder = Las exportaciones van a { $folder }.
export-folder-beside = Las exportaciones van junto a la demo, en { $folder }.
export-waiting = Esperando para exportar…
export-running = Exportando… Esto lee toda la demo y puede tardar un rato.
export-cancelled = No se exportó, porque se detuvo la exportación.
export-saved = Guardado en { $path }. Momentos comprobados contra el original: { $count }.
export-playdemo = En la consola del juego:
export-copy-first = Copia { $name } en la carpeta de tu mod, la que tiene liblist.gam, y luego en la consola del juego:
export-copy = Copiar
settings-export-folder = Carpeta de exportación
settings-export-choose = Elegir…
settings-file-names = Nombres de archivo
settings-clip-name-files = Usar el nombre del clip para el archivo
settings-demo-name-first = Empezar con el nombre de la demo
settings-viewer = Visor de demos
settings-viewer-game = Carpeta de Half-Life
settings-viewer-game-none = Sin elegir
settings-viewer-game-hint = La carpeta que contiene valve. El visor reproduce las demos con los mapas, modelos y sonidos del juego que hay en ella.
settings-executable = Programa del juego
settings-executable-hint = Sirve para encontrar la carpeta de Half-Life. El visor lee los archivos del juego por su cuenta y nunca abre el juego.
settings-executable-not-set = Sin elegir, así que el visor no tiene imagen ni sonido
settings-executable-optional = Sin elegir, y no hace falta mientras la carpeta de Half-Life tenga el juego
settings-executable-missing = { $path } ya no está. El visor sigue leyendo la carpeta de Half-Life.
settings-game-override = Esta vez Lambda Slicer usa { $path }, porque se abrió con --game. Lo que elijas aquí vale desde la próxima vez que se abra.
settings-browse = Examinar…
dialog-executables = Programas
setup-title = Configura el visor de demos
setup-body = La imagen y el sonido necesitan los archivos de Half-Life. Indícale a Lambda Slicer dónde está instalado el juego y los leerá de ahí.
setup-locate-exe = Buscar hl.exe…
setup-locate-folder = Buscar Half-Life…
setup-settings = Configuración
setup-still-works = La línea de tiempo, los eventos y los clips siguen funcionando sin ellos.
setup-relocate = { $folder } ya no tiene Half-Life. Búscalo otra vez.
setup-not-executable = { $name } no es { $executable }. Elige el programa del juego, no un acceso directo.
setup-not-installed = { $folder } no tiene una instalación de Half-Life con carpeta valve.
tab-events = Eventos
tab-clips = Clips
tab-scoreboard = Marcador
list-hide = Ocultar la lista de demos
list-show = Mostrar la lista de demos
demo-details = Detalles
viewer-start = Volver al inicio
viewer-mute = Silenciar
viewer-unmute = Activar sonido
viewer-picture = Imagen y opciones del visor
viewer-needs-files = Necesita los archivos del juego
viewer-no-sound-why = Sin sonido: { $reason }
settings-tab-general = General
settings-tab-library = Biblioteca
settings-tab-clips = Clips
settings-section-game = Half-Life
settings-section-view = Vista
settings-language-system = Idioma de la computadora ({ $name })
settings-section-folders = Carpetas
settings-folders-hint = Las carpetas donde la biblioteca busca demos, incluidas sus subcarpetas.
settings-clip-padding-hint = Cuánto conserva un clip creado alrededor de un evento antes y después de él.
settings-section-picture = Imagen
settings-section-sound = Sonido
settings-gamma = Gamma
settings-brightness = Brillo
settings-texgamma = Gamma de texturas
settings-lightgamma = Gamma de luz
settings-overbright = Iluminación sobrebrillante
settings-gamma-hint = Los ajustes del juego con el mismo nombre (gamma, brightness, texgamma, lightgamma, gl_overbright). Ponlos como en el juego para ver el mismo brillo.
settings-gamma-defaults = Valores del juego
settings-fov-recorded = Campo de visión como se grabó
settings-fov = Campo de visión
settings-zoom-fov = Campo de visión con zoom
settings-viewmodel = Mostrar el arma en mano
settings-see-through = Ver jugadores a través de paredes
settings-see-through-items = También otros modelos, como proyectiles y objetos
settings-see-through-hint = Los jugadores detrás de paredes se ven como siluetas de su color superior, que los juegos por equipos fijan por equipo. Solo se puede ver lo que contiene la demo. El servidor envía los jugadores cercanos a quien grabó, y los servidores que ocultan a los jugadores que quien grabó no puede ver no dejan nada que mostrar.
settings-section-decals = Decals
settings-decals = Máximo
settings-decals-hint = Agujeros de bala, sangre y otras marcas que quedan durante la partida. Solo se quedan los más nuevos, y los más viejos se borran cuando salen nuevos. Los decals que son parte del mapa siempre se quedan.
settings-restore = Volver a la config por defecto…
settings-restore-title = Volver a la config por defecto
settings-restore-list = Estas opciones vuelven a como estaban al principio. La carpeta de Half-Life y las carpetas de la biblioteca se quedan como están.
settings-restore-none = Todas las opciones ya están en su valor por defecto.
settings-restore-columns = Columnas que se ven en la biblioteca
settings-restore-layout = Paneles abiertos y su tamaño
settings-clear-library = Vaciar biblioteca…
settings-clear-library-title = Vaciar biblioteca
settings-clear-library-text = { $count ->
    [one] Lambda Slicer olvida lo que encontró en { $count } demo y vuelve a escanear las carpetas de la biblioteca, lo que puede tardar un rato. Tus archivos de demos, clips y opciones se quedan como están.
    [many] Lambda Slicer olvida lo que encontró en { $count } de demos y vuelve a escanear las carpetas de la biblioteca, lo que puede tardar un rato. Tus archivos de demos, clips y opciones se quedan como están.
   *[other] Lambda Slicer olvida lo que encontró en { $count } demos y vuelve a escanear las carpetas de la biblioteca, lo que puede tardar un rato. Tus archivos de demos, clips y opciones se quedan como están.
}
viewer-zoom = Hacer zoom
viewer-see-through = Ver tras paredes
settings-export-beside = Junto a cada demo
settings-export-hint = Dentro de la carpeta de tu mod, la que tiene liblist.gam como Half-Life\ag, los clips se reproducen con un solo comando playdemo.
settings-file-names-hint = Un clip exportado se nombra con su demo y sus tiempos de inicio y fin, salvo que uses el nombre del clip. El juego solo puede abrir archivos con nombres hechos de letras del alfabeto inglés, números, - y ., así que todo lo demás se cambia por _. Si el nombre de un clip no tiene ninguno de estos caracteres, el archivo se nombra por sus tiempos.

clips-store-moved = No se pudo leer el archivo de clips { $path }, así que se guardó como { $moved } y los clips empiezan vacíos. { $detail }
clips-store-unreadable = No se pudo leer el archivo de clips { $path }: { $detail }. Hasta que se cierre Lambda Slicer, los clips no se guardan, así que no se pierde nada del archivo.
clips-save-failed = No se pudieron guardar los clips en { $path }: { $detail }
notice-dismiss = Cerrar

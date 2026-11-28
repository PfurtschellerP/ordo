tab-med = Sanitario
tab-command = Mando del incidente
# Workspaces of the incident command, by phase (ADR-0061).
ws-lead = Mando
ws-prepare = Preparación
ws-crew = Dotación
crew-tasks = Misiones
crew-transports = Transportes para su unidad
crew-none-here = No forma parte de ninguna dotación en este incidente. El mando del incidente le añade en Preparación – Unidades.
dispatch = Despacho
people = Personal
crew-patients = Mis pacientes
crew-transports-view = Mis transportes
tab-helpers = Intervinientes
tab-devices = Dispositivos
incident-page = Incidente

## The command structure and the map on one page (ADR-0095).
command-space = Estructura
# Its label in the phone's bottom bar.
tab-resources = Unidades
tab-hospitals = Hospitales
tab-free-units = Unidades de transporte libres
tab-free-units-short = Unidades libres
uhs-title = Puestos médicos avanzados
uhs-choose = Elegir PMA
units-title = Unidades
units-empty = Aún no hay unidades.
unit-call-sign = Indicativo
unit-type = Tipo (p. ej. ambulancia, transporte sanitario, equipo sanitario)
unit-organisation = Organización
unit-submit = Añadir unidad
unit-service = Servicio
unit-missing = Indique un indicativo y un tipo.
col-status = Estado
col-set-status = Fijar estado
no-status = –
set-status-label = Fijar estado de { $unit }
unit-status-title = Unidades: estado y misiones
unit-status-empty = Aún no hay unidades en el incidente. Se añaden en Preparación.
# Starts with the button's visible text, so its name matches what is seen.
task-give-label = { task-give }: { $unit }
move-here = Mover aquí
# Starts with the button's visible text, so its name matches what is seen.
move-here-label = { move-here }: { $person } a { $here }
group-units = Unidades
action-add-unit = Añadir unidad

## Incidents and roles (ADR-0046)
incidents-title = Incidentes
incidents-intro = Elija un incidente. Los roles en un incidente los dan el mando del incidente o la planificación.
incidents-loading = Cargando incidentes …
incidents-offline = La lista de incidentes solo está disponible con conexión al servidor.
# Logging out from the incident list: the device is unpaired, so another person can log in.
log-out = Cerrar sesión
log-out-hint = Cerrar sesión desvincula este dispositivo; después otra persona puede iniciar sesión.
incidents-empty = Aún no hay ningún incidente habilitado. El mando del incidente o la planificación dan a las personas un rol en un incidente.
open = Abrir
plan-title = Planificar incidente
plan-name = Nombre del incidente
plan-submit = Crear incidente
grant = Dar rol
incident-state-label = Estado
incident-page-title = Incidente
incident-prepare = Preparar: mapa de situación, unidades y personal con roles y estructura de mando. Inicie el incidente cuando esté en curso.

## The command's overview (ADR-0067)
open-map = Abrir mapa de situación
open-dispatch = Al despacho
open-units = A las unidades
open-label = abiertas
no-open-tasks = No hay misiones abiertas.
persons-on-scene = Intervinientes en el lugar
units-on-scene = Unidades en el lugar
quick-links = Acceso rápido
task-for = Para: { $assignee }
discharged-count = { $n } dados de alta
start-incident = Iniciar incidente
close-incident = Cerrar incidente
incident-paused-label = en pausa
pause-incident = Pausar incidente
resume-incident = Reanudar incidente
incident-pause-hint = Para incidentes de varios días, como las noches de un festival. Durante la pausa todo sigue utilizable; la situación y el diario de operaciones continúan.
incident-details = Datos del incidente
incident-description = Descripción
incident-description-hint = Por ejemplo lugar, organizador, público esperado, particularidades.
incident-starts = Inicio previsto
incident-ends = Fin previsto
incident-details-save = Guardar datos

## Treatment places
places = Puestos de tratamiento

## Transport requests (TRANS-1, TRANS-3)
transports = Transportes
transports-empty = No hay transportes abiertos.

## Overview for the medical lead (UHS-8).
overview = Resumen

## Units (UNIT-1…6).
strength-title = Efectivos en el lugar
strength = Efectivos
strength-legend-title = Efectivos (mandos / mandos intermedios / personal)
strength-numbers = Indique los efectivos como tres números enteros, o déjelo vacío.
leaders = Mandos
sub-leaders = Mandos intermedios
crew-count = Personal
by-organisation = Por organización
by-service = Por servicio
no-service = sin servicio
no-organisation = sin organización
crew = Dotación
crew-none = No hay dotación registrada.
crew-member = Nombre
crew-qualifications = Cualificaciones
crew-add = Añadir a la dotación
crew-remove = Quitar
release = Liberar del incidente
back-in = De vuelta en el incidente

## [impl->dsn~detailed-strength~1]
unit-leaders = Mandos de formación
org-chart = Estructura de mando como organigrama
special-functions-legend = De ellos con funciones especiales
strength-detailed = Indicar los efectivos en detalle: mandos de formación y jefes de sección por separado, funciones especiales como médicos y portadores de ERA aparte (DV 102, 2024)
# "of these 2 × physicians, 6 × BA wearers".
special-functions = de ellos { $listed }
unreported = sin parte de efectivos: { $n }
# "3 – Task accepted, since 10:12"
status-since = { $status }, desde { $since }
crew-label = { crew } de { $unit } ({ $n })
crew-remove-label = Quitar a { $name } de la dotación
release-label = Liberar { $unit } del incidente
back-in-label = { $unit } de vuelta en el incidente
released-list = Unidades liberadas ({ $n })
released-at = Liberada a las { $at }
crew-title = Dotación { $unit }
attention-due = Seguimiento pendiente: { $unit } – { $task }
attention-new-task = Nueva misión: { $task }
attention-unassigned = Misión sin unidad: { $task }
case-register = Admitir paciente
open-in-maps-app = Abrir en la aplicación de mapas
log-report-closed = Aviso cerrado: { $report }
log-report-patient = Paciente de la misión { $task } admitido por { $unit }

## Tasks (TASK-1…4).
tasks-heading = Misiones abiertas
tasks-empty = No hay misiones abiertas.
task-give = Dar misión
task-unit = Para
task-unit-choose = Elegir unidad o puesto
task-template = Plantilla
task-template-none = sin plantilla
task-text = Misión
task-follow-up = Seguimiento en (minutos, opcional)
task-missing = Indique un destinatario y una misión.
task-new = Nueva misión
task-new-word = Misión
task-create = Crear misión
task-kind-label = Tipo
task-kind-other = Otra misión
task-kind-missing = Elija el tipo.
task-unit-later = Asignar más tarde
task-unit-only = Elegir unidad
task-unit-hint = Opcional para un paciente o un suceso: la unidad también puede asignarse más tarde.
task-place = Lugar
task-what = ¿Qué ocurre?
task-what-hint = En pocas palabras, sin datos médicos (van en la ficha del paciente).
task-received-label = Recibido
task-received-hint = Dejar vacío para ahora.
task-text-missing = Describa brevemente de qué se trata.
task-place-missing = Indique el lugar.
task-too-long = El lugar o la descripción es demasiado largo.
task-time-invalid = Indique una hora, p. ej. 14:05.
task-assign-to = Asignar a
task-assign = Asignar
task-unit-missing = Elija una unidad.
task-send-further = Enviar otra unidad
task-send-to = Enviar también
task-send = Enviar
# Opens a small map centred on the Auftrag's pin.
pin-show = Mostrar marcador
pin-show-label = Mostrar marcador: { $task }
my-tasks = Mis misiones
pin-title = Marcador en el mapa (opcional)
pin-hint = Busque una dirección o toque el mapa. Con el teclado: mueva el mapa con las flechas y luego «Colocar marcador en el centro del mapa».
pin-at-centre = Colocar marcador en el centro del mapa
pin-clear = Quitar marcador
pin-none = No hay marcador colocado.
pin-set = Colocar marcador
pin-move = Mover marcador
pin-save = Guardar marcador

## Map object cards (MAP-11).
map-details = Detalles
map-object-gone = Ya no está en el mapa.
card-assign = Asignar unidad
card-open-task = Abrir misión
card-current-task = Misión
card-edit = Editar
card-patients = Pacientes por categoría de triaje
card-open-uhs = Abrir PMA
qualifications-short = Cualificaciones
# "Anna Schmidt removed."
removed = { $what } quitado.
map-details-label = { map-details }: { $what }
card-edit-label = { card-edit }: { $unit }
# "Paramedic: 2"
qualification-count = { $label }: { $n }
crew-names = Nombres ({ $n })
pin-at = Marcador: { $at }
task-received = Recibido a las { $at }
task-taken = Paciente admitido por { $unit } ({ $at })
follow-up-later = Seguimiento en 10 minutos
due = Pendiente
tasks-title = { tasks-heading } ({ $n })
# "In progress: Patient: Stage north"
task-step-label = { $step }: { $task }
# One unit on a task, for accessible names: "RTW 2 – Patient: Stage north".
unit-in-task = { $unit } – { $task }
task-units = Unidades en la misión
# "Follow up at 10:42"
follow-up-at = Seguimiento a las { $at }
# "Due: follow up since 10:42"
follow-up-due = { due }: seguimiento desde las { $at }
# "10:12, ben@…"
task-given-by = Dada a las { $at } por { $by }

## Command structure (CMD-1…4, ADR-0031).
structure-title = Estructura de mando
structure-short = Estructura
structure-empty = Aún no hay estructura de mando.
position = Puesto
position-choose = Elegir puesto
position-create = Crear puesto
position-kind = Tipo
position-kind-choose = Elegir tipo
position-name = Nombre
position-parent = Depende de
position-top = nivel superior
position-missing = Indique un tipo y un nombre.
position-close = Disolver
no-position = sin asignar
no-leader = Dirección: sin cubrir
leader-label = Dirección
leader-set = Asignar o transferir la dirección
uhs-assign = Asignar PMA
leader = { leader-label }: { $name }
# What a member of a position is, under its name in the org chart.
member-uhs = PMA
# Shown, never enforced (ADR-0031).
span-of-control = { $n } subordinados directos: más de 5, atención al ámbito de control.
position-close-label = Disolver { $title }

## A position put on the map, or found there, from the tree (CMD-5).
position-map-set-label = Colocar en el mapa: { $title }
position-map-show-label = Mostrar en el mapa: { $title }
# The place panel's status once a position was chosen from the tree.
map-place-ready = { $label }: toque el lugar o colóquelo en el centro del mapa.
position-of = Posición de { $unit }

## Situation map (MAP-1, MAP-2).
map-title = Mapa de situación
map-short = Mapa
map-label = Mapa de situación. Las flechas mueven, más y menos hacen zoom.
map-loading = Cargando mapa …
map-hint-touch = Mover el mapa con dos dedos
map-hint-wheel = Zoom: Ctrl y rueda del ratón
map-hint-wheel-mac = Zoom: ⌘ y rueda del ratón
map-unavailable = El mapa no puede mostrarse en este dispositivo; la lista de abajo muestra todo lo que contiene.
map-from-device = Mapa base: guardado en este dispositivo (también sin red).
map-from-server = Mapa base: del servidor.
map-no-region = No se ha preparado un mapa base para este incidente: los objetos están sobre un fondo neutro.
map-objects = En el mapa
map-locate = A mi ubicación
map-locating = Buscando la ubicación …
map-located = El mapa muestra su ubicación.
map-locate-denied = La ubicación no está permitida. Permita la ubicación a ORDO en los ajustes del navegador.
my-position = Yo
my-position-show = Mostrar mi ubicación
my-position-shown = El mapa muestra su ubicación (punto «Yo») y el marcador. Se queda en este dispositivo.
map-locate-failed = No se pudo determinar la ubicación.
map-objects-none = Aún no hay nada en el mapa.
map-show = Mostrar
map-remove = Quitar
map-place-hint = Toque el lugar, o mueva el mapa para que el lugar quede bajo la cruz (también con las flechas), y colóquelo en el centro del mapa.
map-place-kind = Qué
map-place-which = Cuál
map-place-choose = Elija
map-place-label = Etiqueta
map-place-submit = Colocar en el centro del mapa
map-place-missing = Elija qué colocar.
incident-location = Lugar del incidente
map-saving = Guardando el mapa base en este dispositivo para usarlo sin red …
map-save-failed = No se pudo guardar el mapa.
map-kind-symbol = Símbolo táctico
map-symbol-label = Etiqueta (opcional)
map-show-label = Mostrar { $label } en el mapa
map-remove-label = Quitar { $label } del mapa

## The map editor (ADR-0064)
map-edit = Editar mapa de situación
map-edit-hint = Coloque en el mapa el lugar del incidente, las unidades, los PMA y los puestos, dibuje zonas y líneas. El mando ve el mapa en Mando – Mapa de situación.
map-edit-not-allowed = El mando del incidente edita el mapa; aquí solo puede consultarse.
map-edit-object = Editar
map-tools = Herramienta
tool-select = Seleccionar
tool-place = Colocar símbolo
tool-area = Zona
tool-line = Línea
map-tap-to-choose = Toque un objeto en el mapa, o edítelo en la lista de abajo.
map-move-hint = Toque el nuevo lugar, o mueva el mapa para que quede bajo la cruz (también con las flechas), y colóquelo en el centro del mapa.
map-move-centre = Colocar en el centro del mapa
map-deselect = Anular selección
reshape-hint = Arrastre las esquinas para cambiar la forma; arrastre una esquina nueva desde el centro de un lado; arrastre la forma para moverla. Sin arrastrar: «Volver a dibujar».
reshape-apply = Aplicar
reshape-discard = Descartar
reshape-redraw = Volver a dibujar
map-reshape-failed = Esta forma no puede arrastrarse aquí; «Volver a dibujar» funciona siempre.
map-edit-label = Editar { $label }
map-chosen = { $label } seleccionado.
map-moved = { $label } movido.
map-placed = { $label } colocado.
map-reshaped = { $label } cambiado.
map-drawn = { $label } guardado.
map-removed = { $label } quitado.
draw-redraw = Volver a dibujar { $label }
log-hint = Cada cambio de este incidente que conoce este dispositivo, con hora, persona y dispositivo, los más recientes primero. Las entradas nunca se cambian: una corrección es una entrada nueva que remite a la anterior.
log-empty = Aún no hay entradas.
log-pending = aún sin enviar
log-by = { $user } · dispositivo { $device }
log-corrects = (corrige la entrada de las { $at })
log-pause-since = Pausa desde las { $at }
log-address = Dirección
log-task-given = Misión para { $to }: { $task }
log-position-closed = { $title } disuelto
log-leader = { $title }: dirección { $leader }
log-map-placed = { $label } colocado en el mapa de situación
log-map-removed = { $label } quitado del mapa de situación

## Map region from the server (MAP-8)
map-region-title = Mapa base
map-region-short = Mapa base
map-region-hint = El mapa base de este incidente para el mapa de situación: un radio del mapa mundial libre (OpenStreetMap a través de Protomaps). Solo lo reciben los dispositivos de los intervinientes de este incidente. Los dispositivos que guardaron un mapa sin conexión lo guardan de nuevo después.
map-region-none = Aún no se ha creado un mapa base para este incidente.
map-region-new = Nuevo mapa base
map-region-lat = Latitud (centro)
map-region-lon = Longitud (centro)
map-region-radius = Radio (km, de 1 a 50)
map-region-from-incident = El centro es el lugar del incidente.
map-region-coordinates = Centro en grados decimales, p. ej. 48.37 y 10.90.
map-region-cut = Crear mapa base
map-region-progress = Progreso
map-region-invalid = Indique una latitud (−85 a 85), una longitud (−180 a 180) y un radio de 1 a 50 km.
draw-category = Tipo
draw-add-corner = Colocar esquina en el centro del mapa
draw-undo = Quitar la última esquina
draw-cancel = Cancelar
draw-finish = Hecho
draw-too-few-area = Una zona necesita al menos tres esquinas.
draw-too-few-line = Una línea necesita al menos dos puntos.

## Helper register (HELP-1, HELP-2, ADR-0071)
helpers-title = Intervinientes
helpers-intro = Datos personales de los intervinientes de este incidente. Solo el mando del incidente los ve.
helpers-intro-roles = Quién tiene qué rol en este incidente. Solo el mando del incidente ve los datos personales.
helpers-empty = Aún no hay nadie registrado.
helper-add-title = Añadir persona
helper-add = Añadir
role-add-title = Dar rol
helper-new-word = Persona
role-new-word = Rol
helper-complete = Completar datos
helper-complete-label = { helper-complete }: { $name }
helper-complete-title = Completar los datos de { $name }
helper-name = Nombre
helper-phone = Teléfono
helper-phone-hint = Con prefijo, p. ej. 0171 1234567 o +43 664 1234567.
helper-phone-invalid = El número de teléfono no es un número válido con prefijo, p. ej. 0171 1234567.
helper-street = Calle y número
helper-postcode = Código postal
helper-postcode-invalid = El código postal no tiene la forma del país, p. ej. 55116.
helper-town = Localidad
helper-role = Rol en el incidente
helper-roles = Roles en el incidente
or = o
guest-duty-ended = El fin del turno ya ha pasado. Para un acceso sin cuenta, inicie de nuevo el turno o cambie su fin.
guest-until-closed = Acceso de invitado hasta el final del incidente
guest-no-command = El mando del incidente necesita una cuenta de ORDO.
guest-access = Acceso por código QR
guest-code-title = Código QR para el acceso
guest-code-refused = No se pudo crear el acceso.
guest-code-alt = Código QR que vincula el dispositivo de este interviniente con el incidente
guest-link = O abra este enlace en el dispositivo:
guest-code-close = Hecho
guest-code-for = Para { $name }: escanear con la cámara de cada dispositivo que use { $name }. Válido hasta { $expires }.
guest-code-until-closed = Para { $name }: escanear con la cámara de cada dispositivo que use { $name }. Válido hasta el final del incidente.
guest-access-label = { guest-access }: { $name }
guest-until = Acceso de invitado hasta { $until }
role-missing = Elija un rol.
helper-account-hint = Elija la cuenta de la organización; se toma su nombre. Dejar vacío para intervinientes sin cuenta.
diet-other = otra …
diet-other-label = Otra dieta
helper-birth-date-label = Fecha de nacimiento
helper-diet = Dieta
diet-none = no indicada
helper-duty-end = Fin del turno
helper-account-label = Cuenta de ORDO
helper-name-missing = Indique un nombre.
helper-too-long = Cada dato puede tener como máximo 200 caracteres.
helper-birth-date = Indique la fecha de nacimiento como fecha.
helper-account = La cuenta de ORDO no es un identificador válido, p. ej. anna@ordo.example.org.
helper-time = Indique el inicio y el fin del turno como fecha con hora.
helper-not-saved = No se pudieron guardar los datos.
duty-start = Iniciar turno
duty-end = Terminar turno
# Starts with the button's visible text, so its name matches what is seen.
duty-start-label = { duty-start }: { $name }
# Starts with the button's visible text, so its name matches what is seen.
duty-end-label = { duty-end }: { $name }
helper-edit-title = Editar { $name }
duty-pause = Pausar turno
duty-resume = Reanudar turno
duty-pause-label = { duty-pause }: { $name }
duty-resume-label = { duty-resume }: { $name }
# "on a break since 14:02"
paused-since = en pausa desde las { $at }
on-duty-since = de servicio desde las { $at }
duty-from-to = Turno { $from } – { $to }
duty-from = Turno desde { $from }
duty-to = Turno hasta { $to }
# The person typed for a crew is not in the helper register (HELP-3).
not-registered = { $name } no está en el registro de intervinientes. ¿Registrar con el nombre y añadir?
register-and-add = Registrar y añadir
# Starts with the button's visible text, so its name matches what is seen.
register-and-add-label = { register-and-add }: { $name }
# Registration cards (HELP-5, ADR-0072).
# "Photograph Meldekarte", with the profile's label of the card ("card" for several).
card-generic = tarjeta
card-photo = Fotografiar { $label }
card-reading = Leyendo la tarjeta …
card-not-recognised = No se ha reconocido ninguna tarjeta. Fotografíela de nuevo: tarjeta plana, código QR completo en la imagen y nítido.
card-read = { $card } de { $name } leída. Compruebe y registre.

## Converted by hand: counts, lists and values of ORDO's types
log-title = Diario de operaciones
task-follow-up-number = Indique los minutos como número entero.
log-pause-ended = Pausa terminada
draw-hint = Toque cada esquina, o mueva el mapa para que la esquina quede bajo la cruz (también con las flechas), y colóquela. Luego «Hecho».
helper-duty-start = Inicio del turno
# "no open tasks", "1 open task", "3 open tasks"
open-tasks-of = { $n ->
    [0] ninguna misión abierta
    [one] 1 misión abierta
   *[other] { $n } misiones abiertas
}
# People on more than one crew or staff list (UNIT-8), e.g. "Ben (RTW 1, care unit south)".
double-booked-people = Asignados más de una vez, quizá contados dos veces: { $people }
# The person being added is on other crews or staff lists already (UNIT-8).
assigned-elsewhere-to = { $person } ya está asignado a { $posts }. ¿Mover, para que nadie cuente dos veces?
# Variables: $unit, $overwritten, $kept
lost-update-of = Atención { $unit }: el estado { $overwritten } fue sustituido al mismo tiempo por { $kept }. Compruébelo.
action-status-of = Estado { $status }
follow-ups-due = { $n ->
    [one] 1 seguimiento pendiente
   *[other] { $n } seguimientos pendientes
}
unreported-strength = { $n ->
    [0] Todas las unidades han dado parte de sus efectivos.
    [one] 1 unidad sin parte de efectivos.
   *[other] { $n } unidades sin parte de efectivos.
}
transports-open = { $n ->
    [0] Transportes: ninguno abierto
   *[other] Transportes: { $n } abiertos
}
summary-map = { $objects ->
    [0] Aún no hay nada en el mapa.
    [one] 1 objeto en el mapa.
   *[other] { $objects } objetos en el mapa.
}
# The state as people read it, with a pause: "Running – paused".
incident-state-paused = { $state } – { incident-paused-label }
# "13.11.2026, 18:00 to 15.11.2026, 23:00"
incident-from-to = { $starts } a { $ends }
incident-from = desde { $starts }
incident-to = hasta { $ends }
# Variables: $user, $role
withdraw-role = Retirar: { $role } a { $user }
# The strength read out: "Strength: 0 leaders, 1 sub-leader, 8 crew, 9 in total".
strength-count = { $count } { $part }
strength-read = { strength }: { $parts }, { $total } en total
strength-special = { $strength }, { $special }
strength-platoon-leaders = Jefes de sección
strength-forces = Intervinientes
# A strength's legend, e.g. "Strength (leaders / sub-leaders / crew)".
strength-legend-of = { $title } ({ $parts })
units-count = { $n ->
    [one] 1 unidad
   *[other] { $n } unidades
}
# Variables: $kind, $received, $place, $text
log-report-line = Aviso recibido: { $kind }, { $received } – { $place } – { $text }
# "Role Physician withdrawn from Anna Schmidt."
role-removed-from = Rol { $role } retirado a { $who }.
# Variables: $action, $task
pin-edit-of = { $action }: { $task }
task-kind-event = Suceso
task-kind-patient = Paciente
# "Patient: Stage north"
task-label-of = { $kind }: { $place }
# How far a task is (TASK-2).
task-phase-received = Recibida
task-phase-assigned = Asignada
task-phase-in-progress = En curso
task-phase-done = Hecha
task-phase-cancelled = Cancelada
# Variables: $kind, $received, $place, $task
log-task-received-line = Misión recibida: { $kind }, { $received } – { $place } – { $task }
log-task-received-to = { $line } – para { $to }
log-task-assigned = Misión «{ $task }» asignada a { $unit }
task-step-open = Abierta
task-step-in-progress = En curso
task-step-done = Hecha
task-step-cancel = Cancelar
# Time left until a follow-up, e.g. "7:05 min left"; "now" at zero.
countdown-now = ahora
# Variables: $minutes, $seconds
countdown-left = quedan { $minutes }:{ $seconds } min
# "Handover: Dr M. → Dr K."
handover-from-to = Relevo: { $from } → { $to }
map-kind-uhs = PMA
map-kind-drawing = Dibujo
map-kind-report = Aviso
map-saved-size = Mapa base guardado en este dispositivo ({ $megabytes } MB), utilizable sin red.
# "With units: 2 (SK I 1 · SK II 1)"
with-units-line = { $label }: { $total } ({ $parts })
# Variables: $module, $event-type
log-unknown = { $module }: { $event-type }
log-planned = Incidente «{ $name }» creado
# Variables: $added, $unit-type, $organisation
log-unit-added-line = { $added } ({ $unit-type }, { $organisation })
log-helper-registered = { $name } añadido
log-helper-saved = Datos de { $name } guardados
log-helper-changes = { $name }: { $changes }
log-cleared = { $label } borrado
log-fields-changed-list = Datos cambiados: { $fields }
# Variables: $unit, $n
log-crew = { $n ->
    [0] Dotación de { $unit }: nadie
    [one] Dotación de { $unit }: 1 persona
   *[other] Dotación de { $unit }: { $n } personas
}
# A unit's step on a task (ADR-0085): "Task “…”: RTW 2 In progress".
log-unit-step-of = Misión «{ $task }»: { $unit } { $status }
log-task-updated-to = Misión «{ $task }»: { $status }
log-task-follow-up-changed = Misión «{ $task }»: seguimiento cambiado
log-assigned-to = { $what } depende de: { $position }
log-assigned-none = { $what }: ya no depende de un puesto
map-region-preparing = Preparando el mapa base …
map-region-tiles = Creando el mapa base: { $done } de { $total } teselas …
map-region-ready-size = Mapa base listo ({ $megabytes } MB).
# Variables: $km, $lat, $lon
map-region-area = Radio de { $km } km alrededor de { $lat }, { $lon }.
map-region-created = Creado el { $at }.
map-region-failed = No se pudo crear el mapa base: { $error }
draw-corners = { $n ->
    [0] Aún no hay ninguna esquina.
    [one] 1 esquina colocada.
   *[other] { $n } esquinas colocadas.
}
# The diets of the helpers still expected or on duty, for catering.
diet-counts-line = Dieta (sin turnos terminados): { $list }
diet-count = { $n } × { $diet }
# Starts with the button's visible text, so its name matches what is seen.
helper-edit-of = { $edit }: { $name }

## Führung – Lage (ADR-0099): the map beside the units and the treatment places.
lage = Situación
# The groups the command sees its units in (UNIT-10); the profile gives each status its group.
group-available = Disponible
group-task = En misión
group-transport = En traslado
group-unavailable = No disponible
group-none = Sin estado
# The first of the type filter's chips: "Alle 14".
filter-all = Todas
units-filter = Tipo
units-only-available = Solo disponibles
units-glance-empty = Ninguna unidad coincide con la selección.
open-lage = Ir a la situación
# "4 von 14 einsatzbereit", the first number large.
units-available-of = de { $total } disponibles
col-type = Tipo
col-free = libres
col-all = total

## Struktur: actions at each position (ADR-0102), forms in a dialog.
position-new = Función
lead-word = Mando
sub-position-word = Subfunción
uhs-word = PMA
map-set-word = Situar
leader-set-label = Asignar el mando: { $title }
position-sub-label = Añadir una subfunción: { $title }
uhs-assign-label = Asignar un PMA: { $title }
# The "+" button that opens the form to add one (ADR-0102).
unit-new = Unidad
# The incident's details, read compactly; ✎ opens the form (ADR-0102).
incident-details-edit = Editar los datos
not-set = sin indicar
strength-detailed-short = Efectivo detallado
yes = sí
no = no

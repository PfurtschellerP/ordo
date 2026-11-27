app-name = ORDO
skip-link = Ir al contenido
pair-title = Vincular dispositivo
pair-intro = Inicie sesión con la cuenta de su organización. Este dispositivo recibe una clave propia que nunca sale de él; ORDO no guarda contraseñas.
pair-login = Iniciar sesión
pair-qr = Vincular con código QR
pair-qr-hint = Para tabletas compartidas: escanee el código QR con un teléfono con sesión iniciada y confirme.
pair-qr-alt = Código QR para vincular este dispositivo
pair-code = Código: { $code }
pair-qr-link = Sin cámara: abrir en otro dispositivo e introducir el código:
pair-waiting = Esperando confirmación …
pair-no-login = No hay inicio de sesión configurado en este servidor.
pair-manual = Desarrollo: vincular el dispositivo a mano
pair-manual-intro = Para instalaciones de prueba sin inicio de sesión: valores del archivo semilla del servidor y de `ordo generate-device-key`.
login-failed = Error al iniciar sesión
relogin-title = Sesión caducada
relogin-intro = La sesión de este dispositivo ha caducado. Inicie sesión de nuevo: los datos del dispositivo se conservan y se sincronizan después.
network-error = Servidor no accesible
pair-user = Usuario (nombre@servidor)
pair-device = ID del dispositivo
pair-key = Clave privada del dispositivo
pair-incident = ID del incidente
pair-submit = Vincular
unpair = Desvincular dispositivo
signed-in-as = Sesión iniciada como
avatar-change = Cambiar foto de perfil
avatar-remove = Quitar foto de perfil
retry = Reintentar
starting = ORDO se está iniciando …
online = Conectado
connection-label = Conexión: { $state }. { $pending }
connecting = Conectando …
offline = Sin conexión: los registros se guardan y se envían más tarde
my-devices = Mis dispositivos
page-tabs = Partes de la página
revoked-title = Dispositivo bloqueado
revoked-text = Este dispositivo ha sido bloqueado. Se han borrado de él su clave y todos los datos del incidente. Para volver a usarlo, vincúlelo de nuevo.
pair-again = Vincular de nuevo
device-name-label = Nombre de este dispositivo
device-name-hint = Cómo aparece el dispositivo en la lista de dispositivos, p. ej. «Tableta PMA norte».
device-revoked = Dispositivo …{ $short } bloqueado
retagged = Paciente { $old }: nuevo número de tarjeta { $new }
uhs-created = { $name } creado
unit-added = Unidad { $unit } añadida
launcher = Cambiar de espacio de trabajo
views = Vistas
incident = Incidente
reason-no-access = Sin acceso en este incidente
reason-not-connected = Disponible tras la primera conexión
no-workspace = Todavía no se ha habilitado ningún espacio de trabajo para este dispositivo en este incidente.

## Navigation and header
menu = Menú
search = Buscar
storage-failed = Almacenamiento del dispositivo lleno o bloqueado: los registros solo se conservan hasta enviarse. No recargue.

## Appearance (UX-3)
appearance = Apariencia
profile = Perfil y dispositivo
hand = Manejo con

## Vim keys (ADR-0090). Key names stay English (ADR-0036).
keymap = Teclado
keys-title = Atajos de teclado
keys-on = Las teclas Vim están activadas en este dispositivo. Funcionan mientras ningún campo de texto tenga el foco y nunca cambian datos.
keys-off = Las teclas Vim están desactivadas. Actívelas en el menú, en «Teclado».
keys-vim = Teclas Vim
keys-turn-on = Activar teclas Vim
keys-turn-off = Desactivar teclas Vim
keys-always = Siempre
key-keys = Teclas
key-does = Efecto
key-help = Esta vista general
key-palette = Paleta de comandos
key-search = Buscar o filtrar
key-move = Al siguiente elemento a la izquierda, abajo, arriba, a la derecha, en toda la página
key-first-last = Primer / último elemento de la página
key-tabs = Pestaña siguiente / anterior
key-views = Primera a cuarta vista
key-menu = Menú
key-rail = Vistas en el borde: j/k para elegir, o abre, l o Esc para volver
key-open = Abrir entrada
key-insert = Al primer campo de entrada
key-scroll = Media página abajo / arriba
key-escape = Salir del campo de texto
key-in-list = En una lista: entrada siguiente / anterior
key-close = Cerrar diálogo o menú
theme-set = Apariencia: { $label }
photo-added = Foto guardada para el paciente { $tag }
photo-removed = Foto quitada para el paciente { $tag }
photo-restored = Foto del paciente { $tag } restaurada
photo-failed = No se pudo leer la imagen.
photo-too-large = La imagen es demasiado grande, incluso reducida.
edit = Editar
save = Guardar
cancel = Cancelar

## Command palette (UX-10)
palette-title = Búsqueda y comandos
palette-placeholder = Paciente, unidad, vista o comando …
palette-results = Resultados
palette-none = No se ha encontrado nada.
palette-keys = elegir
palette-run = ejecutar
palette-close = cerrar
palette-back = atrás
group-views = Vista
group-actions = Comando
my-roles = Roles
no-role = ninguno (planificación de incidentes)
change-incident = Cambiar de incidente
incident-details-saved = Datos del incidente guardados.
incident-times-invalid = El final es anterior al inicio, o una fecha está incompleta.
place-label = Puesto { $n }
places-set = { $n } puestos de tratamiento
transport-history = Historial
transport-requested = { $tag }: transporte solicitado
order-given = { $tag }: orden registrada
order-confirmed-said = Orden confirmada
order-recorded = { $tag }: orden verbal registrada
section-close = Cerrar
# Closes a dialog.
close = Cerrar
done = Hecho
discharge = Alta
readmitted = { $tag } readmitido
hospital-saved = Hospital guardado: { $name }
capacity-saved = Admisión guardada: { $name }

## API versions (ADR-0052).
version-unsupported = Esta aplicación y el servidor no tienen ninguna versión en común. Recargue la página.
crew-set = Dotación de { $unit } guardada
unit-released = { $unit } liberada del incidente
# The patient counts of patients with units' crews.
with-units-label = Con unidades
handed-over = Paciente { $tag } transferido a { $to }, recepción pendiente
handed-over-quick = Transferencia rápida iniciada para el paciente { $tag }
handed-outside = Paciente { $tag } transferido fuera de ORDO a { $receiver }
taken = Paciente { $tag } recibido por { $by }
uhs-closed = { $name } cerrado
uhs-updated = { $name } guardado

## What needs the command's attention (TASK-3): the bell in the header.
attention = Avisos
attention-none = Nada requiere atención en este momento.

## Reports as Aufträge (REP-1…5, ADR-0081); events of ADR-0075 still read.
report-closed = Aviso cerrado
task-given-label = Misión dada
task-patient-taken = Paciente admitido desde la misión
undo = Deshacer
close-label = { section-close }: { $what }
task-recorded = Misión creada: { $place }
follow-up-later-done = Seguimiento aplazado
task-given = Misión dada a { $unit }
structure-saved = Estructura de mando guardada
position-created = { $name } creado
map-saved-label = Mapa de situación guardado

## Address search (ADR-0066)
address-label = Buscar dirección
address-submit = Buscar
address-searching = Buscando …
address-empty = Introduzca una dirección.
address-too-long = Introduzca como máximo 200 caracteres.
address-offline = La búsqueda de direcciones necesita conexión con el servidor.
address-not-found = No se ha encontrado nada. Escríbala de otra forma, por ejemplo con la localidad.
address-off = No hay búsqueda de direcciones configurada en este servidor.
address-refused = La búsqueda de direcciones es para el mando del incidente.
address-failed = La búsqueda de direcciones no responde ahora. Inténtelo más tarde.
address-places = Lugares encontrados
address-attribution = Direcciones de OpenStreetMap, © colaboradores de OpenStreetMap (ODbL).
address-chosen = Elegido: { $label }
notices-back = Atrás

## Stored data that cannot be used (local persistence)
stored-state-unusable = Esta versión no puede leer los datos de este dispositivo ({ $reason }). Se conservan sin cambios; recargue la aplicación o avise a la administración.

## Display names (ORG-7)
display-name = Nombre visible
display-name-hint = Cómo le ven los demás en ORDO. Sin nombre propio, se usa el nombre del inicio de sesión.
display-name-save = Guardar nombre
display-name-reset = Usar el nombre del inicio de sesión
guest-pair-title = Acceso para intervinientes
guest-ended-title = Acceso finalizado
guest-ended = Su acceso a este incidente terminó con su turno. Para un nuevo acceso, diríjase al mando del incidente.
guest-pairing = Vinculando este dispositivo con el código QR …
guest-code-invalid = El código QR ya no es válido (ya usado, caducado o turno terminado). Pida un código nuevo al mando del incidente.
helper-saved = Datos de { $name } guardados.

## Counts
pending = { $n ->
    [0] Todo enviado
    [one] 1 registro aún sin enviar
   *[other] { $n } registros aún sin enviar
}
palette-count = { $n ->
    [0] { palette-none }
    [one] 1 resultado
   *[other] { $n } resultados
}
attention-label = { $n ->
    [0] { attention }: ninguno
   *[other] { attention }: { $n }
}
address-found = { $n ->
    [one] 1 lugar encontrado. Elija uno.
   *[other] { $n } lugares encontrados. Elija uno.
}

## How long ago, from minutes: "just now", "5 min ago", "2 h 5 min ago", "3 days ago"
ago-now = ahora mismo
ago-minutes = hace { $minutes } min
ago-hours = hace { $hours } h
# Variables: $hours, $minutes
ago-hours-minutes = hace { $hours } h { $minutes } min
ago-days = { $days ->
    [one] hace 1 día
   *[other] hace { $days } días
}

## Triage categories in the DE profile: the codes on the triage tags stay.
triage-immediate = SK I
triage-urgent = SK II
triage-minor = SK III
triage-expectant = SK IV
triage-deceased = Fallecido
triage-uninjured = Ileso
# A place kept for a triage category, e.g. "SK I – Place 2" (UHS-13).
category-place = { $category } – puesto { $n }

## Log lines with values of ORDO's types
patient-registered = Paciente { $tag } registrado: { $category }, { $uhs }
patient-retriaged = Paciente { $tag } retriado: { $category }
unit-status-set = { $unit }: estado { $status }
# A unit status with its code, e.g. "3 – Task accepted".
status-with-code = { $code } – { $label }
# Variables: $user, $role
role-granted-to = Rol { $role } dado a { $user }
# Variables: $user, $role
role-withdrawn-from = Rol { $role } retirado a { $user }
incident-state-changed-to = Incidente: { $state }
priority-set = { $tag }: prioridad de transporte fijada
priority-removed = { $tag }: prioridad de transporte retirada
place-assigned-to = { $tag } en { $place }
place-none = { $tag }: sin puesto de tratamiento
transport-status-set-to = Transporte: { $status }
order-status-set-to = Orden: { $status }
task-status-set-to = Misión: { $status }
# "Discharge: Transported"
discharged-as-outcome = { discharge }: { $outcome }
discharged-with = { $tag }: { $outcome }

## Themes, hands and keyboards (UX-3, ADR-0049, ADR-0090)
theme-auto = Automático
theme-day = Día
theme-night = Noche
theme-high-contrast = Alto contraste
hand-right = mano derecha
hand-left = mano izquierda
keymap-standard = Estándar
keymap-vim = Teclas Vim

## Roles offered when granting (prototype; later from the profile)
role-commander = Mando del incidente
role-medic = Servicio sanitario
role-physician = Médico
role-paramedic = Técnico en emergencias

## States
incident-now-paused = Incidente en pausa
incident-now-resumed = Incidente reanudado
incident-planned = Planificado
incident-active = En curso
incident-closed = Cerrado
transport-status-requested = Solicitado
transport-status-assigned = Asignado
transport-status-loading = Cargando
transport-status-departed = En camino
transport-status-arrived = En destino
transport-status-handed-over = Paciente transferido en destino
transport-status-cancelled = Cancelado
order-status-given = Dada
order-status-acknowledged = Recibida
order-status-done = Realizada
order-status-withdrawn = Retirada
outcome-released-on-site = Alta en el lugar
outcome-transported = Trasladado
outcome-refused = Rechazo de asistencia o traslado
outcome-deceased = Fallecido
task-status-open = Abierta
task-status-in-progress = En curso
task-status-done = Hecha
task-status-cancelled = Cancelada

## Dates and numbers, as each language writes them
# Day and month with two digits. Variables: $day, $month, $year
date-of = { $day }/{ $month }/{ $year }
# Variables: $date, $time
date-time-of = { $date }, { $time }
# The decimal separator, as in "1.5 MB".
decimal-separator = ,

## Errors of the device's own workings (pairing, storage, randomness)
no-random = No hay números aleatorios disponibles
no-incident-chosen = Ningún incidente elegido
device-id-invalid = ID del dispositivo: { $error }
private-key-format = Clave privada: se esperan 64 caracteres hexadecimales
photo-not-found = Foto no encontrada
photo-other-version = Foto en otra versión
app-state-unavailable = Estado de la aplicación no disponible
no-secure-context = Almacenamiento seguro no disponible: la aplicación debe abrirse por HTTPS.
no-device-storage = Sin almacenamiento en el dispositivo
# A device's suggested name from its browser and system (ORG-8): "Firefox on Android".
device-name-on = { $browser } en { $system }

## Language of this device (I18N-1)
# Each language names itself in its own file.
language-name = Español
language = Idioma
language-auto = Automático (como el navegador)
language-after-reload = El idioma se aplica al recargar la próxima vez. Recargue solo cuando todo esté enviado.

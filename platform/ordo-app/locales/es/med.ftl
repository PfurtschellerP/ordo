crew-places = Puestos de tratamiento libres
crew-not-held = Este paciente no está (o ya no está) con su unidad. Quien atiende ahora al paciente ve la ficha.
# The UHS as a unit (UHS-12).
uhs-strength = Efectivos del PMA
uhs-strength-hint = El PMA cuenta como una unidad entre los efectivos en el lugar: con estos efectivos, si no con su personal.
register-title = Registrar paciente
register-tag = Número de la tarjeta de triaje
# A card number already registered (PAT-12): where the patient is, and since when.
tag-in-use = La tarjeta { $tag } ya está registrada: { $holder }, desde las { $time }.
# A card number registered again (PAT-12): by whom, where and when.
# Variables: $person, $holder, $time
tag-registered-again = Registrada de nuevo a las { $time } por { $person } ({ $holder }).
tag-twice = Tarjeta registrada dos veces
tag-twice-hint = Dos dispositivos registraron este número de tarjeta mientras estaban sin conexión, o se tecleó mal un número. Quien registró más tarde da a su paciente una tarjeta nueva e introduce su número; lo que este dispositivo documentó desde entonces va con ella.
retag-ask = Su registro llegó más tarde. Dé a su paciente una nueva tarjeta de triaje.
retag-number = Número de la nueva tarjeta
retag-submit = Cambiar número
retag-in-use = La tarjeta { $tag } ya está registrada. Tome una tarjeta sin usar.
register-category = Categoría de triaje
register-uhs = Paciente en
register-submit = Registrar
board-empty = Aún no hay pacientes registrados.
col-retriage = Retriaje
not-sent = aún sin enviar
sent-label = enviado
uhs-empty = Aún no hay ningún PMA. El mando del incidente los crea en Preparación – Unidades.
uhs-name = Nombre del PMA
uhs-capacity = Capacidad (pacientes)
uhs-submit = Crear PMA
uhs-new = Nuevo PMA
no-uhs = Aún no hay ningún PMA en el que pueda documentar. El mando del incidente crea los PMA y asigna su personal.

## Patient list and record
# The workspace of one UHS in the launcher.
ws-uhs = PMA
patients-title = Pacientes
all-patients = Todos los pacientes
registered-word = registrado.
choose-patient = Elija un paciente en la lista para abrir la ficha.
unknown-person = Persona desconocida
patient-title = Paciente { $tag }
not-found = Este paciente no existe en este incidente.
sections = Secciones
triage-title = Triaje
triage-history = Historial
retriage-to = Cambiar triaje
# Variables: $user, $time
by-at = { $time }, { $user }
photo-title = Foto
photo-none = Aún no hay foto.
photo-take = Hacer foto
photo-upload = Subir imagen
photo-busy = Reduciendo la foto …
photo-preview-loading = Vista previa: cargando la foto completa …
photo-preview-unavailable = Vista previa: la foto completa aún no está en el servidor. Se volverá a cargar más tarde.
photo-purpose = Solo para la identificación. La foto se reduce, se guarda sin datos de cámara ni de ubicación y solo se envía a quien puede verla.
# Under the photo: "Taken 14:31 by Anna Schmidt".
photo-taken = Hecha a las { $at } por { $by }
photo-alt = Foto del paciente { $tag }
photo-remove-short = Quitar
photo-remove-label = Quitar foto – paciente { $tag }
new-entry = Nuevo registro
correct = Corregir
nothing-documented = Aún no hay nada documentado.
no-answer = no indicado
# The free-text answer of an open choice (EPCR-12).
other-answer = Otro
now = Ahora
empty-entry = Introduzca al menos un valor.
no-changes = Sin cambios.
check-input = Compruebe los campos marcados.
corrected = corregido
entry-saved = { $form } guardado para el paciente { $tag }
new-entry-label = { new-entry }: { $form }
group-patients = Paciente
action-register = Registrar paciente
action-create-uhs = Crear PMA
action-open-record = Abrir ficha
action-take-photo = Hacer foto
action-entry = Registrar { $form }

## Transport priority
transport-priority = Prioridad de transporte
remove-priority = Sin prioridad de transporte
set-priority = Fijar { $label }
set-priority-label = { $tag }: fijar { $label }
remove-priority-label = { $tag }: sin prioridad de transporte
place = Puesto de tratamiento
no-place = Sin puesto

## The plan of a UHS's places (UHS-14)
place-free = libre
place-fill-label = Ocupar
place-none-title = Sin puesto
register-place-hint = Sugerencia: el primer puesto libre de la categoría de triaje elegida.
# "Fill place 2": the button of a free place.
place-fill = Ocupar { $label }

## Capacity per triage category (UHS-13)
uhs-capacities-legend = Capacidad por categoría de triaje (opcional)
uhs-capacities-hint = Puestos de tratamiento reservados para una categoría de triaje; en total como máximo la capacidad. Los demás puestos no tienen categoría.
# The places per category do not fit in the capacity.
uhs-places-exceed = Los puestos por categoría ({ $kept }) superan la capacidad ({ $total }). Aumente la capacidad o reserve menos puestos.
place-taken = { $label } (ocupado: { $by })
places-free = { $free } de { $total } libres

## Free treatment places per category (UHS-8, UHS-16, ADR-0086)
places-all = Todos los PMA
places-any = sin categoría
places-full = completo
places-none = No hay puestos de tratamiento creados.
places-legend = Una casilla por puesto: llena es ocupado, discontinua es libre. Las categorías completas están rayadas y marcadas «completo».
# "free of 15", after the large number of free places.
free-of = libres de { $total }
# "2 without a place": patients waiting for a place.
without-place = { $n } sin puesto
transports-title = Transportes abiertos
transport-title = Transporte
transport-type = Tipo de transporte
transport-priority-of-request = Urgencia
transport-needs = Necesidades especiales
transport-request = Solicitar transporte
transport-choose = Elija un tipo de transporte y una urgencia.
transport-none = No se ha solicitado transporte.
transport-unit-label = Unidad
transport-destination = Destino (p. ej. hospital)
transport-assign = Asignar

## Transports waiting for a unit, in the bell and on the overview (TASK-3).
transport-waiting = Transporte sin unidad: { $summary } · { $priority }
transports-title-short = Transportes
open-transports = A los transportes
transports-waiting = sin unidad
no-transports-waiting = Ninguna solicitud de transporte en espera.
transports-under-way = { $n } en camino
transport-assign-missing = Elija una unidad; el destino puede indicarse después.
transport-destination-missing = Elija un destino.
# A transport sent before its destination was decided (ADR-0098).
destination-open = Destino pendiente
destination-set-label = Fijar destino
# A destination set before changes on the way, e.g. a hospital stops admitting.
destination-change = Cambiar destino
destination-set = Destino: { $destination }
transport-no-units = Aún no hay unidades en el incidente.
transport-choose-unit = Elegir unidad
# Accessible name of a step button: the step and which transport.
transport-step-label = { $step }: { $transport }
# Short description of a transport for lists and button names.
transport-summary = { $transport-type } para { $uhs }, { $at }
transport-unit = { $unit } → { $destination }

## Medical orders (ORDER-1…4)
orders = Órdenes
orders-title = Órdenes abiertas
orders-empty = No hay órdenes abiertas.
orders-none = No hay órdenes.
order-give = Dar orden
order-kind = Tipo
order-medication = Medicación
order-measure = Medida
order-substance = Principio activo
order-dose = Dosis con unidad
order-route = Vía
order-route-choose = Elegir vía
order-measure-text = Medida
order-note = Nota (opcional)
order-incomplete = Indique la orden completa.
order-records-only = ORDO documenta las órdenes; quien da la orden comprueba la dosis y la indicación.
# "Given 10:12 by petra@…"
order-by = { $at } por { $by }
order-performed-by-label = Realizada por
order-performed-by = { order-performed-by-label } { $person }
# "Done (by paul@…)", in the history: who carried it out, apart from who
# reported it.
order-status-performed = { $status } (por { $person })

## Spoken orders, recorded by the crew and confirmed by their giver (ORDER-6), and who

## ordered a documented medication (ORDER-7).
order-who = ¿Quién da la orden?
order-mine = Yo doy la orden
order-spoken-label = Ordenada verbalmente, por radio o por teléfono
order-ordered-by = Ordenada por
order-channel-label = Transmitida
order-read-back = Repetida y confirmada (circuito cerrado)
order-record = Registrar orden
order-spoken-hint = Quien dio la orden la confirma después en ORDO, si usa ORDO.
order-unconfirmed = Aún sin confirmar
order-confirm = Confirmar
ordered-by-from-order = Tomado de la orden. Registre primero las órdenes verbales en «Órdenes».
ordered-by-no-order = sin orden en ORDO
# "Recorded 10:12 by Anna Becker"
order-recorded-by = Registrada a las { $at } por { $by }
# "Confirmed 10:20 by Petra Schulz"
order-confirmed = Confirmada a las { $at } por { $by }
order-confirm-label = Confirmar orden: { $what }
log-order-confirmed = { $tag }: orden verbal confirmada
# Who ordered a documented medication, taken from its order: "Petra Schulz (order 10:12)"
ordered-by-order = { $giver } (orden { $at })

## Admission and discharge (UHS-3).
stay-title = Recorrido del paciente
stay-open = Dar de alta …
# The stay section on the tab *Übergabe*: admissions and discharges; moving is a handover.
discharge-title = Admisión y alta
triage-open = Cambiar triaje
orders-open = Nueva orden
discharge-outcome = Alta con resultado
discharge-choose = Elija el resultado del alta.
readmit = Readmitir
# "Discharged patients (2)"
discharged-list = Pacientes dados de alta ({ $n })
# "With the transport: RTW → City hospital"
discharge-transport = Con el transporte: { $line }

## Hospitals and the transport board (TRANS-2, TRANS-4).
destination = Destino
destination-choose = Elegir destino
destination-other-option = Otro destino (texto libre)
destination-other = Otro destino
free-units-title = Unidades de transporte libres
free-units-empty = No hay unidades de transporte libres.
hospitals-title = Hospitales
hospitals-empty = Aún no hay hospitales registrados.
# Where the list is kept, on the transport board while it is empty.
hospitals-where = Los hospitales se registran en Preparación – Unidades, pestaña «Hospitales».
hospital-name = Nombre del hospital
hospital-departments = Servicios
hospital-capabilities = Prestaciones
hospital-add = Añadir hospital
hospital-save = Guardar hospital
hospital-edit = Editar
hospital-name-missing = Indique el nombre del hospital.
capacity-legend = Admisión libre (pacientes)
capacity-save = Guardar admisión
capacity-none = Admisión: no comunicada
capacity-numbers = Indique la admisión con números enteros.
sent = Trasladados: { $n }
hospital-edit-label = Editar: { $name }
hospital-edit-title = Editar { $name }
capacity-save-label = { capacity-save }: { $name }

## Planned transports (TRANS-5).
transport-planned = Previsto para (vacío: de inmediato)
transport-planned-invalid = Indique la fecha y la hora completas.
planned-title = Transportes previstos
planned-empty = No hay transportes previstos.
planned-for = Previsto para { $at }

## Documenting a done order (ORDER-5).
document-order = Pasar a la documentación
document-order-label = { document-order }: { $what }
transports-count = Transportes: { $open } abiertos, { $planned } previstos

## Patient pathway (PAT-5).
# "Moved: Patient collection point north → Treatment area"
moved = Trasladado: { $from } → { $to }
patient-moved = { $tag } trasladado a { $to }

## Triage references (PAT-11, ADR-0045).
triage-with-label = Triaje según
triage-with-none = no indicado
triage-aid = Ayuda de triaje
triage-source = Abrir la fuente oficial (ventana nueva)
triage-aid-note = Quien hace el triaje decide la categoría; ORDO no sugiere ninguna ni comprueba ninguna.
# "by PRIOR"
triage-with = según { $name }

## UHS details (UHS-1).
uhs-details = Ubicación y personal
uhs-data = Datos
saved = Guardado.
uhs-name-needed = El PMA necesita un nombre.
uhs-location = Ubicación
uhs-on-map = En el mapa de situación
uhs-not-on-map = aún sin colocar
uhs-place-on-map = Colocar en el mapa
uhs-move-on-map = Mover en el mapa
# The link's visible text, then the unit: the name contains what is seen.
uhs-place-label = { $visible }: { $name }
uhs-staff = Personal
uhs-staff-none = No hay personal introducido.
uhs-resources-title = PMA y personal
uhs-staff-add = Añadir al personal
uhs-save = Guardar
uhs-capacity-number = Indique la capacidad como número entero.

## The record page (EPCR-14, EPCR-15).
record-tabs = Partes de la ficha
record-overview = Resumen
record-timeline = Historial
record-documentation = Documentación
record-documentation-short = Doc.
record-handover = Transferencia
record-at = En
record-triaged = Triaje
record-since = Desde

## The patient list's tabs (chapter 8.6.5, *Tabs*).
patients-tab = Pacientes
register-tab = Registrar
staff-request-tab = Solicitar personal
staff-request-short = Personal
quick-actions = Registro rápido
# The floating button that opens the quick actions on phones.
quick-capture = Registrar
all = Todos
all-kinds = Todo
latest = último
order = Orden
orders-open-title = Órdenes abiertas
orders-all = Todas las órdenes de la documentación
orders-none-open = No hay órdenes abiertas.

## Handover (ADR-0087).
holders-uhs = PMA
holders-units = Unidades
handover-to = Transferencia a { $to }
handover-quick-step = Transferencia rápida por código QR
handover-taken-by = Recibido: { $by }
handover-taken-back = Transferencia retirada: { $by }
handed-outside-step = Transferido a { $receiver } (fuera de ORDO)
handover-open-to = Transferencia a { $to } pendiente
handover-open-quick = Transferencia rápida pendiente
# "Still with SanTrp 1 · handed over 14:31 by Max Muster".
handover-open-since = Aún con { $holder } · transferido a las { $at } por { $by }
handover-open-hint = Hasta la recepción, el puesto anterior sigue documentando; el puesto que recibe confirma con «Recibir».
handover-open-quick-hint = El puesto que recibe escanea el código QR con ORDO o la cámara de su dispositivo y elige adónde va el paciente.
handover-note-short = Nota:
handover-take = Recibir para { $to }
handover-take-word = Recibir
# The take button's accessible name: its words and the patient.
handover-take-label = { $words }: paciente { $tag }
handover-take-to = Recibir para
take-place-hint = Sugerencia: el primer puesto libre de la categoría de triaje del paciente.
handover-take-back = Retirar la transferencia
handover-failed = No se pudo guardar la transferencia.
handover-way = El recorrido del paciente

## The handover document (ADR-0089).
document-title = Documento de transferencia
document-hint = PDF para el puesto que recibe. Todos los datos van también adjuntos en forma legible por máquina, como bundle HL7 FHIR (un adjunto del archivo PDF, en «Adjuntos» en los programas de PDF). Lo crea el servidor de ORDO; para ello el dispositivo necesita conexión.
document-button = Descargar PDF
document-label = Descargar PDF: documento de transferencia paciente { $tag }
# The file name of the saved document.
document-file = Transferencia-{ $tag }.pdf
document-asking = Creándose en el servidor …
document-saved = Descargado.
document-offline = Sin conexión con el servidor. El documento se crea allí; inténtelo de nuevo con conexión.
document-refused = No puede obtener el documento de transferencia de este paciente.
document-failed = No se pudo crear el documento. Inténtelo de nuevo.

## The way as a flow chart (dsn patient-flow).
flow-registered = Registrado a las { $time }
flow-taken = Recibido a las { $time }
flow-taken-back = Transferencia retirada a las { $time }
# "Transport: RTW 2 → City hospital · Requested"
flow-transport = Transporte: { $line } · { $status }
flow-handover = Transferencia
flow-handover-open = Transferencia pendiente
flow-moved = Trasladado
flow-outside = Fuera de ORDO
flow-quick-receiver = Quien escanee el código QR
flow-not-taken = Aún sin recibir
flow-next-edge = Próxima transferencia
handover-next = a quién transferir
handover-title = Transferir paciente
handover-to-label = Transferir a
handover-to-uhs = PMA o puesto de tratamiento
handover-to-uhs-hint = En el incidente. La ficha va con el paciente; el personal del PMA sigue documentando.
handover-to-unit = Unidad
handover-to-unit-hint = En el incidente, p. ej. una ambulancia. Su dotación sigue documentando.
handover-to-transport = Transporte
handover-to-transport-hint = Recibe la unidad de un transporte solicitado; el destino se deduce del transporte.
handover-to-outside = Fuera de ORDO
handover-to-outside-hint = P. ej. un hospital, otro servicio de ambulancias, la policía. El paciente sale del incidente.
handover-target-label = Qué puesto: { $kind }
handover-receiver = A quién
handover-receiver-hint = P. ej. «Hospital central, urgencias» o «Ambulancia Florian Kreis 3/83-1».
handover-receiver-missing = Indique a quién se transfiere el paciente.
handover-target-missing = Elija a quién se transfiere el paciente.
handover-note = Nota de transferencia (opcional)
handover-note-example = p. ej. qué ocurrió en el camino
handover-receiver-any = el puesto que recibe
handover-gets = Lo que recibe { $to }
handover-gets-record = la ficha completa con historial, fotos y órdenes
handover-gets-note = quién transfirió, cuándo, y la nota de transferencia
handover-gets-outside = la transferencia con receptor, hora y nota en el historial; después un documento de transferencia (PDF)
handover-after = Después
handover-after-inside = Hasta la recepción usted sigue documentando, y ambos lados ven «Transferencia pendiente». Tras la recepción aún puede leer la ficha, pero ya no escribir.
handover-after-outside = El paciente sale del incidente de inmediato y cuenta como dado de alta. Si el paciente vuelve: «Readmitir».
handover-button-label = Transferir
handover-button = Transferir a { $to }
quick-handover = Transferencia rápida
quick-handover-hint = Cuando aún no se sabe quién recibe: ORDO muestra un código QR. Quien lo escanee elige adónde va el paciente. El código no contiene datos médicos.
quick-handover-start = Mostrar código QR
quick-code-alt = Código QR de la transferencia rápida
quick-code-hint = Escanear con ORDO («Escanear transferencia») o la cámara del dispositivo que recibe.
scan-handover = Escanear transferencia
scan-handover-hint = Foto del código QR de una transferencia rápida.
scan-reading = Leyendo el código QR …
scan-no-handover = No se ve ningún código QR de transferencia en la foto.
take-title = Recibir al paciente { $tag }
take-from = Hasta ahora con { $holder }. Elija dónde está el paciente con usted.
take-not-here = La ficha aún no está en este dispositivo. Llega en cuanto el dispositivo esté conectado; esta ventana sigue abierta.
take-not-arrived = La transferencia en sí aún no está en este dispositivo. La recepción cuenta en cuanto ambos dispositivos vuelvan a estar conectados.
take-gone = El paciente ya no está en el incidente (dado de alta o transferido fuera).
take-taken = Esta transferencia ya ha sido recibida.
take-nowhere = No figura en el personal de ningún PMA ni en ninguna dotación. El mando del incidente le asigna.
incoming-title = Transferencias para usted
incoming-line = de { $from } a { $to }
record-open = Ficha
# Variables: $tag, $to
handover-bell = Transferencia a { $to }: paciente { $tag }
readmitted-step = Readmitido
cleared = borrado
photo-added-word = Foto añadida
timeline-filter = Filtrar historial
way-registered = Registrado · { $uhs }
registered-as = Registrado: { $category }
entries-all = Los { $n }
entries-all-label = { $form }: los { $n } registros del historial
corrects-entry = corrige el registro de las { $time }
uhs-edit-title = Editar { $name }
uhs-close-title = Cerrar PMA
uhs-close = Cerrar PMA …
uhs-close-confirm = Sí, cerrar
uhs-close-hint = El PMA desaparece de las listas, del mapa y de los recuentos. El diario de operaciones lo conserva.
uhs-close-question = ¿Cerrar de verdad { $name }?
uhs-details-label = { uhs-details } de { $name }
staff-remove-label = Quitar a { $name } del personal
uhs-renamed = PMA renombrado a { $name }

## A UHS asks for staff (UHS-15).
staff-requests-title = Solicitar personal
staff-request-choose-uhs = Elija arriba su PMA en «Mi PMA» para solicitar para él.
staff-request-for = Para { $uhs }: el mando del incidente ve la solicitud y envía a alguien.
staff-request-open = abierta
staff-request-withdrawn = retirada
staff-request-closed = Solicitud cerrada
staff-board-title = Solicitudes de los PMA
staff-send = ¿Quién viene?
staff-assign = Asignar
staff-withdraw = Retirar
staff-request-met = { $person } viene
staff-withdraw-label = { staff-withdraw }: { $request }
case-cancel = Admitir sin misión
register-from-case = De la misión «{ $place }» para { $unit }
staff-request-not-staff = Solo alguien del personal de { $uhs } puede solicitar.
all-uhs = Todos los PMA
read-only-uhs = { $name } (solo lectura)
# Above a record the person may only read (UHS-10).
record-read-only = Solo lectura: no figura en el personal de { $uhs }. El mando del incidente asigna el personal.

## Converted by hand: counts, forms and values of ORDO's types
working-uhs = Mi PMA
# Spoken description of a triage category, so screen readers convey its meaning.
triage-description-immediate = Categoría de triaje I, rojo, tratamiento inmediato
triage-description-urgent = Categoría de triaje II, amarillo, urgente
triage-description-minor = Categoría de triaje III, verde, lesiones leves
triage-description-expectant = Categoría de triaje IV, azul, tratamiento expectante
triage-description-deceased = Fallecido
triage-description-uninjured = Persona afectada, ilesa, sin necesidad de tratamiento
triage-count = { $n ->
    [one] 1 triaje
   *[other] { $n } triajes
}
# Variables: $tag, $category
retriage-of = { col-retriage } { $tag }: { $category }
corrected-entries = { $n ->
    [one] 1 registro corregido
   *[other] { $n } registros corregidos
}
# "Edit Vitalwerte": the button's name with its form.
edit-form = Editar { $form }
value-not-a-number = Introduzca un número.
value-between = Introduzca un valor de { $min } a { $max }.
value-at-least = Introduzca al menos { $min }.
value-at-most = Introduzca como máximo { $max }.
value-impossible = Valor imposible.
value-not-a-date = Introduzca una fecha.
value-not-a-time = Introduzca una hora (hh:mm).
value-too-long = El texto es demasiado largo.
action-retriage-to = Cambiar triaje: { $category }
# "7 of 10 taken · 3 free", or "Patients" without a capacity.
uhs-load-of = de { $capacity } ocupados · { $free } libres
uhs-load-patients = { $n ->
    [one] Paciente
   *[other] Pacientes
}
patient-count = { $n ->
    [one] 1 paciente
   *[other] { $n } pacientes
}
# How a UHS's capacity is split, as typed. Variables: $kept, $total
uhs-places-over = Ahora { $kept } puestos por categoría: más que la capacidad de { $total }.
# Variables: $kept, $rest
uhs-places-within = Ahora { $kept } por categoría, { $rest } sin categoría.
# A category's strip of places, e.g. "SK II: 4 of 4 taken, full".
place-strip-full = { $category }: { $taken } de { $total } ocupados, completo
# Variables: $category, $taken, $total, $free
place-strip-free = { $category }: { $taken } de { $total } ocupados, { $free } libres
transport-step-requested = Solicitado
transport-step-assign = Asignar
transport-step-loading = Cargar
transport-step-departed = En camino
transport-step-arrived = En destino
transport-step-handed-over = Paciente transferido en destino
transport-step-cancel = Cancelar transporte
order-step-give = Dar orden
order-step-acknowledge = Acusar recibo
order-step-done = Realizada
order-step-withdraw = Retirar
order-channel-in-person = en persona
order-channel-radio = por radio
order-channel-phone = por teléfono
order-channel-telemedicine = por telemedicina
# "Ordered verbally by Petra Schulz, by radio, read back and confirmed"
order-spoken-read-back = Ordenada verbalmente por { $giver }, { $channel }, repetida y confirmada
order-spoken-not-read-back = Ordenada verbalmente por { $giver }, { $channel }, no repetida
# "Readmit DE-0042": the button's name with its patient.
readmit-of = Readmitir { $tag }
# "Admission: SK I 2 · SK II 4 · SK III – (as of 10:12, ben@…)"
capacity-of = Admisión: { $parts } (a las { $at }, { $by })
# "Open orders: 3 (1 not confirmed)"
open-orders-count = Órdenes abiertas: { $n }
# Variables: $n, $unconfirmed
open-orders-unconfirmed = Órdenes abiertas: { $n } ({ $unconfirmed } sin confirmar)
triage-stage-pretriage = Pretriaje
triage-stage-triage = Triaje
uhs-staff-count = { $n ->
    [0] { uhs-staff-none }
    [one] Personal: 1 persona
   *[other] Personal: { $n } personas
}
handover-no-transport = No hay transporte con unidad asignada. Solicite un transporte en la sección «Transporte».
handover-no-other = No hay otro puesto en el incidente.
uhs-close-blocked = { $patients ->
    [one] Aún hay 1 paciente aquí. Transfiéralo o dele el alta primero, luego cierre.
   *[other] Aún hay { $patients } pacientes aquí. Transfiéralos o deles el alta primero, luego cierre.
}
staff-need-physician = Médico
staff-need-paramedic = Técnico en emergencias
staff-request-of = Solicitar { $need }
# Variables: $uhs, $need
staff-requested-of = { $uhs } solicita { $need }
# The Auftrag of the person sent after a staff request (UHS-15). Variables: $uhs, $need
staff-task-of = Como { $need } a { $uhs }
# "Patient counts care unit entrance: 5 (SK I 1 · SK III 4), 1 without a place, 2 discharged"
log-counts-total = Recuento de pacientes { $uhs }: { $total }
log-counts-discharged = { $n } dados de alta
log-places = { $uhs }: { $n } puestos de tratamiento
log-transport = { $kind } desde { $uhs }
log-transport-requested = Transporte solicitado: { $transport }
# Variables: $transport, $status, $unit
log-transport-updated-by = Transporte { $transport }: { $status } ({ $unit })
log-transport-updated-to = Transporte { $transport }: { $status }
log-order-updated = { $tag }: orden { $status }

## Führung – Lage (ADR-0099): more of each UHS, without patient data.
# Variables: $uhs, $destination
transport-of-unit = Traslado desde { $uhs } → { $destination }
staff-requested-tag = Personal solicitado: { $need }
waiting-transport-n = { $n ->
    [one] 1 espera traslado
   *[other] { $n } esperan traslado
}
waiting-no-unit = { $n } sin unidad
waiting-transport-total = Esperan traslado: { $n }
# Variables: $all, $here, $discharged
registered-here-discharged = Registrados { $all } · aquí ahora { $here } · dados de alta { $discharged }
staff-go-send = Enviar personal
staff-go-send-label = { staff-go-send }: { $uhs }
# The triage aid on the profile page (PAT-11, ADR-0100).
profile-triage = Triaje en este dispositivo
# The "+" button that opens the form to add one (ADR-0102).
uhs-new-word = PMA
# The "⋯" menu of a transport's further steps (ADR-0102).
transport-more-label = Más pasos: { $transport }

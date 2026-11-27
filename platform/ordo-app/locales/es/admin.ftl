tab-admin = Administración
tab-person = Persona
ws-access = Acceso
ws-info = Información
devices = Dispositivos
devices-intro-own = Sus propios dispositivos vinculados. Bloquee aquí un dispositivo perdido o robado: se desconecta al instante y borra sus datos en cuanto vuelva a conectarse.
devices-intro-all = Todos los dispositivos vinculados de la organización. Bloquee aquí un dispositivo perdido o robado: se desconecta al instante y borra sus datos en cuanto vuelva a conectarse.
devices-offline = La lista de dispositivos solo está disponible con conexión al servidor.
devices-loading = Cargando la lista de dispositivos …
devices-empty = No hay dispositivos vinculados.
this-device = este dispositivo
device-person = Persona
device-paired = Vinculado
device-valid-until = Sesión válida hasta
device-connection = Conexión
device-connected = conectado
device-not-connected = no conectado
# Devices used in the incident, on *Vorbereitung – Personal* (ORG-12).
incident-devices = Dispositivos del incidente
incident-devices-intro = Los dispositivos de todas las personas con un rol en este incidente. Bloquee aquí un dispositivo perdido o robado: se desconecta al instante y borra todos los datos de ORDO en cuanto vuelva a conectarse, también los de otros incidentes.
incident-devices-empty = Aún no hay dispositivos: nadie con un rol en este incidente tiene un dispositivo vinculado.
unknown = desconocido
revoke = Bloquear
revoke-confirm = Sí, bloquear
device-name = Dispositivo …{ $short }
# "Firefox on Android · …4F2A9C": the device's name and the end of its ID (ORG-8).
named-device = { $name } · …{ $short }
revoke-label = Bloquear: dispositivo …{ $short }

## Licences and source code (LIC-1)
notices-title = Licencias y código fuente
notices-ordo-licence = ORDO es software libre bajo la Licencia Pública General Affero de GNU, versión 3 o cualquier versión posterior (AGPL-3.0-or-later). Puede usarse, compartirse y modificarse; quien ofrezca una versión modificada a través de una red debe ofrecer también su código fuente.
notices-licence-text = Texto de la licencia
notices-source = Código fuente de esta versión:
notices-parts = Otros componentes
notices-crates = Bibliotecas de Rust
notices-crates-hint = Los textos de licencia tal como acompañan a las bibliotecas del servidor y de la aplicación, cada texto con las bibliotecas a las que se aplica.
notices-crates-missing = Los avisos de licencia de las bibliotecas de Rust no forman parte de esta compilación. Qué bibliotecas se usan y en qué versión figura en el código fuente (Cargo.lock); las compilaciones oficiales muestran aquí sus avisos de licencia.
info-version = Versión
info-app = Aplicación
info-server = Servidor
info-build-local = compilación local
info-server-offline = El servidor no es accesible en este momento.
info-loading = Consultando …
# "Protocols: sync 1.0–1.2, HTTP 1.0"
info-protocols = Protocolos: sync { $sync }, HTTP { $http }
notices-loading = Cargando avisos de licencia …
notices-used-map = Mapa de situación
notices-used-map-font = Tipografía de las etiquetas del mapa
notices-used-font = Tipografía de la interfaz
notices-used-icons = Iconos de la interfaz
notices-used-symbols = Símbolos tácticos (dibujados al compilar ORDO)
notices-used-storage = Almacenamiento de datos en el dispositivo
notices-used-map-data = Datos cartográficos, cuando hay un mapa base configurado
notices-public-domain = dominio público

## Organisation roles (ORG-9)
org-roles = Permisos
org-roles-title = Permisos en la organización
org-roles-intro = Quién administra la organización o planifica incidentes. Un permiso se aplica en todos los dispositivos de la persona en cuanto se concede, y termina al retirarse o al final del día elegido. Los roles en un incidente los da el mando del incidente en «Preparación – Personal y roles».
org-roles-loading = Cargando roles …
org-roles-empty = Aún no se ha dado ningún rol.
org-roles-offline = Los roles solo pueden verse y cambiarse con conexión al servidor.
org-role-grant-title = Dar rol
org-role-person = Persona (ID de usuario)
org-role-role = Rol
org-role-until-label = Válido hasta (opcional)
org-role-until-hint = Sin fecha, el rol se aplica hasta que se retire.
org-role-grant = Dar
org-role-withdraw = Retirar
org-role-invalid = Indique un ID de usuario, p. ej. anna@ordo.example.
org-role-past = La fecha está en el pasado.
org-role-fixed = de la configuración del servidor
org-role-own-admin = su propia administración no puede retirarse
org-role-lapsed = caducado
# Until when a role counts, e.g. "until 30.11.2026".
org-role-until = hasta el { $date }

## Local accounts (ORG-10)

## Upstream login providers (ORG-11, ADR-0063).
login-providers = Inicio de sesión
providers-title = Inicio de sesión a través de la organización
providers-intro = Conecte el servicio de inicio de sesión de su organización (OpenID Connect, p. ej. Microsoft Entra ID, Keycloak, Authentik, Google). Después aparece como botón propio en la página de inicio de sesión. Antes de guardar, inicia sesión una vez a través de él como prueba.
providers-loading = Cargando proveedores …
providers-not-managed = Este servidor no gestiona proveedores de inicio de sesión (sin acceso a la interfaz del servicio de inicio de sesión). Los proveedores están entonces en su configuración.
provider-local = Cuentas locales de este servidor
provider-change = Cambiar
provider-delete = Quitar
provider-delete-confirm = Sí, quitar
provider-new = Añadir proveedor
provider-name = Nombre en la página de inicio de sesión
provider-name-hint = p. ej. «Cuenta Cruz Roja»
provider-issuer = URL del emisor
provider-issuer-hint = p. ej. https://login.microsoftonline.com/<tenant>/v2.0
provider-client-id = ID de cliente
provider-secret = Secreto de cliente
provider-secret-hint = Se pasa al servicio de inicio de sesión y nunca se vuelve a mostrar en ORDO.
provider-secret-keep = Dejar vacío para conservar el secreto actual.
provider-skip-email = Aceptar también direcciones de correo sin indicador de verificación (p. ej. con Microsoft Entra ID)
provider-redirect = Introducir como URI de redirección en el proveedor:
provider-invalid = Indique un nombre, una URL de emisor (https://…), un ID de cliente y un secreto de cliente.
provider-test = Preparar inicio de sesión de prueba
provider-test-title = Inicio de sesión de prueba
provider-test-open = Abrir inicio de sesión de prueba
provider-test-hint = Vuelva aquí después: el resultado aparece aquí.
provider-save = Guardar proveedor
provider-saved = Proveedor guardado.
provider-removed = Proveedor quitado.
provider-kind = De la configuración ({ $kind })
provider-change-label = { provider-change }: { $name }
provider-delete-label = { provider-delete }: { $name }
provider-delete-question = ¿Quitar «{ $name }»? Ya no serán posibles nuevos inicios de sesión a través de él.
provider-change-title = Cambiar «{ $name }»
provider-test-waiting = Inicie sesión a través de «{ $name }» en una pestaña nueva. El inicio de sesión de prueba es válido 10 minutos; durante ese tiempo la página de inicio de sesión muestra también «Prueba: { $name }».
provider-confirm = Confirmo: se aceptan los inicios de sesión a través de «{ $name }» para este servidor.
provider-test-failed = El inicio de sesión de prueba ha fallado: { $error }. Compruebe los datos y vuelva a probar.
accounts = Cuentas
accounts-title = Cuentas de inicio de sesión
accounts-intro = Cuentas para iniciar sesión en este servidor, p. ej. para intervinientes sin cuenta de la organización. Solo el servicio de inicio de sesión guarda contraseñas; ORDO no las conoce.
accounts-loading = Cargando cuentas …
accounts-empty = Aún no hay cuentas.
accounts-offline = Las cuentas solo pueden verse y cambiarse con conexión al servidor.
accounts-not-managed = Este servidor no gestiona cuentas de inicio de sesión (sin acceso a la interfaz del servicio de inicio de sesión). Las cuentas proceden entonces de la configuración o del sistema de cuentas de la organización.
account-create-title = Crear cuenta
account-email = Dirección de correo (nombre de usuario)
account-name = Nombre
account-password = Primera contraseña
account-new-password = Nueva contraseña
account-password-hint = Al menos 12 caracteres. Comuníquesela a la persona en persona.
account-create = Crear cuenta
account-set-password = Establecer una nueva contraseña
account-save-password = Guardar contraseña
account-delete = Eliminar cuenta
account-delete-confirm = Sí, eliminar cuenta
account-done = Guardado.
account-invalid = Indique una dirección de correo y un nombre.
account-password-length = La contraseña debe tener al menos 12 caracteres (como máximo 72 bytes).
account-exists = Ya existe una cuenta con esta dirección de correo.
account-not-found = Esta cuenta ya no existe.
account-refused = El servicio de inicio de sesión ha rechazado el cambio. Las cuentas de su configuración solo pueden cambiarse allí.
account-unreachable = El servicio de inicio de sesión no es accesible. Inténtelo más tarde.
# The question before deleting an account.
account-delete-question = ¿Eliminar la cuenta de { $name } ({ $email })? La persona ya no podrá iniciar sesión y sus dispositivos quedarán bloqueados.
# The accessible names of an account's buttons.
account-set-password-label = Establecer una nueva contraseña: { $email }
account-delete-label = Eliminar cuenta: { $email }

## Converted by hand: forms and values of ORDO's types
revoke-own-question = ¿Bloquear este dispositivo (…{ $short })? Se desconecta al instante y se borran todos sus datos, también los registros aún sin enviar.
revoke-other-question = ¿Bloquear el dispositivo …{ $short }? Se desconecta al instante y borra todos sus datos en cuanto se conecte. Los registros del dispositivo aún sin enviar se pierden.
# "0.0.0, build 1a2b3c4"
info-build-commit = { $version }, compilación { $commit }
info-build-local-of = { $version }, { info-build-local }
org-role-admin = Administración
org-role-planner = Planificación de incidentes (todos los incidentes)
# The accessible name of the withdraw button. Variables: $role, $person
org-role-withdraw-of = Retirar { $role }: { $person }
provider-test-passed-as = El inicio de sesión de prueba ha funcionado. ORDO lo reconocería como { $user }.
# Variables: $user, $name, $email
provider-test-passed-name-email = El inicio de sesión de prueba ha funcionado. ORDO lo reconocería como { $user } (nombre: { $name }, correo: { $email }).
provider-test-passed-name = El inicio de sesión de prueba ha funcionado. ORDO lo reconocería como { $user } (nombre: { $name }).
provider-test-passed-email = El inicio de sesión de prueba ha funcionado. ORDO lo reconocería como { $user } (correo: { $email }).
provider-not-found = Este proveedor ya no existe, o solo puede cambiarse en la configuración.
provider-not-tested = Primero pruebe con éxito; el inicio de sesión de prueba es válido 10 minutos.
provider-refused = El servicio de inicio de sesión ha rechazado el cambio. ¿Está activada su interfaz para proveedores (DEX_API_CONNECTORS_CRUD)?

## The person's own pages (ADR-0100).
ws-person = Perfil y dispositivos
profile-view = Perfil

app-name = ORDO
skip-link = Aller au contenu
pair-title = Associer l'appareil
pair-intro = Connectez-vous avec le compte de votre organisation. Cet appareil reçoit sa propre clé, qui ne le quitte jamais ; ORDO n'enregistre aucun mot de passe.
pair-login = Se connecter
pair-qr = Associer par code QR
pair-qr-hint = Pour les tablettes partagées : scannez le code QR avec un téléphone connecté et confirmez.
pair-qr-alt = Code QR pour associer cet appareil
pair-code = Code : { $code }
pair-qr-link = Sans caméra : ouvrir sur un autre appareil et saisir le code :
pair-waiting = En attente de confirmation …
pair-no-login = Aucune connexion n'est configurée sur ce serveur.
pair-manual = Développement : associer l'appareil à la main
pair-manual-intro = Pour les installations de test sans connexion : valeurs du fichier d'amorçage du serveur et de `ordo generate-device-key`.
login-failed = Échec de la connexion
relogin-title = Connexion expirée
relogin-intro = La connexion de cet appareil a expiré. Veuillez vous reconnecter – les données de l'appareil sont conservées puis synchronisées.
network-error = Serveur injoignable
pair-user = Utilisateur (nom@serveur)
pair-device = Identifiant de l'appareil
pair-key = Clé privée de l'appareil
pair-incident = Identifiant de l'intervention
pair-submit = Associer
unpair = Se déconnecter
signed-in-as = Connecté en tant que
avatar-change = Changer la photo de profil
avatar-remove = Supprimer la photo de profil
retry = Réessayer
starting = ORDO démarre …
online = Connecté
connection-label = Connexion : { $state }. { $pending }
connecting = Connexion …
offline = Hors ligne – les saisies sont enregistrées et envoyées plus tard
my-devices = Mes appareils
page-tabs = Parties de la page
revoked-title = Appareil bloqué
revoked-text = Cet appareil a été bloqué. Sa clé et toutes les données d'intervention en ont été supprimées. Pour l'utiliser à nouveau, associez-le de nouveau.
pair-again = Associer à nouveau
device-name-label = Nom de cet appareil
device-name-hint = Comment l'appareil apparaît dans la liste des appareils, p. ex. « Tablette PMA nord ».
device-revoked = Appareil …{ $short } bloqué
retagged = Patient { $old } : nouveau numéro de fiche { $new }
uhs-created = { $name } créé
unit-added = Moyen { $unit } ajouté
launcher = Changer d'espace de travail
views = Vues
incident = Intervention
reason-no-access = Pas d'accès dans cette intervention
reason-not-connected = Disponible après la première connexion
no-workspace = Aucun espace de travail n'a encore été ouvert pour cet appareil dans cette intervention.

## Navigation and header
menu = Menu
search = Rechercher
storage-failed = Stockage de l'appareil plein ou bloqué : les saisies ne sont conservées que jusqu'à leur envoi. Ne pas recharger.

## Appearance (UX-3)
appearance = Apparence
profile = Profil et appareil
hand = Utilisé avec

## Vim keys (ADR-0090). Key names stay English (ADR-0036).
keymap = Clavier
keys-title = Raccourcis clavier
keys-on = Les touches Vim sont activées sur cet appareil. Elles fonctionnent tant qu'aucun champ de saisie n'a le focus, et ne modifient jamais de données.
keys-off = Les touches Vim sont désactivées. Activez-les dans le menu sous « Clavier ».
keys-vim = Touches Vim
keys-turn-on = Activer les touches Vim
keys-turn-off = Désactiver les touches Vim
keys-always = Toujours
key-keys = Touches
key-does = Effet
key-help = Cet aperçu
key-palette = Palette de commandes
key-search = Rechercher ou filtrer
key-move = Vers l'élément suivant à gauche, en dessous, au-dessus, à droite – sur toute la page
key-first-last = Premier / dernier élément de la page
key-tabs = Onglet suivant / précédent
key-views = Première à quatrième vue
key-menu = Menu
key-rail = Vues sur le bord : j/k pour choisir, o ouvre, l ou Échap pour revenir
key-open = Ouvrir l'entrée
key-insert = Vers le premier champ de saisie
key-scroll = Une demi-page vers le bas / le haut
key-escape = Quitter le champ de saisie
key-in-list = Dans une liste : entrée suivante / précédente
key-close = Fermer la boîte de dialogue ou le menu
theme-set = Apparence : { $label }
photo-added = Photo enregistrée pour le patient { $tag }
photo-removed = Photo supprimée pour le patient { $tag }
photo-restored = Photo du patient { $tag } restaurée
photo-failed = L'image n'a pas pu être lue.
photo-too-large = L'image est trop grande, même réduite.
edit = Modifier
save = Enregistrer
cancel = Annuler

## Command palette (UX-10)
palette-title = Recherche et commandes
palette-placeholder = Patient, moyen, vue ou commande …
palette-results = Résultats
palette-none = Aucun résultat.
palette-keys = choisir
palette-run = exécuter
palette-close = fermer
palette-back = retour
group-views = Vue
group-actions = Commande
my-roles = Rôles
no-role = aucun (planification d'intervention)
change-incident = Changer d'intervention
incident-details-saved = Détails de l'intervention enregistrés.
incident-times-invalid = La fin est avant le début, ou une date est incomplète.
place-label = Place { $n }
places-set = { $n } places de soins
transport-history = Historique
transport-requested = { $tag } : transport demandé
order-given = { $tag } : prescription enregistrée
order-confirmed-said = Prescription confirmée
order-recorded = { $tag } : prescription orale enregistrée
section-close = Fermer
# Closes a dialog.
close = Fermer
done = Terminé
discharge = Sortie
readmitted = { $tag } réadmis
hospital-saved = Hôpital enregistré : { $name }
capacity-saved = Admission enregistrée : { $name }

## API versions (ADR-0052).
version-unsupported = Cette application et le serveur n'ont aucune version en commun. Veuillez recharger la page.
crew-set = Équipage de { $unit } enregistré
unit-released = { $unit } libéré de l'intervention
# The patient counts of patients with units' crews.
with-units-label = Avec les moyens
handed-over = Patient { $tag } transmis à { $to }, prise en charge en attente
handed-over-quick = Transmission rapide lancée pour le patient { $tag }
handed-outside = Patient { $tag } transmis hors ORDO à { $receiver }
taken = Patient { $tag } pris en charge par { $by }
uhs-closed = { $name } fermé
uhs-updated = { $name } enregistré

## What needs the command's attention (TASK-3): the bell in the header.
attention = Alertes
attention-none = Rien ne demande d'attention pour le moment.

## Reports as Aufträge (REP-1…5, ADR-0081); events of ADR-0075 still read.
report-closed = Signalement clos
task-given-label = Mission donnée
task-patient-taken = Patient admis depuis la mission
undo = Annuler
close-label = { section-close } : { $what }
task-recorded = Mission créée : { $place }
follow-up-later-done = Relance reportée
task-given = Mission donnée à { $unit }
structure-saved = Organisation du commandement enregistrée
position-created = { $name } créé
map-saved-label = Carte de situation enregistrée

## Address search (ADR-0066)
address-label = Rechercher une adresse
address-submit = Rechercher
address-searching = Recherche …
address-empty = Veuillez saisir une adresse.
address-too-long = Veuillez saisir au plus 200 caractères.
address-offline = La recherche d'adresse nécessite une connexion au serveur.
address-not-found = Aucun résultat. Veuillez l'écrire autrement, par exemple avec la commune.
address-off = Aucune recherche d'adresse n'est configurée sur ce serveur.
address-refused = La recherche d'adresse est réservée au commandement.
address-failed = La recherche d'adresse ne répond pas pour le moment. Veuillez réessayer plus tard.
address-places = Lieux trouvés
address-attribution = Adresses d'OpenStreetMap, © contributeurs d'OpenStreetMap (ODbL).
address-chosen = Choisi : { $label }
notices-back = Retour

## Stored data that cannot be used (local persistence)
stored-state-unusable = Cette version ne peut pas lire les données de cet appareil ({ $reason }). Elles sont conservées telles quelles ; veuillez recharger l'application ou prévenir l'administration.

## Display names (ORG-7)
display-name = Nom affiché
display-name-hint = Comment les autres vous voient dans ORDO. Sans nom propre, le nom de la connexion s'applique.
display-name-save = Enregistrer le nom
display-name-reset = Utiliser le nom de la connexion
guest-pair-title = Accès pour les intervenants
guest-ended-title = Accès terminé
guest-ended = Votre accès à cette intervention s'est terminé avec votre service. Pour un nouvel accès, adressez-vous au commandement.
guest-pairing = Association de cet appareil avec le code QR …
guest-code-invalid = Le code QR n'est plus valable (déjà utilisé, expiré ou service terminé). Veuillez demander un nouveau code au commandement.
helper-saved = Coordonnées de { $name } enregistrées.

## Counts
pending = { $n ->
    [0] Tout est envoyé
    [one] { $n } saisie pas encore envoyée
   *[other] { $n } saisies pas encore envoyées
}
palette-count = { $n ->
    [0] { palette-none }
    [one] 1 résultat
   *[other] { $n } résultats
}
attention-label = { $n ->
    [0] { attention } : aucune
   *[other] { attention } : { $n }
}
address-found = { $n ->
    [one] 1 lieu trouvé. Veuillez choisir.
   *[other] { $n } lieux trouvés. Veuillez choisir.
}

## How long ago, from minutes: "just now", "5 min ago", "2 h 5 min ago", "3 days ago"
ago-now = à l'instant
ago-minutes = il y a { $minutes } min
ago-hours = il y a { $hours } h
# Variables: $hours, $minutes
ago-hours-minutes = il y a { $hours } h { $minutes } min
ago-days = { $days ->
    [one] il y a 1 jour
   *[other] il y a { $days } jours
}

## Triage categories in the DE profile: the codes on the triage tags stay.
triage-immediate = SK I
triage-urgent = SK II
triage-minor = SK III
triage-expectant = SK IV
triage-deceased = Décédé
triage-uninjured = Indemne
# A place kept for a triage category, e.g. "SK I – Place 2" (UHS-13).
category-place = { $category } – place { $n }

## Log lines with values of ORDO's types
patient-registered = Patient { $tag } enregistré : { $category }, { $uhs }
patient-retriaged = Patient { $tag } retrié : { $category }
unit-status-set = { $unit } : statut { $status }
# A unit status with its code, e.g. "3 – Task accepted".
status-with-code = { $code } – { $label }
# Variables: $user, $role
role-granted-to = Rôle { $role } donné à { $user }
# Variables: $user, $role
role-withdrawn-from = Rôle { $role } retiré à { $user }
incident-state-changed-to = Intervention : { $state }
priority-set = { $tag } : priorité de transport fixée
priority-removed = { $tag } : priorité de transport retirée
place-assigned-to = { $tag } sur { $place }
place-none = { $tag } : pas de place de soins
transport-status-set-to = Transport : { $status }
order-status-set-to = Prescription : { $status }
task-status-set-to = Mission : { $status }
# "Discharge: Transported"
discharged-as-outcome = { discharge } : { $outcome }
discharged-with = { $tag } : { $outcome }

## Themes, hands and keyboards (UX-3, ADR-0049, ADR-0090)
theme-auto = Automatique
theme-day = Jour
theme-night = Nuit
theme-high-contrast = Contraste élevé
hand-right = main droite
hand-left = main gauche

## Roles offered when granting (prototype; later from the profile)
role-commander = Commandement
role-medic = Secours à personne
role-physician = Médecin
role-paramedic = Secouriste

## States
incident-now-paused = Intervention en pause
incident-now-resumed = Intervention reprise
incident-planned = Planifiée
incident-active = En cours
incident-closed = Close
transport-status-requested = Demandé
transport-status-assigned = Attribué
transport-status-loading = Chargement
transport-status-departed = Parti
transport-status-arrived = À destination
transport-status-handed-over = Patient transmis à destination
transport-status-cancelled = Annulé
order-status-given = Donnée
order-status-acknowledged = Prise en compte
order-status-done = Exécutée
order-status-withdrawn = Retirée
outcome-released-on-site = Laissé sur place
outcome-transported = Transporté
outcome-refused = Refus de soins ou de transport
outcome-deceased = Décédé
task-status-open = Ouverte
task-status-in-progress = En cours
task-status-done = Terminée
task-status-cancelled = Annulée

## Dates and numbers, as each language writes them
# Day and month with two digits. Variables: $day, $month, $year
date-of = { $day }/{ $month }/{ $year }
# Variables: $date, $time
date-time-of = { $date } { $time }
# The decimal separator, as in "1.5 MB".
decimal-separator = ,

## Errors of the device's own workings (pairing, storage, randomness)
no-random = Aucun nombre aléatoire disponible
no-incident-chosen = Aucune intervention choisie
device-id-invalid = Identifiant de l'appareil : { $error }
private-key-format = Clé privée : 64 caractères hexadécimaux attendus
photo-not-found = Photo introuvable
photo-other-version = Photo dans une autre version
app-state-unavailable = État de l'application indisponible
no-secure-context = Stockage sécurisé indisponible : l'application doit être ouverte en HTTPS.
no-device-storage = Pas de stockage sur l'appareil
# A device's suggested name from its browser and system (ORG-8): "Firefox on Android".
device-name-on = { $browser } sur { $system }

## Language of this device (I18N-1)
# Each language names itself in its own file.
language-name = Français
language = Langue
language-auto = Automatique (comme le navigateur)
language-after-reload = La langue s'applique au prochain rechargement. Veuillez recharger seulement quand tout est envoyé.

## The profile page (ADR-0100): the person's settings apart from the device's.
profile-settings = Profil et paramètres
profile-me = Moi
profile-me-hint = S'applique sur tous vos appareils.
profile-device = Cet appareil
profile-device-hint = S'applique à toute personne qui utilise cet appareil.
keymap-vim-use = Utiliser les touches Vim
keymap-vim-hint = Les raccourcis habituels restent valables ; les touches Vim s'y ajoutent.
sign-out-hint = Cet appareil devra ensuite être associé à nouveau ; une autre personne pourra alors se connecter.
language-person-hint = Vous suit sur chaque appareil où vous êtes connecté.

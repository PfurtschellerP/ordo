tab-med = Secours à personne
tab-command = Commandement
# Workspaces of the incident command, by phase (ADR-0061).
ws-lead = Conduite
ws-prepare = Préparation
ws-crew = Équipage
crew-tasks = Missions
crew-transports = Transports pour votre moyen
crew-none-here = Vous ne faites partie d'aucun équipage dans cette intervention. Le commandement vous ajoute sous Préparation – Moyens.
dispatch = Engagement
people = Personnel
crew-patients = Mes patients
crew-transports-view = Mes transports
tab-helpers = Intervenants
tab-devices = Appareils
incident-page = Intervention

## The command structure and the map on one page (ADR-0095).
command-space = Structure
# Its label in the phone's bottom bar.
tab-resources = Moyens
tab-hospitals = Hôpitaux
tab-free-units = Moyens de transport libres
tab-free-units-short = Moyens libres
uhs-title = Postes médicaux avancés
uhs-choose = Choisir le PMA
units-title = Moyens
units-empty = Pas encore de moyens.
unit-call-sign = Indicatif
unit-type = Type (p. ex. ambulance, VSAV, équipe de secouristes)
unit-organisation = Organisation
unit-submit = Ajouter le moyen
unit-service = Service
unit-missing = Veuillez indiquer un indicatif et un type.
col-status = Statut
col-set-status = Définir le statut
no-status = –
set-status-label = Définir le statut de { $unit }
# Starts with the button's visible text, so its name matches what is seen.
task-give-label = { task-give } : { $unit }
move-here = Déplacer ici
# Starts with the button's visible text, so its name matches what is seen.
move-here-label = { move-here } : { $person } vers { $here }
group-units = Moyens
action-add-unit = Ajouter un moyen

## Incidents and roles (ADR-0046)
incidents-title = Interventions
incidents-intro = Choisissez une intervention. Les rôles dans une intervention sont donnés par le commandement ou la planification.
incidents-loading = Chargement des interventions …
incidents-offline = La liste des interventions n'est disponible qu'avec une connexion au serveur.
# Logging out from the incident list: the device is unpaired, so another person can log in.
log-out = Se déconnecter
log-out-hint = La déconnexion dissocie cet appareil ; une autre personne peut alors se connecter.
incidents-empty = Aucune intervention ouverte pour l'instant. Le commandement ou la planification donne aux personnes un rôle dans une intervention.
open = Ouvrir
plan-title = Planifier une intervention
plan-name = Nom de l'intervention
plan-submit = Créer l'intervention
grant = Donner un rôle
incident-state-label = Statut
incident-page-title = Intervention
incident-prepare = Préparer : carte de situation, moyens et personnel avec rôles et organisation du commandement. Démarrez l'intervention quand elle est en cours.

## The command's overview (ADR-0067)
open-map = Ouvrir la carte de situation
open-dispatch = Vers l'engagement
open-units = Vers les moyens
open-label = ouvertes
no-open-tasks = Aucune mission ouverte.
persons-on-scene = Intervenants sur place
units-on-scene = Moyens sur place
quick-links = Accès rapide
task-for = Pour : { $assignee }
discharged-count = { $n } sortis
start-incident = Démarrer l'intervention
close-incident = Clore l'intervention
incident-paused-label = en pause
pause-incident = Mettre l'intervention en pause
resume-incident = Reprendre l'intervention
incident-pause-hint = Pour les interventions sur plusieurs jours, comme les nuits d'un festival. Pendant la pause tout reste utilisable ; la situation et la main courante continuent.
incident-details = Détails de l'intervention
incident-description = Description
incident-description-hint = Par exemple lieu, organisateur, public attendu, particularités.
incident-starts = Début prévu
incident-ends = Fin prévue
incident-details-save = Enregistrer les détails

## Treatment places
places = Places de soins

## Transport requests (TRANS-1, TRANS-3)
transports = Transports
transports-empty = Aucun transport ouvert.

## Overview for the medical lead (UHS-8).
overview = Vue d'ensemble

## Units (UNIT-1…6).
strength-title = Effectifs sur place
strength = Effectif
strength-legend-title = Effectif (chefs / sous-chefs / équipiers)
strength-numbers = Veuillez indiquer l'effectif en trois nombres entiers, ou le laisser vide.
leaders = Chefs
sub-leaders = Sous-chefs
crew-count = Équipiers
by-organisation = Par organisation
by-service = Par service
no-service = sans service
no-organisation = sans organisation
crew = Équipage
crew-none = Aucun équipage enregistré.
crew-member = Nom
crew-qualifications = Qualifications
crew-add = Ajouter à l'équipage
crew-remove = Retirer
release = Libérer de l'intervention
back-in = De retour dans l'intervention

## [impl->dsn~detailed-strength~1]
unit-leaders = Chefs de formation
org-chart = Organisation du commandement en organigramme
special-functions-legend = Dont fonctions spéciales
strength-detailed = Indiquer l'effectif en détail : chefs de formation et chefs de section à part, fonctions spéciales comme médecins et porteurs d'ARI séparément (DV 102, 2024)
# "of these 2 × physicians, 6 × BA wearers".
special-functions = dont { $listed }
unreported = sans compte rendu d'effectif : { $n }
# "3 – Task accepted, since 10:12"
status-since = { $status }, depuis { $since }
crew-label = { crew } de { $unit } ({ $n })
crew-remove-label = Retirer { $name } de l'équipage
release-label = Libérer { $unit } de l'intervention
back-in-label = { $unit } de retour dans l'intervention
released-list = Moyens libérés ({ $n })
released-at = Libéré à { $at }
crew-title = Équipage { $unit }
attention-due = Relance due : { $unit } – { $task }
attention-new-task = Nouvelle mission : { $task }
attention-unassigned = Mission sans moyen : { $task }
case-register = Admettre le patient
open-in-maps-app = Ouvrir dans l'application de cartes
log-report-closed = Signalement clos : { $report }
log-report-patient = Patient de la mission { $task } admis par { $unit }

## Tasks (TASK-1…4).
tasks-heading = Missions ouvertes
tasks-empty = Aucune mission ouverte.
task-give = Donner une mission
task-unit = Pour
task-unit-choose = Choisir un moyen ou une fonction
task-template = Modèle
task-template-none = aucun modèle
task-text = Mission
task-follow-up = Relancer dans (minutes, facultatif)
task-missing = Veuillez indiquer un destinataire et une mission.
task-new = Nouvelle mission
task-new-word = Mission
task-create = Créer la mission
task-kind-label = Type
task-kind-other = Autre mission
task-kind-missing = Veuillez choisir le type.
task-unit-later = Attribuer plus tard
task-unit-hint = Facultatif pour un patient ou un événement : le moyen peut aussi être attribué plus tard.
task-place = Lieu
task-what = Que se passe-t-il ?
task-what-hint = En quelques mots, sans détails médicaux (ils vont dans le dossier patient).
task-received-label = Reçu
task-received-hint = Laisser vide pour maintenant.
task-text-missing = Veuillez décrire brièvement de quoi il s'agit.
task-place-missing = Veuillez indiquer le lieu.
task-too-long = Le lieu ou la description est trop long.
task-time-invalid = Veuillez indiquer une heure, p. ex. 14:05.
task-assign = Attribuer
task-send-further = Envoyer un autre moyen
my-tasks = Mes missions
pin-title = Repère sur la carte (facultatif)
pin-hint = Recherchez une adresse ou touchez la carte. Au clavier : déplacez la carte avec les flèches, puis « Placer le repère au centre de la carte ».
pin-at-centre = Placer le repère au centre de la carte
pin-clear = Retirer le repère
pin-none = Aucun repère placé.
pin-set = Placer le repère
pin-save = Enregistrer le repère
## The Auftrag card (ADR-0104): chips, dialogs, the "⋯" menu, Disposition's tabs.
pin-on-map = Sur la carte
pin-dialog-title = Repère : { $task }
task-more-label = Autres étapes : { $task }
task-no-units = Aucun autre moyen sur place.
tab-tasks = Missions
tab-staff-requests = Demander du personnel

## Map object cards (MAP-11).
map-details = Détails
map-object-gone = N'est plus sur la carte.
card-assign = Attribuer un moyen
card-open-task = Ouvrir la mission
card-current-task = Mission
card-edit = Modifier
card-patients = Patients par catégorie de tri
card-open-uhs = Ouvrir le PMA
qualifications-short = Qualifications
# "Anna Schmidt removed."
removed = { $what } retiré.
map-details-label = { map-details } : { $what }
card-edit-label = { card-edit } : { $unit }
# "Paramedic: 2"
qualification-count = { $label } : { $n }
crew-names = Noms ({ $n })
pin-at = Repère : { $at }
task-received = Reçu à { $at }
task-taken = Patient admis par { $unit } ({ $at })
follow-up-later = Relancer dans 10 minutes
due = Échu
tasks-title = { tasks-heading } ({ $n })
# "In progress: Patient: Stage north"
task-step-label = { $step } : { $task }
# One unit on a task, for accessible names: "RTW 2 – Patient: Stage north".
unit-in-task = { $unit } – { $task }
task-units = Moyens sur la mission
# "Follow up at 10:42"
follow-up-at = Relance à { $at }
# "Due: follow up since 10:42"
follow-up-due = { due } : relance depuis { $at }
# "10:12, ben@…"
task-given-by = Donnée à { $at } par { $by }

## Command structure (CMD-1…4, ADR-0031).
structure-title = Organisation du commandement
structure-short = Structure
structure-empty = Aucune organisation du commandement pour l'instant.
position = Fonction
position-choose = Choisir une fonction
position-create = Créer une fonction
position-kind = Type
position-kind-choose = Choisir le type
position-name = Nom
position-parent = Rattaché à
position-top = niveau supérieur
position-missing = Veuillez indiquer un type et un nom.
position-close = Dissoudre
no-position = non rattaché
no-leader = Direction : non pourvue
leader-label = Direction
leader-set = Attribuer ou transmettre la direction
uhs-assign = Rattacher un PMA
leader = { leader-label } : { $name }
# What a member of a position is, under its name in the org chart.
member-uhs = PMA
# Shown, never enforced (ADR-0031).
span-of-control = { $n } subordonnés directs : plus de 5, attention à l'étendue du contrôle.
position-close-label = Dissoudre { $title }

## A position put on the map, or found there, from the tree (CMD-5).
position-map-set-label = Placer sur la carte : { $title }
position-map-show-label = Afficher sur la carte : { $title }
# The place panel's status once a position was chosen from the tree.
map-place-ready = { $label } : touchez l'endroit ou placez-le au centre de la carte.
position-of = Position de { $unit }

## Situation map (MAP-1, MAP-2).
map-title = Carte de situation
map-short = Carte
map-label = Carte de situation. Les flèches déplacent, plus et moins zooment.
map-loading = Chargement de la carte …
map-hint-touch = Déplacer la carte avec deux doigts
map-hint-wheel = Zoom : Ctrl et molette
map-hint-wheel-mac = Zoom : ⌘ et molette
map-unavailable = La carte ne peut pas être affichée sur cet appareil ; la liste ci-dessous montre tout ce qui s'y trouve.
map-from-device = Fond de carte : enregistré sur cet appareil (aussi sans réseau).
map-from-server = Fond de carte : du serveur.
map-no-region = Aucun fond de carte n'a été préparé pour cette intervention : les objets sont sur un fond neutre.
map-objects = Sur la carte
map-locate = Vers ma position
map-locating = Recherche de la position …
map-located = La carte montre votre position.
map-locate-denied = La localisation n'est pas autorisée. Autorisez la localisation pour ORDO dans les réglages du navigateur.
my-position = Moi
my-position-show = Afficher ma position
my-position-shown = La carte montre votre position (point « Moi ») et le repère. Elle reste sur cet appareil.
map-locate-failed = La position n'a pas pu être déterminée.
map-objects-none = Rien sur la carte pour l'instant.
map-show = Afficher
map-remove = Retirer
map-place-hint = Touchez l'endroit – ou déplacez la carte pour que l'endroit soit sous le réticule (aussi avec les flèches), et placez-le au centre de la carte.
map-place-kind = Quoi
map-place-which = Lequel
map-place-choose = Veuillez choisir
map-place-label = Libellé
map-place-submit = Placer au centre de la carte
map-place-missing = Veuillez choisir ce qu'il faut placer.
incident-location = Lieu de l'intervention
map-saving = Enregistrement du fond de carte sur cet appareil pour une utilisation sans réseau …
map-save-failed = La carte n'a pas pu être enregistrée.
map-kind-symbol = Symbole tactique
map-symbol-label = Libellé (facultatif)
map-show-label = Afficher { $label } sur la carte
map-remove-label = Retirer { $label } de la carte

## The map editor (ADR-0064)
map-edit = Modifier la carte de situation
map-edit-hint = Placez sur la carte le lieu de l'intervention, les moyens, les PMA et les fonctions, dessinez des zones et des lignes. La conduite voit la carte sous Conduite – Carte de situation.
map-edit-not-allowed = Le commandement modifie la carte ; ici, elle peut seulement être consultée.
map-edit-object = Modifier
map-tools = Outil
tool-select = Sélectionner
tool-place = Placer un symbole
tool-area = Zone
tool-line = Ligne
map-tap-to-choose = Touchez un objet sur la carte, ou modifiez-le dans la liste ci-dessous.
map-move-hint = Touchez le nouvel endroit – ou déplacez la carte pour qu'il soit sous le réticule (aussi avec les flèches), et placez-le au centre de la carte.
map-move-centre = Placer au centre de la carte
map-deselect = Désélectionner
reshape-hint = Faites glisser les coins pour changer la forme ; tirez un nouveau coin depuis le milieu d'un côté ; faites glisser la forme pour la déplacer. Sans glisser : « Redessiner ».
reshape-apply = Appliquer
reshape-discard = Abandonner
reshape-redraw = Redessiner
map-reshape-failed = Cette forme ne peut pas être déplacée ici ; « Redessiner » fonctionne toujours.
map-edit-label = Modifier { $label }
map-chosen = { $label } sélectionné.
map-moved = { $label } déplacé.
map-placed = { $label } placé.
map-reshaped = { $label } modifié.
map-drawn = { $label } enregistré.
map-removed = { $label } retiré.
draw-redraw = Redessiner { $label }
log-hint = Chaque modification de cette intervention connue de cet appareil, avec l'heure, la personne et l'appareil, les plus récentes d'abord. Les entrées ne sont jamais modifiées : une correction est une nouvelle entrée qui renvoie à l'ancienne.
log-empty = Pas encore d'entrées.
log-pending = pas encore envoyée
log-by = { $user } · appareil { $device }
log-corrects = (corrige l'entrée de { $at })
log-pause-since = Pause depuis { $at }
log-address = Adresse
log-task-given = Mission pour { $to } : { $task }
log-position-closed = { $title } dissous
log-leader = { $title } : direction { $leader }
log-map-placed = { $label } placé sur la carte de situation
log-map-removed = { $label } retiré de la carte de situation

## Map region from the server (MAP-8)
map-region-title = Fond de carte
map-region-short = Fond de carte
map-region-hint = Le fond de carte de cette intervention pour la carte de situation : un rayon de la carte du monde libre (OpenStreetMap via Protomaps). Seuls les appareils des intervenants de cette intervention le reçoivent. Les appareils qui ont enregistré une carte hors ligne l'enregistrent à nouveau ensuite.
map-region-none = Aucun fond de carte n'a encore été créé pour cette intervention.
map-region-new = Nouveau fond de carte
map-region-lat = Latitude (centre)
map-region-lon = Longitude (centre)
map-region-radius = Rayon (km, de 1 à 50)
map-region-from-incident = Le centre est le lieu de l'intervention.
map-region-coordinates = Centre en degrés décimaux, p. ex. 48.37 et 10.90.
map-region-cut = Créer le fond de carte
map-region-progress = Progression
map-region-invalid = Veuillez indiquer une latitude (−85 à 85), une longitude (−180 à 180) et un rayon de 1 à 50 km.
draw-category = Type
draw-add-corner = Placer un coin au centre de la carte
draw-undo = Retirer le dernier coin
draw-cancel = Annuler
draw-finish = Terminé
draw-too-few-area = Une zone nécessite au moins trois coins.
draw-too-few-line = Une ligne nécessite au moins deux points.

## Helper register (HELP-1, HELP-2, ADR-0071)
helpers-title = Intervenants
helpers-intro = Coordonnées des intervenants de cette intervention. Seul le commandement les voit.
helpers-intro-roles = Qui a quel rôle dans cette intervention. Seul le commandement voit les coordonnées.
helpers-empty = Personne n'est enregistré pour l'instant.
helper-add-title = Ajouter une personne
helper-add = Ajouter
role-add-title = Donner un rôle
helper-new-word = Personne
role-new-word = Rôle
helper-complete = Compléter les coordonnées
helper-complete-label = { helper-complete } : { $name }
helper-complete-title = Compléter les coordonnées de { $name }
helper-name = Nom
helper-phone = Téléphone
helper-phone-hint = Avec l'indicatif, p. ex. 0171 1234567 ou +43 664 1234567.
helper-phone-invalid = Le numéro de téléphone n'est pas un numéro valable avec indicatif, p. ex. 0171 1234567.
helper-street = Rue et numéro
helper-postcode = Code postal
helper-postcode-invalid = Le code postal n'a pas la forme du pays, p. ex. 55116.
helper-town = Commune
helper-role = Rôle dans l'intervention
helper-roles = Rôles dans l'intervention
or = ou
guest-duty-ended = La fin du service est passée. Pour un accès sans compte, redémarrez le service ou modifiez sa fin.
guest-until-closed = Accès invité jusqu'à la fin de l'intervention
guest-no-command = Le commandement nécessite un compte ORDO.
guest-access = Accès par code QR
guest-code-title = Code QR pour l'accès
guest-code-refused = L'accès n'a pas pu être créé.
guest-code-alt = Code QR qui associe l'appareil de cet intervenant à l'intervention
guest-link = Ou ouvrez ce lien sur l'appareil :
guest-code-close = Terminé
guest-code-for = Pour { $name } : scanner avec la caméra de chaque appareil qu'utilise { $name }. Valable jusqu'au { $expires }.
guest-code-until-closed = Pour { $name } : scanner avec la caméra de chaque appareil qu'utilise { $name }. Valable jusqu'à la fin de l'intervention.
guest-access-label = { guest-access } : { $name }
guest-until = Accès invité jusqu'au { $until }
role-missing = Veuillez choisir un rôle.
helper-account-hint = Choisissez le compte de l'organisation ; son nom est repris. Laisser vide pour les intervenants sans compte.
diet-other = autre …
diet-other-label = Autre régime
helper-birth-date-label = Date de naissance
helper-diet = Régime alimentaire
diet-none = non indiqué
helper-duty-end = Fin du service
helper-account-label = Compte ORDO
helper-name-missing = Veuillez indiquer un nom.
helper-too-long = Chaque donnée peut comporter au plus 200 caractères.
helper-birth-date = Veuillez indiquer la date de naissance sous forme de date.
helper-account = Le compte ORDO n'est pas un identifiant valable, p. ex. anna@ordo.example.org.
helper-time = Veuillez indiquer le début et la fin du service sous forme de date avec heure.
helper-not-saved = Les coordonnées n'ont pas pu être enregistrées.
duty-start = Commencer le service
duty-end = Terminer le service
# Starts with the button's visible text, so its name matches what is seen.
duty-start-label = { duty-start } : { $name }
# Starts with the button's visible text, so its name matches what is seen.
duty-end-label = { duty-end } : { $name }
helper-edit-title = Modifier { $name }
duty-pause = Interrompre le service
duty-resume = Reprendre le service
duty-pause-label = { duty-pause } : { $name }
duty-resume-label = { duty-resume } : { $name }
# "on a break since 14:02"
paused-since = en pause depuis { $at }
on-duty-since = en service depuis { $at }
duty-from-to = Service { $from } – { $to }
duty-from = Service à partir de { $from }
duty-to = Service jusqu'à { $to }
# The person typed for a crew is not in the helper register (HELP-3).
not-registered = { $name } n'est pas dans le registre des intervenants. L'enregistrer avec le nom et l'ajouter ?
register-and-add = Enregistrer et ajouter
# Starts with the button's visible text, so its name matches what is seen.
register-and-add-label = { register-and-add } : { $name }
# Registration cards (HELP-5, ADR-0072).
# "Photograph Meldekarte", with the profile's label of the card ("card" for several).
card-generic = carte
card-photo = Photographier { $label }
card-reading = Lecture de la carte …
card-not-recognised = Aucune carte reconnue. Veuillez la photographier à nouveau : carte à plat, code QR entièrement visible et net.
card-read = { $card } de { $name } lue. Veuillez vérifier et enregistrer.

## Converted by hand: counts, lists and values of ORDO's types
log-title = Main courante
task-follow-up-number = Veuillez indiquer les minutes en nombre entier.
log-pause-ended = Pause terminée
draw-hint = Touchez chaque coin – ou déplacez la carte pour que le coin soit sous le réticule (aussi avec les flèches), et placez-le. Puis « Terminé ».
helper-duty-start = Début du service
# People on more than one crew or staff list (UNIT-8), e.g. "Ben (RTW 1, care unit south)".
double-booked-people = Affectés plusieurs fois, peut-être comptés deux fois : { $people }
# The person being added is on other crews or staff lists already (UNIT-8).
assigned-elsewhere-to = { $person } est déjà affecté à { $posts }. Déplacer, pour que personne ne soit compté deux fois ?
# Variables: $unit, $overwritten, $kept
lost-update-of = Attention { $unit } : le statut { $overwritten } a été remplacé en même temps par { $kept }. Veuillez vérifier.
action-status-of = Statut { $status }
follow-ups-due = { $n ->
    [one] { $n } relance due
   *[other] { $n } relances dues
}
unreported-strength = { $n ->
    [0] Tous les moyens ont signalé leur effectif.
    [one] { $n } moyen sans compte rendu d'effectif.
   *[other] { $n } moyens sans compte rendu d'effectif.
}
transports-open = { $n ->
    [0] Transports : aucun ouvert
   *[other] Transports : { $n } ouverts
}
summary-map = { $objects ->
    [0] Rien sur la carte pour l'instant.
    [one] 1 objet sur la carte.
   *[other] { $objects } objets sur la carte.
}
# The state as people read it, with a pause: "Running – paused".
incident-state-paused = { $state } – { incident-paused-label }
# "13.11.2026, 18:00 to 15.11.2026, 23:00"
incident-from-to = { $starts } au { $ends }
incident-from = à partir du { $starts }
incident-to = jusqu'au { $ends }
# Variables: $user, $role
withdraw-role = Retirer : { $role } à { $user }
# The strength read out: "Strength: 0 leaders, 1 sub-leader, 8 crew, 9 in total".
strength-count = { $count } { $part }
strength-read = { strength } : { $parts }, { $total } au total
strength-special = { $strength }, { $special }
strength-platoon-leaders = Chefs de section
strength-forces = Intervenants
# A strength's legend, e.g. "Strength (leaders / sub-leaders / crew)".
strength-legend-of = { $title } ({ $parts })
units-count = { $n ->
    [one] { $n } moyen
   *[other] { $n } moyens
}
# Variables: $kind, $received, $place, $text
log-report-line = Signalement reçu : { $kind }, { $received } – { $place } – { $text }
# "Role Physician withdrawn from Anna Schmidt."
role-removed-from = Rôle { $role } retiré à { $who }.
task-kind-event = Événement
task-kind-patient = Patient
# "Patient: Stage north"
task-label-of = { $kind } : { $place }
# How far a task is (TASK-2).
task-phase-received = Reçue
task-phase-assigned = Attribuée
task-phase-in-progress = En cours
task-phase-done = Terminée
task-phase-cancelled = Annulée
# Variables: $kind, $received, $place, $task
log-task-received-line = Mission reçue : { $kind }, { $received } – { $place } – { $task }
log-task-received-to = { $line } – pour { $to }
log-task-assigned = Mission « { $task } » attribuée à { $unit }
task-step-open = Ouverte
task-step-in-progress = En cours
task-step-done = Terminée
task-step-cancel = Annuler
# Time left until a follow-up, e.g. "7:05 min left"; "now" at zero.
countdown-now = maintenant
# Variables: $minutes, $seconds
countdown-left = encore { $minutes }:{ $seconds } min
# "Handover: Dr M. → Dr K."
handover-from-to = Transmission : { $from } → { $to }
map-kind-uhs = PMA
map-kind-drawing = Dessin
map-kind-report = Signalement
map-saved-size = Fond de carte enregistré sur cet appareil ({ $megabytes } Mo), utilisable sans réseau.
# "With units: 2 (SK I 1 · SK II 1)"
with-units-line = { $label } : { $total } ({ $parts })
# Variables: $module, $event-type
log-unknown = { $module } : { $event-type }
log-planned = Intervention « { $name } » créée
# Variables: $added, $unit-type, $organisation
log-unit-added-line = { $added } ({ $unit-type }, { $organisation })
log-helper-registered = { $name } ajouté
log-helper-saved = Coordonnées de { $name } enregistrées
log-helper-changes = { $name } : { $changes }
log-cleared = { $label } effacé
log-fields-changed-list = Données modifiées : { $fields }
# Variables: $unit, $n
log-crew = { $n ->
    [0] Équipage de { $unit } : personne
    [one] Équipage de { $unit } : 1 personne
   *[other] Équipage de { $unit } : { $n } personnes
}
# A unit's step on a task (ADR-0085): "Task “…”: RTW 2 In progress".
log-unit-step-of = Mission « { $task } » : { $unit } { $status }
log-task-updated-to = Mission « { $task } » : { $status }
log-task-follow-up-changed = Mission « { $task } » : relance modifiée
log-assigned-to = { $what } rattaché à : { $position }
log-assigned-none = { $what } : n'est plus rattaché à une fonction
map-region-preparing = Préparation du fond de carte …
map-region-tiles = Création du fond de carte : { $done } tuiles sur { $total } …
map-region-ready-size = Fond de carte prêt ({ $megabytes } Mo).
# Variables: $km, $lat, $lon
map-region-area = Rayon de { $km } km autour de { $lat }, { $lon }.
map-region-created = Créé le { $at }.
map-region-failed = Le fond de carte n'a pas pu être créé : { $error }
draw-corners = { $n ->
    [0] Aucun coin placé pour l'instant.
    [one] 1 coin placé.
   *[other] { $n } coins placés.
}
# The diets of the helpers still expected or on duty, for catering.
diet-counts-line = Régimes (sans service terminé) : { $list }
diet-count = { $n } × { $diet }
# Starts with the button's visible text, so its name matches what is seen.
helper-edit-of = { $edit } : { $name }

## Führung – Lage (ADR-0099): the map beside the units and the treatment places.
lage = Situation
# The groups the command sees its units in (UNIT-10); the profile gives each status its group.
group-available = Disponible
group-task = En mission
group-transport = En transport
group-unavailable = Indisponible
group-none = Sans statut
# The first of the type filter's chips: "Alle 14".
filter-all = Tous
units-filter = Type
units-only-available = Disponibles seulement
units-glance-empty = Aucun moyen ne correspond au choix.
open-lage = Vers la situation
# "4 von 14 einsatzbereit", the first number large.
units-available-of = sur { $total } disponibles
col-type = Type
col-free = libres
col-all = total

## Struktur: actions at each position (ADR-0102), forms in a dialog.
position-new = Fonction
lead-word = Direction
sub-position-word = Sous-fonction
uhs-word = PMA
map-set-word = Placer
leader-set-label = Désigner la direction : { $title }
position-sub-label = Ajouter une sous-fonction : { $title }
uhs-assign-label = Rattacher un PMA : { $title }
# The "+" button that opens the form to add one (ADR-0102).
unit-new = Moyen
# The incident's details, read compactly; ✎ opens the form (ADR-0102).
incident-details-edit = Modifier les informations
not-set = non indiqué
strength-detailed-short = Effectif détaillé
yes = oui
no = non

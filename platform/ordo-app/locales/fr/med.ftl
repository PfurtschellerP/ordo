crew-places = Places de soins libres
crew-not-held = Ce patient n'est pas (ou plus) avec votre moyen. Qui s'occupe du patient maintenant voit le dossier.
# The UHS as a unit (UHS-12).
uhs-strength = Effectif du PMA
uhs-strength-hint = Le PMA compte comme un moyen parmi les effectifs sur place : avec cet effectif, sinon avec son personnel.
register-title = Enregistrer un patient
register-tag = Numéro de la fiche de tri
# A card number already registered (PAT-12): where the patient is, and since when.
tag-in-use = La fiche { $tag } est déjà enregistrée : { $holder }, depuis { $time }.
# A card number registered again (PAT-12): by whom, where and when.
# Variables: $person, $holder, $time
tag-registered-again = Enregistrée à nouveau à { $time } par { $person } ({ $holder }).
tag-twice = Fiche enregistrée deux fois
tag-twice-hint = Deux appareils ont enregistré ce numéro de fiche alors qu'ils étaient hors ligne, ou un numéro a été mal saisi. Qui a enregistré en dernier donne à son patient une nouvelle fiche et saisit son numéro ; ce que cet appareil a documenté depuis suit.
retag-ask = Votre enregistrement est arrivé plus tard. Veuillez donner une nouvelle fiche de tri à votre patient.
retag-number = Numéro de la nouvelle fiche
retag-submit = Changer le numéro
retag-in-use = La fiche { $tag } est déjà enregistrée. Veuillez prendre une fiche inutilisée.
register-category = Catégorie de tri
register-uhs = Patient à
register-submit = Enregistrer
board-empty = Aucun patient enregistré pour l'instant.
col-retriage = Retri
not-sent = pas encore envoyé
sent-label = envoyé
uhs-empty = Aucun PMA pour l'instant. Le commandement les crée sous Préparation – Moyens.
uhs-name = Nom du PMA
uhs-capacity = Capacité (patients)
uhs-submit = Créer le PMA
uhs-new = Nouveau PMA
no-uhs = Pas encore de PMA où vous pouvez documenter. Le commandement crée les PMA et affecte leur personnel.

## Patient list and record
# The workspace of one UHS in the launcher.
ws-uhs = PMA
patients-title = Patients
all-patients = Tous les patients
registered-word = enregistré.
choose-patient = Choisissez un patient dans la liste pour ouvrir le dossier.
unknown-person = Personne inconnue
patient-title = Patient { $tag }
not-found = Ce patient n'existe pas dans cette intervention.
sections = Sections
triage-title = Tri
triage-history = Historique
retriage-to = Modifier le tri
# Variables: $user, $time
by-at = { $time }, { $user }
photo-title = Photo
photo-none = Pas encore de photo.
photo-take = Prendre une photo
photo-upload = Téléverser une image
photo-busy = Réduction de la photo …
photo-preview-loading = Aperçu – chargement de la photo complète …
photo-preview-unavailable = Aperçu – la photo complète n'est pas encore sur le serveur. Elle sera rechargée plus tard.
photo-purpose = Uniquement pour l'identification. La photo est réduite, enregistrée sans données d'appareil photo ni de localisation et envoyée seulement à qui peut la voir.
# Under the photo: "Taken 14:31 by Anna Schmidt".
photo-taken = Prise à { $at } par { $by }
photo-alt = Photo du patient { $tag }
photo-remove-short = Supprimer
photo-remove-label = Supprimer la photo – patient { $tag }
new-entry = Nouvelle saisie
correct = Corriger
nothing-documented = Rien de documenté pour l'instant.
no-answer = non indiqué
# The free-text answer of an open choice (EPCR-12).
other-answer = Autre
now = Maintenant
empty-entry = Veuillez saisir au moins une valeur.
no-changes = Aucune modification.
check-input = Veuillez vérifier les champs marqués.
corrected = corrigé
entry-saved = { $form } enregistré pour le patient { $tag }
new-entry-label = { new-entry } : { $form }
group-patients = Patient
action-register = Enregistrer un patient
action-create-uhs = Créer un PMA
action-open-record = Ouvrir le dossier
action-take-photo = Prendre une photo
action-entry = Saisir { $form }

## Transport priority
transport-priority = Priorité de transport
remove-priority = Pas de priorité de transport
set-priority = Fixer { $label }
set-priority-label = { $tag } : fixer { $label }
remove-priority-label = { $tag } : pas de priorité de transport
place = Place de soins
no-place = Pas de place

## The plan of a UHS's places (UHS-14)
place-free = libre
place-fill-label = Occuper
place-none-title = Sans place
register-place-hint = Suggestion : la première place libre de la catégorie de tri choisie.
# "Fill place 2": the button of a free place.
place-fill = Occuper { $label }

## Capacity per triage category (UHS-13)
uhs-capacities-legend = Capacité par catégorie de tri (facultatif)
uhs-capacities-hint = Places de soins réservées à une catégorie de tri ; au total au plus la capacité. Les autres places n'ont pas de catégorie.
# The places per category do not fit in the capacity.
uhs-places-exceed = Les places par catégorie ({ $kept }) dépassent la capacité ({ $total }). Veuillez augmenter la capacité ou réserver moins de places.
place-taken = { $label } (occupée : { $by })
places-free = { $free } sur { $total } libres

## Free treatment places per category (UHS-8, UHS-16, ADR-0086)
places-all = Tous les PMA
places-any = sans catégorie
places-full = complet
places-none = Aucune place de soins créée.
places-legend = Une case par place : pleine = occupée, en pointillés = libre. Les catégories complètes sont hachurées et marquées « complet ».
# "free of 15", after the large number of free places.
free-of = libres sur { $total }
# "2 without a place": patients waiting for a place.
without-place = { $n } sans place
transports-title = Transports ouverts
transport-title = Transport
transport-type = Type de transport
transport-priority-of-request = Urgence
transport-needs = Besoins particuliers
transport-request = Demander un transport
transport-choose = Veuillez choisir un type de transport et une urgence.
transport-none = Aucun transport demandé.
transport-unit-label = Moyen
transport-destination = Destination (p. ex. hôpital)
transport-assign = Attribuer

## Transports waiting for a unit, in the bell and on the overview (TASK-3).
transport-waiting = Transport sans moyen : { $summary } · { $priority }
transports-title-short = Transports
open-transports = Vers les transports
transports-waiting = sans moyen
no-transports-waiting = Aucune demande de transport en attente.
transports-under-way = { $n } en route
transport-assign-missing = Veuillez choisir un moyen ; la destination peut suivre plus tard.
transport-destination-missing = Veuillez choisir une destination.
# A transport sent before its destination was decided (ADR-0098).
destination-open = Destination ouverte
destination-set-label = Fixer la destination
# A destination set before changes on the way, e.g. a hospital stops admitting.
destination-change = Changer la destination
destination-set = Destination : { $destination }
transport-no-units = Pas encore de moyens dans l'intervention.
transport-choose-unit = Choisir un moyen
# Accessible name of a step button: the step and which transport.
transport-step-label = { $step } : { $transport }
# Short description of a transport for lists and button names.
transport-summary = { $transport-type } pour { $uhs }, { $at }
transport-unit = { $unit } → { $destination }

## Medical orders (ORDER-1…4)
orders = Prescriptions
orders-title = Prescriptions ouvertes
orders-empty = Aucune prescription ouverte.
orders-none = Aucune prescription.
order-give = Prescrire
order-kind = Type
order-medication = Médicament
order-measure = Mesure
order-substance = Principe actif
order-dose = Dose avec unité
order-route = Voie d'administration
order-route-choose = Choisir la voie
order-measure-text = Mesure
order-note = Remarque (facultatif)
order-incomplete = Veuillez indiquer la prescription complète.
order-records-only = ORDO documente les prescriptions ; la personne qui prescrit vérifie la dose et l'indication.
# "Given 10:12 by petra@…"
order-by = { $at } par { $by }
order-performed-by-label = Exécutée par
order-performed-by = { order-performed-by-label } { $person }
# "Done (by paul@…)", in the history: who carried it out, apart from who
# reported it.
order-status-performed = { $status } (par { $person })

## Spoken orders, recorded by the crew and confirmed by their giver (ORDER-6), and who

## ordered a documented medication (ORDER-7).
order-who = Qui prescrit ?
order-mine = Je prescris
order-spoken-label = Prescrit oralement, par radio ou par téléphone
order-ordered-by = Prescrit par
order-channel-label = Transmis
order-read-back = Répété et confirmé (boucle fermée)
order-record = Enregistrer la prescription
order-spoken-hint = Qui a prescrit confirme plus tard dans ORDO, s'il utilise ORDO.
order-unconfirmed = Pas encore confirmée
order-confirm = Confirmer
ordered-by-from-order = Repris de la prescription. Enregistrez d'abord les prescriptions orales sous « Prescriptions ».
ordered-by-no-order = sans prescription dans ORDO
# "Recorded 10:12 by Anna Becker"
order-recorded-by = Enregistrée à { $at } par { $by }
# "Confirmed 10:20 by Petra Schulz"
order-confirmed = Confirmée à { $at } par { $by }
order-confirm-label = Confirmer la prescription : { $what }
log-order-confirmed = { $tag } : prescription orale confirmée
# Who ordered a documented medication, taken from its order: "Petra Schulz (order 10:12)"
ordered-by-order = { $giver } (prescription { $at })

## Admission and discharge (UHS-3).
stay-title = Parcours du patient
stay-open = Sortie …
# The stay section on the tab *Übergabe*: admissions and discharges; moving is a handover.
discharge-title = Admission et sortie
triage-open = Modifier le tri
orders-open = Nouvelle prescription
discharge-outcome = Sortie avec issue
discharge-choose = Veuillez choisir l'issue de la sortie.
readmit = Réadmettre
# "Discharged patients (2)"
discharged-list = Patients sortis ({ $n })
# "With the transport: RTW → City hospital"
discharge-transport = Avec le transport : { $line }

## Hospitals and the transport board (TRANS-2, TRANS-4).
destination = Destination
destination-choose = Choisir la destination
destination-other-option = Autre destination (texte libre)
destination-other = Autre destination
free-units-title = Moyens de transport libres
free-units-empty = Aucun moyen de transport libre.
hospitals-title = Hôpitaux
hospitals-empty = Aucun hôpital enregistré pour l'instant.
# Where the list is kept, on the transport board while it is empty.
hospitals-where = Les hôpitaux sont enregistrés sous Préparation – Moyens, onglet « Hôpitaux ».
hospital-name = Nom de l'hôpital
hospital-departments = Services
hospital-capabilities = Plateaux techniques
hospital-add = Ajouter un hôpital
hospital-save = Enregistrer l'hôpital
hospital-edit = Modifier
hospital-name-missing = Veuillez indiquer le nom de l'hôpital.
capacity-legend = Admissions libres (patients)
capacity-save = Enregistrer l'admission
capacity-none = Admission : non signalée
capacity-numbers = Veuillez indiquer l'admission en nombres entiers.
sent = Transportés : { $n }
hospital-edit-label = Modifier : { $name }
hospital-edit-title = Modifier { $name }
capacity-save-label = { capacity-save } : { $name }

## Planned transports (TRANS-5).
transport-planned = Prévu pour (vide : immédiatement)
transport-planned-invalid = Veuillez indiquer la date et l'heure complètes.
planned-title = Transports prévus
planned-empty = Aucun transport prévu.
planned-for = Prévu pour { $at }

## Documenting a done order (ORDER-5).
document-order = Reprendre dans la documentation
document-order-label = { document-order } : { $what }
transports-count = Transports : { $open } ouverts, { $planned } prévus

## Patient pathway (PAT-5).
# "Moved: Patient collection point north → Treatment area"
moved = Déplacé : { $from } → { $to }
patient-moved = { $tag } déplacé vers { $to }

## Triage references (PAT-11, ADR-0045).
triage-with-label = Tri selon
triage-with-none = non indiqué
triage-aid = Aide au tri
triage-source = Ouvrir la source officielle (nouvelle fenêtre)
triage-aid-note = La personne qui trie décide de la catégorie ; ORDO n'en suggère aucune et n'en vérifie aucune.
# "by PRIOR"
triage-with = selon { $name }

## UHS details (UHS-1).
uhs-details = Emplacement et personnel
uhs-data = Données
saved = Enregistré.
uhs-name-needed = Le PMA a besoin d'un nom.
uhs-location = Emplacement
uhs-on-map = Sur la carte de situation
uhs-not-on-map = pas encore placé
uhs-place-on-map = Placer sur la carte
uhs-move-on-map = Déplacer sur la carte
# The link's visible text, then the unit: the name contains what is seen.
uhs-place-label = { $visible } : { $name }
uhs-staff = Personnel
uhs-staff-none = Aucun personnel saisi.
uhs-resources-title = PMA et personnel
uhs-staff-add = Ajouter au personnel
uhs-save = Enregistrer
uhs-capacity-number = Veuillez indiquer la capacité en nombre entier.

## The record page (EPCR-14, EPCR-15).
record-tabs = Parties du dossier
record-overview = Vue d'ensemble
record-timeline = Historique
record-documentation = Documentation
record-documentation-short = Doc.
record-handover = Transmission
record-at = À
record-triaged = Trié
record-since = Depuis

## The patient list's tabs (chapter 8.6.5, *Tabs*).
patients-tab = Patients
register-tab = Enregistrer
staff-request-tab = Demander du personnel
staff-request-short = Personnel
quick-actions = Saisie rapide
# The floating button that opens the quick actions on phones.
quick-capture = Saisir
all = Tous
all-kinds = Tout
latest = dernier
order = Prescription
orders-open-title = Prescriptions ouvertes
orders-all = Toutes les prescriptions de la documentation
orders-none-open = Aucune prescription ouverte.

## Handover (ADR-0087).
holders-uhs = PMA
holders-units = Moyens
handover-to = Transmission à { $to }
handover-quick-step = Transmission rapide par code QR
handover-taken-by = Pris en charge : { $by }
handover-taken-back = Transmission reprise : { $by }
handed-outside-step = Transmis à { $receiver } (hors ORDO)
handover-open-to = Transmission à { $to } en attente
handover-open-quick = Transmission rapide en attente
# "Still with SanTrp 1 · handed over 14:31 by Max Muster".
handover-open-since = Encore chez { $holder } · transmis à { $at } par { $by }
handover-open-hint = Jusqu'à la prise en charge, le poste précédent continue de documenter ; le poste qui reçoit confirme avec « Prendre en charge ».
handover-open-quick-hint = Le poste qui reçoit scanne le code QR avec ORDO ou la caméra de son appareil et choisit où va le patient.
handover-note-short = Remarque :
handover-take = Prendre en charge pour { $to }
handover-take-word = Prendre en charge
# The take button's accessible name: its words and the patient.
handover-take-label = { $words } : patient { $tag }
handover-take-to = Prendre en charge pour
take-place-hint = Suggestion : la première place libre de la catégorie de tri du patient.
handover-take-back = Reprendre la transmission
handover-failed = La transmission n'a pas pu être enregistrée.
handover-way = Le parcours du patient

## The handover document (ADR-0089).
document-title = Document de transmission
document-hint = PDF pour le poste qui reçoit. Toutes les données y sont aussi jointes sous forme lisible par machine, en bundle HL7 FHIR (une pièce jointe du fichier PDF, sous « Pièces jointes » dans les logiciels PDF). Le serveur ORDO le crée ; pour cela l'appareil a besoin d'une connexion.
document-button = Télécharger le PDF
document-label = Télécharger le PDF : document de transmission patient { $tag }
# The file name of the saved document.
document-file = Transmission-{ $tag }.pdf
document-asking = Création sur le serveur …
document-saved = Téléchargé.
document-offline = Pas de connexion au serveur. Le document y est créé ; veuillez réessayer avec une connexion.
document-refused = Vous ne pouvez pas récupérer le document de transmission de ce patient.
document-failed = Le document n'a pas pu être créé. Veuillez réessayer.

## The way as a flow chart (dsn patient-flow).
flow-registered = Enregistré à { $time }
flow-taken = Pris en charge à { $time }
flow-taken-back = Transmission reprise à { $time }
# "Transport: RTW 2 → City hospital · Requested"
flow-transport = Transport : { $line } · { $status }
flow-handover = Transmission
flow-handover-open = Transmission en attente
flow-moved = Déplacé
flow-outside = Hors ORDO
flow-quick-receiver = Qui scanne le code QR
flow-not-taken = Pas encore pris en charge
flow-next-edge = Prochaine transmission
handover-next = à qui transmettre
handover-title = Transmettre le patient
handover-to-label = Transmettre à
handover-to-uhs = PMA ou place de soins
handover-to-uhs-hint = Dans l'intervention. Le dossier suit le patient ; le personnel du PMA continue de documenter.
handover-to-unit = Moyen
handover-to-unit-hint = Dans l'intervention, p. ex. une ambulance. Son équipage continue de documenter.
handover-to-transport = Transport
handover-to-transport-hint = Le moyen d'un transport demandé prend en charge ; la destination découle du transport.
handover-to-outside = Hors ORDO
handover-to-outside-hint = P. ex. un hôpital, un autre service d'ambulances, la police. Le patient quitte l'intervention.
handover-target-label = Quel poste : { $kind }
handover-receiver = À qui
handover-receiver-hint = P. ex. « Hôpital central, urgences » ou « Ambulance Florian Kreis 3/83-1 ».
handover-receiver-missing = Veuillez indiquer à qui le patient est transmis.
handover-target-missing = Veuillez choisir à qui le patient est transmis.
handover-note = Note de transmission (facultatif)
handover-note-example = p. ex. ce qui s'est passé en route
handover-receiver-any = le poste qui reçoit
handover-gets = Ce que reçoit { $to }
handover-gets-record = le dossier complet avec historique, photos et prescriptions
handover-gets-note = qui a transmis, quand, et la note de transmission
handover-gets-outside = la transmission avec destinataire, heure et note dans l'historique ; un document de transmission (PDF) suit
handover-after = Ensuite
handover-after-inside = Jusqu'à la prise en charge vous continuez de documenter, et les deux côtés voient « Transmission en attente ». Après la prise en charge, vous pouvez encore lire le dossier, mais plus écrire.
handover-after-outside = Le patient quitte l'intervention aussitôt et compte comme sorti. Si le patient revient : « Réadmettre ».
handover-button-label = Transmettre
handover-button = Transmettre à { $to }
quick-handover = Transmission rapide
quick-handover-hint = Quand on ne sait pas encore qui prend en charge : ORDO affiche un code QR. Qui le scanne choisit où va le patient. Le code ne contient aucune donnée médicale.
quick-handover-start = Afficher le code QR
quick-code-alt = Code QR de la transmission rapide
quick-code-hint = Scanner avec ORDO (« Scanner une transmission ») ou la caméra de l'appareil qui reçoit.
scan-handover = Scanner une transmission
scan-handover-hint = Photo du code QR d'une transmission rapide.
scan-reading = Lecture du code QR …
scan-no-handover = Aucun code QR de transmission visible sur la photo.
take-title = Prendre en charge le patient { $tag }
take-from = Jusqu'ici chez { $holder }. Choisissez où le patient se trouve chez vous.
take-not-here = Le dossier n'est pas encore sur cet appareil. Il arrive dès que l'appareil est connecté ; cette fenêtre reste ouverte.
take-not-arrived = La transmission elle-même n'est pas encore sur cet appareil. La prise en charge compte dès que les deux appareils sont à nouveau connectés.
take-gone = Le patient n'est plus dans l'intervention (sorti ou transmis hors ORDO).
take-taken = Cette transmission a déjà été prise en charge.
take-nowhere = Vous n'êtes au personnel d'aucun PMA ni dans aucun équipage. Le commandement vous affecte.
incoming-title = Transmissions pour vous
incoming-line = de { $from } à { $to }
record-open = Dossier
# Variables: $tag, $to
handover-bell = Transmission à { $to } : patient { $tag }
readmitted-step = Réadmis
cleared = effacé
photo-added-word = Photo ajoutée
timeline-filter = Filtrer l'historique
way-registered = Enregistré · { $uhs }
registered-as = Enregistré : { $category }
entries-all = Toutes les { $n }
entries-all-label = { $form } : les { $n } saisies dans l'historique
corrects-entry = corrige la saisie de { $time }
uhs-edit-title = Modifier { $name }
uhs-close-title = Fermer le PMA
uhs-close = Fermer le PMA …
uhs-close-confirm = Oui, fermer
uhs-close-hint = Le PMA disparaît des listes, de la carte et des décomptes. La main courante le conserve.
uhs-close-question = Vraiment fermer { $name } ?
uhs-details-label = { uhs-details } de { $name }
staff-remove-label = Retirer { $name } du personnel
uhs-renamed = PMA renommé en { $name }

## A UHS asks for staff (UHS-15).
staff-requests-title = Demander du personnel
staff-request-choose-uhs = Choisissez votre PMA ci-dessus sous « Mon PMA » pour faire une demande pour lui.
staff-request-for = Pour { $uhs } : le commandement voit la demande et envoie quelqu'un.
staff-request-open = ouverte
staff-request-withdrawn = retirée
staff-request-closed = Demande close
staff-board-title = Demandes des PMA
staff-send = Qui vient ?
staff-assign = Affecter
staff-withdraw = Retirer
staff-request-met = { $person } arrive
staff-withdraw-label = { staff-withdraw } : { $request }
case-cancel = Admettre sans mission
register-from-case = De la mission « { $place } » pour { $unit }
staff-request-not-staff = Seul le personnel de { $uhs } peut faire une demande.
all-uhs = Tous les PMA
read-only-uhs = { $name } (lecture seule)
# Above a record the person may only read (UHS-10).
record-read-only = Lecture seule : vous ne faites pas partie du personnel de { $uhs }. Le commandement affecte le personnel.

## Converted by hand: counts, forms and values of ORDO's types
working-uhs = Mon PMA
# Spoken description of a triage category, so screen readers convey its meaning.
triage-description-immediate = Catégorie de tri I, rouge, traitement immédiat
triage-description-urgent = Catégorie de tri II, jaune, urgent
triage-description-minor = Catégorie de tri III, vert, blessures légères
triage-description-expectant = Catégorie de tri IV, bleu, traitement d'attente
triage-description-deceased = Décédé
triage-description-uninjured = Personne impliquée, indemne, sans besoin de soins
triage-count = { $n ->
    [one] { $n } tri
   *[other] { $n } tris
}
# Variables: $tag, $category
retriage-of = { col-retriage } { $tag } : { $category }
corrected-entries = { $n ->
    [one] { $n } saisie corrigée
   *[other] { $n } saisies corrigées
}
# "Edit Vitalwerte": the button's name with its form.
edit-form = Modifier { $form }
value-not-a-number = Veuillez saisir un nombre.
value-between = Veuillez saisir une valeur de { $min } à { $max }.
value-at-least = Veuillez saisir au moins { $min }.
value-at-most = Veuillez saisir au plus { $max }.
value-impossible = Valeur impossible.
value-not-a-date = Veuillez saisir une date.
value-not-a-time = Veuillez saisir une heure (hh:mm).
value-too-long = Le texte est trop long.
action-retriage-to = Modifier le tri : { $category }
# "7 of 10 taken · 3 free", or "Patients" without a capacity.
uhs-load-of = sur { $capacity } occupées · { $free } libres
uhs-load-patients = { $n ->
    [one] Patient
   *[other] Patients
}
patient-count = { $n ->
    [one] { $n } patient
   *[other] { $n } patients
}
# How a UHS's capacity is split, as typed. Variables: $kept, $total
uhs-places-over = Actuellement { $kept } places par catégorie : plus que la capacité de { $total }.
# Variables: $kept, $rest
uhs-places-within = Actuellement { $kept } par catégorie, { $rest } sans catégorie.
# A category's strip of places, e.g. "SK II: 4 of 4 taken, full".
place-strip-full = { $category } : { $taken } sur { $total } occupées, complet
# Variables: $category, $taken, $total, $free
place-strip-free = { $category } : { $taken } sur { $total } occupées, { $free } libres
transport-step-requested = Demandé
transport-step-assign = Attribuer
transport-step-loading = Charger
transport-step-departed = Parti
transport-step-arrived = À destination
transport-step-handed-over = Patient transmis à destination
transport-step-cancel = Annuler le transport
order-step-give = Prescrire
order-step-acknowledge = Prendre en compte
order-step-done = Exécutée
order-step-withdraw = Retirer
order-channel-in-person = en personne
order-channel-radio = par radio
order-channel-phone = par téléphone
order-channel-telemedicine = par télémédecine
# "Ordered verbally by Petra Schulz, by radio, read back and confirmed"
order-spoken-read-back = Prescrit oralement par { $giver }, { $channel }, répété et confirmé
order-spoken-not-read-back = Prescrit oralement par { $giver }, { $channel }, non répété
# "Readmit DE-0042": the button's name with its patient.
readmit-of = Réadmettre { $tag }
# "Admission: SK I 2 · SK II 4 · SK III – (as of 10:12, ben@…)"
capacity-of = Admission : { $parts } (au { $at }, { $by })
# "Open orders: 3 (1 not confirmed)"
open-orders-count = Prescriptions ouvertes : { $n }
# Variables: $n, $unconfirmed
open-orders-unconfirmed = Prescriptions ouvertes : { $n } ({ $unconfirmed } non confirmées)
triage-stage-pretriage = Pré-tri
triage-stage-triage = Tri
uhs-staff-count = { $n ->
    [0] { uhs-staff-none }
    [one] Personnel : 1 personne
   *[other] Personnel : { $n } personnes
}
handover-no-transport = Aucun transport avec un moyen attribué. Demandez un transport dans la section « Transport ».
handover-no-other = Aucun autre poste dans l'intervention.
uhs-close-blocked = { $patients ->
    [one] { $patients } patient est encore ici. Transmettez ou faites sortir d'abord, puis fermez.
   *[other] { $patients } patients sont encore ici. Transmettez ou faites sortir d'abord, puis fermez.
}
staff-need-physician = Médecin
staff-need-paramedic = Secouriste
staff-request-of = Demander { $need }
# Variables: $uhs, $need
staff-requested-of = { $uhs } demande { $need }
# The Auftrag of the person sent after a staff request (UHS-15). Variables: $uhs, $need
staff-task-of = En tant que { $need } vers { $uhs }
# "Patient counts care unit entrance: 5 (SK I 1 · SK III 4), 1 without a place, 2 discharged"
log-counts-total = Nombre de patients { $uhs } : { $total }
log-counts-discharged = { $n } sortis
log-places = { $uhs } : { $n } places de soins
log-transport = { $kind } depuis { $uhs }
log-transport-requested = Transport demandé : { $transport }
# Variables: $transport, $status, $unit
log-transport-updated-by = Transport { $transport } : { $status } ({ $unit })
log-transport-updated-to = Transport { $transport } : { $status }
log-order-updated = { $tag } : prescription { $status }

## Führung – Lage (ADR-0099): more of each UHS, without patient data.
# Variables: $uhs, $destination
transport-of-unit = Transport depuis { $uhs } → { $destination }
staff-requested-tag = Personnel demandé : { $need }
waiting-transport-n = { $n ->
    [one] 1 attend un transport
   *[other] { $n } attendent un transport
}
waiting-no-unit = { $n } sans moyen
waiting-transport-total = Attendent un transport : { $n }
# Variables: $all, $here, $discharged
registered-here-discharged = Enregistrés { $all } · ici maintenant { $here } · sortis { $discharged }
staff-go-send = Envoyer du personnel
staff-go-send-label = { staff-go-send } : { $uhs }

tab-admin = Administration
tab-person = Personne
ws-access = Accès
ws-info = Infos
devices = Appareils
devices-intro-own = Vos propres appareils associés. Bloquez ici un appareil perdu ou volé : il est déconnecté aussitôt et efface ses données dès qu'il se reconnecte.
devices-intro-all = Tous les appareils associés de l'organisation. Bloquez ici un appareil perdu ou volé : il est déconnecté aussitôt et efface ses données dès qu'il se reconnecte.
devices-offline = La liste des appareils n'est disponible qu'avec une connexion au serveur.
devices-loading = Chargement de la liste des appareils …
devices-empty = Aucun appareil associé.
this-device = cet appareil
device-person = Personne
device-paired = Associé
device-valid-until = Connexion valable jusqu'au
device-connection = Connexion
device-connected = connecté
device-not-connected = non connecté
# Devices used in the incident, on *Vorbereitung – Personal* (ORG-12).
incident-devices = Appareils de l'intervention
incident-devices-intro = Les appareils de toutes les personnes ayant un rôle dans cette intervention. Bloquez ici un appareil perdu ou volé : il est déconnecté aussitôt et efface toutes les données ORDO dès qu'il se reconnecte, y compris celles d'autres interventions.
incident-devices-empty = Pas encore d'appareils : personne ayant un rôle dans cette intervention n'a d'appareil associé.
unknown = inconnu
revoke = Bloquer
revoke-confirm = Oui, bloquer
device-name = Appareil …{ $short }
# "Firefox on Android · …4F2A9C": the device's name and the end of its ID (ORG-8).
named-device = { $name } · …{ $short }
revoke-label = Bloquer : appareil …{ $short }

## Licences and source code (LIC-1)
notices-title = Licences et code source
notices-ordo-licence = ORDO est un logiciel libre sous licence publique générale GNU Affero, version 3 ou toute version ultérieure (AGPL-3.0-or-later). Il peut être utilisé, partagé et modifié ; quiconque propose une version modifiée sur un réseau doit aussi en proposer le code source.
notices-licence-text = Texte de la licence
notices-source = Code source de cette version :
notices-parts = Autres composants
notices-crates = Bibliothèques Rust
notices-crates-hint = Les textes de licence tels qu'ils accompagnent les bibliothèques du serveur et de l'application, chaque texte avec les bibliothèques auxquelles il s'applique.
notices-crates-missing = Les mentions de licence des bibliothèques Rust ne font pas partie de cette compilation. Les bibliothèques utilisées et leurs versions figurent dans le code source (Cargo.lock) ; les compilations officielles affichent ici leurs mentions de licence.
info-version = Version
info-app = Application
info-server = Serveur
info-build-local = compilation locale
info-server-offline = Le serveur est injoignable pour le moment.
info-loading = Interrogation …
# "Protocols: sync 1.0–1.2, HTTP 1.0"
info-protocols = Protocoles : sync { $sync }, HTTP { $http }
notices-loading = Chargement des mentions de licence …
notices-used-map = Carte de situation
notices-used-map-font = Police des libellés de la carte
notices-used-font = Police de l'interface
notices-used-icons = Icônes de l'interface
notices-used-symbols = Symboles tactiques (dessinés à la compilation d'ORDO)
notices-used-storage = Stockage des données sur l'appareil
notices-used-map-data = Données cartographiques, lorsqu'un fond de carte est configuré
notices-public-domain = domaine public

## Organisation roles (ORG-9)
org-roles = Autorisations
org-roles-title = Autorisations dans l'organisation
org-roles-intro = Qui administre l'organisation ou planifie les interventions. Une autorisation s'applique sur tous les appareils de la personne dès qu'elle est donnée, et prend fin quand elle est retirée ou à la fin du jour choisi. Les rôles dans une intervention sont donnés par le commandement sous « Préparation – Personnel et rôles ».
org-roles-loading = Chargement des rôles …
org-roles-empty = Aucun rôle donné pour l'instant.
org-roles-offline = Les rôles ne peuvent être consultés et modifiés qu'avec une connexion au serveur.
org-role-grant-title = Donner un rôle
org-role-new-word = Rôle
org-role-person = Personne (identifiant)
org-role-role = Rôle
org-role-until-label = Valable jusqu'au (facultatif)
org-role-until-hint = Sans date, le rôle s'applique jusqu'à son retrait.
org-role-grant = Donner
org-role-withdraw = Retirer
org-role-invalid = Veuillez indiquer un identifiant, p. ex. anna@ordo.example.
org-role-past = La date est dans le passé.
org-role-fixed = issu de la configuration du serveur
org-role-own-admin = votre propre administration ne peut pas être retirée
org-role-lapsed = expiré
# Until when a role counts, e.g. "until 30.11.2026".
org-role-until = jusqu'au { $date }

## Local accounts (ORG-10)

## Upstream login providers (ORG-11, ADR-0063).
login-providers = Connexion
providers-title = Connexion par l'organisation
providers-intro = Raccordez le service de connexion de votre organisation (OpenID Connect, p. ex. Microsoft Entra ID, Keycloak, Authentik, Google). Il apparaît ensuite comme un bouton distinct sur la page de connexion. Avant d'enregistrer, vous vous connectez une fois par ce service pour tester.
providers-loading = Chargement des fournisseurs …
providers-not-managed = Ce serveur ne gère pas les fournisseurs de connexion (pas d'accès à l'interface du service de connexion). Les fournisseurs figurent alors dans sa configuration.
provider-local = Comptes locaux de ce serveur
provider-change = Modifier
provider-delete = Supprimer
provider-delete-confirm = Oui, supprimer
provider-new = Ajouter un fournisseur
provider-name = Nom sur la page de connexion
provider-name-hint = p. ex. « Compte Croix-Rouge »
provider-issuer = URL de l'émetteur
provider-issuer-hint = p. ex. https://login.microsoftonline.com/<tenant>/v2.0
provider-client-id = ID client
provider-secret = Secret client
provider-secret-hint = Transmis au service de connexion et plus jamais affiché dans ORDO.
provider-secret-keep = Laisser vide pour conserver le secret actuel.
provider-skip-email = Accepter aussi les adresses e-mail sans indicateur de vérification (p. ex. avec Microsoft Entra ID)
provider-redirect = À saisir comme URI de redirection chez le fournisseur :
provider-invalid = Veuillez indiquer un nom, une URL d'émetteur (https://…), un ID client et un secret client.
provider-test = Préparer la connexion de test
provider-test-title = Connexion de test
provider-test-open = Ouvrir la connexion de test
provider-test-hint = Revenez ici ensuite : le résultat s'affiche ici.
provider-save = Enregistrer le fournisseur
provider-saved = Fournisseur enregistré.
provider-removed = Fournisseur supprimé.
provider-kind = Issu de la configuration ({ $kind })
provider-change-label = { provider-change } : { $name }
provider-delete-label = { provider-delete } : { $name }
provider-delete-question = Supprimer « { $name } » ? Les nouvelles connexions par ce fournisseur ne seront plus possibles.
provider-change-title = Modifier « { $name } »
provider-test-waiting = Connectez-vous par « { $name } » dans un nouvel onglet. La connexion de test est valable 10 minutes ; pendant ce temps, la page de connexion affiche aussi « Test : { $name } ».
provider-confirm = Je confirme : les connexions par « { $name } » sont acceptées pour ce serveur.
provider-test-failed = La connexion de test a échoué : { $error }. Veuillez vérifier les données et tester à nouveau.
accounts = Comptes
accounts-title = Comptes de connexion
accounts-intro = Comptes pour se connecter à ce serveur, p. ex. pour des intervenants sans compte de l'organisation. Seul le service de connexion enregistre les mots de passe ; ORDO ne les connaît pas.
accounts-loading = Chargement des comptes …
accounts-empty = Aucun compte pour l'instant.
accounts-offline = Les comptes ne peuvent être consultés et modifiés qu'avec une connexion au serveur.
accounts-not-managed = Ce serveur ne gère pas les comptes de connexion (pas d'accès à l'interface du service de connexion). Les comptes proviennent alors de la configuration ou du système de comptes de l'organisation.
account-create-title = Créer un compte
account-new-word = Compte
account-email = Adresse e-mail (identifiant)
account-name = Nom
account-password = Premier mot de passe
account-new-password = Nouveau mot de passe
account-password-hint = Au moins 12 caractères. Communiquez-le à la personne en personne.
account-create = Créer le compte
account-set-password = Définir un nouveau mot de passe
account-save-password = Enregistrer le mot de passe
account-delete = Supprimer le compte
account-delete-confirm = Oui, supprimer le compte
account-done = Enregistré.
account-invalid = Veuillez indiquer une adresse e-mail et un nom.
account-password-length = Le mot de passe doit comporter au moins 12 caractères (au plus 72 octets).
account-exists = Il existe déjà un compte avec cette adresse e-mail.
account-not-found = Ce compte n'existe plus.
account-refused = Le service de connexion a refusé la modification. Les comptes issus de sa configuration ne peuvent être modifiés que là.
account-unreachable = Le service de connexion est injoignable. Veuillez réessayer plus tard.
# The question before deleting an account.
account-delete-question = Supprimer le compte de { $name } ({ $email }) ? La personne ne pourra plus se connecter et ses appareils seront bloqués.
# The accessible names of an account's buttons.
account-set-password-label = Définir un nouveau mot de passe : { $email }
account-delete-label = Supprimer le compte : { $email }

## Converted by hand: forms and values of ORDO's types
revoke-own-question = Bloquer cet appareil (…{ $short }) ? Il est déconnecté aussitôt et toutes ses données sont effacées, y compris les saisies pas encore envoyées.
revoke-other-question = Bloquer l'appareil …{ $short } ? Il est déconnecté aussitôt et efface toutes ses données dès qu'il se connecte. Les saisies pas encore envoyées depuis l'appareil sont perdues.
# "0.0.0, build 1a2b3c4"
info-build-commit = { $version }, compilation { $commit }
info-build-local-of = { $version }, { info-build-local }
org-role-admin = Administration
org-role-planner = Planification (toutes les interventions)
# The accessible name of the withdraw button. Variables: $role, $person
org-role-withdraw-of = Retirer { $role } : { $person }
provider-test-passed-as = La connexion de test a réussi. ORDO la reconnaîtrait comme { $user }.
# Variables: $user, $name, $email
provider-test-passed-name-email = La connexion de test a réussi. ORDO la reconnaîtrait comme { $user } (nom : { $name }, e-mail : { $email }).
provider-test-passed-name = La connexion de test a réussi. ORDO la reconnaîtrait comme { $user } (nom : { $name }).
provider-test-passed-email = La connexion de test a réussi. ORDO la reconnaîtrait comme { $user } (e-mail : { $email }).
provider-not-found = Ce fournisseur n'existe plus, ou il ne peut être modifié que dans la configuration.
provider-not-tested = Veuillez d'abord tester avec succès ; la connexion de test est valable 10 minutes.
provider-refused = Le service de connexion a refusé la modification. Son interface pour les fournisseurs est-elle activée (DEX_API_CONNECTORS_CRUD) ?

## The person's own pages (ADR-0100).
ws-person = Profil et appareils
profile-view = Profil

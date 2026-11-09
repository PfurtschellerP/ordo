---
status: accepted
date: 2026-11-09
---

# Profile pictures: ORDO, then the identity provider, then Gravatar

## Context and Problem Statement

Faces help to recognise people on scene: in the roster, next to the signed-in person,
later on the map and in the incident log. People should be able to set their own picture,
and organisations that keep pictures in their identity provider or people with a Gravatar
should not have to upload one again.

## Decision Outcome

Decided by the project lead on 2026-11-09: a person's picture is resolved in this order,
by the server:

1. **ORDO:** a picture the person set in ORDO (menu → Gerät → Profilbild). The app takes
   or chooses a photo, downscales it on the device to at most 256 px and 128 KiB and
   re-encodes it as JPEG, without camera or location metadata.
2. **Identity provider:** the OpenID Connect `picture` claim (HTTPS only), fetched by the
   server at every login. Dex does not forward this claim from upstream providers today,
   so with the bundled Dex this source is usually empty.
3. **Gravatar:** looked up by the server at login by the SHA-256 hash of the e-mail
   address (`d=404`), only if `ORDO_GRAVATAR` is on (default on).

Without a picture the app shows the person's initials. Pictures travel over the
authenticated sync connection (`getAvatars`, `setAvatar`, `avatars`); devices never
contact the identity provider's picture host or Gravatar themselves. The server accepts
only JPEG, PNG and WebP (no SVG) up to 128 KiB. Pictures are decorative: the name is
always written next to them.

### Consequences

* Good, because people are recognisable at a glance, and most need to do nothing.
* Good, because devices make no requests to third parties; a field server without
  internet simply keeps the pictures it has.
* Bad, because the Gravatar lookup sends a hash of the e-mail address to Gravatar
  (Automattic, USA) at each login. Organisations that do not want this, or have no legal
  basis for it, set `ORDO_GRAVATAR=off`.
* Neutral, because every person with an account can see the pictures of others in the
  installation, like their names.

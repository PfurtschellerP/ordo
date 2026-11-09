---
status: accepted
date: 2026-11-09
---

# Attachments: encrypted, content-addressed blobs in an object store

## Context and Problem Statement

Photos, and later documents, ECG strips or audio, are much larger than events. Inside
events ([ADR-0028](0028-patient-photos-in-sealed-events.md)) they reach every device of
the compartment at once, bloat the event store and the sync, and fill the prototype's
browser storage after a few dozen photos. The design must stay offline-first, need-to-know
and federated.

## Considered Options

* Photos inside events (ADR-0028)
* Blobs as rows in the event database (SQLite/PostgreSQL)
* Events keep a reference; the encrypted blob lives in an object store and travels separately

## Decision Outcome

Chosen option: "Events keep a reference; the blob lives in an object store" (accepted by
the project lead on 2026-11-09), because
the event log stays small and fast, and blob storage can scale on its own.

* **Encrypted on the device.** The device encrypts each blob with a fresh random key
  (XChaCha20-Poly1305). That key travels only inside the sealed event of the protected
  compartment. Whoever may read the event can open the blob; the object store only ever
  sees ciphertext.
* **Content-addressed.** The event names the blob by the BLAKE3 hash of its ciphertext,
  plus size and media type. Events are signed, so every copy of a blob can be checked,
  and identical blobs are stored once.
* **A small preview stays inline.** The event carries a thumbnail of a few kilobytes, so
  lists and records show something at once, even offline.
* **Separate transfer.** Blobs are uploaded and downloaded over HTTP (`/blobs/<hash>`),
  not over the event WebSocket, with the same session and the same need-to-know check as
  the event. The device keeps an upload queue like its event outbox. The server accepts
  the event before the blob arrives; other devices show "not loaded yet" until it does.
* **Storage behind one interface.** The server uses the `object_store` crate (Apache
  Arrow, Apache-2.0): a local directory by default (single binary, field servers), or an
  S3-compatible store for high availability and distribution.
* **Only fully open-source S3-compatible stores.** Supported and tested is what is free
  software as a whole, without an open-core or commercial-only edition: the reference is
  [Garage](https://garagehq.deuxfleurs.fr) (AGPL-3.0, Deuxfleurs; designed for small
  geo-distributed self-hosting, runs on modest hardware), offered in the Compose bundle
  and used in the tests. Ceph RGW (LGPL) and SeaweedFS (Apache-2.0) also qualify. MinIO
  is not supported: its community edition lost its management features in 2025, binaries
  stopped, and the repository was archived in February 2026. Devices keep blobs in the browser's file system (OPFS,
  [ADR-0034](0034-device-storage-sqlite-opfs.md)) and fetch full images on demand or for
  the patients of their own UHS.
* **Federation.** Partner servers fetch blobs from the home server under the same
  federation agreement as events.
* **Retention.** Blobs follow the retention of their compartment (EPCR-8). After deletion
  the event keeps the hash, so the history shows that a photo existed.

### Consequences

* Good, because events stay small; large media no longer limit the number of patients.
* Good, because the object store never sees plaintext and can be any S3-compatible system.
* Good, because offline use stays intact: the taking device has the photo at once,
  others get the preview at once and the full image when connected.
* Bad, because a second transfer path needs its own tests (resume, retries, quota).

## More Information

Supersedes ADR-0028. Implemented together with the new device storage (ADR-0034), right
after the first MANV features (roadmap).

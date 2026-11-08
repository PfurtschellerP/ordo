---
status: proposed
date: 2026-11-08
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

Proposed option: "Events keep a reference; the blob lives in an object store", because
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
  Arrow): a local directory by default (single binary), or any S3-compatible store for
  high availability and distribution, e.g. Garage (designed for small geo-distributed
  self-hosting) or Ceph. Devices keep blobs in the browser's file system (OPFS,
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

Supersedes ADR-0028. Implemented together with the new device storage (roadmap).

---
status: accepted
date: 2026-11-10
---

# API versions: `major.minor` per specification, negotiated per connection and request

## Context and Problem Statement

Devices work offline for days and then reconnect to a server that was updated in between;
field servers run older releases than the organisation's main server; federation partners
upgrade on their own schedule. A change that breaks an older client must not take it out
of service in the middle of an incident. [API-2](../requirements/06-federation-and-interfaces.md#api-2--independent-spec-versions)
asks for specifications versioned apart from the software, and for servers that announce
which versions they support (architecture 8.7: `/.well-known/ordo`). So far neither the
sync protocol nor the HTTP API carries a version.

## Considered Options

* No versions: clients and server must always be updated together
* A version in the URL (`/v1/…`), as many HTTP APIs do
* Microversions (as in OpenStack): every request or connection names the `major.minor`
  version the client speaks; the server announces its supported range and answers in the
  version asked for

## Decision Outcome

Chosen option: microversions, one version line per specification:

* **Format:** `major.minor`, e.g. `1.4`. A change that adds something a client may ignore
  (a new message, an optional field) raises `minor`; a change that breaks older clients
  raises `major`. Each version is recorded in the specification's changelog.
* **Sync protocol:** the server's `challenge` names the range it supports (`versions`:
  `min`, `max`). The device answers with `version` in its `hello`; the server answers in
  that version and confirms it in `welcome` (`version`). A `hello` without a version is
  the oldest version, `1.0`, so devices from before this decision keep working. A version
  outside the range is refused with `unsupportedVersion`. The server sends nothing newer
  than the version agreed on that connection.
* **HTTP API:** a request names the version in the header `ORDO-API-Version: 1.0`;
  without it, the oldest supported version applies. The response repeats the version it
  answers in. A version outside the range gets `406 Not Acceptable` with the supported
  range.
* **Discovery:** `GET /.well-known/ordo` lists the supported range of each specification,
  so tools and partners can check before connecting.
* **Event payloads** keep their own `schema_version` per event type (rule 3 of AGENTS.md):
  they are signed and forwarded unchanged, so they are versioned apart from the protocol.
* **Tests:** CI connects with the oldest supported version and without a version, besides
  the newest.
* The first version of both is `1.0`: draft 1 of the sync protocol and the HTTP endpoints
  as they are now. Federation gets its own version line when it is specified.

### Consequences

* Good, because a server can move on while old devices and partners keep working, and a
  device that is too old learns so clearly instead of failing on a message it cannot read.
* Good, because the version is in every exchange, so problems in the field can be traced
  to the versions involved.
* Bad, because the server must keep the behaviour of every supported version until it
  drops it from the range; dropping a version is itself a breaking change and is announced
  ahead.

## More Information

* [OpenStack microversion specification](https://specs.openstack.org/openstack/api-sig/guidelines/microversion_specification.html)
* Specification: [`spec/sync/protocol.md`](../../spec/sync/protocol.md) §11 Versions

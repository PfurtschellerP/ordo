# Database migrations of the server

Versioned schema changes of the server's SQLite database, applied by sqlx at start in
version order, each exactly once. The table `_sqlx_migrations` records them with a
checksum.

- One change, one new file: `NNNN_what_it_does.sql`, numbered after the last one.
- Never edit or delete a migration that has been released. sqlx refuses to start if an
  applied migration's checksum changed, and that is intended.
- `0001_initial.sql` is the schema at the first release. It uses `IF NOT EXISTS` so that
  databases from before the first release adopt it.
- Data in the database is immutable events and wrapped keys: a migration changes tables
  and indexes, never stored events ([ADR-0001](../../../docs/adr/0001-local-first-event-sourcing.md)).

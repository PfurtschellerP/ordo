-- Display names (ORG-7): the identity provider's, stored at every login, and one the
-- person chose, which wins while it is set.
CREATE TABLE people (
    user_id TEXT PRIMARY KEY,
    provider_name TEXT,
    chosen_name TEXT
) STRICT;

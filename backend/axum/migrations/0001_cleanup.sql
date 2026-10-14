-- The settings of the cleanup: one row, none until they are changed from their defaults.
CREATE TABLE settings (
    id             INTEGER PRIMARY KEY CHECK (id = 1),
    temporary_days INTEGER NOT NULL,
    journal_mb     INTEGER NOT NULL,
    notify         INTEGER NOT NULL CHECK (notify IN (0, 1))
) STRICT;

-- The items chosen in the page the last time, as a JSON array of their ids.
CREATE TABLE selection (
    id    INTEGER PRIMARY KEY CHECK (id = 1),
    items TEXT    NOT NULL
) STRICT;

-- The scheduled cleanups: when (a JSON schedule) and what (a JSON array of item ids).
CREATE TABLE rules (
    id         TEXT    PRIMARY KEY,
    name       TEXT    NOT NULL,
    enabled    INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    schedule   TEXT    NOT NULL,
    items      TEXT    NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

-- The cleanups done, by hand (no rule) or by a rule (its name then), with what each item freed.
CREATE TABLE history (
    id      INTEGER PRIMARY KEY,
    at      INTEGER NOT NULL,
    rule    TEXT,
    freed   INTEGER NOT NULL,
    results TEXT    NOT NULL
) STRICT;

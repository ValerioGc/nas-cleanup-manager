-- Cleanup settings in a single row, absent until the defaults are changed.
CREATE TABLE settings (
    id             INTEGER PRIMARY KEY CHECK (id = 1),
    temporary_days INTEGER NOT NULL,
    journal_mb     INTEGER NOT NULL,
    notify         INTEGER NOT NULL CHECK (notify IN (0, 1))
) STRICT;

-- Last item selection as a JSON array of ids.
CREATE TABLE selection (
    id    INTEGER PRIMARY KEY CHECK (id = 1),
    items TEXT    NOT NULL
) STRICT;

-- Scheduled cleanups with a JSON schedule and a JSON array of item ids.
CREATE TABLE rules (
    id         TEXT    PRIMARY KEY,
    name       TEXT    NOT NULL,
    enabled    INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    schedule   TEXT    NOT NULL,
    items      TEXT    NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

-- Completed cleanups with the space freed by each item. The rule is null for a manual run.
CREATE TABLE history (
    id      INTEGER PRIMARY KEY,
    at      INTEGER NOT NULL,
    rule    TEXT,
    freed   INTEGER NOT NULL,
    results TEXT    NOT NULL
) STRICT;

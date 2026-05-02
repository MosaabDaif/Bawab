CREATE TABLE IF NOT EXISTS scans (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  student_id TEXT    NOT NULL,
  action     TEXT    NOT NULL CHECK(action IN ('IN', 'OUT')),
  timestamp  TEXT    NOT NULL   -- ISO 8601 / RFC-3339 from chrono::Utc
);

CREATE TABLE IF NOT EXISTS admins (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  username      TEXT    NOT NULL UNIQUE,
  password_hash TEXT    NOT NULL
);

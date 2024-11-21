CREATE TABLE commits (
    id BYTEA PRIMARY KEY,
    subject TEXT NOT NULL,
    message TEXT,
    parent BYTEA REFERENCES commits (id) ON DELETE SET NULL,
    merge_parent BYTEA REFERENCES commits (id) ON DELETE SET NULL,
    commit_time TIMESTAMP NOT NULL
);
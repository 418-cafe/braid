-- objects managed by an external consumer of the braid library
CREATE TABLE "external_object" (
    id bytea PRIMARY KEY
);

CREATE TABLE "user" (
    "id" VARCHAR(255) PRIMARY KEY
);

CREATE TABLE "register" (
    "id" bytea PRIMARY KEY
);

CREATE TABLE "register_entry" (
    "register" bytea NOT NULL REFERENCES "register"(id),
    "key" VARCHAR(1023) NOT NULL,
    "entry_external_object" bytea REFERENCES "external_object"(id),
    "entry_register" bytea REFERENCES "register"(id),

    CONSTRAINT external_object_and_register_not_both_null CHECK ("entry_external_object" IS NOT NULL OR "entry_register" IS NOT NULL),
    CONSTRAINT external_object_and_register_one_is_null CHECK ("entry_external_object" IS NULL OR "entry_register" IS NULL),

    PRIMARY KEY (register, "key")
);

-- individual commits, regardless of how they are applied. this allows
-- commits to retain their identity even if they are applied to different
-- parent commits or in a different way
CREATE TABLE "commit" (
    id bytea PRIMARY KEY,
    subject VARCHAR(4095),
    body TEXT,
    author VARCHAR(255) NOT NULL REFERENCES "user"(id),
    authored TIMESTAMPTZ NOT NULL
);

-- the separate implementation of a commit. a commit can have multiple implementations
CREATE TABLE "commit_impl" (
    id bytea PRIMARY KEY,
    "commit" bytea NOT NULL REFERENCES "commit"(id),
    parent bytea REFERENCES commit_impl(id),
    merge_parent bytea REFERENCES commit_impl(id),
    committer VARCHAR(255) NOT NULL REFERENCES "user"(id),
    "committed" TIMESTAMPTZ NOT NULL

    -- todo: tree, etc
);

CREATE TABLE branch (
    name VARCHAR(255) NOT NULL PRIMARY KEY,
    tip bytea NOT NULL REFERENCES commit_impl(id),
    is_default BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE "save" (
    id bytea PRIMARY KEY,
    parent bytea REFERENCES "save"(id),
    "when" TIMESTAMPTZ NOT NULL,
    "content" bytea REFERENCES "external_object"(id)
);

-- committed is null == object is not present at previous commit but has been saved
-- save is null == object has been committed but not saved
-- they cannot both be null because even if never committed, saved once and
--      then deleted and saved again, there is a save record for the delete
CREATE TABLE "state" (
    branch VARCHAR(255) NOT NULL REFERENCES branch(name),
    "key" VARCHAR(4095) NOT NULL,
    "committed" bytea REFERENCES "external_object"(id),
    "save" bytea REFERENCES "save"(id),

    CONSTRAINT committed_and_saved_not_both_null CHECK ("committed" IS NOT NULL OR "save" IS NOT NULL),

    PRIMARY KEY (branch, "key")
);
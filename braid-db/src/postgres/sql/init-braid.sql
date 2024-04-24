CREATE OR REPLACE PROCEDURE braid.init_braid()
LANGUAGE plpgsql
AS $init_braid$
BEGIN
    -- ENUMS
    CREATE TYPE braid.object_kind AS ENUM (
        'content',
        'commit',
        'save',
        'save_register',
        'register'
    );

    CREATE TYPE braid.save_entry_kind AS ENUM (
        'executable',
        'content'
    );

    -- TYPES
    CREATE TYPE braid.register_entry_record AS (
        key varchar(255),
        content bytea,
        is_executable boolean
    );

    CREATE DOMAIN braid.register_entry_records AS braid.register_entry_record[];

    CREATE TYPE braid.save_register_entry_record AS (
        key varchar(255),
        save bytea
    );

    CREATE DOMAIN braid.save_register_entry_records AS braid.save_register_entry_record[];

    -- ODB TABLES
    CREATE TABLE braid.objects (
        id bytea PRIMARY KEY,
        kind braid.object_kind NOT NULL,

        CHECK (octet_length(id) = 32)
    );

    CREATE TABLE braid.content_register (
        id bytea PRIMARY KEY,
        is_content boolean NOT NULL,

        FOREIGN KEY (id) REFERENCES braid.objects(id)
    );

    CREATE TABLE braid.content (
        id bytea PRIMARY KEY,

        FOREIGN KEY (id) REFERENCES braid.content_register(id)
    );

    CREATE TABLE braid.register (
        id bytea PRIMARY KEY,

        FOREIGN KEY (id) REFERENCES braid.content_register(id)
    );

    CREATE TABLE braid.register_entry (
        register bytea NOT NULL,
        key varchar(255) NOT NULL,
        content bytea NOT NULL,
        is_executable boolean NOT NULL,

        PRIMARY KEY (key, content),

        FOREIGN KEY (register) REFERENCES braid.register(id),
        FOREIGN KEY (content) REFERENCES braid.content_register(id),

        CHECK (
            key NOT LIKE E'%\n%' AND
            key NOT LIKE E'%\r%' AND
            key NOT LIKE '%/%'
        )
    );

    CREATE TABLE braid.save_parent (
        id bytea PRIMARY KEY,
        is_commit boolean NOT NULL,

        FOREIGN KEY (id) REFERENCES braid.objects(id)
    );

    CREATE TABLE braid.save (
        id bytea PRIMARY KEY,
        author varchar(255) NOT NULL,
        date timestamp with time zone NOT NULL,
        kind braid.save_entry_kind NOT NULL,
        content bytea NOT NULL,
        parent bytea NOT NULL,

        FOREIGN KEY (id) REFERENCES braid.save_parent (id),
        FOREIGN KEY (parent) REFERENCES braid.save_parent (id),
        FOREIGN KEY (content) REFERENCES braid.content (id)
    );

    CREATE TABLE braid.save_register (
        id bytea PRIMARY KEY,

        FOREIGN KEY (id) REFERENCES braid.objects(id)
    );

    CREATE TABLE braid.save_register_entry (
        save_register bytea NOT NULL,
        key varchar(255) NOT NULL,
        save bytea NOT NULL,

        PRIMARY KEY (save_register, key, save),

        FOREIGN KEY (save_register) REFERENCES braid.save_register (id),
        FOREIGN KEY (save) REFERENCES braid.save (id),

        CHECK (
            key NOT LIKE E'%\n%' AND
            key NOT LIKE E'%\r%'
        )
    );

    CREATE TABLE braid.commit (
        id bytea PRIMARY KEY,
        register bytea NOT NULL,
        parent bytea,
        merge_parent bytea,
        rebase_of bytea,
        saves bytea NOT NULL,
        date timestamp with time zone NOT NULL,
        committer varchar(255) NOT NULL,
        summary text NOT NULL,
        body text NOT NULL,

        FOREIGN KEY (id) REFERENCES braid.save_parent (id),
        FOREIGN KEY (register) REFERENCES braid.register (id),
        FOREIGN KEY (parent) REFERENCES braid.commit (id),
        FOREIGN KEY (merge_parent) REFERENCES braid.commit (id),
        FOREIGN KEY (rebase_of) REFERENCES braid.commit (id),
        FOREIGN KEY (saves) REFERENCES braid.save_register (id)
    );

    -- UPSERTS
    CREATE PROCEDURE braid.create_object(object_id bytea, object_kind braid.object_kind) AS $$
    DECLARE inserted bytea;
    BEGIN
        INSERT INTO braid.objects (id, kind)
        VALUES (object_id, object_kind)
        ON CONFLICT DO NOTHING
        RETURNING id INTO inserted;

        IF inserted IS NULL AND EXISTS (
            SELECT id FROM braid.objects o WHERE o.id = object_id AND o.kind != object_kind
        ) THEN
            RAISE EXCEPTION 'Object with id % already exists and is not a %', id, kind;
        END IF;
    END $$ LANGUAGE plpgsql;

    CREATE PROCEDURE braid.create_content(id bytea) AS $$
    BEGIN
        CALL braid.create_object(id, 'content');

        INSERT INTO braid.content_register (id, is_content)
        VALUES (id, TRUE);

        INSERT INTO braid.content (id)
        VALUES (id);
    END $$ LANGUAGE plpgsql;

    CREATE PROCEDURE braid.create_register(id bytea, entries braid.register_entry_records) AS $$
    DECLARE inserted bytea;
    BEGIN
        CALL braid.create_object(id, 'register');

        INSERT INTO braid.content_register (id, is_content)
        VALUES (id, FALSE)
        ON CONFLICT DO NOTHING;

        INSERT INTO braid.register (id)
        VALUES (id)
        ON CONFLICT DO NOTHING;

        WITH c AS (
            SELECT e.key, e.content, e.is_executable,
                (SELECT is_content FROM braid.content_register AS cr WHERE cr.id = e.content) as is_content
            FROM UNNEST(entries) as e
        )
        INSERT INTO braid.register_entry (register, key, content, is_executable)
        SELECT id, c.key, c.content, CASE WHEN c.is_content THEN c.is_executable ELSE FALSE END
        FROM c
        ON CONFLICT DO NOTHING;
    END $$ LANGUAGE plpgsql;

    CREATE PROCEDURE braid.create_save(id bytea, author varchar(255), date timestamp with time zone,
        kind braid.save_entry_kind, content bytea, parent bytea) AS $$
    BEGIN
        CALL braid.create_object(id, 'save');

        INSERT INTO braid.save_parent (id, is_commit)
        VALUES (id, FALSE)
        ON CONFLICT DO NOTHING;

        INSERT INTO braid.save (id, author, date, kind, content, parent)
        VALUES (id, author, date, kind, content, parent)
        ON CONFLICT DO NOTHING;
    END $$ LANGUAGE plpgsql;

    CREATE PROCEDURE braid.create_save_register(id bytea, entries braid.save_register_entry_records) AS $$
    BEGIN
        CALL braid.create_object(id, 'save_register');

        INSERT INTO braid.save_register (id)
        VALUES (id)
        ON CONFLICT DO NOTHING;

        WITH c AS (
            SELECT e.key, e.save
            FROM UNNEST(entries) as e
        )
        INSERT INTO braid.save_register_entry (save_register, key, save)
        SELECT id, c.key, c.save
        FROM c
        ON CONFLICT DO NOTHING;
    END $$ LANGUAGE plpgsql;

    CREATE PROCEDURE braid.create_commit(id bytea, register bytea, parent bytea, merge_parent bytea, rebase_of bytea,
        saves bytea, date timestamp with time zone, committer varchar(255), summary text, body text) AS $$
    BEGIN
        CALL braid.create_object(id, 'commit');

        INSERT INTO braid.save_parent (id, is_commit)
        VALUES (id, TRUE)
        ON CONFLICT DO NOTHING;

        INSERT INTO braid.commit (id, register, parent, merge_parent, rebase_of, saves, date, committer, summary, body)
        VALUES (id, register, parent, merge_parent, rebase_of, saves, date, committer, summary, body)
        ON CONFLICT DO NOTHING;
    END $$ LANGUAGE plpgsql;

    -- READS
    CREATE FUNCTION braid.get_register(register_id bytea)
    RETURNS TABLE(key varchar(255), content bytea, is_executable boolean, is_content boolean) AS $$
    BEGIN
        RETURN QUERY
        SELECT re.key, re.content, re.is_executable, cr.is_content
        FROM braid.register_entry AS re
        INNER JOIN braid.content_register AS cr ON re.content = cr.id
        WHERE re.register = register_id;
    END $$ LANGUAGE plpgsql;

    CREATE FUNCTION braid.get_save(save_id bytea)
    RETURNS TABLE(id bytea, author varchar(255), date timestamp with time zone,
        kind braid.save_entry_kind, content bytea, parent bytea, is_commit boolean) AS $$
    BEGIN
        RETURN QUERY
        SELECT s.id, s.author, s.date, s.kind, s.content, s.parent, sp.is_commit
        FROM braid.save as s
        INNER JOIN braid.save_parent as sp ON s.parent = sp.id
        WHERE s.id = save_id;
    END $$ LANGUAGE plpgsql;

    CREATE FUNCTION braid.get_save_register(save_register_id bytea)
    RETURNS TABLE(key varchar(255), save bytea) AS $$
    BEGIN
        RETURN QUERY
        SELECT sre.key, sre.save
        FROM braid.save_register_entry as sre
        WHERE sre.save_register = save_register_id;
    END $$ LANGUAGE plpgsql;

    CREATE FUNCTION braid.get_commit(commit_id bytea)
    RETURNS TABLE(id bytea, register bytea, parent bytea, merge_parent bytea, rebase_of bytea,
        saves bytea, date timestamp with time zone, committer varchar(255), summary text, body text) AS $$
    BEGIN
        RETURN QUERY
        SELECT c.id, c.register, c.parent, c.merge_parent, c.rebase_of, c.saves, c.date, c.committer, c.summary, c.body
        FROM braid.commit as c
        WHERE c.id = commit_id;
    END $$ LANGUAGE plpgsql;
END;
$init_braid$;
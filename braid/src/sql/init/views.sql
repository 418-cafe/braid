CREATE VIEW save_lineage AS
WITH RECURSIVE lineage AS (
    SELECT
        sv.id,
        st.branch,
        st."key",
        sv.parent,
        sv."when",
        sv."content",
        0 AS depth
    FROM "state" st
    JOIN "save" sv
    ON st.save = sv.id

    UNION ALL

    SELECT
        sv.id,
        l.branch,
        l."key",
        sv.parent,
        sv."when",
        sv."content",
        l.depth + 1
    FROM lineage l
    JOIN save sv
    ON l.parent = sv.id
)
SELECT
    id,
    branch,
    "key",
    parent,
    "when",
    "content",
    depth
FROM
    lineage;

CREATE OR REPLACE VIEW register_path AS
WITH RECURSIVE register_cte AS (
    SELECT
        re.register AS root_register,
        CAST(re.key AS TEXT) AS path,
        re.entry_external_object AS external_object,
        re.entry_register
    FROM register_entry re

    UNION ALL

    SELECT
        rc.root_register,
        rc.path || '/' || re.key AS path,
        re.entry_external_object,
        re.entry_register
    FROM register_cte rc
    JOIN register_entry re
    ON rc.entry_register = re.register
)
SELECT
    root_register,
    path,
    external_object
FROM register_cte
WHERE external_object IS NOT NULL;

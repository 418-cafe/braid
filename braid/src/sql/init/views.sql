CREATE VIEW save_lineage AS
WITH RECURSIVE lineage AS (
    -- Anchor: Start with the save referenced in the state table
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

    -- Recursive part: Get the parent save recursively
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

-- Two more fixed columns at the end of every board: Done (the crew finished it and someone
-- confirmed it) and Closed (won't be done). An issue in either is closed, and its chat is
-- read-only. Columns people add sit between Ready to dev and Done.

ALTER TABLE project_issue_columns DROP CONSTRAINT project_issue_columns_fixed_check;
ALTER TABLE project_issue_columns
    ADD CONSTRAINT project_issue_columns_fixed_check CHECK (fixed IN ('backlog', 'ready', 'done', 'closed'));

INSERT INTO project_issue_columns (project_id, name, position, fixed)
-- 1000 and 1001 are DONE_POSITION and CLOSED_POSITION in routes/project_issues.rs.
SELECT id, 'Done', 1000, 'done' FROM projects
UNION ALL
SELECT id, 'Closed', 1001, 'closed' FROM projects
ON CONFLICT (project_id, fixed) DO NOTHING;

-- Issues already closed: the crew's finished ones to Done, the rest to Closed, oldest first.
UPDATE project_issues i SET column_id = c.id, position = r.n
FROM project_issue_columns c,
     (SELECT id, row_number() OVER (PARTITION BY project_id ORDER BY closed_at NULLS FIRST, number) AS n
      FROM project_issues WHERE status = 'closed') r
WHERE r.id = i.id AND c.project_id = i.project_id
  AND c.fixed = CASE WHEN i.crew_state = 'done' THEN 'done' ELSE 'closed' END;

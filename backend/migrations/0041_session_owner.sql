-- Every chat belongs to the person who started it. Sessions used to be scoped only to an
-- organization, so in a shared org one member could list, open and post into another member's
-- chats, and Home surfaced their replies and to-dos: their memories and private conversation
-- leaked to colleagues. Access is now checked against this owner.
ALTER TABLE sessions ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE CASCADE;

-- Backfill: whoever sent the chat's first message, then whatever else records who the chat was for.
UPDATE sessions s SET user_id = (
    SELECT ci.user_id FROM messages m JOIN channel_identities ci ON ci.id = m.sender_channel_identity_id
    WHERE m.session_id = s.id ORDER BY m.created_at LIMIT 1
) WHERE s.user_id IS NULL;

UPDATE sessions s SET user_id = (
    SELECT ci.user_id FROM turn_jobs tj JOIN channel_identities ci ON ci.id = tj.sender_channel_identity_id
    WHERE tj.session_id = s.id ORDER BY tj.created_at LIMIT 1
) WHERE s.user_id IS NULL;

UPDATE sessions s SET user_id = (SELECT p.user_id FROM projects p WHERE p.session_id = s.id ORDER BY p.created_at LIMIT 1)
WHERE s.user_id IS NULL;

UPDATE sessions s SET user_id = (SELECT r.user_id FROM reminders r WHERE r.session_id = s.id ORDER BY r.created_at LIMIT 1)
WHERE s.user_id IS NULL;

UPDATE sessions s SET user_id = (
    SELECT ci.user_id FROM agent_sessions a JOIN channel_identities ci ON ci.id = a.sender_channel_identity_id
    WHERE a.session_id = s.id ORDER BY a.started_at LIMIT 1
) WHERE s.user_id IS NULL;

UPDATE sessions s SET user_id = (SELECT d.user_id FROM agent_delegations d WHERE d.session_id = s.id ORDER BY d.created_at LIMIT 1)
WHERE s.user_id IS NULL;

UPDATE sessions s SET user_id = (SELECT j.user_id FROM scheduled_jobs j WHERE j.session_id = s.id ORDER BY j.created_at LIMIT 1)
WHERE s.user_id IS NULL;

UPDATE sessions s SET user_id = (SELECT p.user_id FROM agent_plans p WHERE p.session_id = s.id ORDER BY p.created_at LIMIT 1)
WHERE s.user_id IS NULL;

-- Still unknown: in an organization of one, the chat can only have been theirs. Anything left
-- after that (a chat with no trace of who started it, in a shared org) stays ownerless, and
-- nobody can open it: the safe outcome when there's no telling whose it is.
UPDATE sessions s SET user_id = (SELECT min(m.user_id::text)::uuid FROM memberships m WHERE m.org_id = s.org_id)
WHERE s.user_id IS NULL AND (SELECT count(*) FROM memberships m WHERE m.org_id = s.org_id) = 1;

CREATE INDEX sessions_user_id_idx ON sessions (user_id);

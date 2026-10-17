-- A project moved into an organization reaches its owners.
ALTER TABLE notifications DROP CONSTRAINT IF EXISTS notifications_kind_check;
ALTER TABLE notifications ADD CONSTRAINT notifications_kind_check
    CHECK (kind IN ('account', 'subscription', 'quota', 'promo', 'deploy', 'project'));

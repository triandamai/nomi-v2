-- migrations/0101_profile_extensions.sql
ALTER TABLE user_profiles
    ADD COLUMN IF NOT EXISTS location TEXT,
    ADD COLUMN IF NOT EXISTS show_stats_publicly BOOLEAN NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS show_crew_publicly BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS showcase_project_ids UUID[] NOT NULL DEFAULT '{}'::uuid[];

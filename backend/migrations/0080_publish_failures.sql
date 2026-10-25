-- Why a publish failed, as Shipyard reported it on the failed deployment: the step and phase
-- (validate, build, configure, deploy, finalize), its message and the end of the log, plus
-- what the app printed when it failed while starting. The Deploy sheet shows it, and the
-- project's chat gets it so Koda can fix the app and publish again.
ALTER TABLE project_publishes ADD COLUMN failure JSONB;

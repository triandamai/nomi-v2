// A project's versions as the Versions tab lists them (backend routes::project_versions).
import type { ProjectVersion } from '$lib/types';

/** Who made it: the crew member, else the person; null when unknown. */
export function versionAuthor(version: ProjectVersion): string | null {
	return version.by.agent ?? version.by.name;
}

/** Restore is offered for any kept version but the one the project is on. */
export function canRestore(version: ProjectVersion): boolean {
	return version.restorable && !version.current;
}

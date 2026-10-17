import type { CrewRosterMember } from '$lib/crew';
import type { HomeSummary, Preferences, Profile, SessionSummary } from '$lib/types';

/** Everything home blocks read: the home route's load plus the app layout's. */
export interface HomeData {
	profile: Profile;
	preferences: Preferences;
	crew: CrewRosterMember[];
	recentSessions: SessionSummary[];
	summary: HomeSummary | null;
}

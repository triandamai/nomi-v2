// The backend's own shapes, as the app shell reads them (backend/crates/nomi-server/src/routes).
// Only the fields v2 uses; the adapters next to this file turn them into the design system's props.

/** `GET /api/profile`. */
export type Profile = {
	display_name: string | null;
	username: string | null;
	email: string;
	avatar_url: string | null;
};

/** `GET /api/preferences`. */
export type Preferences = {
	theme: 'light' | 'dark' | 'system';
	timezone: string;
	language: string;
	/** False until the person saved a timezone: `timezone` is then only the UTC fallback. */
	has_stored_timezone: boolean;
};

/** `GET /api/usage/brief`. */
export type UsageBrief = {
	plan: { id: string; name: string; monthly_tokens: number; custom_quota: boolean };
	month: string;
	tokens_used: number;
	/** A bigger plan is offered than theirs. */
	can_upgrade: boolean;
};

/** One crew member in `GET /api/agents` (`{ agents }`). */
export type CrewMember = {
	agent_type: string;
	name: string;
	role: string;
	is_dynamic: boolean;
	/** What it's doing for this person right now, across all chats. */
	state: 'working' | 'waiting' | 'done' | 'idle';
	status: string;
};

/** One chat in `GET /api/sessions` (`{ sessions }`), latest first. */
export type SessionSummary = {
	id: string;
	title: string | null;
	last_message: { content: string; created_at: string } | null;
	updated_at: string;
	/** The project this chat is the own chat of (Build). */
	project_id: string | null;
	pinned: boolean;
	in_project_id: string | null;
};

export type OrgRole = 'owner' | 'admin' | 'member';

/** One workspace in `GET /api/orgs`. */
export type OrgSummary = { id: string; name: string; role: OrgRole; members: number; projects: number };

/** `GET /api/orgs`: the person's workspaces, and the one projects are listed from (null until picked). */
export type OrgList = { orgs: OrgSummary[]; project_org_id: string | null };

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
	/** Since when it's pinned (newest pin first is the list's job). */
	pinned_at?: string | null;
	in_project_id: string | null;
	/** A crew member is at work in it right now. */
	agent_active?: boolean;
};

export type OrgRole = 'owner' | 'admin' | 'member';

/** One workspace in `GET /api/orgs`. */
export type OrgSummary = { id: string; name: string; role: OrgRole; members: number; projects: number };

/** `GET /api/orgs`: the person's workspaces, and the one projects are listed from (null until picked). */
export type OrgList = { orgs: OrgSummary[]; project_org_id: string | null };

// The chat's shapes (backend/crates/nomi-server/src/routes/sessions.rs, message_rerun.rs and
// model_options.rs). The content blocks the chat core reads are typed; the rest pass through as
// `{ kind }` for the rich replies to draw.

/** What a failed run's Try again reruns (nomi_agent_core::content_block::RetryTarget). */
export type RetryTarget = { target: 'turn'; turn_job_id: string } | { target: 'delegation'; delegation_id: string } | { target: 'last_message' };

/**
 * A message's content block. `reasoning` is the crew's thinking; `run_failed` an error bubble
 * (with a retry when trying again can help); `quota_notice` the month's allowance used up;
 * `suggestions` follow-ups to ask; `stopped` what a stop ended.
 */
export type ContentBlock = { kind: string; [field: string]: unknown };

/** The blocks the chat core reads, by their fields. */
export type KnownBlock =
	| { kind: 'reasoning'; text: string }
	| { kind: 'run_failed'; retry?: RetryTarget }
	| { kind: 'quota_notice'; reason: string }
	| { kind: 'suggestions'; items: string[] }
	| { kind: 'stopped'; tasks: unknown[] };

export type MessageMemory = { id: string; content: string; kind: string };

/** One message in `GET /api/sessions/:id/messages` (`{ messages, has_more }`), oldest first. */
export type MessageItem = {
	id: string;
	sender: 'user' | 'assistant';
	content: string;
	content_blocks: ContentBlock[] | null;
	created_at: string;
	my_feedback: 'up' | 'down' | null;
	agent_display_name: string | null;
	memory_count: number;
	steps?: { tool: string; detail: string | null; ok: boolean; output?: string }[];
	sources?: { title: string | null; url: string }[];
	/** A teammate's message in a shared project chat. */
	author_name?: string;
	edited_at?: string | null;
	/** Earlier takes of the person's message and its replies. */
	versions?: number;
	saved_memories?: MessageMemory[];
	used_memories?: MessageMemory[];
};

/** A message with its markdown rendered (and sanitised) on the server. */
export type RenderedMessage = MessageItem & { content_html: string };

/** `POST /api/sessions/:id/messages`: the person's message, and the supervisor's answer when it
 * was a stop (nothing is queued then). */
export type SendResult = { user_message: MessageItem; supervisor_reply?: MessageItem };

/** `POST …/regenerate` and `PATCH …/messages/:id`: the latest turn running again. */
export type RerunResult = { message_id: string; version: number; total: number; edited_at: string | null };

/** One earlier take of a person's message (`GET …/versions`), its replies rendered. */
export type MessageVersion = {
	version: number;
	text: string;
	replies: { content: string; content_html: string; agent_display_name: string | null; created_at: string }[];
	created_at: string;
};

/** `GET /api/sessions/:id/thinking` (`Default` in the picker is medium). */
export type ThinkingLevel = 'low' | 'medium' | 'high' | 'max';

/** One model a chat can use. `key` is `admin:<id>` or `conn:<connection>:<model>`. */
export type ModelOption = { key: string; label: string; model_id: string; free: boolean; enabled: boolean };

/** `GET /api/llm/model-options`: models grouped by who pays, Nomi's credits (`nomi`) or the
 * person's own key (`connection`). */
export type ModelGroup = {
	id: string;
	source: 'nomi' | 'connection';
	provider_id: string;
	has_logo: boolean;
	label: string;
	connection_id: string | null;
	error: 'unreachable' | null;
	models: ModelOption[];
};

/** The model a chat answers with (or the person's default). */
export type ModelChoice = { key: string; label: string; model_id: string; provider_id: string; provider_label: string; has_logo: boolean; chat_only: boolean };

export type ModelOptions = { groups: ModelGroup[]; current: ModelChoice | null };

/** `GET /api/projects`: the chosen workspace's projects, which a chat can be filed under. */
export type ProjectList = { org_id: string | null; projects: { id: string; name: string }[] };

/** `GET /api/home`: what Home's Today strip sums up (each list holds at most four). */
export type HomeSummary = {
	since: string;
	timezone: string;
	while_you_were_out: { kind: string; agent: string; title: string; detail: string; session_id: string | null; at: string }[];
	today: { id: string; run_at: string; label: string; agent: string; recurrence: string | null }[];
	plans: { kind: 'todo' | 'plan'; title: string; agent: string; done: number; total: number; session_id: string; updated_at: string }[];
	while_you_were_out_total: number;
	today_total: number;
	plans_total: number;
};

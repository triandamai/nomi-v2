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

/** One notification in `GET /api/notifications` (kind: account, subscription, quota, promo, deploy, project). */
export type NotificationItem = { id: string; kind: string; title: string; body: string; link: string | null; read: boolean; created_at: string };

/** `GET /api/notifications?filter=&page=`: a page (20) of the inbox, newest first, with each filter's count. */
export type NotificationPage = {
	items: NotificationItem[];
	unread: number;
	page: number;
	pages: number;
	page_size: number;
	total: number;
	counts: { all: number; unread: number; read: number; promo: number };
};

/** A reminder Kala keeps (status: active, fired, done or cancelled; recurrence: daily, weekly or monthly). */
export type ReminderItem = {
	id: string;
	session_id: string;
	title: string;
	notes: string | null;
	due_at: string;
	recurrence: string | null;
	recurrence_weekday: number | null;
	recurrence_day_of_month: number | null;
	status: string;
	created_by: string;
	last_fired_at: string | null;
	created_at: string;
};

/** Work an agent will do later (the core scheduler), listed beside the reminders. */
export type ScheduledTask = { id: string; label: string; run_at: string; recurrence: string | null; agent: string; session_id: string };

/** `GET /api/reminders`: the person's zone, active reminders, the last 20 finished, and scheduled work. */
export type RemindersList = { timezone: string; upcoming: ReminderItem[]; past: ReminderItem[]; scheduled_tasks: ScheduledTask[] };

/** `GET|PUT /api/notifications/preferences`: which emails the person gets. */
export type EmailPreferences = { email_account: boolean; email_promos: boolean };

/** `GET|PUT /api/profile`: the person's profile with what they've done (activity days are in their zone). */
export type ProfileDetails = {
	display_name: string | null;
	username: string | null;
	email: string;
	avatar_url: string | null;
	location: string | null;
	joined_at: string;
	plan_name: string;
	show_stats_publicly: boolean;
	show_crew_publicly: boolean;
	showcase_project_ids: string[];
	showcase_projects: { id: string; name: string; description: string | null; status: string; deploy_url: string | null; build_duration_secs: number | null }[];
	stats: {
		lifetime_tokens: number;
		peak_day_tokens: number;
		peak_day_date: string | null;
		longest_task_secs: number | null;
		longest_task_name: string | null;
		longest_streak_days: number;
		/** "1-9 Aug". */
		longest_streak_range: string | null;
		current_streak_days: number;
		best_streak_target: number;
	};
	activity: { date: string; tokens: number }[];
	/** The last 30 days. */
	top_models: { label: string; tokens: number; percentage: number }[];
	/** kind: "chats" or "projects". */
	crew_usage: { agent_id: string; name: string; count: number; kind: string }[];
};

/** `GET /api/auth/google`: how the person signs in (configured: whether this Nomi offers Google). */
export type SignInMethods = { configured: boolean; password: boolean; google_email: string | null };

/** One of the person's signed-in devices (`GET /api/devices`); `current` is the one asking. */
export type DeviceItem = {
	id: string;
	platform: string;
	name: string | null;
	brand: string | null;
	model: string | null;
	os_name: string | null;
	os_version: string | null;
	app_version: string;
	presence: string;
	last_seen_at: string;
	created_at: string;
	current: boolean;
};

/** `GET /api/usage?month=YYYY-MM`: one month of the person's usage, days in their zone. */
export type UsageMonth = {
	plan: { id: string; name: string; card_tone: string; monthly_tokens: number; custom_quota: boolean };
	month: string;
	current_month: string;
	timezone: string;
	/** What the allowance counts: Nomi's own models, cached input at a tenth. */
	tokens_used: number;
	input_tokens: number;
	output_tokens: number;
	cached_input_tokens: number;
	own_key_tokens: number;
	calls: number;
	spend_usd: number;
	by_day: { date: string; tokens: number; spend_usd: number }[];
	/** source: "nomi" or "own_key". */
	by_model: { admin_model_id: string | null; label: string; provider: string; model_id: string; source: string; tokens: number; calls: number; spend_usd: number }[];
};

/** A plan people can be on (`GET /api/plans`). */
export type PlanOffer = {
	id: string;
	slug: string;
	name: string;
	description: string;
	monthly_tokens: number;
	price_label: string;
	features: string[];
	card_tone: string;
	promo_label: string | null;
	promo_price_label: string | null;
	promo_ends_at: string | null;
	is_default: boolean;
	is_active: boolean;
	sort_order: number;
};

export type PlansForUser = { plans: PlanOffer[]; current_plan_id: string; monthly_tokens: number; custom_quota: boolean; override_until: string | null };

/** One of the person's own provider keys (`GET /api/llm/connections`). */
export type OwnConnection = { id: string; provider_id: string; label: string; auth_type: string; kind: string; base_url: string | null; api_key_masked: string | null; created_at: string };

/** A provider Nomi can connect to (`GET /api/llm/providers`). */
export type ProviderDefinition = {
	id: string;
	name: string;
	description: string;
	has_logo: boolean;
	category: 'popular' | 'other' | 'custom';
	recommended: boolean;
	auth_type: 'api_key' | 'oauth' | 'custom_endpoint';
	kind: string;
	default_base_url: string | null;
	doc_url: string | null;
	model_count: number;
};

/** `GET /api/llm/models`: Nomi's models, and which one Koda builds with. */
export type UserModels = {
	admin_models: { id: string; label: string; provider: string; provider_label: string; model_id: string }[];
	coding: { kind: 'default' } | { kind: 'same_as_chat' } | { kind: 'admin'; admin_model_id: string };
	default_coding_model_id: string | null;
};

/** One thing the crew remembers (`GET /api/memory`); weight 0.1–5, starting at 1. */
export type MemoryItem = {
	id: string;
	content: string;
	weight: number;
	/** preference, person, routine, goal or fact. */
	kind: string;
	created_at: string;
	updated_at: string;
	uses: number;
	last_used_at: string | null;
	needs_check: boolean;
};

/** `GET /api/connections`: Google (the services granted) and Hevy, as connected now. */
export type ConnectionsOverview = {
	google: { configured: boolean; email: string | null; services: string[] };
	hevy: { provider: string; account_name: string | null; connected_at: string; last_used_at: string | null } | null;
};

/** `GET /api/connections/github`: oauth says whether "Connect GitHub" can sign in on GitHub. */
export type GithubStatus = { oauth: boolean; connection: { login: string; kind: string; created_at: string; last_used_at: string | null } | null };

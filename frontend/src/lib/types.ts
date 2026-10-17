import type { Locale } from '$lib/i18n';
export interface SessionSummary {
	id: string;
	channel: string;
	chat_type: string;
	chat_id: string;
	title: string | null;
	last_message: { content: string; created_at: string } | null;
	agent_active: boolean;
	updated_at: string;
	project_id: string | null;
}

export interface TodoItem {
	id: string;
	text: string;
	status: 'pending' | 'in_progress' | 'done';
}

export interface TableColumn {
	key: string;
	label: string;
}

export type ContentBlock =
	| { kind: 'file_write'; project_id: string; path: string; content: string; previous_content: string | null }
	| { kind: 'file_writes'; project_id: string; files: { path: string; content: string; previous_content: string | null }[] }
	| { kind: 'file_delete'; project_id: string; path: string; previous_content?: string | null }
	| { kind: 'todo_list'; items: TodoItem[] }
	| { kind: 'table'; variant: 'data' | 'comparison'; columns: TableColumn[]; rows: Record<string, unknown>[] }
	| { kind: 'workspace_connect'; reason: 'not_connected' | 'service_not_allowed' | string; services: string[] }
	/** A connector the request needs (Connections → its page); `reason` is not_connected or key_rejected. */
	| { kind: 'connector_connect'; connector: string; reason: 'not_connected' | 'key_rejected' | string }
	| { kind: 'project_link'; project_id: string; session_id: string; name: string }
	| { kind: 'quota_notice'; reason: string }
	| {
			kind: 'questions';
			intro: string | null;
			questions: { question: string; options: { label: string; detail?: string | null }[]; multiple?: boolean }[];
	  }
	| {
			kind: 'approval_request';
			id: string;
			tool_name: string;
			description: string;
			input: Record<string, unknown>;
			/** `cancelled`: the user stopped the agent before deciding. */
			status: 'pending' | 'approved' | 'denied' | 'cancelled';
			decided_at: string | null;
	  }
	| { kind: 'plan'; plan_id: string; agent_session_id: string; title: string; version: number }
	/** The model's thinking; shown collapsed on the agent's next reply (see $lib/reasoning). */
	| { kind: 'reasoning'; text: string }
	/** Something the crew read out loud (the speak tool); `url` is the recording. */
	| { kind: 'voice'; url: string; text: string; voice: string | null }
	/** A reminder going off, posted by the Reminders agent. */
	| {
			kind: 'reminder';
			reminder_id: string;
			title: string;
			notes: string | null;
			due_at: string;
			recurrence: string | null;
			/** Where the reminder stands now (added when the chat loads). */
			state?: 'open' | 'done' | 'snoozed' | 'cancelled' | 'gone';
			snoozed_until?: string;
	  };

export interface MessageItem {
	id: string;
	sender: 'user' | 'assistant';
	content: string;
	content_blocks: ContentBlock[] | null;
	created_at: string;
	my_feedback: 'up' | 'down' | null;
	agent_display_name: string | null;
	/** How many memories this reply drew on. */
	memory_count: number;
	/** What the crew did on the way to this reply (tools used), oldest first. */
	steps?: ReplyStep[];
	/** Pages the crew read or found for this reply. */
	sources?: ReplySource[];
	/** Who wrote it, when it's a teammate's message in a shared project chat. */
	author_name?: string;
}

export interface ReplyStep {
	tool: string;
	/** What it was used on: the search, page, file, command, or who it went to. */
	detail: string | null;
	ok: boolean;
	/** What a command printed (its end). */
	output?: string;
}

export interface ReplySource {
	title: string | null;
	url: string;
}

export interface AgentPlanItem {
	id: string;
	title: string;
	version: number;
	content: string | null;
	created_at: string;
}

export interface AgentPlansResponse {
	plans: AgentPlanItem[];
}

export interface RenderedMessage extends MessageItem {
	content_html: string;
}

export interface LlmAdminModelOption {
	id: string;
	label: string;
	provider: string;
	model_id: string;
}

export type LlmUserSelection =
	| { kind: 'admin'; admin_model_id: string }
	| { kind: 'custom'; label: string; provider: string; model_id: string; api_key_masked: string; base_url: string | null };

/** Which model Koda builds the person's projects with. */
export type LlmCodingSelection = { kind: 'default' } | { kind: 'same_as_chat' } | { kind: 'admin'; admin_model_id: string };

export interface LlmModelsResponse {
	admin_models: LlmAdminModelOption[];
	selection: LlmUserSelection | null;
	coding: LlmCodingSelection;
	/** The coding model an admin picked (what 'default' means), if any. */
	default_coding_model_id: string | null;
}

export interface AdminLlmModel {
	id: string;
	label: string;
	provider: string;
	model_id: string;
	base_url: string | null;
	is_default: boolean;
	api_key_masked: string;
	/** USD per million tokens; null until set. */
	input_usd_per_mtok: number | null;
	output_usd_per_mtok: number | null;
	/** Input read from the provider's prompt cache; null charges it at the input price. */
	cached_input_usd_per_mtok: number | null;
	/** Reads files people's own models can't open. */
	is_files_model: boolean;
	/** What Koda builds projects with, unless the person picked another. */
	is_coding_model: boolean;
	/** Switched off: nobody can pick it and nothing is routed to it. */
	enabled: boolean;
	/** What an admin said it opens besides text; null when it's worked out from the model id. */
	media_inputs: MediaInput[] | null;
	/** What it opens, listed or worked out. */
	media_support: MediaInput[];
}

export type MediaInput = 'image' | 'pdf' | 'audio' | 'video';

export interface PersonalityVersion {
	version: number;
	description: string;
	created_at: string;
	is_current: boolean;
}

export interface PersonalityHistoryResponse {
	versions: PersonalityVersion[];
}

/** Which personality a chat replies in (GET /api/sessions/:id/personality). */
export interface ChatPersonality {
	/** This chat's own personality, when it has one. */
	chat: string | null;
	/** The person's default, used by chats without their own. */
	default: string | null;
}

export type MemoryKind = 'preference' | 'person' | 'routine' | 'goal' | 'fact';

/** Why a reply missed, given with a thumbs-down. */
export type FeedbackReason = 'wrong_memory' | 'not_relevant' | 'too_long' | 'other';

/** A memory a reply drew on (GET …/messages/:id/memories). */
export interface UsedMemory {
	id: string;
	content: string;
	kind: MemoryKind;
	/** Replaced, merged or faded since: no longer recalled. */
	archived: boolean;
}

export interface MemoryItem {
	id: string;
	content: string;
	weight: number;
	kind: MemoryKind;
	last_used_at: string | null;
	/** Strong and not confirmed in a long while: worth asking whether it's still true. */
	needs_check: boolean;
	created_at: string;
	updated_at: string;
	/** How many of Nomi's replies drew on this memory. */
	uses: number;
	embedding: number[];
}

export interface MemoryListResponse {
	memories: MemoryItem[];
}

export interface DashboardStats {
	total_users: number;
	tokens_today: number;
	tokens_all_time: number;
	running_agents: number;
}

export interface RunningAgentItem {
	agent_session_id: string;
	session_id: string;
	agent_type: string;
	agent_display_name: string;
	channel: string;
	started_at: string;
	last_activity_at: string;
	current_phase: string;
	current_phase_detail: string | null;
}

export interface UserAgentGroup {
	user_id: string;
	label: string;
	agents: RunningAgentItem[];
}

export interface AgentsResponse {
	users: UserAgentGroup[];
}

export interface AgentStatus {
	agent_session_id: string;
	agent_type: string;
	current_phase: string;
	current_phase_detail: string | null;
}

export interface DynamicAgent {
	id: string;
	name: string;
	system_prompt: string;
	intent_label: string;
	intent_description: string;
	granted_tools: string[];
	supports_todos: boolean;
	supports_plans: boolean;
	can_delegate: boolean;
	is_active: boolean;
	/** The agent's look (see ShapePicker). */
	shape: string;
	tone: string;
	motion: string;
}

export interface AdminUserSummary {
	id: string;
	email: string;
	is_platform_admin: boolean;
	is_staff: boolean;
	org_count: number;
	plan_name: string;
	/** Their allowance is an admin's override. */
	custom_quota: boolean;
	monthly_tokens: number;
	/** Used on Nomi's models this calendar month (UTC). */
	tokens_used: number;
}

/** Someone's plan, allowance, use and change history (Admin → Users → Plan & quota). */
export interface AdminSubscription {
	plan: Plan;
	quota_override: number | null;
	override_until: string | null;
	note: string | null;
	monthly_tokens: number;
	month: string;
	tokens_used: number;
	history: { plan_name: string; quota_override: number | null; override_until: string | null; note: string | null; changed_by_email: string | null; created_at: string }[];
}

export interface AdminUserListResponse {
	users: AdminUserSummary[];
	total: number;
}

export interface PermissionGrant {
	id: string;
	scope_type: 'admin' | 'org';
	org_id: string | null;
	org_name: string | null;
	resource: string;
	actions: string[];
	created_at: string;
}

export interface MembershipRow {
	org_id: string;
	org_name: string;
	role: string;
}

export interface AdminUserDetail {
	id: string;
	email: string;
	is_platform_admin: boolean;
	display_name: string | null;
	username: string | null;
	permissions: PermissionGrant[];
	memberships: MembershipRow[];
}

export interface OrgOption {
	id: string;
	name: string;
}

export interface Profile {
	display_name: string | null;
	username: string | null;
	email: string;
	avatar_url: string | null;
}

export type Theme = 'light' | 'dark' | 'system';
/** Appearance theme (stored as accent_color). */
export type AccentColor =
	| 'canopy'
	| 'coral-reef'
	| 'borneo-dusk'
	| 'phantom'
	| 'senja-jakarta'
	| 'lautan'
	| 'matcha'
	| 'saffron'
	| 'lagoon'
	| 'orchid'
	| 'ruby'
	| 'graphite';

export interface Preferences {
	theme: Theme;
	accent_color: AccentColor;
	timezone: string;
	/** "en" or "id": the app's language and the crew's. */
	language: Locale;
	has_stored_timezone: boolean;
	/** Features pinned to the drawer, in order; null until customised (see $lib/drawerPins). */
	drawer_pins?: string[] | null;
}

export interface ProjectSummary {
	id: string;
	session_id: string;
	name: string;
	description: string | null;
	status: 'planning' | 'building' | 'ready';
	created_at: string;
	updated_at: string;
	/** Who started it. */
	created_by: string;
	open_issues: number;
}

export interface ProjectList {
	/** The organization shown; null until the person picks one for projects. */
	org_id: string | null;
	projects: ProjectSummary[];
}

export type OrgRole = 'owner' | 'admin' | 'member';

export interface OrgSummary {
	id: string;
	name: string;
	role: OrgRole;
	members: number;
	projects: number;
}

export interface OrgList {
	orgs: OrgSummary[];
	project_org_id: string | null;
}

export interface OrgMember {
	user_id: string;
	name: string;
	email: string | null;
	avatar_url: string | null;
	role: OrgRole;
	joined_at: string;
}

export interface OrgInvite {
	code: string;
	email: string | null;
	role: OrgRole;
	created_at: string;
	expires_at: string;
}

export interface OrgDetail {
	id: string;
	name: string;
	my_role: OrgRole;
	me: string;
	members: OrgMember[];
	/** Only owners and admins see these. */
	invites: OrgInvite[];
}

export interface InviteInfo {
	org_id: string;
	org_name: string;
	inviter: string;
	role: OrgRole;
	state: 'open' | 'expired' | 'used' | 'member' | 'other_address';
	/** The invited address, partly hidden. */
	email: string | null;
}

/** Something deleting a message also deletes (see the backend's message_delete). */
export interface RelatedItem {
	id: string;
	label: string;
	amount_cents: number | null;
	kind: string | null;
}

export interface MessageDeletion {
	message_ids: string[];
	transactions: RelatedItem[];
	reminders: RelatedItem[];
	plans: RelatedItem[];
	tasks: RelatedItem[];
	files: RelatedItem[];
}

export type IssueKind = 'feature' | 'enhancement' | 'bug';
export type IssueSeverity = 'low' | 'medium' | 'high' | 'critical';

export interface ProjectIssue {
	number: number;
	title: string;
	body: string;
	kind: IssueKind;
	severity: IssueSeverity;
	status: 'open' | 'closed';
	created_by: string | null;
	mine: boolean;
	closed_at: string | null;
	created_at: string;
	updated_at: string;
}

export interface IssueList {
	issues: ProjectIssue[];
	counts: { open: number; closed: number };
}

export interface ProjectFileSummary {
	path: string;
	content_type: string;
	size_bytes: number;
}

export interface ProjectDetail {
	id: string;
	name: string;
	description: string | null;
	plan: string | null;
	status: 'planning' | 'building' | 'ready';
	/** What Koda builds it with; 'static' projects are plain HTML, previewed without WebContainer. */
	stack: 'sveltekit' | 'svelte' | 'react' | 'vue' | 'astro' | 'static';
	files_version: number;
	files: ProjectFileSummary[];
	/** The project's chat. */
	session_id: string;
	org_id: string;
	org_name: string;
	open_issues: number;
}

export type AdminStreamFrame =
	| {
			kind: 'AgentSessionStarted';
			agent_session_id: string;
			session_id: string;
			agent_type: string;
			agent_display_name: string;
			channel: string;
			sender_label: string;
	  }
	| { kind: 'AgentSessionEnded'; agent_session_id: string; session_id: string; reason: string }
	| { kind: 'AgentPhaseChanged'; agent_session_id: string; session_id: string; phase: string; detail: string | null };

export interface AgentEventItem {
	id: string;
	session_id: string | null;
	agent_session_id: string | null;
	agent_type: string | null;
	agent_display_name: string | null;
	event_type: string;
	created_at: string;
	tool_name: string | null;
	is_error: boolean | null;
}

/** GET /api/home — the three status cards on Home. */
export interface HomeSummary {
	since: string;
	timezone: string;
	while_you_were_out: {
		kind: 'finished' | 'failed' | 'stopped' | 'needs_you' | 'reminder' | 'reply';
		agent: string;
		title: string;
		detail: string;
		session_id: string | null;
		at: string;
	}[];
	today: { id: string; run_at: string; label: string; agent: string; recurrence: string | null }[];
	plans: { kind: 'todo' | 'plan'; title: string; agent: string; done: number; total: number; session_id: string; updated_at: string }[];
	/** Full counts; the lists above carry at most four. */
	while_you_were_out_total: number;
	today_total: number;
	plans_total: number;
}

/** GET /api/home/{updates,today,plans}: one page of a Home section. */
export interface HomeSectionPage<T> {
	since: string;
	timezone: string;
	items: T[];
	total: number;
	page: number;
	per_page: number;
}

/** GET /api/money */
export interface MoneySummary {
	/** The money month shown, by the month it starts in. */
	month: string;
	/** The day money months start (1 = calendar months). */
	start_day: number;
	period_start: string;
	period_end: string;
	timezone: string;
	/** Spending in the period. */
	total_cents: number;
	/** Money in (income) in the period. */
	income_cents: number;
	previous_total_cents: number;
	transaction_count: number;
	by_category: { category: string; cents: number; count: number }[];
	by_day: { date: string; cents: number }[];
	transactions: {
		id: string;
		occurred_at: string;
		amount_cents: number;
		category: string;
		description: string;
		kind: 'expense' | 'income';
		/** Who added it: 'manual' (the person), 'agent' (Finley, from a chat) or 'import'. */
		source: 'manual' | 'agent' | 'import';
		/** When it was recorded (occurred_at is when it happened). */
		created_at: string;
		items: { name: string; quantity: number; unit_amount_cents: number | null; amount_cents: number; category: string | null }[];
	}[];
	months: string[];
	budgets: { category: string; limit_cents: number; spent_cents: number }[];
}

/** One reminder, from the Reminders agent's own table (GET /api/reminders). */
export interface Reminder {
	id: string;
	session_id: string;
	title: string;
	notes: string | null;
	due_at: string;
	recurrence: 'daily' | 'weekly' | 'monthly' | null;
	recurrence_weekday: number | null;
	recurrence_day_of_month: number | null;
	status: 'active' | 'fired' | 'done' | 'cancelled';
	created_by: 'user' | 'agent';
	last_fired_at: string | null;
	created_at: string;
}

/** An agent task scheduled for later (core scheduler), shown on the Reminders page. */
export interface ScheduledTask {
	id: string;
	label: string;
	run_at: string;
	recurrence: string | null;
	agent: string;
	session_id: string;
}

/** A chat's thinking level (see ThinkingMenu). */
export type ThinkingLevel = 'off' | 'low' | 'medium' | 'high';

/** A Google service the Workspace agent can use. */
export type WorkspaceService = 'gmail' | 'sheets' | 'docs' | 'drive' | 'calendar';

/** GET /api/connections/google: this user's own Google account, if connected. */
export interface GoogleConnection {
	configured: boolean;
	connection: { email: string; services: WorkspaceService[]; connected_at: string } | null;
	activity: { service: WorkspaceService; summary: string; link: string | null; created_at: string }[];
	services: WorkspaceService[];
}

/** The plan someone is on (GET /api/usage/brief). */
export interface UsagePlan {
	/** The plan's slug ("free", "pro"…). */
	id: string;
	name: string;
	card_tone: string;
	/** Tokens of Nomi's own models they may use each month: an admin's override, else the plan's. */
	monthly_tokens: number;
	/** Their allowance is an admin's override. */
	custom_quota: boolean;
}

/** A plan, as Admin → Plans and the plans sheet show it (GET /api/plans). */
export interface Plan {
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
}

export interface PlansForUser {
	plans: Plan[];
	current_plan_id: string;
	monthly_tokens: number;
	custom_quota: boolean;
	override_until: string | null;
}

export type NotificationKind = 'account' | 'subscription' | 'quota' | 'promo' | 'deploy' | 'project';

export interface NotificationItem {
	id: string;
	kind: NotificationKind;
	title: string;
	body: string;
	link: string | null;
	read: boolean;
	created_at: string;
}

/** This month's allowance and how much of it is used (GET /api/usage/brief). */
export interface UsageBrief {
	plan: UsagePlan;
	month: string;
	tokens_used: number;
	/** A bigger plan is offered than theirs. */
	can_upgrade: boolean;
}

/** One month of usage and spend (GET /api/usage). */
export interface UsageMonth {
	plan: UsagePlan;
	month: string;
	current_month: string;
	timezone: string;
	/** Tokens of Nomi's own models: what the allowance counts (cached input at a tenth). */
	tokens_used: number;
	input_tokens: number;
	output_tokens: number;
	/** The part of input_tokens the provider served from its prompt cache. */
	cached_input_tokens: number;
	/** Tokens sent with the person's own API key (never billed). */
	own_key_tokens: number;
	calls: number;
	spend_usd: number;
	by_day: { date: string; tokens: number; spend_usd: number }[];
	/** One row per model actually called, kept together by the admin model's id (so a renamed model keeps its history). */
	by_model: { admin_model_id: string | null; label: string; provider: string; model_id: string; source: 'nomi' | 'own_key'; tokens: number; calls: number; spend_usd: number }[];
}

/** Admin → Usage: one month (UTC) of everyone's tokens, per chat model and embedding model. */
export interface AdminUsage {
	month: string;
	current_month: string;
	chat: { tokens: number; input_tokens: number; output_tokens: number; cached_input_tokens: number; own_key_tokens: number; calls: number; people: number; spend_usd: number };
	embedding: { tokens: number; calls: number; estimated_calls: number };
	by_day: { date: string; chat_tokens: number; embedding_tokens: number }[];
	models: {
		admin_model_id: string | null;
		label: string;
		deleted: boolean;
		provider: string;
		model_id: string;
		source: 'nomi' | 'own_key';
		input_tokens: number;
		output_tokens: number;
		cached_input_tokens: number;
		tokens: number;
		calls: number;
		people: number;
		spend_usd: number;
	}[];
	embedding_models: { provider: string; model_id: string; tokens: number; calls: number; estimated_calls: number; people: number; current: boolean }[];
}

/** A connected health or fitness app (Connections → Hevy). */
export interface HealthConnection {
	provider: string;
	account_name: string | null;
	connected_at: string;
	last_used_at: string | null;
}

/** Every connector's state for the Connections list. */
export interface ConnectionsOverview {
	google: { configured: boolean; email: string | null; services: string[] };
	hevy: HealthConnection | null;
}

/** One publish of a project to a Shipyard environment. */
export interface ProjectPublish {
	id: string;
	environment: string;
	status: 'running' | 'succeeded' | 'failed';
	url: string | null;
	error: string | null;
	files_version: number;
	/** The publish this one rolled back to. */
	rollback_of: string | null;
	can_roll_back: boolean;
	created_by: string | null;
	created_at: string;
	finished_at: string | null;
}

export interface ShipyardService {
	name: string;
	service_id: string;
	type: string;
	status: string;
	replicas: number;
	running: number;
	urls: string[];
	last_deployment: { id: string; status: string; created_at: string } | null;
}

export interface DeployOverview {
	/** Whether this Nomi can publish at all. */
	available: boolean;
	role: 'admin' | 'member';
	app: string | null;
	database: 'postgres' | 'sqlite' | 'none';
	url: string | null;
	publishes: ProjectPublish[];
	/** What runs in production now; null when nothing does. */
	live: { environment: string; services: ShipyardService[]; last_publish: unknown } | null;
}

export interface PublishDetail extends ProjectPublish {
	deployments: { service: string; deployment_id: string; status: string }[];
}

export interface DeploymentDetail {
	id: string;
	service: string;
	status: string;
	image: string | null;
	created_at: string;
	finished_at: string | null;
	steps: { name: string; status: string; started_at: string | null; finished_at: string | null }[];
	logs: { level: string; message: string; at: string }[];
}

export interface ContainerUsage {
	service: string;
	container_id: string;
	stats: {
		cpu_percent: number;
		memory_usage_bytes: number;
		memory_limit_bytes: number;
		memory_percent: number;
		net_rx_bytes: number;
		net_tx_bytes: number;
		block_read_bytes: number;
		block_write_bytes: number;
		pids: number;
		timestamp: string;
	} | null;
	limits: { cpu: number | null; memory_mb: number | null };
}

export interface ShipyardEvent {
	id: string;
	type: string;
	environment: string | null;
	service: string | null;
	data: Record<string, unknown>;
	created_at: string;
}

export interface ProjectPerson {
	user_id: string;
	name: string;
	email: string | null;
	avatar_url: string | null;
	org_role: OrgRole;
	project_role: 'admin' | 'member' | null;
	is_owner: boolean;
	added_at: string | null;
}

export interface ProjectMembers {
	access: 'org' | 'members';
	my_role: 'admin' | 'member';
	me: string;
	people: ProjectPerson[];
}

export type DatabaseSource = 'development' | 'production';

export interface DatabaseTable {
	schema: string;
	name: string;
	rows_estimate: number;
}

export interface QueryResult {
	columns: string[];
	rows: unknown[][];
	truncated: boolean;
	rows_affected: number | null;
	took_ms: number;
}

export interface TransferOrgSummary {
	id: string;
	name: string;
	is_personal: boolean;
	owner_id: string | null;
	owner_name: string | null;
	my_role: string | null;
}

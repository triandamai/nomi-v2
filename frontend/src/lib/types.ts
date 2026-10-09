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
	| { kind: 'file_delete'; project_id: string; path: string }
	| { kind: 'todo_list'; items: TodoItem[] }
	| { kind: 'table'; variant: 'data' | 'comparison'; columns: TableColumn[]; rows: Record<string, unknown>[] }
	| { kind: 'workspace_connect'; reason: 'not_connected' | 'service_not_allowed' | string; services: string[] }
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
	/** A reminder going off, posted by the Reminders agent. */
	| { kind: 'reminder'; reminder_id: string; title: string; notes: string | null; due_at: string; recurrence: string | null };

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

export interface LlmModelsResponse {
	admin_models: LlmAdminModelOption[];
	selection: LlmUserSelection | null;
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
	/** Reads files people's own models can't open. */
	is_files_model: boolean;
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
export type AccentColor = 'canopy' | 'coral-reef' | 'borneo-dusk' | 'phantom' | 'senja-jakarta';

export interface Preferences {
	theme: Theme;
	accent_color: AccentColor;
	timezone: string;
	/** "en" or "id": the app's language and the crew's. */
	language: Locale;
	has_stored_timezone: boolean;
}

export interface ProjectSummary {
	id: string;
	session_id: string;
	name: string;
	description: string | null;
	status: 'planning' | 'building' | 'ready';
	created_at: string;
	updated_at: string;
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
	files: ProjectFileSummary[];
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
	month: string;
	timezone: string;
	total_cents: number;
	previous_total_cents: number;
	transaction_count: number;
	by_category: { category: string; cents: number; count: number }[];
	by_day: { date: string; cents: number }[];
	transactions: { id: string; occurred_at: string; amount_cents: number; category: string; description: string }[];
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

/** The plan someone is on (everyone is on Free until Pro launches). */
export interface UsagePlan {
	id: 'free' | 'pro';
	/** Tokens of Nomi's own models included each month. */
	monthly_tokens: number;
}

/** This month's allowance and how much of it is used (GET /api/usage/brief). */
export interface UsageBrief {
	plan: UsagePlan;
	month: string;
	tokens_used: number;
}

/** One month of usage and spend (GET /api/usage). */
export interface UsageMonth {
	plan: UsagePlan;
	month: string;
	current_month: string;
	timezone: string;
	/** Tokens of Nomi's own models: what the allowance counts. */
	tokens_used: number;
	input_tokens: number;
	output_tokens: number;
	/** Tokens sent with the person's own API key (never billed). */
	own_key_tokens: number;
	calls: number;
	spend_usd: number;
	by_day: { date: string; tokens: number; spend_usd: number }[];
	by_model: { label: string; provider: string; model_id: string; source: 'nomi' | 'own_key'; tokens: number; calls: number; spend_usd: number }[];
}

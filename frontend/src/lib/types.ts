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
	| { kind: 'reasoning'; text: string };

export interface MessageItem {
	id: string;
	sender: 'user' | 'assistant';
	content: string;
	content_blocks: ContentBlock[] | null;
	created_at: string;
	my_feedback: 'up' | 'down' | null;
	agent_display_name: string | null;
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
}

export interface PersonalityVersion {
	version: number;
	description: string;
	created_at: string;
	is_current: boolean;
}

export interface PersonalityHistoryResponse {
	versions: PersonalityVersion[];
}

export interface MemoryItem {
	id: string;
	content: string;
	weight: number;
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
export type AccentColor = 'green' | 'blue' | 'purple' | 'pink' | 'orange' | 'teal';

export interface Preferences {
	theme: Theme;
	accent_color: AccentColor;
	timezone: string;
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
}

/** One scheduled reminder (GET /api/reminders). */
export interface Reminder {
	id: string;
	label: string;
	run_at: string;
	recurrence: 'daily' | 'weekly' | 'monthly' | null;
	recurrence_weekday: number | null;
	recurrence_day_of_month: number | null;
	agent: string;
	status: 'active' | 'completed' | 'cancelled';
	last_fired_at: string | null;
	session_id: string;
}

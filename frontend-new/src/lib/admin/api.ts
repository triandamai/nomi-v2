// The admin API's answers, as the backend serializes them (backend/crates/nomi-server/src/routes:
// admin_*.rs, plans.rs, notifications.rs, maintenance.rs, llm_models.rs, provider_connections.rs,
// settings.rs, tools.rs, dynamic_agents.rs, support.rs, deploy.rs). Dates are RFC 3339 strings.

// Overview, usage and live agents (admin_dashboard.rs, admin_usage.rs) ------------------------------

export type Dashboard = {
	total_users: number;
	tokens_today: number;
	tokens_all_time: number;
	running_agents: number;
	sandbox: { running: number; starting: number; error: number; minutes_this_month: number };
};

export type AdminUsage = {
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
		source: 'nomi' | 'own_key' | string;
		input_tokens: number;
		output_tokens: number;
		cached_input_tokens: number;
		tokens: number;
		calls: number;
		people: number;
		spend_usd: number;
	}[];
	embedding_models: { provider: string; model_id: string; tokens: number; calls: number; estimated_calls: number; people: number; current: boolean }[];
	sandbox_minutes: { user_id: string; email: string | null; minutes: number }[];
};

export type RunningAgent = {
	agent_session_id: string;
	session_id: string;
	agent_type: string;
	agent_display_name: string;
	channel: string;
	started_at: string;
	last_activity_at: string;
	current_phase: string;
	current_phase_detail: string | null;
};

export type AgentsResponse = { users: { user_id: string; label: string; agents: RunningAgent[] }[] };

export type AgentEvent = {
	id: string;
	session_id: string | null;
	agent_session_id: string | null;
	agent_type: string | null;
	agent_display_name: string | null;
	event_type: string;
	created_at: string;
	tool_name: string | null;
	is_error: boolean | null;
};

/** What `/api/admin/agents/ws` sends (relayed at /admin/agents/ws). */
export type AgentFrame =
	| { kind: 'AgentSessionStarted'; agent_session_id: string; session_id: string; agent_type: string; agent_display_name: string; channel: string; sender_label: string }
	| { kind: 'AgentSessionEnded'; agent_session_id: string; session_id: string; reason: string }
	| { kind: 'AgentPhaseChanged'; agent_session_id: string; session_id: string; phase: string; detail: string | null };

// Users and access (admin_users.rs, admin_access.rs, plans.rs) ---------------------------------------

export type UserSummary = {
	id: string;
	email: string;
	is_platform_admin: boolean;
	is_staff: boolean;
	org_count: number;
	plan_name: string;
	custom_quota: boolean;
	monthly_tokens: number;
	tokens_used: number;
};

export type UserList = { users: UserSummary[]; total: number };

export type PermissionGrant = {
	id: string;
	scope_type: 'admin' | 'org' | string;
	org_id: string | null;
	org_name: string | null;
	resource: string;
	actions: string[];
	created_at: string;
};

export type Membership = { org_id: string; org_name: string; role: 'owner' | 'admin' | 'member' | string };

export type UserDetail = {
	id: string;
	email: string;
	is_platform_admin: boolean;
	display_name: string | null;
	username: string | null;
	permissions: PermissionGrant[];
	memberships: Membership[];
};

export type AdminOrg = { id: string; name: string };

export type Plan = {
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

export type AdminPlan = Plan & { subscribers: number };

export type Subscription = {
	plan: Plan;
	quota_override: number | null;
	override_until: string | null;
	note: string | null;
	monthly_tokens: number;
	month: string;
	tokens_used: number;
	history: { plan_name: string; quota_override: number | null; override_until: string | null; note: string | null; changed_by_email: string | null; created_at: string }[];
};

// Notifications and maintenance --------------------------------------------------------------------

export type Broadcast = { id: string; title: string; body: string; link: string | null; audience: string; recipients: number; read: number; created_at: string };

export type Maintenance = { enabled: boolean; message: string; until: string | null; updated_at: string };

// Models and providers (llm_models.rs, provider_connections.rs, settings.rs) -----------------------

export type AdminModel = {
	id: string;
	label: string;
	provider: string;
	model_id: string;
	base_url: string | null;
	is_default: boolean;
	/** The model's own key, masked ("...abcd"); "" when it has none. Never the key. */
	api_key_masked: string;
	connection_id: string | null;
	connection_label: string | null;
	input_usd_per_mtok: number | null;
	output_usd_per_mtok: number | null;
	cached_input_usd_per_mtok: number | null;
	is_files_model: boolean;
	is_coding_model: boolean;
	media_inputs: string[] | null;
	media_support: string[];
	enabled: boolean;
};

export type CatalogProvider = {
	id: string;
	name: string;
	description: string;
	icon: string;
	has_logo: boolean;
	category: 'popular' | 'other' | 'custom';
	recommended: boolean;
	auth_type: 'api_key' | 'oauth' | 'custom_endpoint';
	kind: string;
	default_base_url: string | null;
	doc_url: string | null;
	model_count: number;
};

export type Connection = {
	id: string;
	provider_id: string;
	label: string;
	auth_type: 'api_key' | 'custom_endpoint' | string;
	kind: string;
	base_url: string | null;
	header_names: string[];
	api_key_masked: string | null;
	created_at: string;
	models_in_use: string[];
};

export type FetchedModel = { id: string; label: string | null };

export type EmbeddingSettings = { provider: string; model_id: string; base_url: string | null; api_key_masked: string };

// Tools (tools.rs) -------------------------------------------------------------------------------

export type KeySource = 'saved' | 'env' | 'missing';

export type WebSearchSettings = {
	provider: 'tavily' | 'serper' | 'brave' | 'searxng';
	max_results: number;
	max_results_limit: number;
	base_url: string | null;
	providers: { id: string; needs_key: boolean; env_var: string; key_source: KeySource }[];
};

export type ReadPageSettings = { max_chars: number };

export type SpeakSettings = {
	voice_id: string;
	voice_name: string | null;
	model_id: string;
	stability: number;
	similarity_boost: number;
	style: number;
	use_speaker_boost: boolean;
	speed: number;
	output_format: string;
	output_formats: string[];
	max_chars: number;
	max_chars_limit: number;
	key_source: KeySource;
	env_var: string;
	models: { model_id: string; name: string }[];
};

export type ToolEntry = {
	name: string;
	description: string;
	enabled: boolean;
	required: boolean;
	agent_only: boolean;
	configurable: boolean;
	ready: boolean;
	used_by: string[];
	settings: unknown;
};

export type ToolGroup = { key: string; kind: 'crew' | 'agent' | 'custom'; label: string; tools: ToolEntry[] };

export type SearchTest = { ok: boolean; provider: string; results: { title: string; url: string; snippet: string }[]; error: string | null };

export type VoiceOptions = {
	voices: { voice_id: string; name: string; category: string | null; description: string | null; preview_url: string | null }[];
	models: { model_id: string; name: string }[];
};

// Custom agents (dynamic_agents.rs) ----------------------------------------------------------------

export type DynamicAgent = {
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
	shape: string;
	tone: string;
	motion: string;
};

// Support (support.rs) ---------------------------------------------------------------------------

export type Article = { id: string; title: string; body: string; tags: string[]; published: boolean; created_at: string; updated_at: string };

export type Draft = { id: string; question: string; title: string; body: string; status: string; times_asked: number; last_asked_at: string; article_id: string | null; created_at: string };

export type SupportKey = { id: string; name: string; prefix: string; product_name: string; created_at: string; last_used_at: string | null; revoked_at: string | null };

export type AskedQuestion = { id: string; question: string; answer: string; answered: boolean; customer: string | null; key_name: string | null; created_at: string };

// Configuration and payment (admin_configuration.rs, admin_payment.rs) ----------------------------

export type ConfigField = {
	name: string;
	env: string | null;
	kind: 'text' | 'number' | 'secret' | 'choice' | 'domain';
	choices?: string[];
	/** The saved value, else the environment's; always null for a secret. */
	value: string | null;
	set: boolean;
	source: 'saved' | 'environment' | 'none';
};

export type ConfigSection = { name: string; fields: ConfigField[] };

export type Configuration = { sections: ConfigSection[]; can_manage: boolean };

export type TestResult = { success: boolean; message: string };

// Integrations: Shipyard (deploy.rs, admin_shipyard.rs) ------------------------------------------

export type ShipyardStatus = {
	configured: boolean;
	can_reveal: boolean;
	webhook_url: string | null;
	webhook_registered: boolean | null;
	webhook_enabled: boolean | null;
	webhook_target: string | null;
	webhook_from_env: boolean;
	registered_at: string | null;
	last_event_at: string | null;
};

export type ShipyardApp = {
	org_id: string;
	org_name: string;
	key: string;
	name: string;
	environments: string[];
	project: { id: string; name: string; session_id: string } | null;
};

export type AppsOverview = { apps: ShipyardApp[]; candidates: { org_id: string; projects: { id: string; name: string }[] }[]; errors: string[] };

export type WebhookEndpoint = { id: string; url: string; events: string[]; enabled: boolean; org: string | null; created_at: string };

export type Delivery = {
	id: string;
	event_id: string;
	event_type: string;
	status: string;
	attempts: number;
	last_status_code: number | null;
	last_error: string | null;
	created_at: string;
	delivered_at: string | null;
};

export type SyncedOrg = { id: string; name: string; synced_at: string | null; projects: number; published: number };

export type AppDetail = {
	key: string;
	dev_database: boolean;
	environments: {
		environment: string;
		services: { name: string; type: string; status: string; replicas: number; running: number; urls: string[] }[];
		last_publish: { status: string; urls: string[]; created_at: string } | null;
	}[];
};

export type AppSecret = { ref: string; name: string; kind: string; environment: string | null; used_by: string[]; updated_at: string };

export type ReceivedEvent = {
	id: string;
	type: string;
	project_id: string | null;
	project_name: string | null;
	environment: string | null;
	service: string | null;
	created_at: string;
	received_at: string;
};

export type EventFeed = { events: ReceivedEvent[]; page: number; pages: number; total: number; types: unknown };

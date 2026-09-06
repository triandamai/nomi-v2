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

export interface MessageItem {
	id: string;
	sender: 'user' | 'assistant';
	content: string;
	created_at: string;
	my_feedback: 'up' | 'down' | null;
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

export interface DashboardStats {
	total_users: number;
	tokens_today: number;
	tokens_all_time: number;
	running_agents: number;
}

export interface RunningAgentItem {
	agent_session_id: string;
	agent_type: string;
	channel: string;
	started_at: string;
	last_activity_at: string;
}

export interface UserAgentGroup {
	user_id: string;
	label: string;
	agents: RunningAgentItem[];
}

export interface AgentsResponse {
	users: UserAgentGroup[];
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

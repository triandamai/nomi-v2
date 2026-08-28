export interface SessionSummary {
	id: string;
	channel: string;
	chat_type: string;
	chat_id: string;
	last_message: { content: string; created_at: string } | null;
	agent_active: boolean;
	updated_at: string;
}

export interface MessageItem {
	id: string;
	sender: 'user' | 'assistant';
	content: string;
	created_at: string;
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

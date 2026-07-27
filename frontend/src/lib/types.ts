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

// Connections → MCP servers: the error codes POST /api/connections/mcp answers with, in words.
import { m } from '$lib/paraglide/messages';

export function mcpError(code: string | undefined): string {
	switch (code) {
		case 'missing_name':
			return m.mcp_err_missing_name();
		case 'missing_url':
			return m.mcp_err_missing_url();
		case 'invalid_url':
			return m.mcp_err_invalid_url();
		case 'command_not_supported':
			return m.mcp_err_command();
		case 'already_added':
			return m.mcp_err_already_added();
		case 'too_many':
			return m.mcp_err_too_many();
		default:
			return m.mcp_err_failed();
	}
}

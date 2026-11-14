import { describe, expect, it } from 'vitest';
import { mcpError } from './mcpServers';

describe('mcpError', () => {
	it('words each code, and anything else as a failure', () => {
		expect(mcpError('invalid_url')).toContain('http');
		expect(mcpError('command_not_supported')).toContain('commands');
		expect(mcpError('already_added')).toContain('already');
		expect(mcpError(undefined)).toBe(mcpError('server_error'));
	});
});

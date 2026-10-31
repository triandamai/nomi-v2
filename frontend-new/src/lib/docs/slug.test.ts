import { describe, expect, it } from 'vitest';
import { slugify } from './slug';

describe('slugify', () => {
	it('makes the anchors the pen exports use', () => {
		expect(slugify('Code, terminal & phone')).toBe('code-terminal-phone');
		expect(slugify('Space · radius · icons')).toBe('space-radius-icons');
		expect(slugify('ApprovalCard')).toBe('approval-card');
		expect(slugify('KVRow')).toBe('kv-row');
		expect(slugify('DS Header')).toBe('ds-header');
	});
});

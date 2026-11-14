import { describe, expect, it } from 'vitest';
import { fieldLabel, sectionLabel } from './configuration';

describe('configuration labels', () => {
	it('names each section and field in the person’s language', () => {
		expect(sectionLabel('smtp')).toEqual({ title: 'Email', description: 'The server Nomi sends codes, invites and notices through.' });
		expect(fieldLabel('sandbox', 'idle_stop_secs')).toBe('Stop when idle for (seconds)');
	});

	it('shows a section or field the backend adds later by its own name', () => {
		expect(sectionLabel('billing_v2')).toEqual({ title: 'Billing v2', description: undefined });
		expect(fieldLabel('smtp', 'retry_after_secs')).toBe('Retry after secs');
	});
});

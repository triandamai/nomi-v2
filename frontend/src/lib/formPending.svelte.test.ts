import { describe, expect, it, vi } from 'vitest';
import { FormPending } from './formPending.svelte';

type Submit = Parameters<FormPending['submit']>[0];
type After = Exclude<Awaited<ReturnType<FormPending['submit']>>, void>;
type AfterInput = Parameters<After>[0];

describe('FormPending', () => {
	it('is on from submit until the server answer is applied, keeping fields when asked', async () => {
		const sending = new FormPending({ reset: false });
		expect(sending.on).toBe(false);
		const after = (await sending.submit({} as Submit)) as After;
		expect(sending.on).toBe(true);
		const update = vi.fn(async () => {
			expect(sending.on).toBe(true);
		});
		await after({ update } as unknown as AfterInput);
		expect(update).toHaveBeenCalledWith({ reset: false });
		expect(sending.on).toBe(false);
	});

	it('turns off even when applying the answer fails', async () => {
		const sending = new FormPending();
		const after = (await sending.submit({} as Submit)) as After;
		const update = vi.fn(async () => {
			throw new Error('offline');
		});
		await expect(after({ update } as unknown as AfterInput)).rejects.toThrow('offline');
		expect(update).toHaveBeenCalledWith({ reset: true });
		expect(sending.on).toBe(false);
	});
});

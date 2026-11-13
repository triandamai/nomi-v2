import { describe, expect, it } from 'vitest';
import { modelFromForm } from './models';

const form = (fields: Record<string, string>) => {
	const data = new FormData();
	for (const [key, value] of Object.entries(fields)) data.set(key, value);
	return data;
};

describe('a model from the admin form', () => {
	it('is a model on a connected provider, with its prices, opening what the backend detects', () => {
		expect(modelFromForm(form({ label: ' DeepSeek Flash ', source: 'connection', connection_id: 'c1', model_id: 'deepseek-flash', input_usd_per_mtok: '0.27', output_usd_per_mtok: '1.1', cached_input_usd_per_mtok: '', media_mode: 'auto' }))).toEqual({
			ok: true,
			body: { label: 'DeepSeek Flash', connection_id: 'c1', model_id: 'deepseek-flash', input_usd_per_mtok: 0.27, output_usd_per_mtok: 1.1, cached_input_usd_per_mtok: null, media_inputs: null },
		});
	});

	it('is a model with its own provider and key, opening the kinds listed', () => {
		expect(modelFromForm(form({ label: 'Claude', source: 'own', provider: 'anthropic', api_key: 'sk-test', base_url: '', model_id: 'claude-sonnet', media_mode: 'listed', media_image: 'on', media_pdf: 'on' }))).toEqual({
			ok: true,
			body: { label: 'Claude', provider: 'anthropic', model_id: 'claude-sonnet', api_key: 'sk-test', base_url: null, input_usd_per_mtok: null, output_usd_per_mtok: null, cached_input_usd_per_mtok: null, media_inputs: ['image', 'pdf'] },
		});
	});

	it('keeps an own key when it is left empty while editing', () => {
		const read = modelFromForm(form({ label: 'Claude', source: 'own', provider: 'anthropic', api_key: '', model_id: 'claude-sonnet', media_mode: 'auto' }), { editing: true });
		expect(read.ok && 'api_key' in read.body ? read.body.api_key : 'missing').toBeNull();
	});

	it('says what is missing or wrong', () => {
		for (const fields of <Record<string, string>[]>[
			{ label: '', source: 'connection', connection_id: 'c1', model_id: 'm' },
			{ label: 'A', source: 'connection', connection_id: '', model_id: 'm' },
			{ label: 'A', source: 'own', provider: '', model_id: 'm' },
			{ label: 'A', source: 'connection', connection_id: 'c1', model_id: ' ' },
			{ label: 'A', source: 'connection', connection_id: 'c1', model_id: 'm', input_usd_per_mtok: '-1' },
			{ label: 'A', source: 'connection', connection_id: 'c1', model_id: 'm', output_usd_per_mtok: 'cheap' },
		]) {
			expect(modelFromForm(form(fields)).ok).toBe(false);
		}
	});
});

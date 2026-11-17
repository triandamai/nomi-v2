import { describe, expect, it } from 'vitest';
import {
	blockedReason,
	composeMessage,
	extensionOf,
	formatBytes,
	hiddenLine,
	kindLabel,
	progressDetail,
	readyCount,
	splitAttachments,
	uploadFailure,
	visionSplit,
	type ChipState,
} from './attachments';

const ref = (id: string, name: string, kind = 'pdf', mime = 'application/pdf', size = 4404019) => `<attachment id="${id}" name="${name}" kind="${kind}" mime="${mime}" size="${size}"/>`;

describe('message references', () => {
	it('puts each file’s tag after the typed text, and reads them back', () => {
		const text = composeMessage('  What did I spend?  ', [ref('a', 'bca-sep-2026.pdf'), ref('b', 'struk.jpg', 'image', 'image/jpeg', 120)]);
		expect(text).toBe(`What did I spend?\n\n${ref('a', 'bca-sep-2026.pdf')}\n\n${ref('b', 'struk.jpg', 'image', 'image/jpeg', 120)}`);
		expect(splitAttachments(text)).toEqual({
			text: 'What did I spend?',
			files: [
				{ id: 'a', name: 'bca-sep-2026.pdf', kind: 'pdf', mime: 'application/pdf', size: 4404019 },
				{ id: 'b', name: 'struk.jpg', kind: 'image', mime: 'image/jpeg', size: 120 },
			],
		});
	});

	it('leaves a message without files alone, and reads a masked size as unknown', () => {
		expect(splitAttachments('Hi')).toEqual({ text: 'Hi', files: [] });
		expect(splitAttachments('<attachment id="x" name="a.pdf" kind="pdf" mime="application/pdf" size="•••"/>').files[0].size).toBeUndefined();
	});
});

describe('formats', () => {
	it('writes sizes with a decimal comma', () => {
		expect(formatBytes(512)).toBe('512 B');
		expect(formatBytes(4404019)).toBe('4,2 MB');
		expect(formatBytes(18 * 1024)).toBe('18 KB');
		expect(formatBytes(15 * 1024 * 1024)).toBe('15 MB');
	});

	it('names a file’s type the way the chip does', () => {
		expect(kindLabel({ name: 'bca-sep-2026.pdf', kind: 'pdf' })).toBe('PDF');
		expect(kindLabel({ name: 'struk-belanja-nov.jpg', kind: 'image' })).toBe('JPG');
		expect(kindLabel({ name: 'README', kind: 'text' })).toBe('TEXT');
		expect(extensionOf('kasir-setup.dmg')).toBe('.dmg');
		expect(extensionOf('Makefile')).toBe('');
	});

	it('counts real bytes while uploading', () => {
		expect(progressDetail(0.42, 5 * 1024 * 1024)).toBe('42% · 2,1 of 5 MB');
		expect(progressDetail(0, 5 * 1024 * 1024)).toBe('0% · 0 of 5 MB');
	});
});

describe('upload failures', () => {
	it('tells the reasons apart by what came back', () => {
		expect(uploadFailure(0)).toBe('connection_lost');
		expect(uploadFailure(507)).toBe('storage_full');
		expect(uploadFailure(413)).toBe('too_large');
		expect(uploadFailure(415)).toBe('rejected');
		expect(uploadFailure(500)).toBe('server_error');
		expect(uploadFailure(400)).toBe('server_error');
	});
});

describe('what keeps Send off', () => {
	const chip = (status: ChipState['status'], name = 'a.pdf'): ChipState => ({ key: name, name, size: 1, status });

	it('sends when every chip is ready', () => {
		expect(blockedReason([chip('ready')])).toBeNull();
		expect(blockedReason([])).toBeNull();
	});

	it('names a refused file first, then failed uploads, then a model that can’t see, then uploads still going', () => {
		expect(blockedReason([chip('failed', 'x.pdf'), chip('rejected', 'kasir-setup.dmg')])).toEqual({ kind: 'rejected', name: 'kasir-setup.dmg' });
		expect(blockedReason([chip('failed', 'x.pdf'), chip('failed', 'y.pdf')])).toEqual({ kind: 'failed', count: 2 });
		expect(blockedReason([chip('warning'), chip('uploading')])).toEqual({ kind: 'warning' });
		expect(blockedReason([chip('uploading')])).toEqual({ kind: 'uploading' });
	});

	it('counts the files ready', () => {
		expect(readyCount([chip('ready', 'a'), chip('failed', 'b'), chip('warning', 'c')])).toBe(2);
	});
});

describe('models that can see images', () => {
	const groups = [
		{ source: 'connection' as const, label: 'Anthropic', models: [{ key: 'k1', label: 'claude-sonnet', enabled: true, capabilities: ['vision', 'code'] }, { key: 'k2', label: 'deepseek-v3', enabled: true, capabilities: ['code'] }] },
		{ source: 'nomi' as const, label: 'Nomi', models: [{ key: 'n1', label: 'gemini-flash', enabled: true, capabilities: ['vision'] }, { key: 'n2', label: 'off', enabled: false, capabilities: ['vision'] }] },
	];

	it('keeps the switched-on models that see, grouped by who pays, and counts the rest', () => {
		const split = visionSplit(groups, 'k2');
		expect(split.groups.map((group) => [group.source, group.models.map((model) => model.label)])).toEqual([
			['connection', ['claude-sonnet']],
			['nomi', ['gemini-flash']],
		]);
		expect(split.hidden).toEqual(['deepseek-v3']);
	});

	it('says what’s hidden, the current model first', () => {
		expect(hiddenLine(['deepseek-v3'], 'deepseek-v3')).toEqual({ name: 'deepseek-v3', others: 0 });
		expect(hiddenLine(['a', 'deepseek-v3', 'b'], 'deepseek-v3')).toEqual({ name: 'deepseek-v3', others: 2 });
	});
});

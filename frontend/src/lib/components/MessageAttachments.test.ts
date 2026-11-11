// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { render, text } from '../../test-stubs/render';
import MessageAttachments from './MessageAttachments.svelte';
import type { Attachment } from '$lib/attachments';

describe('MessageAttachments', () => {
	it('renders images, videos, audio, and generic files', async () => {
		const files: Attachment[] = [
			{ id: 'att-img', name: 'photo.png', kind: 'image', size: 1024, mime: 'image/png' },
			{ id: 'att-vid', name: 'clip.mp4', kind: 'video', size: 2048, mime: 'video/mp4' },
			{ id: 'att-aud', name: 'sound.mp3', kind: 'audio', size: 512, mime: 'audio/mp3' },
			{ id: 'att-doc', name: 'readme.pdf', kind: 'pdf', size: 4096, mime: 'application/pdf' },
			{ name: 'inline.txt', kind: 'text', text: 'Hello inline text' },
		];

		const root = render(MessageAttachments, { files });
		expect(root.querySelector('img[alt="photo.png"]')).toBeTruthy();
		expect(text(root)).toContain('clip.mp4');
		expect(text(root)).toContain('sound.mp3');
		expect(text(root)).toContain('readme.pdf');
		expect(text(root)).toContain('inline.txt');
	});
});

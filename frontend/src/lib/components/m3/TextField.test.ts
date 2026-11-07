// @vitest-environment jsdom
import { afterEach, describe, expect, it } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';
import TextField from './TextField.svelte';

let instance: Record<string, unknown> | undefined;
afterEach(() => {
	if (instance) unmount(instance);
	instance = undefined;
	document.body.innerHTML = '';
});

function render(props: Record<string, unknown>) {
	const root = document.body.appendChild(document.createElement('div'));
	instance = mount(TextField, { target: root, props: { label: 'Password', ...props } });
	return root;
}

describe('TextField', () => {
	it('shows and hides what was typed in a secret field', () => {
		const root = render({ type: 'password', value: 'hunter2' });
		const input = root.querySelector('input')!;
		const eye = root.querySelector<HTMLButtonElement>('.m3-field__eye')!;
		expect(input.type).toBe('password');
		expect(eye.type).toBe('button');
		expect(eye.getAttribute('aria-pressed')).toBe('false');

		eye.click();
		flushSync();
		expect(input.type).toBe('text');
		expect(eye.getAttribute('aria-pressed')).toBe('true');

		eye.click();
		flushSync();
		expect(input.type).toBe('password');
	});

	it('has no eye on other fields', () => {
		const root = render({ type: 'email' });
		expect(root.querySelector('.m3-field__eye')).toBeNull();
		expect(root.querySelector('input')!.type).toBe('email');
	});

	it('marks an invalid field and says why under it', () => {
		const root = render({ error: true, supportingText: 'Nope' });
		expect(root.querySelector('input')!.getAttribute('aria-invalid')).toBe('true');
		expect(root.querySelector('.m3-field__supporting--error')?.textContent).toBe('Nope');
	});
});

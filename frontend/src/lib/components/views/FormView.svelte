<!-- Several details to fill in at once. Choices are chips (like the questions card), dates and
     times use the app's pickers; sending posts the answers as one "Label: value" message. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { useChatSend } from '$lib/chatSend';
	import TextField from '$lib/components/m3/TextField.svelte';
	import DatePicker from '$lib/components/m3/DatePicker.svelte';
	import TimePicker from '$lib/components/m3/TimePicker.svelte';
	import Checkbox from '$lib/components/m3/Checkbox.svelte';
	import ViewFrame from './ViewFrame.svelte';
	import { composeForm } from '$lib/views/format';
	import type { FormField, FormView, ViewProps } from '$lib/views/types';

	let { view, answered = false }: ViewProps<FormView> = $props();

	const send = useChatSend();
	const initial = (field: FormField) => (field.default === undefined || typeof field.default === 'boolean' ? '' : String(field.default));
	// svelte-ignore state_referenced_locally
	let values = $state<Record<string, string>>(Object.fromEntries(view.fields.map((f) => [f.key, initial(f)])));
	// svelte-ignore state_referenced_locally
	let picks = $state<Record<string, string[]>>(Object.fromEntries(view.fields.map((f) => [f.key, f.type === 'select' && f.default !== undefined ? [String(f.default)] : []])));
	// svelte-ignore state_referenced_locally
	let checks = $state<Record<string, boolean>>(Object.fromEntries(view.fields.map((f) => [f.key, f.default === true])));
	let sent = $state(false);
	const done = $derived(answered || sent);

	function answer(field: FormField): string {
		switch (field.type) {
			case 'select':
			case 'multiselect':
				return picks[field.key].join(', ');
			case 'checkbox':
				return checks[field.key] ? m.view_yes() : '';
			default:
				return values[field.key] ?? '';
		}
	}

	const missing = $derived(view.fields.some((f) => f.required && answer(f).trim() === ''));
	const message = $derived(composeForm(view.title, view.fields, Object.fromEntries(view.fields.map((f) => [f.key, answer(f)]))));

	function toggle(field: FormField, option: string) {
		if (done) return;
		const current = picks[field.key];
		picks[field.key] = current.includes(option) ? current.filter((o) => o !== option) : field.type === 'multiselect' ? [...current, option] : [option];
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		if (done || missing || !message || !send) return;
		send(message);
		sent = true;
	}
</script>

<ViewFrame title={view.title}>
	<form class="form" onsubmit={submit}>
		{#if view.intro}<p class="form__intro">{view.intro}</p>{/if}
		<fieldset class="form__fields" disabled={done}>
			{#each view.fields as field (field.key)}
				{@const label = field.required ? `${field.label} *` : field.label}
				{#if field.type === 'select' || field.type === 'multiselect'}
					<div class="form__choice" role="group" aria-label={label}>
						<span class="form__label">{label}</span>
						<div class="form__chips">
							{#each field.options ?? [] as option (option)}
								{@const on = picks[field.key].includes(option)}
								<button type="button" class="chip" class:chip--on={on} aria-pressed={on} onclick={() => toggle(field, option)}>
									{#if on}<svg width="16" height="16" viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" /></svg>{/if}
									{option}
								</button>
							{/each}
						</div>
					</div>
				{:else if field.type === 'checkbox'}
					<Checkbox bind:checked={checks[field.key]} {label} disabled={done} />
				{:else if field.type === 'date'}
					<DatePicker {label} bind:value={values[field.key]} placeholder={field.placeholder} required={field.required} disabled={done} />
				{:else if field.type === 'time'}
					<TimePicker {label} bind:value={values[field.key]} placeholder={field.placeholder} required={field.required} disabled={done} />
				{:else if field.type === 'textarea'}
					<label class="form__area">
						<span class="form__label">{label}</span>
						<textarea rows="3" bind:value={values[field.key]} placeholder={field.placeholder} required={field.required}></textarea>
					</label>
				{:else}
					<TextField {label} bind:value={values[field.key]} type={field.type === 'number' ? 'number' : 'text'} inputmode={field.type === 'number' ? 'decimal' : undefined} placeholder={field.placeholder} required={field.required} />
				{/if}
			{/each}
		</fieldset>
		{#if done}
			<p class="form__done">{m.view_answered()}</p>
		{:else}
			<div class="form__actions">
				<button type="submit" class="form__send" disabled={missing || !message || !send}>{view.submit_label ?? m.view_send()}</button>
			</div>
		{/if}
	</form>
</ViewFrame>

<style>
	.form {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.form__intro {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		line-height: 1.45;
	}
	.form__fields {
		display: flex;
		flex-direction: column;
		gap: 14px;
		margin: 0;
		padding: 0;
		border: none;
		min-width: 0;
	}
	.form__label {
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface-variant);
	}
	.form__choice,
	.form__area {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.form__chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		min-height: 36px;
		padding: 6px 14px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.9375rem;
		cursor: pointer;
		transition: background-color var(--nomi-motion-effects-fast);
	}
	.chip:hover:not(:disabled) {
		background: var(--md-sys-color-surface-container-high);
	}
	.chip:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.chip--on {
		border-color: transparent;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.chip:disabled {
		cursor: default;
	}
	.form__area textarea {
		box-sizing: border-box;
		width: 100%;
		padding: 12px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-extra-small);
		background: transparent;
		color: inherit;
		font: inherit;
		resize: vertical;
	}
	.form__area textarea:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -1px;
	}
	.form__actions {
		display: flex;
		justify-content: flex-end;
	}
	.form__send {
		min-height: 40px;
		padding: 0 20px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		font: inherit;
		font-weight: 600;
		cursor: pointer;
	}
	.form__send:hover:not(:disabled) {
		background: color-mix(in srgb, var(--md-sys-color-on-primary) 8%, var(--md-sys-color-primary));
	}
	.form__send:disabled {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 12%, transparent);
		color: color-mix(in srgb, var(--md-sys-color-on-surface) 38%, transparent);
		cursor: default;
	}
	.form__send:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.form__done {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	@media (max-width: 600px) {
		.chip,
		.form__send {
			min-height: 44px;
		}
	}
</style>

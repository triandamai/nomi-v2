<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import { onMount } from 'svelte';
	import AuthShell from '$lib/components/AuthShell.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let code = $state('');
	let submitting = $state(false);
	let formEl = $state<HTMLFormElement>();
	let inputEl = $state<HTMLInputElement>();

	// Seconds until another code can be sent, counted down on the page.
	let resendAt = $state(0);
	let now = $state(Date.now());
	const resendIn = $derived(Math.max(0, Math.ceil((resendAt - now) / 1000)));

	$effect(() => {
		const seconds = form && 'resendIn' in form && typeof form.resendIn === 'number' ? form.resendIn : data.verification?.resend_in;
		if (typeof seconds === 'number') resendAt = Date.now() + seconds * 1000;
	});

	onMount(() => {
		inputEl?.focus();
		const timer = setInterval(() => (now = Date.now()), 1000);
		return () => clearInterval(timer);
	});

	const expired = $derived(data.expired || Boolean(form && 'expired' in form && form.expired));
	const sendsLeft = $derived(form && 'sendsLeft' in form && typeof form.sendsLeft === 'number' ? form.sendsLeft : (data.verification?.sends_left ?? 0));
	const digits = $derived(code.replace(/\D/g, ''));

	function onInput() {
		code = code.replace(/\D/g, '').slice(0, 6);
		// A full code (typed or pasted) goes straight in.
		if (code.length === 6 && !submitting) formEl?.requestSubmit();
	}

	function countdown(seconds: number) {
		return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
	}
</script>

<AuthShell
	title={data.isNew ? m.verify_title_new() : m.verify_title()}
	subtitle={data.verification ? m.verify_subtitle({ email: data.verification.email }) : m.verify_subtitle_expired()}
	switchPrompt={m.verify_wrong_account()}
	switchLabel={m.verify_start_over()}
	switchHref={data.isNew ? '/register' : '/login'}
>
	{#if expired}
		<div class="flex flex-col gap-5">
			<p class="md-body-large" role="alert" style="color: var(--md-sys-color-error)">
				{form?.error ?? m.verify_expired()}
			</p>
			<form method="POST" action="?/cancel">
				<Button type="submit" variant="gradient" size="m" class="w-full">{m.verify_back_to_sign_in()}</Button>
			</form>
		</div>
	{:else}
		<form
			bind:this={formEl}
			method="POST"
			action="?/verify"
			class="flex flex-col gap-5"
			use:enhance={({ cancel }) => {
				if (submitting) return cancel();
				submitting = true;
				return async ({ update, result }) => {
					submitting = false;
					await update({ reset: false });
					if (result.type === 'failure') {
						code = '';
						inputEl?.focus();
					}
				};
			}}
		>
			{#if form?.error && !(form && 'resent' in form)}
				<p class="md-body-medium" role="alert" style="color: var(--md-sys-color-error)">{form.error}</p>
			{/if}
			{#if form && 'resent' in form && form.resent}
				<p class="md-body-medium" role="status" style="color: var(--md-sys-color-primary)">{m.verify_resent()}</p>
			{/if}
			<label class="code">
				<span class="code__label">{m.verify_code_label()}</span>
				<input
					bind:this={inputEl}
					bind:value={code}
					oninput={onInput}
					id="code"
					name="code"
					class="code__input"
					inputmode="numeric"
					autocomplete="one-time-code"
					pattern="[0-9 ]*"
					maxlength="6"
					placeholder="••••••"
					aria-describedby="code-hint"
					required
				/>
				<span class="code__hint" id="code-hint">{m.verify_code_hint()}</span>
			</label>
			<Button type="submit" variant="gradient" size="m" class="w-full" disabled={submitting || digits.length !== 6}>
				{submitting ? m.verify_checking() : m.verify_submit()}
			</Button>
		</form>

		<form method="POST" action="?/resend" class="resend" use:enhance={() => async ({ update }) => update({ reset: false })}>
			{#if sendsLeft <= 0}
				<span class="resend__note">{m.verify_no_more_codes()}</span>
			{:else if resendIn > 0}
				<span class="resend__note">{m.verify_resend_in({ time: countdown(resendIn) })}</span>
			{:else}
				<span class="resend__note">{m.verify_no_code()}</span>
				<button type="submit" class="resend__button">{m.verify_resend()}</button>
			{/if}
		</form>
	{/if}
</AuthShell>

<style>
	.code {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.code__label {
		font-size: var(--md-sys-typescale-body-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
	.code__input {
		width: 100%;
		box-sizing: border-box;
		padding: 14px 16px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-large);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-mono);
		font-size: 2rem;
		font-weight: 600;
		letter-spacing: 0.5em;
		text-align: center;
		text-indent: 0.5em;
	}
	.code__input:focus {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -1px;
		border-color: transparent;
	}
	.code__input::placeholder {
		color: var(--md-sys-color-outline-variant);
	}
	.code__hint {
		font-size: var(--md-sys-typescale-body-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
	.resend {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: center;
		gap: 6px;
		margin-top: 20px;
		font-size: 0.875rem;
	}
	.resend__note {
		color: var(--md-sys-color-on-surface-variant);
	}
	.resend__button {
		padding: 0;
		border: 0;
		background: none;
		color: var(--md-sys-color-primary);
		font: inherit;
		font-weight: 600;
		cursor: pointer;
	}
</style>

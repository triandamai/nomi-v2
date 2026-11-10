<script lang="ts">
	// Auth / Enter code (pUDXz), 1:1: where the code went (· change goes back to the form), the six
	// boxes (a full code goes in by itself), the resend countdown from the backend, the webmail the
	// address uses, the note for a new account, and the ways off. A wrong code says how many tries
	// are left; a locked or expired one offers to start over.
	import { enhance } from '$app/forms';
	import { onMount } from 'svelte';
	import { ArrowLeft } from '@lucide/svelte';
	import AuthLink from '$lib/ds/auth/AuthLink.svelte';
	import AuthNote from '$lib/ds/auth/AuthNote.svelte';
	import CodeInput from '$lib/ds/auth/CodeInput.svelte';
	import MailAppButton from '$lib/ds/auth/MailAppButton.svelte';
	import ErrorCallout from '$lib/ds/conversation/ErrorCallout.svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import { countdown, secondsUntil } from '$lib/auth/format';
	import type { CodeProblem } from '$lib/auth/problems';
	import { m } from '$lib/paraglide/messages';

	let { data, form } = $props();

	let code = $state('');
	let checking = $state(false);
	let verifyForm = $state<HTMLFormElement>();
	let cancelForm = $state<HTMLFormElement>();
	const loaded = () => data;
	let resendAt = $state(Date.now() + loaded().resendIn * 1000);
	let sendsLeft = $state(loaded().sendsLeft);
	let now = $state(Date.now());
	const resendIn = $derived(secondsUntil(resendAt, now));

	const problem = $derived<CodeProblem | null>(form?.problem ?? null);
	const ended = $derived(data.expired || problem === 'expired' || problem === 'locked');
	const errorId = 'code-error';

	// The backend's numbers after a resend (or "wait N seconds") restart the countdown.
	$effect(() => {
		if (!form) return;
		if (form.resent && form.resendIn !== undefined && form.sendsLeft !== undefined) {
			resendAt = Date.now() + form.resendIn * 1000;
			sendsLeft = form.sendsLeft;
		} else if (form.problem === 'wait' && form.seconds) {
			resendAt = Date.now() + form.seconds * 1000;
		} else if (form.problem === 'too_many_codes') {
			sendsLeft = 0;
		}
	});

	onMount(() => {
		const timer = setInterval(() => (now = Date.now()), 1000);
		return () => clearInterval(timer);
	});

	function sayLine(): { text: string; tone: 'danger' | 'quiet' } | null {
		if (checking) return { text: m.verify_checking(), tone: 'quiet' };
		if (form?.resent) return { text: m.verify_resent(), tone: 'quiet' };
		switch (problem) {
			case 'wrong_code':
				return { text: form?.attemptsLeft === 1 ? m.verify_wrong_last() : m.verify_wrong({ left: String(form?.attemptsLeft ?? 0) }), tone: 'danger' };
			case 'incomplete':
				return { text: m.verify_incomplete(), tone: 'danger' };
			case 'wait':
				return { text: m.verify_wait(), tone: 'quiet' };
			case 'too_many_codes':
				return { text: m.verify_too_many_codes(), tone: 'quiet' };
			case 'email_failed':
				return { text: m.verify_email_failed(), tone: 'danger' };
			case 'unreachable':
				return { text: m.auth_unreachable(), tone: 'danger' };
			case 'failed':
				return { text: m.verify_failed(), tone: 'danger' };
			default:
				return null;
		}
	}
	const line = $derived(sayLine());
</script>

<svelte:head><title>{m.verify_page_title()}</title></svelte:head>

<form method="POST" action="?/cancel" bind:this={cancelForm}>
	<button
		type="submit"
		aria-label={m.verify_change_label({ email: data.email })}
		class="flex cursor-pointer items-center gap-1.5 rounded-xs text-caption outline-offset-2 hover:underline hover:decoration-text-3 hover:underline-offset-2 max-sm:min-h-11"
	>
		<span class="inline-flex text-text-2"><Icon icon={ArrowLeft} size={14} /></span>
		<span class="font-mono text-text-2">{data.email}</span>
		<span class="text-text-3">{m.verify_change()}</span>
	</button>
</form>

<div class="flex flex-col gap-1.5">
	<h1 class="text-display font-medium text-text">{m.verify_title()}</h1>
	<p class="text-body leading-normal text-text-2">
		{data.minutes && !ended ? m.verify_sub({ email: data.email, minutes: String(data.minutes) }) : m.verify_sub_short({ email: data.email })}
	</p>
</div>

{#if ended}
	<ErrorCallout
		title={problem === 'locked' ? m.verify_locked() : m.verify_expired()}
		code={problem === 'locked' ? '410 · too_many_attempts' : '410 · expired'}
		actions={[{ label: m.verify_start_over(), onclick: () => cancelForm?.requestSubmit() }]}
	/>
{:else}
	<form
		method="POST"
		action="?/verify"
		bind:this={verifyForm}
		class="flex flex-col gap-5"
		use:enhance={({ cancel }) => {
			if (checking) return cancel();
			checking = true;
			return async ({ result, update }) => {
				await update({ reset: false });
				checking = false;
				if (result.type === 'failure') code = '';
			};
		}}
	>
		<CodeInput
			name="code"
			bind:value={code}
			autofocus
			invalid={problem === 'wrong_code'}
			describedby={line?.tone === 'danger' ? errorId : undefined}
			oncomplete={() => verifyForm?.requestSubmit()}
		/>
	</form>

	<div class="flex items-center gap-2">
		{#if line}
			<p id={errorId} role={line.tone === 'danger' ? 'alert' : 'status'} class={['min-w-0 flex-1 text-caption', line.tone === 'danger' ? 'text-danger' : 'text-text-2']}>{line.text}</p>
		{:else}
			<p class="min-w-0 flex-1 font-mono text-label text-text-3">{m.verify_paste()}</p>
		{/if}
		{#if sendsLeft <= 0}
			<span class="font-mono text-label text-text-3">{m.verify_no_more()}</span>
		{:else if resendIn > 0}
			<span class="font-mono text-label text-text-2" role="timer">{m.verify_resend_in({ time: countdown(resendIn) })}</span>
		{:else}
			<form method="POST" action="?/resend" use:enhance={() => async ({ update }) => update({ reset: false })}>
				<button
					type="submit"
					class="cursor-pointer rounded-xs font-mono text-label text-text-2 outline-offset-2 hover:text-text hover:underline hover:underline-offset-2 max-sm:min-h-11">{m.verify_resend()}</button
				>
			</form>
		{/if}
	</div>

	{#if data.mailApp}
		<div class="flex gap-2"><MailAppButton app={data.mailApp} /></div>
	{/if}
{/if}

{#if data.isNew}<AuthNote text={m.verify_new_note()} />{/if}

<p class="flex flex-wrap items-center gap-x-4">
	<AuthLink label={m.auth_back_to_sign_in()} href="/sign-in" />
	<span class="text-caption text-text-3">{m.verify_spam()}</span>
</p>

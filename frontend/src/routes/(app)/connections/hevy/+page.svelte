<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	// Connecting Hevy: the person pastes their own API key (Hevy Pro → Settings → Developer). Nomi
	// checks it with Hevy before saving it; Raga then works in their Hevy account.
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import { timeAgo } from '$lib/i18n';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let apiKey = $state('');
	let saving = $state(false);
	let confirmOpen = $state(false);
	let justConnected = $state(false);

	const ERRORS: Record<string, () => string> = {
		missing_key: m.hevy_err_missing,
		key_rejected: m.hevy_err_rejected,
		unreachable: m.hevy_err_unreachable,
		rejected: m.hevy_err_rejected,
		disconnect_failed: m.hevy_err_disconnect,
	};
	const errorText = $derived(form && 'error' in form && form.error ? (ERRORS[form.error] ?? m.hevy_err_server)() : null);

	const CAN_DO = [m.hevy_can_log, m.hevy_can_routines, m.hevy_can_progress, m.hevy_can_measure];
</script>

<div class="page">
	<div class="page__inner">
		<a class="back" href="/connections">
			<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M15 18l-6-6 6-6" /></svg>
			{m.conn_title()}
		</a>
		<header class="head">
			<AgentShape agent="health" size={56} />
			<div>
				<h1 class="md-display-small title">Hevy</h1>
				<p class="md-body-large lede">{m.hevy_lede()}</p>
			</div>
		</header>

		{#if !data.loaded}
			<p class="notice" role="alert">{m.conn_load_failed()}</p>
		{:else if data.connection}
			<section class="card" aria-labelledby="hevy-account">
				<h2 id="hevy-account" class="card__title">{data.connection.account_name ?? m.hevy_your_account()}</h2>
				<p class="card__meta">
					<span class="on">{m.hevy_connected()}</span>
					· {m.hevy_connected_since({ when: timeAgo(data.connection.connected_at) })}
					{#if data.connection.last_used_at}· {m.hevy_last_used({ when: timeAgo(data.connection.last_used_at) })}{/if}
				</p>
				<p class="card__body">{m.hevy_only_you()}</p>
				<div class="actions">
					{#if data.resume && justConnected}
						<Button variant="filled" size="m" href="/chat/{data.resume}">{m.hevy_back_to_chat()}</Button>
					{/if}
					<Button variant="outlined" size="m" onclick={() => (confirmOpen = true)}>{m.conn_disconnect()}</Button>
				</div>
			</section>
		{:else}
			{#if data.keyRejected}
				<p class="notice" role="alert">{m.hevy_key_stopped()}</p>
			{/if}
			<section class="card" aria-labelledby="hevy-connect">
				<h2 id="hevy-connect" class="card__title">{m.hevy_connect_title()}</h2>
				<ol class="steps">
					<li><span class="steps__n">1</span><span>{m.hevy_step_pro()}</span></li>
					<li>
						<span class="steps__n">2</span>
						<span>{m.hevy_step_key()} <a href="https://hevy.com/settings?developer" target="_blank" rel="noreferrer">hevy.com/settings?developer</a></span>
					</li>
					<li><span class="steps__n">3</span><span>{m.hevy_step_paste()}</span></li>
				</ol>
				<form
					method="POST"
					action="?/connect"
					class="form"
					use:enhance={() => {
						saving = true;
						return async ({ result, update }) => {
							saving = false;
							if (result.type === 'success') {
								apiKey = '';
								justConnected = true;
								await invalidateAll();
							} else {
								await update({ reset: false });
							}
						};
					}}
				>
					<TextField id="hevy_api_key" name="api_key" type="password" autocomplete="off" label={m.hevy_key_label()} bind:value={apiKey} />
					{#if errorText}
						<p class="error" role="alert">{errorText}</p>
					{/if}
					<div class="actions">
						<Button type="submit" variant="filled" size="m" disabled={saving || !apiKey.trim()}>{saving ? m.hevy_checking() : m.hevy_connect()}</Button>
					</div>
				</form>
				<p class="card__note">{m.hevy_key_note()}</p>
			</section>
		{/if}

		<section class="can" aria-labelledby="hevy-can">
			<h2 id="hevy-can" class="can__title">{m.hevy_can_title()}</h2>
			<ul class="can__list">
				{#each CAN_DO as line, i (i)}
					<li>{line()}</li>
				{/each}
			</ul>
			<p class="can__soon">{m.hevy_soon_monitors()}</p>
		</section>
	</div>
</div>

<Dialog bind:open={confirmOpen} headline={m.hevy_disconnect_title()}>
	<p class="dialog-body">{m.hevy_disconnect_body()}</p>
	{#snippet actions()}
		<Button variant="text" onclick={() => (confirmOpen = false)}>{m.common_cancel()}</Button>
		<form
			method="POST"
			action="?/disconnect"
			use:enhance={() => async ({ update }) => {
				confirmOpen = false;
				await update();
			}}
		>
			<Button type="submit" variant="filled">{m.conn_disconnect()}</Button>
		</form>
	{/snippet}
</Dialog>

<style>
	.page {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.page__inner {
		display: flex;
		flex-direction: column;
		gap: 20px;
		max-width: 640px;
		margin: 0 auto;
	}
	.back {
		display: inline-flex;
		align-items: center;
		align-self: flex-start;
		gap: 4px;
		min-height: 44px;
		margin-left: -6px;
		padding: 0 10px 0 4px;
		border-radius: var(--md-sys-shape-corner-full);
		color: var(--md-sys-color-on-surface-variant);
		font-weight: 600;
		text-decoration: none;
	}
	.back:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.back:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 16px;
	}
	.title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.lede {
		margin: 6px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.notice,
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.card {
		display: flex;
		flex-direction: column;
		gap: 14px;
		padding: 22px 24px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.card__title {
		margin: 0;
		font-size: 1.25rem;
		font-weight: 700;
	}
	.card__meta,
	.card__body,
	.card__note {
		margin: 0;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.card__note {
		font-size: 0.8125rem;
	}
	.on {
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}
	.steps {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.steps li {
		display: flex;
		align-items: baseline;
		gap: 12px;
		font-size: 0.9375rem;
	}
	.steps__n {
		display: grid;
		flex: none;
		place-items: center;
		width: 24px;
		height: 24px;
		border-radius: 50%;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-size: 0.8125rem;
		font-weight: 700;
	}
	.steps a {
		color: var(--md-sys-color-primary);
		overflow-wrap: anywhere;
	}
	.form {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.can {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 0 4px;
	}
	.can__title {
		margin: 0;
		font-size: 1rem;
		font-weight: 650;
		color: var(--md-sys-color-on-surface);
	}
	.can__list {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin: 0;
		padding-left: 20px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.can__soon {
		margin: 4px 0 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.dialog-body {
		margin: 0;
	}
</style>

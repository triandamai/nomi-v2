<script lang="ts">
	// A shared reply or chat: read-only, no composer, no actions.
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import AgentByline from '$lib/components/ds/identity/AgentByline.svelte';
	import Button from '$lib/components/ds/actions/Button.svelte';
	import Radio from '$lib/components/m3/Radio.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import { createCopier } from '$lib/components/ds/actions/copier.svelte';
	import { crewId } from '$lib/crewId';
	import { REPORT_REASONS, reasonLabel, sharedMeta, viewsLabel } from '$lib/sharedPage';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const copier = createCopier();
	let reporting = $state(false);
	let reason = $state('');
	let note = $state('');

	const time = (iso: string) => new Date(iso).toLocaleString(getLocale(), { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' });
</script>

<svelte:head><title>{m.shared_title()}</title><meta name="robots" content="noindex" /></svelte:head>

<main class="shared">
	{#if data.state === 'ready'}
		{#if data.chat.own && data.chat.share_id && data.chat.session_id}
			<section class="shared__owner" aria-label={m.shared_owner_label()}>
				<p class="shared__owner-text">{m.shared_owner()} {viewsLabel(data.chat.views ?? 0)}</p>
				<div class="shared__owner-actions">
					<Button variant="secondary" label={copier.copied ? m.shared_copied() : m.share_copy()} onclick={() => copier.copy(page.url.href)} />
					<form method="POST" action="?/stop" use:enhance>
						<input type="hidden" name="session_id" value={data.chat.session_id} />
						<input type="hidden" name="share_id" value={data.chat.share_id} />
						<Button variant="ghost-danger" type="submit" label={m.shared_stop()} />
					</form>
				</div>
				{#if form && 'stopError' in form && form.stopError}<p class="shared__note" role="alert">{form.stopError}</p>{/if}
			</section>
		{/if}
		<header class="shared__head">
			<p class="nomi-meta">{m.shared_title()} · {sharedMeta(data.chat.shared_by, data.chat.shared_at, getLocale())}</p>
			{#if data.chat.title}<h1 class="md-headline-medium">{data.chat.title}</h1>{/if}
			{#if data.chat.masked}<p class="shared__note">{m.shared_masked()}</p>{/if}
		</header>
		<ol class="shared__messages">
			{#each data.chat.messages as message, index (index)}
				<li class="shared__message" class:shared__message--user={message.sender === 'user'}>
					{#if message.sender === 'assistant'}
						<AgentByline agent={crewId(message.agent_display_name)} name={message.agent_display_name ?? 'Nomi'} meta={time(message.created_at)} />
					{/if}
					<div class="shared__text message-markdown">{@html message.html}</div>
				</li>
			{/each}
		</ol>
		<footer class="shared__foot">
			{#if data.chat.expires_at}<p class="shared__note">{m.shared_expires({ when: time(data.chat.expires_at) })}</p>{/if}
			{#if form && 'reported' in form && form.reported}
				<p class="shared__note" role="status">{m.shared_report_sent()}</p>
			{:else if !data.chat.own}
				{#if reporting}
					<form class="shared__report" method="POST" action="?/report" use:enhance>
						<fieldset class="shared__reasons">
							<legend class="shared__legend">{m.shared_report_why()}</legend>
							{#each REPORT_REASONS as option (option)}
								<Radio name="reason" value={option} label={reasonLabel(option)} bind:group={reason} />
							{/each}
						</fieldset>
						<TextField label={m.shared_report_note()} name="note" bind:value={note} maxlength={1000} />
						{#if form && 'reportError' in form && form.reportError}<p class="shared__note" role="alert">{form.reportError}</p>{/if}
						<div class="shared__owner-actions">
							<Button type="submit" variant="inverse" label={m.shared_report_send()} disabled={!reason} />
							<Button variant="ghost" label={m.shared_report_cancel()} onclick={() => (reporting = false)} />
						</div>
					</form>
				{:else}
					<button type="button" class="shared__report-link" onclick={() => (reporting = true)}>{m.shared_report()}</button>
				{/if}
			{/if}
		</footer>
	{:else if data.state === 'sign_in'}
		<p class="shared__note">{m.shared_sign_in()}</p>
		<Button label={m.shared_sign_in_action()} href="/login?redirect_to={encodeURIComponent(page.url.pathname)}" />
	{:else}
		<p class="shared__note">{form && 'stopped' in form && form.stopped ? m.shared_stopped() : m.shared_gone()}</p>
	{/if}
</main>

<style>
	:global(body) {
		background: var(--nomi-color-ground);
	}
	.shared {
		display: flex;
		max-width: 720px;
		flex-direction: column;
		gap: 24px;
		margin: 0 auto;
		padding: 48px 16px;
		color: var(--md-sys-color-on-surface);
	}
	.shared__head {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.shared__head h1,
	.shared__note {
		margin: 0;
	}
	.shared__note {
		color: var(--md-sys-color-on-surface-variant);
		font-size: var(--md-sys-typescale-body-medium-size);
	}
	.shared__messages {
		display: flex;
		flex-direction: column;
		gap: 20px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.shared__message {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.shared__message--user {
		align-items: flex-end;
	}
	.shared__message--user .shared__text {
		max-width: 80%;
		padding: 10px 14px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.shared__owner {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.shared__owner-text {
		margin: 0;
		color: var(--md-sys-color-on-surface);
		font-size: var(--md-sys-typescale-body-medium-size);
	}
	.shared__owner-actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}
	.shared__owner-actions form {
		margin: 0;
	}
	.shared__foot,
	.shared__report {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.shared__reasons {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin: 0;
		padding: 0;
		border: 0;
	}
	.shared__legend {
		margin-bottom: 4px;
		color: var(--md-sys-color-on-surface);
		font-size: var(--md-sys-typescale-body-medium-size);
	}
	.shared__report-link {
		align-self: flex-start;
		min-height: 44px;
		padding: 0 4px;
		border: 0;
		border-radius: var(--md-sys-shape-corner-small);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: var(--md-sys-typescale-body-medium-size);
		text-decoration: underline;
		cursor: pointer;
	}
	.shared__report-link:hover {
		color: var(--md-sys-color-on-surface);
	}
	.shared__report-link:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.shared__text {
		font-size: var(--md-sys-typescale-body-large-size);
		line-height: var(--md-sys-typescale-body-large-line-height);
	}
</style>

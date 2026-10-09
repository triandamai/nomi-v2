<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { enhance } from '$app/forms';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import { refreshUnread } from '$lib/notifications.svelte';
	import type { NotificationKind } from '$lib/types';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const items = $derived(data.inbox.items);
	let emailAccount = $state(true);
	let emailPromos = $state(true);
	$effect(() => {
		emailAccount = data.preferences.email_account;
		emailPromos = data.preferences.email_promos;
	});
	let prefsForm: HTMLFormElement | undefined = $state();

	// Each kind wears a crew look: Nomi for the account, the citrus reminder look for promos.
	const KIND: Record<NotificationKind, { agent: string; label: () => string }> = {
		account: { agent: 'nomi', label: m.notif_kind_account },
		subscription: { agent: 'supervisor', label: m.notif_kind_subscription },
		quota: { agent: 'money', label: m.notif_kind_quota },
		promo: { agent: 'reminders', label: m.notif_kind_promo },
	};

	function when(at: string): string {
		const date = new Date(at);
		const sameDay = date.toDateString() === new Date().toDateString();
		return new Intl.DateTimeFormat(getLocale(), sameDay ? { timeStyle: 'short' } : { dateStyle: 'medium' }).format(date);
	}
</script>

<div class="inbox">
	<div class="inbox__inner">
		<header class="inbox__head">
			<div>
				<h1 class="md-display-small inbox__title">{m.notif_title()}</h1>
				<p class="md-body-large inbox__lede">{m.notif_lede()}</p>
			</div>
			{#if data.inbox.unread > 0}
				<form method="POST" action="?/readAll" use:enhance={() => async ({ update }) => { await update(); void refreshUnread(); }}>
					<Button type="submit" variant="tonal" size="s">{m.notif_read_all()}</Button>
				</form>
			{/if}
		</header>

		{#if items.length === 0}
			<div class="empty">
				<AgentShape agent="nomi" face size={64} />
				<p>{data.older ? m.notif_no_older() : m.notif_empty()}</p>
			</div>
		{:else}
			<ul class="list">
				{#each items as item (item.id)}
					{@const kind = KIND[item.kind] ?? KIND.account}
					<li>
						<form method="POST" action="?/open" use:enhance={() => async ({ update }) => { await update(); void refreshUnread(); }}>
							<input type="hidden" name="id" value={item.id} />
							{#if item.link}<input type="hidden" name="link" value={item.link} />{/if}
							<button type="submit" class="item" class:item--unread={!item.read}>
								<AgentShape agent={kind.agent} size={40} />
								<span class="item__text">
									<span class="item__meta">
										<span class="item__kind">{kind.label()}</span>
										<time datetime={item.created_at}>{when(item.created_at)}</time>
									</span>
									<span class="item__title">{item.title}</span>
									<span class="item__body">{item.body}</span>
								</span>
								{#if !item.read}<span class="item__dot" aria-label={m.notif_unread()}></span>{/if}
							</button>
						</form>
					</li>
				{/each}
			</ul>
			{#if data.inbox.next_before}
				<a class="older" href={`?before=${encodeURIComponent(data.inbox.next_before)}`}>{m.notif_older()}</a>
			{/if}
		{/if}

		<section class="prefs" aria-labelledby="prefs-title">
			<h2 id="prefs-title" class="prefs__title">{m.notif_email_title()}</h2>
			<form method="POST" action="?/preferences" bind:this={prefsForm} use:enhance={() => async ({ update }) => update({ reset: false })}>
				<input type="hidden" name="email_account" value={String(emailAccount)} />
				<input type="hidden" name="email_promos" value={String(emailPromos)} />
				<div class="pref">
					<span class="pref__text">
						<span class="pref__label">{m.notif_email_account()}</span>
						<span class="pref__hint">{m.notif_email_account_hint()}</span>
					</span>
					<Switch bind:checked={emailAccount} aria-label={m.notif_email_account()} onchange={() => queueMicrotask(() => prefsForm?.requestSubmit())} />
				</div>
				<div class="pref">
					<span class="pref__text">
						<span class="pref__label">{m.notif_email_promos()}</span>
						<span class="pref__hint">{m.notif_email_promos_hint()}</span>
					</span>
					<Switch bind:checked={emailPromos} aria-label={m.notif_email_promos()} onchange={() => queueMicrotask(() => prefsForm?.requestSubmit())} />
				</div>
			</form>
		</section>
	</div>
</div>

<style>
	.inbox {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.inbox__inner {
		display: flex;
		flex-direction: column;
		gap: 16px;
		max-width: 720px;
		margin: 0 auto;
	}
	.inbox__head {
		display: flex;
		align-items: flex-end;
		justify-content: space-between;
		flex-wrap: wrap;
		gap: 12px;
	}
	.inbox__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.inbox__lede {
		margin: 4px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 48px 16px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface-variant);
		text-align: center;
	}
	.list {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.item {
		display: flex;
		align-items: flex-start;
		gap: 14px;
		width: 100%;
		padding: 14px 16px;
		border: none;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.item:hover {
		background: var(--md-sys-color-surface-container);
	}
	.item:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.item--unread {
		background: var(--md-sys-color-surface-container-high);
	}
	.item__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.item__meta {
		display: flex;
		gap: 8px;
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.item__kind {
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}
	.item__title {
		font-weight: 700;
	}
	.item--unread .item__title {
		font-weight: 800;
	}
	.item__body {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
		white-space: pre-line;
	}
	.item__dot {
		flex: none;
		width: 10px;
		height: 10px;
		margin-top: 6px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-primary);
	}
	.older {
		align-self: center;
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}
	.prefs {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-top: 16px;
		padding: 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
	}
	.prefs__title {
		margin: 0 0 4px;
		font-family: var(--md-sys-typescale-title-medium-font);
		font-size: var(--md-sys-typescale-title-medium-size);
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.pref {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding: 8px 0;
	}
	.pref__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.pref__label {
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}
	.pref__hint {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>

<!-- Settings › Data: the person's share links that still open, each with a way to revoke it. -->
<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { shareSummary } from '$lib/sharedPage';
	import type { ShareLink } from './+page.server';

	let { shares }: { shares: ShareLink[] } = $props();
</script>

{#if shares.length === 0}
	<p class="empty">{m.account_shares_empty()}</p>
{:else}
	<ul class="rows">
		{#each shares as share (share.id)}
			<li class="row">
				<span class="row__text">
					<a class="row__headline" href="/chat/{share.session_id}">{share.title || m.account_share_untitled()}</a>
					<span class="row__supporting">{shareSummary(share, getLocale())}</span>
				</span>
				<form method="POST" action="?/revokeShare" use:enhance>
					<input type="hidden" name="id" value={share.id} />
					<input type="hidden" name="session_id" value={share.session_id} />
					<Button variant="text" size="s" type="submit">{m.account_share_revoke()}</Button>
				</form>
			</li>
		{/each}
	</ul>
{/if}

<style>
	.rows {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin: 0;
		padding: 0;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-extra-large);
		list-style: none;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 64px;
		padding: 10px 12px 10px 14px;
		background: var(--md-sys-color-surface-container-lowest);
	}
	.row__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.row__headline {
		overflow: hidden;
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
		font-weight: 600;
		text-decoration: none;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.row__headline:hover {
		text-decoration: underline;
	}
	.row__headline:focus-visible {
		border-radius: var(--md-sys-shape-corner-extra-small);
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.row__supporting {
		overflow-wrap: anywhere;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.empty {
		margin: 0 8px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
</style>

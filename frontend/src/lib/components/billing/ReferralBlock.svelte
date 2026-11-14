<script lang="ts">
	// "Invite friends" on Billing: the person's referral code, what it earns, and how it's going.
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import type { ReferralInfo } from '$lib/types';

	let { referral }: { referral: ReferralInfo } = $props();

	/** How long "Copied" shows after the code is copied. */
	const COPIED_MS = 2000;
	let copied = $state(false);
	let failed = $state(false);

	async function copy() {
		failed = false;
		try {
			await navigator.clipboard.writeText(referral.code);
			copied = true;
			setTimeout(() => (copied = false), COPIED_MS);
		} catch {
			failed = true;
		}
	}
</script>

<section class="panel" aria-labelledby="invite-heading">
	<h2 id="invite-heading" class="panel__title">{m.billing_invite()}</h2>
	<p class="muted">{m.billing_invite_sentence({ percent: referral.referee_discount_percent, days: referral.referrer_bonus_days })}</p>
	<div class="code">
		<code class="code__value">{referral.code}</code>
		<Button variant="tonal" size="s" onclick={copy}>{copied ? m.billing_invite_copied() : m.billing_invite_copy()}</Button>
	</div>
	{#if failed}<p class="error" role="alert">{m.billing_invite_copy_failed()}</p>{/if}
	<p class="muted">{m.billing_invite_counts({ referred: referral.referred_count, rewarded: referral.rewarded_count })}</p>
</section>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		gap: 10px;
		min-width: 0;
		padding: 18px 20px;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.panel__title {
		margin: 0;
		font-size: 1.0625rem;
		font-weight: 700;
	}
	.muted {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.code {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 10px;
	}
	.code__value {
		padding: 8px 14px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-low);
		font-family: var(--md-ref-typeface-mono);
		font-size: 1.125rem;
		font-weight: 700;
		letter-spacing: 0.08em;
		overflow-wrap: anywhere;
	}
</style>

<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	// The month's tokens ran out mid-chat: say so where the person is looking, with how far over
	// they are and the two ways on (their own model key, or a bigger plan).
	import { page } from '$app/state';
	import Button from '$lib/components/m3/Button.svelte';
	import WavyProgress from '$lib/components/m3/WavyProgress.svelte';
	import { formatTokens, usageShare } from '$lib/usage';
	import type { UsageBrief } from '$lib/types';

	const usage = $derived((page.data as { usage?: UsageBrief | null }).usage ?? null);
</script>

<div class="quota" role="status">
	<div class="quota__head">
		<span class="quota__icon" aria-hidden="true">
			<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<path d="M12 9v4M12 17h.01" />
				<path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z" />
			</svg>
		</span>
		<p class="quota__title">{m.quota_card_title()}</p>
	</div>
	<p class="quota__body">{m.quota_card_body()}</p>
	{#if usage}
		<div class="quota__meter">
			<WavyProgress value={usageShare(usage.tokens_used, usage.plan.monthly_tokens)} tone="ember" label={m.usage_of_tokens({ used: formatTokens(usage.tokens_used), total: formatTokens(usage.plan.monthly_tokens) })} />
			<span class="quota__detail">{m.usage_of_tokens({ used: formatTokens(usage.tokens_used), total: formatTokens(usage.plan.monthly_tokens) })}</span>
		</div>
	{/if}
	<div class="quota__actions">
		<Button variant="filled" size="s" href="/models">{m.quota_card_add_key()}</Button>
		<Button variant="tonal" size="s" href="/billing">{m.quota_card_plans()}</Button>
	</div>
</div>

<style>
	.quota {
		display: flex;
		flex-direction: column;
		gap: 10px;
		max-width: 420px;
		padding: 18px 20px;
		border-radius: var(--nomi-shape-bubble-start);
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	.quota__head {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.quota__icon {
		display: inline-flex;
		flex: none;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border-radius: 50%;
		background: var(--md-sys-color-error);
		color: var(--md-sys-color-on-error);
	}
	.quota__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.125rem;
		font-weight: 650;
		line-height: 1.35;
	}
	.quota__body {
		margin: 0;
		font-size: 0.875rem;
		line-height: 1.5;
	}
	.quota__meter {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.quota__detail {
		font-size: 0.75rem;
		opacity: 0.85;
	}
	.quota__actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.quota .quota__actions :global(a) {
		text-decoration: none;
	}
</style>

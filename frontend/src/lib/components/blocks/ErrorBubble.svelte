<script lang="ts">
	// Something stopped the crew, said where the person is looking: what went wrong in plain words
	// and, when running it again can help, a Try again button right on it.
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';

	let {
		text,
		onretry,
		retrying = false,
		retryError = null,
	}: {
		text: string;
		/** Offers Try again; left out when there's nothing to rerun. */
		onretry?: () => void;
		retrying?: boolean;
		/** Why the last Try again didn't go through. */
		retryError?: string | null;
	} = $props();
</script>

<div class="error-bubble" role="alert">
	<span class="error-bubble__icon" aria-hidden="true">
		<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
			<circle cx="12" cy="12" r="9" />
			<path d="M12 8v4.5M12 16h.01" />
		</svg>
	</span>
	<div class="error-bubble__body">
		<p class="error-bubble__text">{text}</p>
		{#if retryError}<p class="error-bubble__detail">{retryError}</p>{/if}
		{#if onretry}
			<div class="error-bubble__actions">
				<Button variant="filled" size="s" disabled={retrying} onclick={onretry}>
					{retrying ? m.run_failed_retrying() : m.run_failed_retry()}
				</Button>
			</div>
		{/if}
	</div>
</div>

<style>
	.error-bubble {
		display: flex;
		gap: 12px;
		align-items: flex-start;
		max-width: 460px;
		padding: 14px 16px;
		border-radius: var(--nomi-shape-bubble-start);
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	.error-bubble__icon {
		display: inline-flex;
		flex: none;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: 50%;
		background: var(--md-sys-color-error);
		color: var(--md-sys-color-on-error);
	}
	.error-bubble__body {
		display: flex;
		flex-direction: column;
		gap: 10px;
		min-width: 0;
		padding-top: 5px;
	}
	.error-bubble__text {
		margin: 0;
		font-size: 0.9375rem;
		line-height: 1.5;
		overflow-wrap: anywhere;
		white-space: pre-line;
	}
	.error-bubble__detail {
		margin: 0;
		font-size: 0.8125rem;
		opacity: 0.85;
	}
	.error-bubble__actions {
		display: flex;
		gap: 8px;
	}
</style>

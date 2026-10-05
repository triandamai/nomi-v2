<script lang="ts">
	let {
		open = $bindable(false),
		message,
		actionLabel,
		onaction,
		timeout = 4000,
	}: {
		open?: boolean;
		message: string;
		actionLabel?: string;
		onaction?: () => void;
		/** ms before auto-dismiss; 0 keeps it until dismissed. */
		timeout?: number;
	} = $props();

	$effect(() => {
		if (!open || timeout <= 0) return;
		const handle = setTimeout(() => (open = false), timeout);
		return () => clearTimeout(handle);
	});
</script>

{#if open}
	<div class="m3-snackbar" role="status">
		<span class="m3-snackbar__message">{message}</span>
		{#if actionLabel}
			<button
				type="button"
				class="m3-snackbar__action"
				onclick={() => {
					onaction?.();
					open = false;
				}}
			>
				{actionLabel}
			</button>
		{/if}
	</div>
{/if}

<style>
	.m3-snackbar {
		position: fixed;
		left: 50%;
		bottom: 24px;
		z-index: 60;
		translate: -50% 0;
		display: flex;
		align-items: center;
		gap: 12px;
		min-width: min(344px, calc(100vw - 32px));
		max-width: calc(100vw - 32px);
		box-sizing: border-box;
		padding: 12px 8px 12px 18px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-inverse-surface);
		color: var(--md-sys-color-inverse-on-surface);
		box-shadow: var(--md-sys-elevation-shadow-level3);
		animation: m3-snackbar-in var(--nomi-motion-spatial-default);
	}
	.m3-snackbar__message {
		flex: 1;
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: var(--md-sys-typescale-body-medium-size);
	}
	.m3-snackbar__action {
		height: 40px;
		padding: 0 14px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: var(--md-sys-color-inverse-primary);
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: var(--md-sys-typescale-label-large-size);
		font-weight: 600;
		cursor: pointer;
	}
	@keyframes m3-snackbar-in {
		from {
			translate: -50% 24px;
			opacity: 0;
		}
	}
</style>

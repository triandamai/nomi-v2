<script lang="ts">
	import { paymentStatusLabel } from '$lib/billing';
	import type { PaymentStatus } from '$lib/types';

	// A payment's state as a small badge: colour from the theme roles, and the word beside it so
	// it never relies on colour alone.
	let { status }: { status: PaymentStatus } = $props();
</script>

<span class="status status--{status}">{paymentStatusLabel(status)}</span>

<style>
	.status {
		display: inline-block;
		padding: 2px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		font-size: 0.75rem;
		font-weight: 700;
		white-space: nowrap;
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
	}
	.status--paid {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.status--pending {
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.status--failed,
	.status--expired {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
</style>

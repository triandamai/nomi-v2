<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Toast from './Toast.svelte';
	import { toasts } from './toasts.svelte';
</script>

<section aria-label={m.toast_region()} class="region">
	{#each toasts.items as toast (toast.id)}
		<div class="slot">
			<Toast
				kind={toast.kind}
				title={toast.title}
				detail={toast.detail}
				icon={toast.icon}
				action={toast.action && {
					label: toast.action.label,
					onclick: () => {
						toast.action?.onclick();
						toasts.dismiss(toast.id);
					},
				}}
				ondismiss={() => toasts.dismiss(toast.id)}
			/>
		</div>
	{/each}
</section>

<style>
	.region {
		position: fixed;
		right: 16px;
		bottom: 16px;
		z-index: 50;
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 8px;
		pointer-events: none;
	}
	@media (max-width: 599px) {
		.region {
			left: 16px;
		}
	}
	.slot {
		pointer-events: auto;
	}
</style>

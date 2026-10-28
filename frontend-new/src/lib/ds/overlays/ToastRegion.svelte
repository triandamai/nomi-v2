<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Toast from './Toast.svelte';
	import { toasts } from './toasts.svelte';
</script>

<section aria-label={m.toast_region()} class="pointer-events-none fixed right-4 bottom-4 z-50 flex flex-col items-end gap-2 max-sm:left-4">
	{#each toasts.items as toast (toast.id)}
		<div class="pointer-events-auto">
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

<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Toast from './Toast.svelte';
	import { raise, topLayer } from './topLayer';
	import { toasts } from './toasts.svelte';

	// Bottom right, 400 wide, the full width less 16 a side on a narrow screen. It sits in the top
	// layer (topLayer) so a dialog or sheet never covers it, and comes forward with each new toast.
	let region = $state<HTMLElement>();
	$effect(() => {
		if (toasts.items.length && region) raise(region);
	});
</script>

<div class="contents">
	<section
		bind:this={region}
		{@attach topLayer}
		popover="manual"
		aria-label={m.toast_region()}
		class="pointer-events-none fixed inset-auto right-4 bottom-4 m-0 flex w-fit flex-col items-end gap-2 overflow-visible border-0 bg-transparent p-0 max-sm:left-4 max-sm:w-auto"
	>
		{#each toasts.items as toast (toast.id)}
			<div class="pointer-events-auto w-100 max-w-full">
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
</div>

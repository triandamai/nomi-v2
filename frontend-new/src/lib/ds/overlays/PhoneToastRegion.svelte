<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import PhoneToast from '../mobile/PhoneToast.svelte';
	import { raise, topLayer } from './topLayer';
	import { toasts } from './toasts.svelte';

	// Flow / Mobile Build (I8tuK0, J1r3Q) → the app's toasts on a phone: Mobile / Toast at the top of
	// the screen, under the status bar, 16 from the sides, newest last. A tap on one (outside its
	// action) lets it go; an error, which waits on a desktop until dismissed, goes after 10 seconds
	// here too, since a phone's toast covers the screen's top. Toasts come from the same store as
	// ToastRegion's, and a screen shows one region or the other.
	const ERROR_MS = 10_000;
	$effect(() => {
		const errors = toasts.items.filter((toast) => toast.kind === 'error');
		const timers = errors.map((toast) => setTimeout(() => toasts.dismiss(toast.id), ERROR_MS));
		return () => timers.forEach(clearTimeout);
	});
	// In the top layer (topLayer), so a sheet or dialog never covers it; forward with each new toast.
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
		class="pointer-events-none fixed inset-x-4 top-safe bottom-auto m-0 flex w-auto flex-col gap-2 overflow-visible border-0 bg-transparent p-0 pt-2"
	>
		{#each toasts.items as toast (toast.id)}
			<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
			<div class="pointer-events-auto" onclick={(event) => !(event.target as HTMLElement).closest('button') && toasts.dismiss(toast.id)}>
				<PhoneToast
					kind={toast.kind}
					title={toast.title}
					detail={toast.detail}
					action={toast.action && {
						label: toast.action.label,
						onclick: () => {
							toast.action?.onclick();
							toasts.dismiss(toast.id);
						},
					}}
				/>
			</div>
		{/each}
	</section>
</div>

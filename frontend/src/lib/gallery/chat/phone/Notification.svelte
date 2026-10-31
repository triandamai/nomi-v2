<script lang="ts">
	import Mascot from '$lib/components/ds/agents/Mascot.svelte';

	// A phone notification from Nomi, as the design draws one: the system's, not the app's UI.
	type Props = { title: string; body: string; time?: string };
	let { title, body, time = 'now' }: Props = $props();
</script>

<div class="notification flex w-89.5 max-w-full gap-3 bg-surface p-3.5 outline -outline-offset-1 outline-border">
	<span class="flex size-9.5 shrink-0 items-center justify-center rounded-md bg-crew-nomi-bg"><Mascot agent="nomi" size={32} /></span>
	<span class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class="flex items-center">
			<span class="min-w-0 flex-1 text-phone-sm font-semibold text-text">Nomi</span>
			<span class="text-ui text-text-3">{time}</span>
		</span>
		<span class="text-body font-semibold text-text">{title}</span>
		<span class="text-phone-sm leading-snug text-text-2">{body}</span>
	</span>
</div>

<style>
	/* The system's notification corners (22), not one of the app's radii. */
	.notification {
		border-radius: 22px;
	}
</style>

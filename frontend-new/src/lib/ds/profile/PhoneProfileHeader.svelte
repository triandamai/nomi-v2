<script lang="ts">
	import { Camera } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile — You & Settings, step 7 (Profile) → Person: the picture round at 72 (or the
	// initials at 22, the nearest size the scale has to the pen's 24), 14 from the name at 22 as the page's title, the mono line under it (the
	// username and when they joined) and Change photo at 14 with its camera, a 44 target.
	type Props = { name: string; line: string; src?: string; photo: string; busy?: boolean; onphoto: () => void };
	let { name, line, src, photo, busy = false, onphoto }: Props = $props();

	const initials = $derived(
		name
			.trim()
			.split(/\s+/)
			.slice(0, 2)
			.map((part) => part[0]?.toUpperCase() ?? '')
			.join(''),
	);
</script>

<div class="flex items-center gap-3.5 px-5 pt-1 pb-4.5">
	<span class="inline-flex size-18 shrink-0 items-center justify-center overflow-hidden rounded-full bg-surface-2 text-text outline -outline-offset-1 outline-border">
		{#if src}<img {src} alt="" class="size-full object-cover" />{:else}<span aria-hidden="true" class="text-phone-display font-medium">{initials}</span>{/if}
	</span>
	<div class="flex min-w-0 flex-1 flex-col gap-1">
		<h1 class="truncate text-phone-display font-semibold text-text">{name}</h1>
		<p class="truncate font-mono text-label text-text-3">{line}</p>
		<button
			type="button"
			disabled={busy}
			onclick={onphoto}
			class="-my-2.5 inline-flex h-11 cursor-pointer items-center gap-1.5 self-start pt-1 text-phone-sm text-text outline-offset-1 focus-visible:outline-2 focus-visible:outline-focus disabled:cursor-default disabled:opacity-60"
		>
			<Icon icon={Camera} size={14} />{photo}
		</button>
	</div>
</div>

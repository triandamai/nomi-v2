<script lang="ts">
	// App / Settings / Profile (U9Gnrg) → Avatar: the person's picture, or their initials, larger
	// than Identity's Avatar: 56 with corners of 10 at the top of the page (initials 13/500), 40 with
	// 6 by the photo controls (initials in mono). Identity's Avatar is the old app's twin, so its
	// props stay as they are.
	type Props = { name: string; src?: string; size?: 56 | 40 };
	let { name, src, size = 56 }: Props = $props();

	const initials = $derived(
		name
			.trim()
			.split(/\s+/)
			.slice(0, 2)
			.map((part) => part[0]?.toUpperCase() ?? '')
			.join(''),
	);
</script>

<span
	class={[
		'inline-flex shrink-0 items-center justify-center overflow-hidden bg-surface-2 text-text-2 outline -outline-offset-1 outline-border-strong',
		size === 56 ? 'size-14 rounded-md text-ui font-medium' : 'size-10 rounded-sm font-mono text-label',
	]}
>
	{#if src}<img {src} alt="" class="size-full object-cover" />{:else}<span aria-hidden="true">{initials}</span>{/if}
</span>

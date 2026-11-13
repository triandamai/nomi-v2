<script lang="ts">
	// Admin → Stat tile: one number worth watching, as a flat bordered card (no shadow): what it
	// counts, the real figure in mono (shortened where it's long, the full one in its title), and a
	// line on what it covers. `live` puts the live dot before the name while something is running,
	// so colour is never the only sign. The pen has no admin screens: built from Status and Meter.
	type Props = {
		label: string;
		value: string;
		/** The exact figure, when `value` is shortened ("1.2K" → "1,234"). */
		exact?: string;
		note?: string;
		live?: boolean;
	};
	let { label, value, exact, note, live = false }: Props = $props();
</script>

<div class="flex min-w-0 flex-col gap-2 rounded-md bg-bg p-4 outline -outline-offset-1 outline-border">
	<span class="flex items-center gap-1.5 text-caption text-text-2">
		{#if live}<span data-dot aria-hidden="true" class="size-1.5 shrink-0 rounded-full bg-accent"></span>{/if}
		<span class="min-w-0 truncate">{label}</span>
	</span>
	<span title={exact} class="font-mono text-display text-text tabular-nums">{value}</span>
	{#if note}<span class="text-caption text-text-3">{note}</span>{/if}
</div>

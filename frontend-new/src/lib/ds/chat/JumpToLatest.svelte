<script lang="ts">
	import { ArrowDown } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Jump to Latest: floats above the composer once you've scrolled up, saying
	// how many replies came in since.
	type Props = { count: number; onclick: () => void };
	let { count, onclick }: Props = $props();
</script>

<button
	type="button"
	class="inline-flex h-7.5 cursor-pointer items-center gap-1.5 rounded-full border border-border-strong bg-surface-2 px-2.75 text-caption text-text shadow-float outline-offset-2 transition-colors hover:bg-border pointer-coarse:min-h-11"
	{onclick}
>
	<Icon icon={ArrowDown} size={14} />{count === 0 ? m.jump_latest() : count === 1 ? m.jump_new_one() : m.jump_new({ count: String(count) })}
</button>

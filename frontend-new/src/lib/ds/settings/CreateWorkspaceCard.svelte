<script lang="ts">
	import { Plus } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / Workspaces (l222Mz) → Create Card: room for another workspace, as tall as a
	// workspace card and outlined in the quiet border, its content centred 10 apart: the plus in a
	// 40px outlined square, the title (13/500) and what it's for.
	type Props = { title: string; body: string; onclick?: () => void };
	let { title, body, onclick }: Props = $props();
</script>

<button
	type="button"
	{onclick}
	class="flex min-h-59 cursor-pointer flex-col items-center justify-center gap-2.5 rounded-md p-4.5 text-center outline -outline-offset-1 outline-border hover:bg-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
>
	<span class="inline-flex size-10 items-center justify-center rounded-md text-text outline -outline-offset-1 outline-border-strong"><Icon icon={Plus} size={18} /></span>
	<span class="text-ui font-medium text-text">{title}</span>
	<span class="text-caption text-text-3">{body}</span>
</button>

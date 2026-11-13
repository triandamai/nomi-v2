<script lang="ts">
	import { Plus } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / Profile (U9Gnrg) → Add Showcase: room for one more featured project, an
	// outlined 44px button with the plus and its label in the middle. It opens the list of live
	// projects (the page's menu), so it passes on the menu's aria and click.
	type Props = {
		label: string;
		disabled?: boolean;
		onclick?: () => void;
		[attribute: `aria-${string}`]: unknown;
	};
	let { label, disabled = false, onclick, ...rest }: Props = $props();
</script>

<button
	type="button"
	{disabled}
	{onclick}
	{...rest}
	class="flex h-11 w-full cursor-pointer items-center justify-center gap-2 rounded-md border border-border text-ui text-text-2 outline-offset-2 hover:enabled:bg-surface hover:enabled:text-text disabled:cursor-default disabled:opacity-50"
>
	<span class="inline-flex text-text-3"><Icon icon={Plus} size={14} /></span>
	<span>{label}</span>
</button>

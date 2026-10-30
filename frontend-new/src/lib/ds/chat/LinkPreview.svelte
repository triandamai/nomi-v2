<script lang="ts">
	import { Globe } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Link Preview: a page the answer points to, as one card that opens it in a new
	// tab. The picture is the page's own, decorative next to its title.
	type Props = { href: string; host: string; title: string; description?: string; image?: string };
	let { href, host, title, description, image }: Props = $props();
</script>

<a
	{href}
	target="_blank"
	rel="noopener noreferrer"
	class="flex w-full items-center overflow-hidden rounded-md border border-border-strong bg-bg pr-3 outline-offset-2 transition-colors hover:bg-surface"
>
	<span class="flex min-w-0 flex-1 flex-col gap-1.5 p-3.5">
		<span class="flex items-center gap-1.5 text-text-3">
			<Icon icon={Globe} size={12} />
			<span class="truncate font-mono text-label">{host}</span>
		</span>
		<span class="text-ui font-medium text-text">{title}</span>
		{#if description}<span class="text-caption text-text-3">{description}</span>{/if}
	</span>
	{#if image}
		<img src={image} alt="" loading="lazy" referrerpolicy="no-referrer" class="h-25 w-35 shrink-0 rounded-sm bg-surface-2 object-cover max-sm:h-18 max-sm:w-24" />
	{/if}
</a>

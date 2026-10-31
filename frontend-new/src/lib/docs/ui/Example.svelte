<script lang="ts">
	import type { Snippet } from 'svelte';
	import CodeBlock from '$lib/ds/chat/CodeBlock.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import { highlight } from '../highlight';

	// A live example: the real component on the page's background, in either theme, with the
	// code that draws it one click away.
	type Props = { title?: string; description?: string; code?: string; align?: 'center' | 'start'; children: Snippet };
	let { title, description, code, align = 'center', children }: Props = $props();

	let showCode = $state(false);
	let theme = $state('page');
	const id = $props.id();
</script>

<figure class="flex flex-col overflow-hidden rounded-md outline -outline-offset-1 outline-border">
	{#if title || description}
		<figcaption class="flex flex-col gap-0.5 border-b border-border px-4 pt-3 pb-2.75">
			{#if title}<span class="text-ui font-medium text-text">{title}</span>{/if}
			{#if description}<span class="text-caption text-text-3">{description}</span>{/if}
		</figcaption>
	{/if}
	<div
		data-theme={theme === 'page' ? undefined : theme}
		class={['flex min-h-36 flex-wrap gap-3 overflow-x-auto bg-bg p-8 max-md:p-4', align === 'center' ? 'items-center justify-center' : 'items-start justify-start']}
	>
		{@render children()}
	</div>
	<div class="flex flex-wrap items-center gap-2 border-t border-border bg-surface px-3 py-2">
		<Segmented
			label="Example theme"
			surface="surface"
			bind:value={theme}
			options={[
				{ value: 'page', label: 'Page' },
				{ value: 'dark', label: 'Dark' },
				{ value: 'light', label: 'Light' },
			]}
		/>
		{#if code}
			<span class="ml-auto">
				<Button variant="ghost" label={showCode ? 'Hide code' : 'Show code'} aria-expanded={showCode} aria-controls={id} onclick={() => (showCode = !showCode)} />
			</span>
		{/if}
	</div>
	{#if code}
		<div {id} hidden={!showCode} class="border-t border-border p-3">
			<CodeBlock {code} language="svelte" tokens={highlight(code)} />
		</div>
	{/if}
</figure>

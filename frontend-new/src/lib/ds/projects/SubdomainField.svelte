<script lang="ts">
	import { LoaderCircle } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Build — Workspace tools (U16wU) → Domain: the name of the app's own address, picked
	// before Nomi's apps domain (shown after it, in mono), with what the hosting said about it as
	// it's typed: checking, free (accent), or why not (danger) with free names to take instead.
	export type SubdomainCheck = { state: 'idle' } | { state: 'checking' } | { state: 'available' } | { state: 'taken'; message: string; suggestions?: string[] };
	type Props = {
		value?: string;
		suffix: string;
		check: SubdomainCheck;
		labels: { field: string; checking: string; available: string; suggestions: string };
		oninput?: (value: string) => void;
		onpick?: (name: string) => void;
	};
	let { value = $bindable(''), suffix, check, labels, oninput, onpick }: Props = $props();

	const id = $props.id();
</script>

<div class="flex flex-col gap-2">
	<div
		class={[
			'flex h-9 items-center rounded-sm border bg-bg px-2.75 focus-within:border-accent focus-within:shadow-focus',
			check.state === 'taken' ? 'border-danger-border' : 'border-border-strong',
		]}
	>
		<input
			{id}
			type="text"
			aria-label={labels.field}
			aria-invalid={check.state === 'taken' ? true : undefined}
			aria-describedby="{id}-check"
			autocomplete="off"
			spellcheck="false"
			bind:value
			oninput={() => oninput?.(value)}
			class="min-w-0 flex-1 bg-transparent font-mono text-ui text-text outline-none"
		/>
		<span class="shrink-0 font-mono text-ui text-text-3">.{suffix}</span>
	</div>
	<p id="{id}-check" aria-live="polite" class="flex min-h-4 items-center gap-1.5 text-caption">
		{#if check.state === 'checking'}
			<span class="inline-flex animate-spin text-text-3 motion-reduce:animate-none"><Icon icon={LoaderCircle} size={12} /></span><span class="text-text-3">{labels.checking}</span>
		{:else if check.state === 'available'}
			<span class="text-accent">{labels.available}</span>
		{:else if check.state === 'taken'}
			<span class="text-danger">{check.message}</span>
		{/if}
	</p>
	{#if check.state === 'taken' && check.suggestions?.length}
		<div class="flex flex-wrap items-center gap-1.5">
			<span class="font-mono text-label text-text-3">{labels.suggestions}</span>
			{#each check.suggestions as name (name)}
				<button type="button" onclick={() => onpick?.(name)} class="h-6 cursor-pointer rounded-xs bg-surface-2 px-2 font-mono text-label text-text-2 outline-offset-1 hover:text-text">{name}</button>
			{/each}
		</div>
	{/if}
</div>

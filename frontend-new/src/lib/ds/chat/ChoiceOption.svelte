<script lang="ts">
	import type { Snippet } from 'svelte';
	import Tag from '../status/Tag.svelte';
	import CheckMark from '../inputs/CheckMark.svelte';

	// One answer in a Decision, Clarify or Plan Approval card: its number (the key that picks it),
	// title, tradeoff and the recommendation, with a real radio or checkbox. The chosen radio takes
	// the accent; checked boxes don't (many can be). Anything extra (a text field) goes underneath.
	type Props = {
		type: 'radio' | 'checkbox';
		name: string;
		checked: boolean;
		title: string;
		detail?: string;
		/** Marks the crew's recommendation, e.g. "Suggested". */
		tag?: string;
		/** Shown as the number that picks it from the keyboard. */
		key?: number;
		/** Which side the radio or checkbox sits on. */
		control?: 'start' | 'end';
		onchange: (checked: boolean) => void;
		children?: Snippet;
	};
	let { type, name, checked, title, detail, tag, key, control = 'end', onchange, children }: Props = $props();

	const chosen = $derived(type === 'radio' && checked);
</script>

{#snippet mark()}
	{#if type === 'radio'}
		<input
			type="radio"
			{name}
			{checked}
			onchange={(event) => onchange(event.currentTarget.checked)}
			class="size-4 shrink-0 cursor-pointer appearance-none rounded-full border border-border-strong outline-offset-2 checked:border-5 checked:border-accent"
		/>
	{:else}
		<CheckMark {name} {checked} onchange={(next) => onchange(next)} />
	{/if}
{/snippet}

<div data-choice class={['flex flex-col gap-2 rounded-sm border transition-colors', chosen ? 'border-accent bg-accent-bg' : 'border-border-strong hover:bg-surface']}>
	<label class="flex cursor-pointer items-center gap-3 px-3 py-2.5 pointer-coarse:min-h-11">
		{#if control === 'start'}{@render mark()}{/if}
		{#if key !== undefined}
			<span aria-hidden="true" class="inline-flex size-5 shrink-0 items-center justify-center rounded-xs bg-surface-2 font-mono text-label text-text-2">{key}</span>
		{/if}
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<span class="flex flex-wrap items-center gap-2">
				<span class="text-ui font-medium text-text">{title}</span>
				{#if tag}<Tag kind="go" label={tag} />{/if}
			</span>
			{#if detail}<span class="text-caption text-text-3">{detail}</span>{/if}
		</span>
		{#if control === 'end'}{@render mark()}{/if}
	</label>
	{#if children}<div class="px-3 pb-2.5">{@render children()}</div>{/if}
</div>

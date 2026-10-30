<script lang="ts">
	import { Check } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import CheckMark from '../inputs/CheckMark.svelte';
	import type { CrewId } from '../agents/crew';

	// Chat responses → Selection: many of many, each with what it is and what it costs, and a
	// running count (and total, from the caller) of what you picked. Unpicked rows recede.
	type Item = { name: string; detail?: string; amount?: string };
	type Props = {
		agent: CrewId;
		question: string;
		items: Item[];
		/** Which items start picked. */
		selected?: boolean[];
		/** The running total of the picked items, formatted, e.g. "Rp 285.990 / month". */
		total?: string;
		/** What confirming does, e.g. "Track 3". */
		confirmLabel: string;
		/** Every change to what's picked, so the caller can update the total. */
		onchange?: (selected: boolean[]) => void;
		onconfirm: (indices: number[]) => void;
		onskip?: () => void;
	};
	let { agent, question, items, selected, total, confirmLabel, onchange, onconfirm, onskip }: Props = $props();

	const initial = () => items.map((_, index) => selected?.[index] ?? false);
	let picked = $state(initial());

	const count = $derived(picked.filter(Boolean).length);
	const all = $derived(items.length > 0 && count === items.length);

	function set(next: boolean[]) {
		picked = next;
		onchange?.(next);
	}
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md border border-border-strong bg-bg">
	<header class="flex items-center gap-2.5 border-b border-border bg-surface py-2.5 pr-2 pl-3.5">
		<Mascot {agent} size={20} />
		<p class="min-w-0 flex-1 text-ui font-medium text-text">{question}</p>
		<button type="button" class="cursor-pointer rounded-xs px-1.5 text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={() => set(items.map(() => !all))}>
			{all ? m.selection_none() : m.selection_all()}
		</button>
	</header>
	<ul>
		{#each items as item, index (index)}
			<li class={index > 0 ? 'border-t border-border' : undefined}>
				<label class="flex cursor-pointer items-center gap-3 px-3.5 py-2.5 hover:bg-surface pointer-coarse:min-h-11">
					<CheckMark checked={picked[index]} onchange={(checked) => set(picked.map((on, at) => (at === index ? checked : on)))} />
					<span class="flex min-w-0 flex-1 flex-col gap-0.5">
						<span class={['text-ui', picked[index] ? 'text-text' : 'text-text-2']}>{item.name}</span>
						{#if item.detail}<span class="font-mono text-label text-text-3">{item.detail}</span>{/if}
					</span>
					{#if item.amount}<span class={['shrink-0 font-mono text-caption', picked[index] ? 'text-text' : 'text-text-3']}>{item.amount}</span>{/if}
				</label>
			</li>
		{/each}
	</ul>
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 py-2.5">
		<span class="min-w-0 flex-1 basis-40 font-mono text-label text-text-3">{m.selection_count({ count: String(count) })}{total ? ` · ${total}` : ''}</span>
		<span class="ml-auto flex items-center gap-2">
			{#if onskip}<Button label={m.action_skip()} variant="ghost" onclick={onskip} />{/if}
			<Button label={confirmLabel} iconEnd={Check} disabled={count === 0} onclick={() => onconfirm(picked.flatMap((on, index) => (on ? [index] : [])))} />
		</span>
	</footer>
</section>

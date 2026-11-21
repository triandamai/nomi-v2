<script lang="ts" module>
	import type { CrewId } from '../agents/crew';

	export type IssueCard = {
		number: number;
		title: string;
		/** Kind · severity · points, in mono. */
		tags: string;
		/** How the crew is getting on, or the deadline; `crew` puts their mascot by it. */
		status?: { text: string; tone?: 'late' | 'failed'; crew?: CrewId };
		/** The issue's own chat, once the crew picked it up. */
		chatHref?: string;
		/** The crew couldn't finish it: offer to hand it back. */
		retry?: boolean;
	};
	export type IssueColumn = { id: string; name: string; hint?: string; cards: IssueCard[] };
</script>

<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import Button from '../actions/Button.svelte';
	import Link from '../actions/Link.svelte';
	import Select from '../inputs/Select.svelte';

	// Beyond the pen: the workspace's Issues sheet, the old app's issue board rebuilt from this
	// system's parts. Columns (Backlog and Ready to dev first, then the project's own, Done and
	// Closed) of cards on the surface, 272 wide and 12 apart, scrolling sideways. A card moves by
	// dragging it onto another column (it lands where it's dropped) or with its "Move to" select
	// (keyboard, touch). Moving one into Ready to dev hands it to the crew; the card then says who
	// has it and how it's going (danger when they couldn't finish, warning when it's overdue).
	type Props = {
		columns: IssueColumn[];
		labels: { board: string; empty: string; moveTo: (number: number) => string; retry: string; openChat: string };
		onmove: (number: number, columnId: string, index: number) => void;
		onretry?: (number: number) => void;
		/** Where a card dropped at `y` lands among the column's other cards. */
		dropAt?: (cards: { top: number; height: number }[], y: number) => number;
	};
	let { columns, labels, onmove, onretry, dropAt = (cards) => cards.length }: Props = $props();

	let dragging = $state<number | null>(null);
	let over = $state<string | null>(null);
	const options = $derived(columns.map((column) => ({ value: column.id, label: column.name })));

	function drop(event: DragEvent, column: IssueColumn) {
		event.preventDefault();
		const number = dragging;
		dragging = null;
		over = null;
		if (number === null) return;
		const cards = [...(event.currentTarget as HTMLElement).querySelectorAll<HTMLElement>('[data-card]')]
			.filter((card) => card.dataset.card !== String(number))
			.map((card) => {
				const box = card.getBoundingClientRect();
				return { top: box.top, height: box.height };
			});
		onmove(number, column.id, dropAt(cards, event.clientY));
	}
</script>

<div role="region" aria-label={labels.board} class="flex w-full items-start gap-3 overflow-x-auto pb-2">
	{#each columns as column (column.id)}
		<section
			aria-label={column.name}
			data-column={column.id}
			class={['flex w-68 shrink-0 flex-col gap-2 rounded-md p-2.5 outline -outline-offset-1', over === column.id ? 'bg-surface-2 outline-border-strong' : 'bg-surface outline-border']}
			ondragover={(event) => {
				if (dragging === null) return;
				event.preventDefault();
				over = column.id;
			}}
			ondragleave={(event) => {
				if (!(event.currentTarget as HTMLElement).contains(event.relatedTarget as Node)) over = over === column.id ? null : over;
			}}
			ondrop={(event) => drop(event, column)}
		>
			<header class="flex items-center gap-2 px-0.5">
				<h3 class="min-w-0 flex-1 truncate text-ui font-medium text-text">{column.name}</h3>
				<span class="font-mono text-label text-text-3">{column.cards.length}</span>
			</header>
			{#if column.hint}<p class="px-0.5 text-caption text-text-3">{column.hint}</p>{/if}
			{#if column.cards.length}
				<ul class="flex flex-col gap-2">
					{#each column.cards as card (card.number)}
						<li
							data-card={card.number}
							draggable="true"
							class={['flex cursor-grab flex-col gap-2 rounded-sm bg-bg p-3 outline -outline-offset-1 outline-border active:cursor-grabbing', dragging === card.number && 'opacity-50']}
							ondragstart={(event) => {
								dragging = card.number;
								event.dataTransfer?.setData('text/plain', `#${card.number}`);
							}}
							ondragend={() => {
								dragging = null;
								over = null;
							}}
						>
							<p class="text-ui text-text"><span class="font-mono text-text-3">#{card.number}</span> {card.title}</p>
							<p class="font-mono text-label text-text-3">{card.tags}</p>
							{#if card.status}
								<p class={['flex items-center gap-1.5 text-caption', card.status.tone === 'failed' ? 'text-danger' : card.status.tone === 'late' ? 'text-warning' : 'text-text-2']}>
									{#if card.status.crew}<Mascot agent={card.status.crew} size={16} />{/if}{card.status.text}
								</p>
							{/if}
							<div class="flex flex-wrap items-center gap-2">
								<div class="min-w-0 flex-1">
									<Select label={labels.moveTo(card.number)} {options} value={column.id} onchange={(target) => target !== column.id && onmove(card.number, target, columns.find((one) => one.id === target)?.cards.length ?? 0)} />
								</div>
								{#if card.retry && onretry}<Button variant="secondary" label={labels.retry} onclick={() => onretry(card.number)} />{/if}
							</div>
							{#if card.chatHref}<Link href={card.chatHref} label={labels.openChat} />{/if}
						</li>
					{/each}
				</ul>
			{:else}
				<p class="rounded-sm px-3 py-4 text-center text-caption text-text-3 outline -outline-offset-1 outline-border outline-dashed">{labels.empty}</p>
			{/if}
		</section>
	{/each}
</div>

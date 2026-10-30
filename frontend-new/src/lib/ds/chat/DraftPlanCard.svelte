<script lang="ts">
	import { ChevronDown, FilePenLine, GripVertical, Play, Plus, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Tag from '../status/Tag.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';
	import { stepNumber } from './stepState';

	// Chat responses → Draft plan: editable, nothing runs yet. Reorder (drag, or the handle with the
	// arrow keys), reassign or remove steps, add one; then run it, ask for changes or discard it.
	type Step = { label: string; owner: { agent: CrewId; name: string } };
	type Props = {
		intro: string;
		steps: Step[];
		estimate: string;
		onmove?: (from: number, to: number) => void;
		onremove?: (index: number) => void;
		onassign?: (index: number) => void;
		onadd?: () => void;
		onrun?: () => void;
		onchanges?: () => void;
		ondiscard?: () => void;
	};
	let { intro, steps, estimate, onmove, onremove, onassign, onadd, onrun, onchanges, ondiscard }: Props = $props();

	let dragging: number | null = $state(null);

	function onhandlekey(event: KeyboardEvent, index: number) {
		const to = event.key === 'ArrowUp' ? index - 1 : event.key === 'ArrowDown' ? index + 1 : null;
		if (to === null || to < 0 || to >= steps.length) return;
		event.preventDefault();
		onmove?.(index, to);
	}
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md border border-border-strong bg-bg">
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 py-2.5">
		<span class="inline-flex text-text-2"><Icon icon={FilePenLine} size={14} /></span>
		<span class="flex-1 text-ui font-medium text-text">{m.draft_title()}</span>
		<Tag kind="neutral" label={m.draft_tag()} />
	</header>
	<p class="px-3.5 pt-3 pb-1 text-ui leading-normal text-text-2">{intro}</p>
	<ol class="flex flex-col px-3.5 py-1">
		{#each steps as step, index (index)}
			<li
				class={['flex items-center gap-2.5 py-2', dragging === index && 'opacity-40']}
				draggable="true"
				ondragstart={() => (dragging = index)}
				ondragend={() => (dragging = null)}
				ondragover={(event) => event.preventDefault()}
				ondrop={() => {
					if (dragging !== null && dragging !== index) onmove?.(dragging, index);
					dragging = null;
				}}
			>
				<button
					type="button"
					aria-label={m.draft_move({ index: String(index + 1) })}
					onkeydown={(event) => onhandlekey(event, index)}
					class="inline-flex cursor-grab rounded-xs text-text-3 outline-offset-2 hover:text-text-2"><Icon icon={GripVertical} size={14} /></button
				>
				<span class="font-mono text-label text-text-3">{stepNumber(index)}</span>
				<span class="min-w-0 flex-1 text-ui text-text">{step.label}</span>
				<button
					type="button"
					onclick={() => onassign?.(index)}
					aria-haspopup="menu"
					class="inline-flex h-6.5 cursor-pointer items-center gap-1.5 rounded-sm border border-border-strong pr-2 pl-1 outline-offset-1 hover:bg-surface-2"
				>
					<Mascot agent={step.owner.agent} size={20} />
					<span class="text-caption text-text-2">{step.owner.name}</span>
					<span class="inline-flex text-text-3"><Icon icon={ChevronDown} size={12} /></span>
				</button>
				<button
					type="button"
					aria-label={m.draft_remove({ index: String(index + 1) })}
					onclick={() => onremove?.(index)}
					class="inline-flex cursor-pointer rounded-xs p-0.5 text-text-3 outline-offset-2 hover:text-text"><Icon icon={X} size={14} /></button
				>
			</li>
		{/each}
		<li>
			<button type="button" onclick={onadd} class="flex w-full cursor-pointer items-center gap-2.5 rounded-xs py-2 text-ui text-text-3 outline-offset-2 hover:text-text-2">
				<Icon icon={Plus} size={14} />
				{m.draft_add_step()}
			</button>
		</li>
	</ol>
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 py-2.5">
		<span class="min-w-0 flex-1 font-mono text-label text-text-3">{estimate}</span>
		<Button label={m.draft_discard()} variant="ghost" iconStart={X} onclick={ondiscard} />
		<Button label={m.draft_changes()} variant="secondary" onclick={onchanges} />
		<Button label={m.draft_run()} iconEnd={Play} onclick={onrun} />
	</footer>
</section>

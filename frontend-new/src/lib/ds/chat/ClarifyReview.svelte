<script lang="ts">
	import { Info, ListChecks, Pencil, Sparkles } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import IconButton from '../actions/IconButton.svelte';
	import Tag from '../status/Tag.svelte';

	// Chat responses → Clarify / Review: your answers before anything is planned. An answer the
	// crew filled in for you is marked Default, with the reason written underneath. Nothing runs
	// yet: the plan comes next.
	type Answer = { topic: string; value: string; isDefault?: boolean };
	type Props = {
		answers: Answer[];
		/** Why a default was picked, in a sentence. */
		assumption?: string;
		onedit: (index: number) => void;
		onback: () => void;
		onconfirm: () => void;
	};
	let { answers, assumption, onedit, onback, onconfirm }: Props = $props();
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md border border-border-strong bg-bg">
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 py-2.5">
		<span class="inline-flex text-text-2"><Icon icon={ListChecks} size={14} /></span>
		<p class="min-w-0 flex-1 text-ui font-medium text-text">{m.clarify_review_title()}</p>
		<span class="font-mono text-label text-text-3">{m.clarify_progress({ n: String(answers.length), total: String(answers.length) })}</span>
	</header>
	<dl class="px-3.5 py-1">
		{#each answers as answer, index (index)}
			<div class={['flex flex-wrap items-center gap-x-3 py-1', index > 0 && 'border-t border-border']}>
				<dt class="w-27.5 shrink-0 text-caption text-text-3">{answer.topic}</dt>
				<dd class="flex min-w-0 flex-1 basis-48 items-center gap-3">
					<span class="min-w-0 flex-1 text-ui text-text">{answer.value}</span>
					{#if answer.isDefault}<Tag kind="neutral" label={m.clarify_default()} />{/if}
					<IconButton icon={Pencil} label={m.clarify_edit({ topic: answer.topic })} variant="ghost" iconSize={14} onclick={() => onedit(index)} />
				</dd>
			</div>
		{/each}
	</dl>
	{#if assumption}
		<p class="flex gap-2 px-3.5 pb-3 text-caption text-text-3">
			<span class="inline-flex h-4 shrink-0 items-center"><Icon icon={Info} size={12} /></span>{assumption}
		</p>
	{/if}
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 py-2.5">
		<span class="min-w-0 flex-1 basis-48 text-caption text-text-3">{m.clarify_review_note()}</span>
		<span class="ml-auto flex items-center gap-2">
			<Button label={m.action_back()} variant="secondary" onclick={onback} />
			<Button label={m.clarify_make_plan()} iconEnd={Sparkles} onclick={onconfirm} />
		</span>
	</footer>
</section>

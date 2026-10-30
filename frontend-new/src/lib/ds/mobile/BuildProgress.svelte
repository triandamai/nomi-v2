<script lang="ts">
	import { Bell, LoaderCircle, Square } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';

	// Mobile / Build Progress: one card while Koda builds. The real step and file, which step of
	// how many (the bars are the plan's steps, not a percentage), and that the app can be closed:
	// a notification says when it's live.
	type Props = { project: string; elapsed: string; step: string; current: number; total: number; label: string; eta: string; onstop: () => void };
	let { project, elapsed, step, current, total, label, eta, onstop }: Props = $props();
</script>

<section class="flex w-full flex-col gap-3 rounded-3xl bg-surface p-4 outline -outline-offset-1 outline-border">
	<p class="flex items-center gap-2">
		<Mascot agent="koda" size={20} />
		<span class="min-w-0 flex-1 text-phone-sm font-medium text-text">{m.bprogress_title({ project })}</span>
		<span class="font-mono text-label text-text-2">{elapsed}</span>
	</p>
	<div aria-hidden="true" class="flex gap-1">
		{#each { length: total } as _, index (index)}
			<span data-step class={['h-1 flex-1 rounded-full', index < current - 1 ? 'bg-accent' : index === current - 1 ? 'bg-text' : 'bg-border-strong']}></span>
		{/each}
	</div>
	<p class="flex items-center gap-2">
		<span class="inline-flex shrink-0 animate-spin text-text-2 motion-reduce:animate-none"><Icon icon={LoaderCircle} size={14} /></span>
		<span class="min-w-0 flex-1 font-mono text-caption text-text">{step}</span>
	</p>
	<p aria-live="polite" class="text-ui text-text-2">{m.bprogress_line({ current: String(current), total: String(total), label, eta })}</p>
	<p class="flex items-center gap-2 rounded-lg bg-surface-2 px-3 py-2.5 text-ui text-text-2">
		<span class="inline-flex shrink-0"><Icon icon={Bell} size={14} /></span><span class="min-w-0 flex-1">{m.bprogress_note()}</span>
	</p>
	<button type="button" class="flex h-11 cursor-pointer items-center justify-center gap-1.5 rounded-md border border-border-strong text-phone-sm text-text outline-offset-2" onclick={onstop}>
		<Icon icon={Square} size={14} />{m.build_stop()}
	</button>
</section>

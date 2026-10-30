<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Mascot from '../agents/Mascot.svelte';
	import Status from '../status/Status.svelte';
	import LiveStep from '../conversation/LiveStep.svelte';

	// Build → Running: the estimate collapsed while Koda builds. The real step it's on and its time
	// (never a progress bar), a stop, and which of the plan's steps are done (mint), current, or
	// still to come, with the crew's rough time left.
	type Props = { started: string; step: string; elapsed?: string; current: number; total: number; label: string; eta?: string; onstop?: () => void };
	let { started, step, elapsed, current, total, label, eta, onstop }: Props = $props();
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md border border-border bg-surface">
	<header class="flex items-center gap-2.5 border-b border-border px-4 py-3.5">
		<Mascot agent="koda" size={20} />
		<span class="text-ui font-medium text-text">Koda</span>
		<span class="min-w-0 flex-1 font-mono text-label text-text-3">{m.build_running({ time: started })}</span>
		<Status kind="working" agent="koda" label={m.build_working({ name: 'Koda' })} />
	</header>
	<LiveStep {step} {elapsed} stopLabel={m.build_stop()} {onstop} />
	<div class="flex flex-col gap-2 px-4 pt-1 pb-3.5">
		<div aria-hidden="true" class="flex gap-1">
			{#each { length: total } as _, index (index)}
				<span data-step class={['h-1 flex-1 rounded-xs', index < current - 1 ? 'bg-accent' : index === current - 1 ? 'bg-text-2' : 'bg-border-strong']}></span>
			{/each}
		</div>
		<p class="flex items-center gap-2">
			<span class="min-w-0 flex-1 text-caption text-text-2">{m.build_step({ current: String(current), total: String(total), label })}</span>
			{#if eta}<span class="font-mono text-label text-text-3">{eta}</span>{/if}
		</p>
	</div>
</section>

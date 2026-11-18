<script lang="ts">
	import { Hourglass, MonitorX, Power } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/ds/actions/Button.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import { latestLine, type SandboxPhase } from './sandbox';

	// The preview pane while the sandbox isn't serving the app yet (Code, terminal & phone preview
	// uR6dq): what it's waiting for and, while it wakes or starts the app, the last thing it printed.
	// The app itself is framed only once the sandbox answers with its address.
	let { phase, log, error, onretry }: { phase: SandboxPhase; log: string; error: string | null; onretry: () => void } = $props();

	const progress = $derived(phase === 'waking' || phase === 'starting' ? latestLine(log) : null);
</script>

<div class="flex flex-col items-start gap-3">
	{#if phase === 'failed'}
		<EmptyState icon={MonitorX} title={m.sandbox_failed_title()} body={error || m.sandbox_failed_body()} />
	{:else if phase === 'stopped'}
		<EmptyState icon={Power} title={m.sandbox_stopped_title()} body={m.sandbox_stopped_body()} />
	{:else if phase === 'building'}
		<EmptyState icon={Hourglass} title={m.sandbox_building_title()} body={m.sandbox_building_body()} />
	{:else if phase === 'starting'}
		<EmptyState icon={Hourglass} title={m.sandbox_starting_title()} body={m.sandbox_starting_body()} />
	{:else}
		<EmptyState icon={Hourglass} title={phase === 'loading' ? m.sandbox_loading_title() : m.sandbox_waking_title()} body={m.sandbox_waking_body()} />
	{/if}
	{#if progress}
		<p data-progress class="max-w-full truncate font-mono text-label text-text-2" aria-live="polite">{progress}</p>
	{/if}
	{#if phase === 'failed' || phase === 'stopped'}<Button variant="secondary" label={m.sandbox_start_again()} onclick={onretry} />{/if}
</div>

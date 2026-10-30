<script lang="ts">
	import Status from '../status/Status.svelte';
	import type { CrewId } from '../agents/crew';

	// A project in a list: its app tile (tinted in a crew colour, initial in the app's own serif),
	// what it is, whether it's live and where, and when it last changed.
	type Props = {
		href: string;
		name: string;
		description?: string;
		tone: CrewId;
		status: { kind: 'live' | 'ready' | 'idle' | 'failed' | 'attention'; label: string };
		url?: string;
		updated: string;
	};
	let { href, name, description, tone, status, url, updated }: Props = $props();
</script>

<a {href} class="flex w-full items-center gap-4 rounded-sm border border-border px-3.75 py-2.75 outline-offset-1 hover:border-border-strong">
	<span class="flex min-w-0 flex-1 items-center gap-3">
		<span class="inline-flex size-8 shrink-0 items-center justify-center rounded-sm" style="background: var(--crew-{tone}-bg)">
			<span class="font-app text-body font-semibold" style="color: var(--crew-{tone}-ink)">{name.charAt(0).toUpperCase()}</span>
		</span>
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<span class="truncate text-ui font-medium text-text">{name}</span>
			{#if description}<span class="truncate text-caption text-text-3">{description}</span>{/if}
		</span>
	</span>
	<span class="flex w-60 shrink-0 flex-col gap-1 max-md:hidden">
		<Status kind={status.kind} label={status.label} />
		{#if url}<span class="font-mono text-label break-all text-text-3">{url}</span>{/if}
	</span>
	<span class="font-mono text-label text-text-3">{updated}</span>
</a>

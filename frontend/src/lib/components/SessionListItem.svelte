<script lang="ts">
	import type { SessionSummary } from '$lib/types';

	let { session }: { session: SessionSummary } = $props();

	function timeAgo(iso: string): string {
		const diffMs = Date.now() - new Date(iso).getTime();
		const minutes = Math.floor(diffMs / 60000);
		if (minutes < 1) return 'just now';
		if (minutes < 60) return `${minutes}m`;
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return `${hours}h`;
		return `${Math.floor(hours / 24)}d`;
	}
</script>

<a
	href={`/chat/${session.id}`}
	class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm hover:bg-neutral-100"
>
	<span class="relative flex h-2 w-2 shrink-0">
		{#if session.agent_active}
			<span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-amber-400 opacity-75"></span>
			<span class="relative inline-flex h-2 w-2 rounded-full bg-amber-500"></span>
		{/if}
	</span>
	<span class="flex-1 truncate text-neutral-700">
		{session.last_message?.content ?? 'New chat'}
	</span>
	<span class="shrink-0 text-xs text-neutral-400">{timeAgo(session.updated_at)}</span>
</a>

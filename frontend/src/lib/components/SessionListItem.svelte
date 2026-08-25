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

<a href={`/chat/${session.id}`} class="m3-session-item">
	<span class="relative flex h-2 w-2 shrink-0">
		{#if session.agent_active}
			<span
				class="absolute inline-flex h-full w-full animate-ping rounded-full opacity-75"
				style="background: var(--md-sys-color-tertiary)"
			></span>
			<span class="relative inline-flex h-2 w-2 rounded-full" style="background: var(--md-sys-color-tertiary)"></span>
		{/if}
	</span>
	<span class="md-body-medium flex-1 truncate" style="color: var(--md-sys-color-on-surface)">
		{session.last_message?.content ?? 'New chat'}
	</span>
	<span class="md-body-small shrink-0" style="color: var(--md-sys-color-outline)">{timeAgo(session.updated_at)}</span>
</a>

<style>
	.m3-session-item {
		display: flex;
		align-items: center;
		gap: 8px;
		border-radius: var(--md-sys-shape-corner-small);
		padding: 8px 12px;
		text-decoration: none;
	}
	.m3-session-item:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
</style>

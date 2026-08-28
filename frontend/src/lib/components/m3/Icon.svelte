<script lang="ts">
	export type IconName =
		| 'dashboard'
		| 'settings'
		| 'logout'
		| 'plus'
		| 'chat-bubble'
		| 'chevron-left'
		| 'chevron-right'
		| 'agents'
		| 'more';

	let { name, size = 20 }: { name: IconName; size?: number } = $props();
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="2"
	stroke-linecap="round"
	stroke-linejoin="round"
	aria-hidden="true"
	focusable="false"
>
	{#if name === 'dashboard'}
		<rect x="4" y="4" width="7" height="7" rx="1.5" />
		<rect x="13" y="4" width="7" height="7" rx="1.5" />
		<rect x="4" y="13" width="7" height="7" rx="1.5" />
		<rect x="13" y="13" width="7" height="7" rx="1.5" />
	{:else if name === 'settings'}
		<circle cx="12" cy="12" r="3" />
		<circle cx="12" cy="12" r="8" stroke-dasharray="2 3.2" />
	{:else if name === 'logout'}
		<path d="M15 4h3a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-3" />
		<line x1="10" y1="12" x2="3" y2="12" />
		<polyline points="6.5 7.5 3 12 6.5 16.5" />
	{:else if name === 'plus'}
		<line x1="12" y1="5" x2="12" y2="19" />
		<line x1="5" y1="12" x2="19" y2="12" />
	{:else if name === 'chat-bubble'}
		<path d="M4 5h16v11H9l-4 4V5z" />
	{:else if name === 'chevron-left'}
		<polyline points="15 18 9 12 15 6" />
	{:else if name === 'chevron-right'}
		<polyline points="9 18 15 12 9 6" />
	{:else if name === 'agents'}
		<rect x="5" y="8" width="14" height="10" rx="2" />
		<line x1="12" y1="8" x2="12" y2="4" />
		<circle cx="12" cy="3" r="1" />
		<circle cx="9" cy="13" r="1" />
		<circle cx="15" cy="13" r="1" />
	{:else if name === 'more'}
		<circle cx="12" cy="5" r="1.5" fill="currentColor" stroke="none" />
		<circle cx="12" cy="12" r="1.5" fill="currentColor" stroke="none" />
		<circle cx="12" cy="19" r="1.5" fill="currentColor" stroke="none" />
	{/if}
</svg>

<script lang="ts">
	import type { AttachmentKind } from '$lib/attachments';

	// One icon per kind of attached file, in the chips and cards of the composer and the chat.
	let { kind, size = 16 }: { kind: AttachmentKind; size?: number } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
	{#if kind === 'voice'}
		<rect x="9" y="3" width="6" height="11" rx="3" /><path d="M5 11a7 7 0 0 0 14 0M12 18v3" />
	{:else if kind === 'audio'}
		<path d="M9 18V5l12-2v13" /><circle cx="6" cy="18" r="3" /><circle cx="18" cy="16" r="3" />
	{:else if kind === 'image'}
		<rect x="3" y="3" width="18" height="18" rx="3" /><circle cx="9" cy="9" r="2" /><path d="m21 15-4.5-4.5L6 21" />
	{:else if kind === 'video'}
		<rect x="2" y="5" width="14" height="14" rx="3" /><path d="m22 8-6 4 6 4z" />
	{:else if kind === 'spreadsheet'}
		<rect x="3" y="3" width="18" height="18" rx="2" /><path d="M3 9h18M3 15h18M9 3v18" />
	{:else if kind === 'presentation'}
		<rect x="2" y="3" width="20" height="13" rx="2" /><path d="M12 16v5M8 21h8" />
	{:else if kind === 'pdf'}
		<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z" /><path d="M14 3v5h5" /><path d="M9 13h6M9 17h4" />
	{:else}
		<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z" /><path d="M14 3v5h5" />
	{/if}
</svg>

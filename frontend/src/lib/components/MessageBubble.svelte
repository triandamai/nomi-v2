<script lang="ts">
	import type { MessageItem } from '$lib/types';

	let { message }: { message: MessageItem } = $props();
</script>

<div class="flex {message.sender === 'user' ? 'justify-end' : 'justify-start'}">
	<div
		class="md-body-large max-w-md px-4 py-2"
		style={message.sender === 'user'
			? `background: var(--md-sys-color-primary-container); color: var(--md-sys-color-on-primary-container); border-radius: var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-extra-small) var(--md-sys-shape-corner-large)`
			: `background: var(--md-sys-color-surface-container-high); color: var(--md-sys-color-on-surface); border-radius: var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-extra-small)`}
	>
		{message.content}
	</div>
</div>

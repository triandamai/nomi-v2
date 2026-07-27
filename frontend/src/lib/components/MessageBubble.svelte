<script lang="ts">
	import type { MessageItem } from '$lib/types';

	let { message }: { message: MessageItem } = $props();
</script>

<div class="flex {message.sender === 'user' ? 'justify-end' : 'justify-start'}">
	<div
		class="max-w-md rounded-2xl px-4 py-2 {message.sender === 'user'
			? 'bg-neutral-900 text-white'
			: 'bg-neutral-100 text-neutral-900'}"
	>
		{message.content}
	</div>
</div>

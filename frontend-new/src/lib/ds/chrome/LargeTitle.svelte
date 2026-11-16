<script lang="ts">
	import type { Snippet } from 'svelte';

	// Flow / Mobile Chat — Start & answer → Chats → Large Title: a phone tab's name, 32 at 600 with
	// −0.6 tracking, padded [4,20,10,20]. `trailing` sits at its end (an action, where a tab has one).
	type Props = { title: string; trailing?: Snippet };
	let { title, trailing }: Props = $props();
</script>

<div class="flex items-center gap-3 px-5 pt-1 pb-2.5">
	<h1 class="min-w-0 flex-1 text-phone-large font-semibold tracking-phone-large text-text">{title}</h1>
	{@render trailing?.()}
</div>

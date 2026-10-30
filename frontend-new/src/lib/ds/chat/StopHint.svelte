<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	// Chat responses → Stop Hint: while a reply streams, stop it here or with Escape.
	let { onstop }: { onstop: () => void } = $props();
</script>

<button
	type="button"
	aria-keyshortcuts="Escape"
	class="inline-flex cursor-pointer items-center gap-2 rounded-sm bg-text px-2 py-1 text-caption text-bg outline-offset-2 pointer-coarse:min-h-11"
	onclick={onstop}
>
	{m.reply_stop()}<span aria-hidden="true" class="font-mono text-label text-text-3">esc</span>
</button>

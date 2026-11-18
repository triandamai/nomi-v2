<!-- Reply: the quick tiles, Good on. -->
<script lang="ts">
	import { Copy, Share, ThumbsDown, ThumbsUp } from '@lucide/svelte';
	import SheetTiles from '$lib/ds/overlays/SheetTiles.svelte';
</script>

<div class="w-97.5 max-w-full rounded-t-sheet bg-surface pt-4">
	<SheetTiles
		tiles={[
			{ label: 'Good', icon: ThumbsUp, pressed: true, onclick: () => {} },
			{ label: 'Bad', icon: ThumbsDown, onclick: () => {} },
			{ label: 'Copy', icon: Copy, onclick: () => {} },
			{ label: 'Share', icon: Share, onclick: () => {} },
		]}
	/>
</div>

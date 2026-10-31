<!-- Under a reply: byline, then copy, like, dislike, share and more. -->
<script lang="ts">
	import Byline from '$lib/ds/mobile/Byline.svelte';
	import ActionBar from '$lib/ds/mobile/ActionBar.svelte';
</script>

<div class="flex w-89.5 max-w-full flex-col gap-5">
	<Byline agent="dana" name="Dana" meta="14:02" />
	<ActionBar text="Food is the jump." onlike={() => {}} ondislike={() => {}} onshare={() => {}} onmore={() => {}} />
</div>

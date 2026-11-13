<!-- Sections: with a count, and a day. -->
<script lang="ts">
	import InboxHeading from '$lib/ds/inbox/InboxHeading.svelte';
</script>

<div class="flex flex-col gap-2">
	<InboxHeading label="Needs you" count={3} />
	<InboxHeading label="Today · Wed 21 Oct" />
</div>

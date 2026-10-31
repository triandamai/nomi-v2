<!-- A chat and a project: the icon tells them apart; the time is short and relative. -->
<script lang="ts">
	import RecentRow from '$lib/ds/navigation/RecentRow.svelte';
</script>

<div class="flex w-56 flex-col gap-0.5">
	<RecentRow href="#chat" kind="chat" title="Budget belanja Oktober" when="2h" />
	<RecentRow href="#project" kind="project" title="salah-daily-tracker" when="2d" />
</div>

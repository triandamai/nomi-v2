<!-- The Core group: always on. -->
<script lang="ts">
	import CrewGroup from '$lib/ds/settings/CrewGroup.svelte';
	import CrewMemberRow from '$lib/ds/settings/CrewMemberRow.svelte';
</script>

<div class="w-84 max-w-full">
	<CrewGroup label="Core" note="always on">
		<li><CrewMemberRow agent="nomi" name="Nomi" role="Talks with you, routes work" state="on" href="?member=chitchat" /></li>
		<li><CrewMemberRow agent="supervisor" name="Supervisor" role="Watches the crew, stops it on ask" state="on" href="?member=supervisor" /></li>
	</CrewGroup>
</div>

<!-- Renaming: press F2 on a card, or Rename in its menu. Enter saves, Escape cancels. -->
<script lang="ts">
	import ProjectCard from '$lib/ds/projects/ProjectCard.svelte';

	let name = $state('salah-daily-tracker');
	let renaming = $state(false);
</script>

<ProjectCard
	{name}
	description="Daily salah tracker with countdown and streaks"
	tone="nomi"
	status="live"
	url="salah-daily-tracker-0ab8f8.trian.space"
	footer="2 chats · deploy #15 · 2d ago"
	href="#open"
	{renaming}
	actions={{ onrename: () => (renaming = true) }}
	onrename={(next) => {
		name = next;
		renaming = false;
	}}
	oncancelrename={() => (renaming = false)}
/>

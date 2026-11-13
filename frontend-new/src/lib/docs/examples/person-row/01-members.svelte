<!-- The person themselves, and an invite still open. -->
<script lang="ts">
	import PersonRow from '$lib/ds/settings/PersonRow.svelte';
</script>

<ul class="flex w-full max-w-220 flex-col border-t border-border">
	<PersonRow name="Mario Prasetyo" initial="M" you="you" meta="ko••@gmail.com · joined 14 Jul">
		{#snippet controls()}<span class="text-caption text-text-2">Admin</span>{/snippet}
	</PersonRow>
	<PersonRow name="andi@kopikita.id" initial="A" meta="invited 2d ago · expires in 5 days">
		{#snippet controls()}<span class="font-mono text-label text-warning">pending · Member</span>{/snippet}
	</PersonRow>
</ul>

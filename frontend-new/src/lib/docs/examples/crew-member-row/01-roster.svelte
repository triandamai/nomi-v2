<!-- Dana chosen, Kala at work, a custom member waiting on you. -->
<script lang="ts">
	import CrewMemberRow from '$lib/ds/settings/CrewMemberRow.svelte';
</script>

<ul class="flex w-84 max-w-full flex-col gap-0.5">
	<li><CrewMemberRow agent="dana" name="Dana" role="Money records" state="on" href="?member=money" selected /></li>
	<li><CrewMemberRow agent="kala" name="Kala" role="Reminders" state="working" href="?member=reminders" /></li>
	<li><CrewMemberRow initial="B" name="Bima" role="Investor updates" state="waiting" tone="warning" href="?member=bima" /></li>
</ul>

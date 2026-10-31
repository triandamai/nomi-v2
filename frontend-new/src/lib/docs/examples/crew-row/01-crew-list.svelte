<!-- The crew in a list: who they are, what they look after, and their state when it is known. -->
<script lang="ts">
	import CrewRow from '$lib/ds/identity/CrewRow.svelte';
</script>

<div class="flex w-full max-w-75 flex-col">
	<CrewRow agent="dana" name="Dana" role="Transactions, budgets, subscriptions" state="idle" />
	<CrewRow agent="kala" name="Kala" role="Reminders on time" />
	<CrewRow agent="maya" name="Maya" role="Reads attachments" />
</div>

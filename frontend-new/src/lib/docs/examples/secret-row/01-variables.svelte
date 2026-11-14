<!-- A variable the person set, and one Nomi manages. -->
<script lang="ts">
	import SecretRow from '$lib/ds/projects/SecretRow.svelte';
</script>

<ul class="flex w-full max-w-170 flex-col border-t border-border">
	<SecretRow name="ADMIN_PIN" meta="updated 2d ago · Mario" masked="••••••" remove="Remove ADMIN_PIN" onremove={() => {}} />
	<SecretRow name="DATABASE_URL" meta="managed · rotates with the database" masked="set by Nomi" remove="Remove DATABASE_URL" />
</ul>

<!-- How a member works: their model, and what they do now. -->
<script lang="ts">
	import CrewDetailRow from '$lib/ds/settings/CrewDetailRow.svelte';
	import Select from '$lib/ds/inputs/Select.svelte';
</script>

<div class="flex w-full max-w-212 flex-col border-t border-border">
	<CrewDetailRow title="Model" description="Koda builds with this one.">
		{#snippet control()}<div class="w-60"><Select label="Model" options={[{ value: 'default', label: 'Nomi’s pick' }]} value="default" /></div>{/snippet}
	</CrewDetailRow>
	<CrewDetailRow title="Now" description="Idle" />
</div>

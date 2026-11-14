<!-- A list page: its name, what it's for, its main action. -->
<script lang="ts">
	import { Plus } from '@lucide/svelte';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
</script>

<div class="w-full max-w-180">
	<PageHeader title="Plans" intro="The plans people can be on. New accounts start on the default plan.">
		{#snippet actions()}<Button label="Add a plan" variant="inverse" iconStart={Plus} />{/snippet}
	</PageHeader>
</div>

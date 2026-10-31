<!-- Variants: primary once per screen, secondary beside it, danger for what deletes. -->
<script lang="ts">
	import { Send, Trash2 } from '@lucide/svelte';
	import PhoneButton from '$lib/ds/mobile/PhoneButton.svelte';
</script>

<div class="flex w-89.5 max-w-full flex-col gap-5">
	<PhoneButton label="Approve & send" icon={Send} onclick={() => {}} />
	<PhoneButton label="Don't send" variant="secondary" onclick={() => {}} />
	<PhoneButton label="Delete reply" variant="danger" icon={Trash2} onclick={() => {}} />
</div>

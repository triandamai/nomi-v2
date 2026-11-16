<!-- Variants: what needs the person, what opens, what stops. -->
<script lang="ts">
	import { ArrowRight, Square, Wrench } from '@lucide/svelte';
	import PhonePill from '$lib/ds/actions/PhonePill.svelte';
</script>

<div class="flex flex-wrap gap-3">
	<PhonePill variant="inverse" label="Ask Koda to fix" icon={Wrench} onclick={() => {}} />
	<PhonePill label="Open" icon={ArrowRight} href="#open" />
	<PhonePill variant="ghost" label="Stop" icon={Square} onclick={() => {}} />
</div>

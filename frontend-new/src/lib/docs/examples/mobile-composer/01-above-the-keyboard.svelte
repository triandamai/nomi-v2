<!-- Above the keyboard: attach, the message, the mic; send appears once there is text. -->
<script lang="ts">
	import Composer from '$lib/ds/mobile/Composer.svelte';
</script>

<div class="w-97.5 max-w-full">
	<Composer onsend={() => {}} onattach={() => {}} onrecord={() => {}} />
</div>

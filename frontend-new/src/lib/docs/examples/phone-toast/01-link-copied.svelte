<!-- Link copied: what happened, what follows, and one action. -->
<script lang="ts">
	import PhoneToast from '$lib/ds/mobile/PhoneToast.svelte';
</script>

<div class="w-89.5 max-w-full">
	<PhoneToast kind="success" title="Link copied" detail="Expires in 7 days" action={{ label: 'Undo', onclick: () => {} }} />
</div>

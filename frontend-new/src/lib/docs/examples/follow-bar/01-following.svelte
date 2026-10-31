<!-- Following: across the top in the teammate's colour, with a way to stop. -->
<script lang="ts">
	import FollowBar from '$lib/ds/build/FollowBar.svelte';
</script>

<div class="w-full max-w-130">
	<FollowBar name="Sari" peer={1} where="Code · +page.svelte" onstop={() => {}} />
</div>

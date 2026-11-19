<!-- Chat: back, the title and who answers, and the chat's menu. -->
<script lang="ts">
	import PhoneChatNavBar from '$lib/ds/thread/PhoneChatNavBar.svelte';
</script>

<div class="w-97.5 max-w-full">
	<PhoneChatNavBar title="Food spending in October" who={{ agent: 'dana', name: 'Dana' }} onback={() => {}} onmenu={() => {}} />
</div>

<!-- The bar: like, dislike (it asks why), regenerate with its menu. -->
<script lang="ts">
	import ReplyBar from '$lib/ds/thread/ReplyBar.svelte';
</script>

<div class="w-full max-w-180 pb-56">
	<ReplyBar text="Rp 2,1 jt on food in October." meta="14:02" onfeedback={async () => true} ondislike={async () => true} onregenerate={() => {}} ondelete={() => {}} />
</div>

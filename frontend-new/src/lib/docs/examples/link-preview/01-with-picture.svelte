<!-- With the page's picture: one card that opens the page in a new tab. -->
<script lang="ts">
	import kosRoom from '$lib/gallery/images/kos-room.jpg';
	import LinkPreview from '$lib/ds/chat/LinkPreview.svelte';
</script>

<div class="w-full max-w-170">
	<LinkPreview
		href="#"
		host="kosku.id"
		title="Kos Mawar Tebet — kamar AC, kamar mandi dalam"
		description="Rp 2.200.000 / bulan · 5 menit ke Stasiun Tebet · listrik token sendiri"
		image={kosRoom}
	/>
</div>

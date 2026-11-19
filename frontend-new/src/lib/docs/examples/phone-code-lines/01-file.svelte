<!-- A file on a phone: press Enter on a line, then Shift+↓, to ask about lines. -->
<script lang="ts">
	import PhoneCodeLines from '$lib/ds/projects/PhoneCodeLines.svelte';
	import { plainLines } from '$lib/ds/chat/code';
</script>

<div class="w-89.5 max-w-full"><PhoneCodeLines label="index.html" lines={plainLines('<h1>Warung Kopi Bu Siti</h1>\n{#each menu as item}\n  <MenuItem {item} />\n{/each}')} onask={() => {}} /></div>

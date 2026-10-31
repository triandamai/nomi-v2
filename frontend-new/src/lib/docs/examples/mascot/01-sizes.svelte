<!-- Sizes: 96 on its own, 32 in lists, 20 beside a name, 16 where the pen draws it. -->
<script lang="ts">
	import Mascot from '$lib/ds/agents/Mascot.svelte';
</script>

<div class="flex items-end gap-6">
	<Mascot agent="nomi" label="Nomi" />
	<Mascot agent="nomi" size={32} label="Nomi" />
	<Mascot agent="nomi" size={20} label="Nomi" />
	<Mascot agent="nomi" size={16} label="Nomi" />
</div>

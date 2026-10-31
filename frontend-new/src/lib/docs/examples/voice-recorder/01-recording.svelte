<!-- Recording: the real input level, the time so far and the language it transcribes in. -->
<script lang="ts">
	import VoiceRecorder from '$lib/ds/chat/VoiceRecorder.svelte';

	// Input levels from 0 to 1, oldest first; the last ten are still settling.
	const levels = [6, 10, 16, 22, 14, 8, 18, 26, 20, 12, 6, 9, 15, 24, 28, 18, 10, 7, 12, 20, 16, 9, 5, 8, 14, 22, 18, 11, 6, 4].map((height) => (height - 4) / 24);
</script>

<div class="w-full max-w-170">
	<VoiceRecorder elapsed="00:14" {levels} fading={10} language="Bahasa Indonesia" oncancel={() => {}} ondone={() => {}} />
</div>

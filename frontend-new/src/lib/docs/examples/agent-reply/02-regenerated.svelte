<!-- Regenerated: the arrows next to the name step between the versions. -->
<script lang="ts">
	import AgentReply from '$lib/ds/thread/AgentReply.svelte';
	import Prose from '$lib/ds/thread/Prose.svelte';

	let current = $state(2);
</script>

<div class="w-full max-w-180">
	<AgentReply agent="dana" name="Dana" meta="14:05 · 11s" version={{ current, total: 2, onprev: () => (current = 1), onnext: () => (current = 2) }}>
		<Prose html={current === 2 ? '<p>Rp 1,8 jt on food once office lunches are taken out.</p>' : '<p>Rp 2,1 jt on food in October.</p>'} />
	</AgentReply>
</div>

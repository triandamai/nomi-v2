<!-- Answer: the byline, the folded reasoning, the prose and the bar under it. -->
<script lang="ts">
	import AgentReply from '$lib/ds/thread/AgentReply.svelte';
	import Prose from '$lib/ds/thread/Prose.svelte';
	import ReplyBar from '$lib/ds/thread/ReplyBar.svelte';
	import Reasoning from '$lib/ds/chat/Reasoning.svelte';
</script>

<div class="w-full max-w-180">
	<AgentReply agent="dana" name="Dana" meta="14:02 · 9s">
		<Reasoning label="Thought for 8s" text="Food is up against September. Checking whether that's GoFood or groceries." />
		<Prose html="<p>Rp 2,1 jt on food in October, up 20% from September. Groceries barely moved; the jump is GoFood, mostly weekday orders after 21:00.</p>" />
		<ReplyBar text="Rp 2,1 jt on food in October." onfeedback={async () => true} ondislike={async () => true} onregenerate={() => {}} />
	</AgentReply>
</div>

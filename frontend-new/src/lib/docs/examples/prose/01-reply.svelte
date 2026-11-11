<!-- A reply: paragraphs, a list, inline code and a link, as the server renders them. -->
<script lang="ts">
	import Prose from '$lib/ds/thread/Prose.svelte';

	const html =
		'<p>Rp 2,1 jt on food in October, up <strong>20%</strong> from September.</p><ul><li>Groceries barely moved.</li><li>GoFood after 21:00 is the jump.</li></ul><p>Your budget lives in <code>budget-keluarga</code>; see <a href="https://nomi.app" target="_blank" rel="noopener noreferrer">the project</a>.</p>';
</script>

<div class="w-full max-w-180"><Prose {html} /></div>

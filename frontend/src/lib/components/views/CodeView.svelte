<!-- Code to read or copy, as the design system's code block (Chat responses → Code): numbered,
     coloured with the nm-code-* tokens, copy takes the code alone. -->
<script lang="ts">
	import CodeBlock from '$lib/components/ds/chat/CodeBlock.svelte';
	import { highlight } from '$lib/codeTokens';
	import type { CodeView, ViewProps } from '$lib/views/types';

	let { view }: ViewProps<CodeView> = $props();

	const code = $derived(view.code.replace(/\n+$/, ''));
	const tokens = $derived(highlight(code, view.language));
</script>

<CodeBlock {code} language={view.language} file={view.file} {tokens} />

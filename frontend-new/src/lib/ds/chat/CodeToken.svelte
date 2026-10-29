<script lang="ts">
	import type { CodeToken } from './code';

	// One highlighted piece of code, coloured by the nm-code-* tokens (comments recede to text-3,
	// punctuation to text-2).
	let { token }: { token: CodeToken } = $props();
</script>

<span
	class={[
		token.kind === 'keyword' && 'text-code-keyword',
		token.kind === 'string' && 'text-code-string',
		token.kind === 'fn' && 'text-code-fn',
		token.kind === 'number' && 'text-code-number',
		token.kind === 'type' && 'text-code-type',
		token.kind === 'comment' && 'text-text-3',
		token.kind === 'punct' && 'text-text-2',
	]}>{token.text}</span
>

<!-- Highlighted: tokens coloured with nm-code-*, the language and the file named. -->
<script lang="ts">
	import CodeBlock from '$lib/ds/chat/CodeBlock.svelte';
	import type { CodeLine } from '$lib/ds/chat/code';

	const tokens: CodeLine[] = [
		[{ text: "// Weekly food budget from October's average", kind: 'comment' }],
		[
			{ text: 'export function ', kind: 'keyword' },
			{ text: 'weeklyBudget', kind: 'fn' },
			{ text: '(total: ' },
			{ text: 'number', kind: 'type' },
			{ text: ', weeks = ' },
			{ text: '4', kind: 'number' },
			{ text: ') {' },
		],
		[{ text: '  ' }, { text: 'return ', kind: 'keyword' }, { text: 'Math.' }, { text: 'round', kind: 'fn' }, { text: '(total / weeks)' }],
		[{ text: '}' }],
	];
	// Copy takes the code as plain text.
	const code = tokens.map((line) => line.map((token) => token.text).join('')).join('\n');
</script>

<div class="w-full max-w-170">
	<CodeBlock language="typescript" file="src/lib/budget.ts" {code} {tokens} />
</div>

<!-- Apply on approval: Koda's edits to one file, with who is asking. -->
<script lang="ts">
	import Diff from '$lib/ds/chat/Diff.svelte';
</script>

<div class="w-full max-w-170">
	<Diff
		path="src/lib/budget.ts"
		hunks={[
			{
				header: '@@ -2,4 +2,6 @@ export function weeklyBudget',
				lines: [
					{ kind: 'context', old: 2, new: 2, text: 'export function weeklyBudget(total: number, weeks = 4) {' },
					{ kind: 'del', old: 3, text: '  return Math.round(total / weeks)' },
					{ kind: 'add', new: 3, text: '  const base = Math.round(total / weeks)' },
					{ kind: 'add', new: 4, text: '  if (base < 300_000) return base' },
					{ kind: 'add', new: 5, text: '  return base - 25_000' },
					{ kind: 'context', old: 4, new: 6, text: '}' },
				],
			},
		]}
		request={{ agent: 'koda', name: 'Koda', files: 1 }}
		onapply={() => {}}
		onreject={() => {}}
	/>
</div>

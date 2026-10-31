<!-- Failed: the failure sums up at the foot, says what is live, and offers the fix. -->
<script lang="ts">
	import Terminal from '$lib/ds/build/Terminal.svelte';
</script>

<Terminal
	run="run #3 · after Koda's weekend-slots change"
	lines={[
		{ kind: 'command', text: '$ npm test', time: '08:52:51' },
		{ kind: 'ok', text: '✓ tests/menu.test.ts (6)' },
		{ kind: 'fail', text: '✗ tests/order.test.ts (4) · 1 failed' },
		{ kind: 'fail', text: '  ✗ pickup slots on weekends › Sat after 11:00' },
		{ kind: 'out', text: '    at tests/order.test.ts:38:22' },
		{ kind: 'error', text: 'Tests  1 failed · 9 passed (10) · 2.3s' },
		{ kind: 'blank' },
		{ kind: 'warn', text: 'deploy skipped: tests failed · live stays on deploy #2' },
	]}
	failure={{ title: '1 test failed: no weekend pickup slots after 11:00', detail: 'Nothing went live. Customers still see deploy #2.', onfix: () => {} }}
/>

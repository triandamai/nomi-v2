<!-- A start that failed, with Ask Koda to fix. -->
<script lang="ts">
	import PhoneTerminal from '$lib/ds/projects/PhoneTerminal.svelte';
</script>

<div class="flex h-100 w-97.5 max-w-full flex-col">
	<PhoneTerminal
		label="dev server · sandbox"
		lines={[{ kind: 'command', text: '$ npm run dev' }, { kind: 'ok', text: '✓ 214 modules transformed' }, { kind: 'fail', text: "Cannot find module './PickupSlots.svelte'" }]}
		failure={{ title: 'The preview didn’t start', detail: 'Terminal shows what went wrong.', fix: 'Ask Koda to fix', onfix: () => {} }}
	/>
</div>

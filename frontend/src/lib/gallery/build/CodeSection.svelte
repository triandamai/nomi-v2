<script lang="ts">
	import CodeView from '$lib/components/ds/build/CodeView.svelte';
	import Terminal from '$lib/components/ds/build/Terminal.svelte';
	import DeviceBar from '$lib/components/ds/build/DeviceBar.svelte';
	import PhoneFrame from '$lib/components/ds/build/PhoneFrame.svelte';
	import CodeQuote from '$lib/components/ds/build/CodeQuote.svelte';
	import type { CodeLine } from '$lib/components/ds/chat/code';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';
	import WarungApp from './WarungApp.svelte';

	const noop = () => {};
	const tree = [
		{ name: 'src', depth: 0, kind: 'folder' as const },
		{ name: 'routes', depth: 1, kind: 'folder' as const },
		{ name: 'menu', depth: 2, kind: 'folder' as const },
		{ name: '+page.svelte', depth: 3, kind: 'file' as const, active: true },
		{ name: '+page.server.ts', depth: 3, kind: 'file' as const },
		{ name: 'lib', depth: 1, kind: 'folder' as const },
		{ name: 'menu.ts', depth: 2, kind: 'file' as const },
		{ name: 'PickupSlots.svelte', depth: 2, kind: 'file' as const },
		{ name: 'OrderSummary.svelte', depth: 2, kind: 'file' as const },
		{ name: 'db/schema.sql', depth: 1, kind: 'database' as const },
		{ name: 'tests', depth: 0, kind: 'folder' as const },
		{ name: 'order.test.ts', depth: 1, kind: 'file' as const },
		{ name: 'package.json', depth: 0, kind: 'text' as const },
	];
	const lines: CodeLine[] = [
		[{ text: '<script ', kind: 'punct' }, { text: 'lang', kind: 'fn' }, { text: '=', kind: 'punct' }, { text: '"ts"', kind: 'string' }, { text: '>', kind: 'punct' }],
		[{ text: '  ' }, { text: 'import ', kind: 'keyword' }, { text: '{ menu, slots } ' }, { text: 'from ', kind: 'keyword' }, { text: "'$lib/menu'", kind: 'string' }],
		[{ text: '  ' }, { text: 'import ', kind: 'keyword' }, { text: 'PickupSlots ' }, { text: 'from ', kind: 'keyword' }, { text: "'$lib/PickupSlots.svelte'", kind: 'string' }],
		[],
		[{ text: '  ' }, { text: 'let ', kind: 'keyword' }, { text: 'cart: ' }, { text: 'CartItem', kind: 'type' }, { text: '[] = []' }],
		[
			{ text: '  ' },
			{ text: 'const ', kind: 'keyword' },
			{ text: 'total = ' },
			{ text: '$derived', kind: 'fn' },
			{ text: '(cart.' },
			{ text: 'reduce', kind: 'fn' },
			{ text: '((s, i) ⇒ s + i.price * i.qty, ' },
			{ text: '0', kind: 'number' },
			{ text: '))' },
		],
		[{ text: '<\/script>', kind: 'punct' }],
		[],
		[{ text: '<h1>', kind: 'punct' }, { text: 'Warung Kopi Bu Siti' }, { text: '</h1>', kind: 'punct' }],
		[{ text: '{#each ', kind: 'keyword' }, { text: 'menu ' }, { text: 'as ', kind: 'keyword' }, { text: 'item' }, { text: '}', kind: 'keyword' }],
		[{ text: '  ' }, { text: '<MenuItem ', kind: 'type' }, { text: '{item} ' }, { text: 'on:add', kind: 'fn' }, { text: '={() ⇒ cart.' }, { text: 'push', kind: 'fn' }, { text: '(item)} ' }, { text: '/>', kind: 'punct' }],
		[{ text: '{/each}', kind: 'keyword' }],
		[],
		[
			{ text: '<PickupSlots ', kind: 'type' },
			{ text: 'from', kind: 'fn' },
			{ text: '=', kind: 'punct' },
			{ text: '"07:00" ', kind: 'string' },
			{ text: 'to', kind: 'fn' },
			{ text: '=', kind: 'punct' },
			{ text: '"11:00" ', kind: 'string' },
			{ text: 'every', kind: 'fn' },
			{ text: '={', kind: 'punct' },
			{ text: '15', kind: 'number' },
			{ text: '} />', kind: 'punct' },
		],
		[{ text: '<OrderSummary ', kind: 'type' }, { text: '{cart} {total} ' }, { text: '/>', kind: 'punct' }],
	];
	const code = { branch: 'main@9d2e41a', tree, path: 'src/routes/menu/+page.svelte', meta: 'read-only · edited by Koda 08:41', lines, onask: noop, onasklines: noop };

	const published = [
		{ kind: 'command' as const, text: '$ npm run build', time: '08:41:02' },
		{ kind: 'out' as const, text: 'vite v5.4.2 building for production…' },
		{ kind: 'ok' as const, text: '✓ 214 modules transformed.' },
		{ kind: 'out' as const, text: 'build/client/_app/immutable/nodes/menu.js   48.2 kB │ gzip 14.1 kB' },
		{ kind: 'ok' as const, text: '✓ built in 9.6s' },
		{ kind: 'blank' as const },
		{ kind: 'command' as const, text: '$ npm test', time: '08:41:13' },
		{ kind: 'ok' as const, text: '✓ tests/menu.test.ts (6)' },
		{ kind: 'ok' as const, text: '✓ tests/order.test.ts (4)' },
		{ kind: 'out' as const, text: 'Tests  10 passed (10) · 2.1s' },
		{ kind: 'blank' as const },
		{ kind: 'command' as const, text: '$ nomi deploy --prod', time: '08:41:16' },
		{ kind: 'out' as const, text: 'uploading 9 files · 212 kB' },
		{ kind: 'ok' as const, text: '→ warung-kopi-menu-3f9a12.trian.space · deploy #2' },
		{ kind: 'ok' as const, text: '✓ live in 01:12' },
	];
	const failed = [
		{ kind: 'command' as const, text: '$ npm run build', time: '08:52:40' },
		{ kind: 'ok' as const, text: '✓ built in 9.9s' },
		{ kind: 'blank' as const },
		{ kind: 'command' as const, text: '$ npm test', time: '08:52:51' },
		{ kind: 'ok' as const, text: '✓ tests/menu.test.ts (6)' },
		{ kind: 'fail' as const, text: '✗ tests/order.test.ts (4) · 1 failed' },
		{ kind: 'fail' as const, text: '  ✗ pickup slots on weekends › Sat after 11:00' },
		{ kind: 'fail' as const, text: '    expected 4 slots, received 0' },
		{ kind: 'out' as const, text: '    at tests/order.test.ts:38:22' },
		{ kind: 'error' as const, text: 'Tests  1 failed · 9 passed (10) · 2.3s' },
		{ kind: 'blank' as const },
		{ kind: 'warn' as const, text: 'deploy skipped: tests failed · live stays on deploy #2' },
	];
</script>

<GallerySection
	title="Code, terminal & phone"
	description="The preview pane's other two tabs and the phone switch. Code and Terminal are read-only: what Koda wrote and what Koda ran. Changes always go through the chat."
>
	<div class="flex flex-col gap-6">
		<GalleryBoard title="Code view" note="file tree + read-only file · lines can be selected and asked about" inline>
			<div class="flex flex-col gap-5">
				<CodeView {...code} />
				<CodeView {...code} selection={[14, 15]} />
			</div>
		</GalleryBoard>
		<GalleryBoard title="Terminal" note="what Koda ran, in order · read-only · failures sum up in one line" inline>
			<div class="flex flex-col gap-5">
				<Terminal run="run #2 · publish deploy #2" lines={published} />
				<Terminal
					run="run #3 · after Koda's weekend-slots change"
					lines={failed}
					failure={{ title: '1 test failed: no weekend pickup slots after 11:00', detail: 'Nothing went live. Customers still see deploy #2.', onfix: noop }}
				/>
			</div>
		</GalleryBoard>
		<div class="flex flex-col gap-6 lg:flex-row lg:items-start lg:gap-8">
			<div class="lg:w-110 lg:shrink-0"><GalleryBoard title="Phone preview" note="real phone size · light app inside the dark workspace" inline>
				<div class="flex flex-col gap-5">
					<DeviceBar device="iPhone 15 · 393 × 852" ondevice={noop} onphone={noop} />
					<PhoneFrame>
						<WarungApp />
					</PhoneFrame>
				</div>
			</GalleryBoard></div>
			<div class="min-w-0 flex-1"><GalleryBoard title="Code quote" note="selected lines, quoted in the composer" inline>
				<CodeQuote file="+page.svelte" range="14–15" snippet={'<PickupSlots from="07:00" to="11:00" …'} onremove={noop} />
			</GalleryBoard></div>
		</div>
	</div>
</GallerySection>


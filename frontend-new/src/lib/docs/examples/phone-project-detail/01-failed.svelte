<!-- Failed: what broke leads, with Ask Koda to fix. -->
<script lang="ts">
	import { Wrench } from '@lucide/svelte';
	import PhoneProjectDetail from '$lib/ds/projects/PhoneProjectDetail.svelte';
</script>

<div class="h-200 w-97.5 max-w-full outline -outline-offset-1 outline-border">
	<PhoneProjectDetail
		name="invoice-sinar-jaya"
		tone="kala"
		status="deploy #6 failed · #5 still live"
		dot="failed"
		alert={{ title: 'Deploy failed', detail: 'build error in src/lib/pdf.ts · #5 still live', action: { label: 'Ask Koda to fix', icon: Wrench, onclick: () => {} } }}
		workspaceHref="#workspace"
		url="invoice-sinar-jaya.trian.space"
		parts={{ code: '41 files · Svelte', url: '…41d2.trian.space', data: 'Postgres · 3 tables', deploys: '6 · last built in 41s', members: 'Trian (owner), you' }}
		onexport={() => {}}
		chats={[{ title: 'Build an invoice tracker', age: '5d', href: '#chat' }]}
		backLabel="Projects"
		onback={() => {}}
		onmore={() => {}}
	/>
</div>

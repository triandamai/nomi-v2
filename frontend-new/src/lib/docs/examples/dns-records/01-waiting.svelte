<!-- A domain waiting for DNS: one record found, two still waiting. -->
<script lang="ts">
	import DnsRecords from '$lib/ds/projects/DnsRecords.svelte';
</script>

<div class="w-full max-w-170">
	<DnsRecords
		checked="last checked 08:51 · every 5 min"
		records={[
			{ type: 'A', name: '@', value: '76.76.21.21', found: false },
			{ type: 'CNAME', name: 'www', value: 'warung-kopi-menu-3f9a12.trian.space', found: true },
			{ type: 'TXT', name: '_nomi-verify', value: 'nomi-verify=4be1c07', found: false },
		]}
		labels={{ heading: 'Set these at your registrar', type: 'Type', name: 'Name', value: 'Value', status: 'Status', found: 'found', waiting: 'waiting', copy: 'Copy' }}
	/>
</div>

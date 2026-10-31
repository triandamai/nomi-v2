<!-- Comparable rows: numbers in mono, a total, and where the data came from. -->
<script lang="ts">
	import DataTable from '$lib/ds/chat/DataTable.svelte';
</script>

<div class="w-full max-w-170">
	<DataTable
		title="Food spending by week"
		columns={['Week', 'Groceries', 'GoFood', 'Total', 'vs Sep']}
		rows={[
			['1–7 Oct', '312.000', '168.000', '480.000', { value: '+12%', tone: 'bad' }],
			['8–14 Oct', '286.000', '241.000', '527.000', { value: '+31%', tone: 'bad' }],
			['15–21 Oct', '301.000', '322.000', '623.000', { value: '+48%', tone: 'bad' }],
			['22–28 Oct', '274.000', '196.000', '470.000', { value: '−6%', tone: 'good' }],
		]}
		total={['October', '1.173.000', '927.000', '2.100.000', { value: '+20%', tone: 'bad' }]}
		source="Rp · from Dana · 142 transactions"
	/>
</div>

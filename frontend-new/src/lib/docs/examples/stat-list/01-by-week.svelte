<!-- By week: each row's total and change on one line, the breakdown under it, the total last. -->
<script lang="ts">
	import StatList from '$lib/ds/mobile/StatList.svelte';
</script>

<div class="w-89.5 max-w-full">
	<StatList
		title="Food by week"
		sub="Oct · Rp"
		rows={[
			{ label: '1–7 Oct', value: '480rb', delta: { value: '+12%', tone: 'bad' }, detail: 'Groceries 312rb · GoFood 168rb' },
			{ label: '8–14 Oct', value: '527rb', delta: { value: '+31%', tone: 'bad' }, detail: 'Groceries 286rb · GoFood 241rb' },
			{ label: '15–21 Oct', value: '623rb', delta: { value: '+48%', tone: 'bad' }, detail: 'Groceries 301rb · GoFood 322rb' },
			{ label: '22–28 Oct', value: '470rb', delta: { value: '−6%', tone: 'good' }, detail: 'Groceries 274rb · GoFood 196rb' },
		]}
		total={{ label: 'October', value: 'Rp 2,1 jt', delta: { value: '+20%', tone: 'bad' } }}
	/>
</div>

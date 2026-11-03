import { describe, expect, it } from 'vitest';
import { citedSources } from './citations';

describe('citedSources', () => {
	it('lists each cited page once, by its number, titled from the pages the crew read', () => {
		const text = 'Rent goes up in January [2](https://kosku.id/mawar). It was Rp 2 jt [1](https://www.example.com/a), see [2](https://kosku.id/mawar).';
		expect(citedSources(text, [{ title: 'Kos Mawar Tebet', url: 'https://kosku.id/mawar' }])).toEqual([
			{
				n: 1,
				url: 'https://www.example.com/a',
				title: 'example.com',
				host: 'example.com',
			},
			{
				n: 2,
				url: 'https://kosku.id/mawar',
				title: 'Kos Mawar Tebet',
				host: 'kosku.id',
			},
		]);
	});

	it('takes only numbered web links as citations', () => {
		expect(citedSources('See [the rules](https://a.example) and [1](mailto:x@y.z)')).toEqual([]);
	});
});

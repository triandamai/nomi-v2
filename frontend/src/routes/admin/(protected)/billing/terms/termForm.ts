/** The most a term can take off. */
export const MAX_DISCOUNT_PERCENT = 100;

export interface TermPayload {
	months: number;
	discount_percent: number;
	label: string;
	is_active: boolean;
	sort_order: number;
}

const whole = (raw: FormDataEntryValue | null): number | null => {
	if (typeof raw !== 'string' || raw.trim() === '') return null;
	const value = Number(raw);
	return Number.isInteger(value) ? value : null;
};

/** The term form as the backend takes it; null when the months or the discount aren't usable. */
export function readTerm(data: FormData): TermPayload | null {
	const months = whole(data.get('months'));
	const discount = whole(data.get('discount_percent'));
	if (months === null || months < 1) return null;
	if (discount === null || discount < 0 || discount > MAX_DISCOUNT_PERCENT) return null;
	const label = data.get('label');
	return {
		months,
		discount_percent: discount,
		label: typeof label === 'string' ? label.trim() : '',
		is_active: data.get('is_active') === 'true',
		sort_order: whole(data.get('sort_order')) ?? 0,
	};
}

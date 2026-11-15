import type { CouponPayload } from '$lib/types';

/** A percent coupon can't take off more than the whole price. */
export const MAX_PERCENT = 100;
/** Ends at the close of the chosen day, like a plan's promo. */
const END_OF_DAY = 'T23:59:59';
const START_OF_DAY = 'T00:00:00';

/** What was wrong with the form, for the page to say in words. */
export type CouponFormProblem = 'code' | 'value' | 'limits' | 'dates';

export type CouponForm = { ok: true; payload: CouponPayload } | { ok: false; problem: CouponFormProblem };

const text = (data: FormData, name: string): string => {
	const value = data.get(name);
	return typeof value === 'string' ? value.trim() : '';
};

/** A whole number, or null when the box is empty or holds something else. */
const whole = (raw: string): number | null => {
	if (raw === '') return null;
	const value = Number(raw);
	return Number.isInteger(value) ? value : null;
};

/** The chosen day's start or end as an instant; null when there's no day or it isn't one. */
function day(raw: string, time: string): string | null {
	if (!/^\d{4}-\d{2}-\d{2}$/.test(raw)) return null;
	const date = new Date(`${raw}${time}`);
	return Number.isNaN(date.getTime()) ? null : date.toISOString();
}

/** The coupon form as the backend takes it, or what's wrong with it. */
export function readCoupon(data: FormData): CouponForm {
	const code = text(data, 'code').toUpperCase();
	if (!code || /\s/.test(code)) return { ok: false, problem: 'code' };

	const kind = text(data, 'discount_kind') === 'amount' ? 'amount' : 'percent';
	const value = whole(text(data, 'discount_value'));
	if (value === null || value < 1 || (kind === 'percent' && value > MAX_PERCENT)) return { ok: false, problem: 'value' };

	const cap = text(data, 'max_redemptions');
	const maxRedemptions = whole(cap);
	const perUser = whole(text(data, 'per_user_limit')) ?? 1;
	const minMonths = whole(text(data, 'min_months')) ?? 1;
	if ((cap !== '' && (maxRedemptions === null || maxRedemptions < 1)) || perUser < 1 || minMonths < 1) return { ok: false, problem: 'limits' };

	const startRaw = text(data, 'starts_at');
	const endRaw = text(data, 'ends_at');
	const startsAt = startRaw ? day(startRaw, START_OF_DAY) : null;
	const endsAt = endRaw ? day(endRaw, END_OF_DAY) : null;
	if ((startRaw && !startsAt) || (endRaw && !endsAt) || (startsAt && endsAt && endsAt < startsAt)) return { ok: false, problem: 'dates' };

	const description = text(data, 'description');
	return {
		ok: true,
		payload: {
			code,
			description: description || null,
			discount_kind: kind,
			discount_value: value,
			max_redemptions: maxRedemptions,
			per_user_limit: perUser,
			starts_at: startsAt,
			ends_at: endsAt,
			plan_ids: data.getAll('plan_ids').filter((id): id is string => typeof id === 'string' && id.length > 0),
			min_months: minMonths,
			first_payment_only: data.get('first_payment_only') === 'true',
			is_active: data.get('is_active') === 'true',
		},
	};
}

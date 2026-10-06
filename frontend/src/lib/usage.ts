// Formatting for token usage and spend (Billing & usage, and the drawer's usage meter).

import { getLocale } from '$lib/paraglide/runtime';

/** "1.2M", "48K", "950": tokens, short. */
export function formatTokens(tokens: number): string {
	return new Intl.NumberFormat(getLocale(), { notation: 'compact', maximumFractionDigits: 1 }).format(tokens);
}

/** Every token, for tables and tooltips: "1,234,567". */
export function formatTokensFull(tokens: number): string {
	return new Intl.NumberFormat(getLocale()).format(tokens);
}

/** US dollars, with cents; tiny amounts keep enough digits to not read as zero. */
export function formatUsd(amount: number): string {
	const digits = amount > 0 && amount < 0.01 ? 4 : 2;
	return new Intl.NumberFormat(getLocale(), { style: 'currency', currency: 'USD', minimumFractionDigits: digits, maximumFractionDigits: digits }).format(amount);
}

/** Share of the allowance used, 0..1 (more than 1 when over). */
export function usageShare(used: number, allowance: number): number {
	return allowance > 0 ? used / allowance : 0;
}

/** "42%", rounding up so any use at all shows as at least 1%. */
export function formatShare(share: number): string {
	const percent = share > 0 && share < 0.01 ? 1 : Math.round(share * 100);
	return new Intl.NumberFormat(getLocale(), { style: 'percent' }).format(percent / 100);
}

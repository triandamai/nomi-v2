import { describe, expect, it } from 'vitest';
import { countdown, displayEmail, isEmail, mailApp, secondsUntil } from './format';

describe('displayEmail', () => {
	it('shows the address the API hid with dots, as the design draws it', () => {
		expect(displayEmail('k******@gmail.com')).toBe('k••••••@gmail.com');
		expect(displayEmail('a**@example.com')).toBe('a••@example.com');
	});

	it('leaves an address that is already hidden, or not hidden at all, as it is', () => {
		expect(displayEmail('ko••••@gmail.com')).toBe('ko••••@gmail.com');
		expect(displayEmail('')).toBe('');
	});
});

describe('countdown', () => {
	it('reads as minutes and seconds', () => {
		expect(countdown(42)).toBe('0:42');
		expect(countdown(60)).toBe('1:00');
		expect(countdown(125)).toBe('2:05');
		expect(countdown(5)).toBe('0:05');
	});

	it('never goes below zero or shows a fraction', () => {
		expect(countdown(-3)).toBe('0:00');
		expect(countdown(41.2)).toBe('0:42');
	});
});

describe('secondsUntil', () => {
	it('counts whole seconds down to a moment, and stops at zero', () => {
		expect(secondsUntil(10_000, 0)).toBe(10);
		expect(secondsUntil(10_000, 9_001)).toBe(1);
		expect(secondsUntil(10_000, 10_000)).toBe(0);
		expect(secondsUntil(10_000, 12_000)).toBe(0);
	});
});

describe('mailApp', () => {
	it('knows Gmail and Outlook addresses, hidden or not', () => {
		expect(mailApp('k******@gmail.com')).toBe('gmail');
		expect(mailApp('ana@googlemail.com')).toBe('gmail');
		expect(mailApp('ANA@Outlook.com')).toBe('outlook');
		for (const domain of ['hotmail.com', 'live.com', 'msn.com', 'outlook.co.id', 'hotmail.co.uk']) expect(mailApp(`a@${domain}`)).toBe('outlook');
	});

	it('offers no app for any other address', () => {
		for (const email of ['a@nomi.test', 'a@yahoo.com', 'a@gmail.com.evil.test', 'not an email', '']) expect(mailApp(email)).toBeNull();
	});
});

describe('isEmail', () => {
	it('takes what the backend takes as an address', () => {
		for (const email of ['ana@example.com', 'a.b+c@sub.example.co.id']) expect(isEmail(email)).toBe(true);
	});

	it('refuses what is plainly not one', () => {
		for (const email of ['', 'ana', 'ana@', '@example.com', 'ana@example', 'ana@.com', 'ana@example.', 'ana @example.com']) expect(isEmail(email)).toBe(false);
	});
});

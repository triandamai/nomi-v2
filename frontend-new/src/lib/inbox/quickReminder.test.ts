import { describe, expect, it } from 'vitest';
import { dueAt, localNow, parseReminder } from './quickReminder';

// Wed 21 Oct 2026, 08:14 where the person is (the Inbox in the pen).
const now = { date: '2026-10-21', time: '08:14' };

describe('reading a quick reminder', () => {
	it('reads what, the day, the time and how often from one line', () => {
		expect(parseReminder('pay PLN electricity every 25th at 10am', now)).toEqual({ title: 'Pay PLN electricity', date: '2026-10-25', time: '10:00', repeat: 'monthly' });
	});

	it('puts a time later today on today, and one already gone on tomorrow', () => {
		expect(parseReminder('Stretch at 4pm', now)).toEqual({ title: 'Stretch', date: '2026-10-21', time: '16:00' });
		expect(parseReminder('call Siti at 7', now)).toEqual({ title: 'Call Siti', date: '2026-10-22', time: '07:00' });
	});

	it('reads clock times, with or without am and pm', () => {
		expect(parseReminder('stand-up 9:30', now)).toMatchObject({ title: 'Stand-up', time: '09:30' });
		expect(parseReminder('lunch at 12pm', now)).toMatchObject({ time: '12:00' });
		expect(parseReminder('backup at 12am', now)).toMatchObject({ time: '00:00', date: '2026-10-22' });
		expect(parseReminder('call mum 6pm', now)).toMatchObject({ title: 'Call mum', time: '18:00' });
	});

	it('leaves numbers that aren’t a time in the title', () => {
		expect(parseReminder('Netflix renews · Rp 186.000 on fri at 09:00', now)).toEqual({ title: 'Netflix renews · Rp 186.000', date: '2026-10-23', time: '09:00' });
	});

	it('repeats every day, week or month', () => {
		expect(parseReminder('every day at 6:30 gym', now)).toEqual({ title: 'Gym', date: '2026-10-22', time: '06:30', repeat: 'daily' });
		expect(parseReminder('gym every monday at 6:30', now)).toEqual({ title: 'Gym', date: '2026-10-26', time: '06:30', repeat: 'weekly' });
		expect(parseReminder('water plants weekly at 9', now)).toEqual({ title: 'Water plants', date: '2026-10-21', time: '09:00', repeat: 'weekly' });
		expect(parseReminder('rent monthly at 9', now)).toEqual({ title: 'Rent', date: '2026-10-21', time: '09:00', repeat: 'monthly' });
	});

	it('keeps a weekday that is today on today while its time is still ahead', () => {
		expect(parseReminder('review every wednesday at 9', now)).toMatchObject({ date: '2026-10-21', repeat: 'weekly' });
		expect(parseReminder('review every wednesday at 8', now)).toMatchObject({ date: '2026-10-28', repeat: 'weekly' });
	});

	it('moves a day of the month that has gone, or doesn’t exist, to the next month that has it', () => {
		expect(parseReminder('every 21st at 8am', now)).toEqual({ title: '', date: '2026-11-21', time: '08:00', repeat: 'monthly' });
		expect(parseReminder('invoice on the 31st at 9', { date: '2026-11-05', time: '10:00' })).toEqual({ title: 'Invoice', date: '2026-12-31', time: '09:00' });
	});

	it('reads today, tomorrow and the day after', () => {
		expect(parseReminder('remind me to water the plants tomorrow', now)).toEqual({ title: 'Water the plants', date: '2026-10-22' });
		expect(parseReminder('pick up the suit the day after tomorrow at 5pm', now)).toEqual({ title: 'Pick up the suit', date: '2026-10-23', time: '17:00' });
		expect(parseReminder('pay rent today at 8pm', now)).toEqual({ title: 'Pay rent', date: '2026-10-21', time: '20:00' });
	});

	it('counts “in 10 minutes” from now, across midnight too', () => {
		expect(parseReminder('in 10 minutes take the pizza out', now)).toEqual({ title: 'Take the pizza out', date: '2026-10-21', time: '08:24' });
		expect(parseReminder('sleep in 30 min', { date: '2026-10-31', time: '23:50' })).toEqual({ title: 'Sleep', date: '2026-11-01', time: '00:20' });
		expect(parseReminder('check the oven in 2 hours', now)).toMatchObject({ date: '2026-10-21', time: '10:14' });
	});

	it('reads Indonesian as well', () => {
		expect(parseReminder('bayar listrik PLN setiap tanggal 25 jam 10 pagi', now)).toEqual({ title: 'Bayar listrik PLN', date: '2026-10-25', time: '10:00', repeat: 'monthly' });
		expect(parseReminder('ingatkan saya minum obat setiap hari jam 9 malam', now)).toEqual({ title: 'Minum obat', date: '2026-10-21', time: '21:00', repeat: 'daily' });
		expect(parseReminder('besok jam 4 sore telepon ibu', now)).toEqual({ title: 'Telepon ibu', date: '2026-10-22', time: '16:00' });
		expect(parseReminder('dalam 2 jam lagi angkat jemuran', now)).toEqual({ title: 'Angkat jemuran', date: '2026-10-21', time: '10:14' });
		expect(parseReminder('rapat setiap minggu jam 10', now)).toEqual({ title: 'Rapat', date: '2026-10-21', time: '10:00', repeat: 'weekly' });
		expect(parseReminder('olahraga setiap hari minggu pukul 06.30', now)).toEqual({ title: 'Olahraga', date: '2026-10-25', time: '06:30', repeat: 'weekly' });
		expect(parseReminder('jemput anak hari ini jam 1 siang', now)).toEqual({ title: 'Jemput anak', date: '2026-10-21', time: '13:00' });
	});

	it('says nothing about when, when the line doesn’t say', () => {
		expect(parseReminder('buy milk', now)).toEqual({ title: 'Buy milk' });
		expect(parseReminder('   ', now)).toEqual({ title: '' });
	});
});

describe('when a quick reminder is due', () => {
	it('is the day and the time, read in the person’s zone by the backend', () => {
		expect(dueAt({ title: 'Rent', date: '2026-10-25', time: '10:00' })).toBe('2026-10-25T10:00');
	});

	it('isn’t known without both', () => {
		expect(dueAt({ title: 'Rent', date: '2026-10-25' })).toBeUndefined();
		expect(dueAt({ title: 'Rent', time: '10:00' })).toBeUndefined();
	});

	it('starts from the date and time where the person is', () => {
		expect(localNow(new Date('2026-10-21T01:14:30Z'), 'Asia/Jakarta')).toEqual({ date: '2026-10-21', time: '08:14' });
		expect(localNow(new Date('2026-10-21T01:14:30Z'), 'Nowhere/Atlantis')).toEqual({ date: '2026-10-21', time: '01:14' });
	});
});

# Build notes — phone-tabs

New design-system pieces, each with its docs entry and a live example:
`thread/PhoneAskCard`, `thread/PhoneTodayList`, `thread/PhoneCrewRow`, `inbox/PhoneInboxCard`,
`inbox/PhoneTimelineRow`, `settings/PhonePerson`, `settings/PhonePlanCard`,
`settings/PhoneSettingsGroup`.

Reused as they are: `LargeTitle`, `PhoneFilters`, `PhonePill`, `Fab`, `EmptyState`, `BottomSheet`,
`SheetOption`, `SheetRow`, `Starters`, `QuickReminder`, `EarlierToggle`.

Routes:
- `/` (Home), `/inbox` and `/settings` render the phone screen when `phoneView` says so, else the
  desktop page as before.
- `/settings` no longer always redirects: a phone gets You (`+page.svelte`), a desktop Profile.
- The shell's person carries `username` for You's line.
- The layout adds Inbox and the settings index to `ownTitle`, and a back link to You on a settings
  page on a phone.

Where the build differs from the pen, on purpose:
- The Today rows' titles and lines are the app's own ("Reminders", not "Reminders today").
- The Try label is the one `Starters` draws, 4 to the left of the pen's.
- The Inbox card's main pill says what the app can do ("Open"), not the pen's per-kind labels
  ("Open invite", "Open project"), which the notification doesn't say.

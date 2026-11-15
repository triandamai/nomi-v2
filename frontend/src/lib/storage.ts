/** Storage is entered in MB; from this many MB the form also shows it in GB. */
export const MB_PER_GB = 1024;

/** "512 MB" below a gigabyte, "1 GB" / "1.5 GB" from there. */
export function formatStorage(mb: number): string {
	if (mb < MB_PER_GB) return `${mb} MB`;
	return `${Number((mb / MB_PER_GB).toFixed(1))} GB`;
}

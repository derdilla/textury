export function parseCustomDate(dateObj: any): number {
  if (!dateObj) return 0;

  try {
    const arr = Array.isArray(dateObj) ? dateObj : Object.values(dateObj);
    if (!arr || arr.length < 5) return 0;

    const year = Number(arr[0]);
    const dayOfYear = Number(arr[1]);
    const hour = Number(arr[2]);
    const minute = Number(arr[3]);
    const second = Number(arr[4]);

    const date = new Date(Date.UTC(year, 0, 1, hour, minute, second));
    date.setUTCDate(date.getUTCDate() + (dayOfYear - 1));

    return date.getTime();
  } catch {
    return 0;
  }
}

export function formatExpiryDuration(minutes: number | null): string {
  if (!minutes || minutes <= 0) return "";

  const days = Math.floor(minutes / (24 * 60));
  const hours = Math.floor((minutes % (24 * 60)) / 60);
  const mins = minutes % 60;

  const parts: string[] = [];

  if (days > 0) {
    parts.push(`${days} day${days === 1 ? '' : 's'}`);
  }
  if (hours > 0) {
    parts.push(`${hours} hour${hours === 1 ? '' : 's'}`);
  }
  if (mins > 0 || parts.length === 0) {
    parts.push(`${mins} minute${mins === 1 ? '' : 's'}`);
  }

  return `Expires in ${parts.join(' ')}`;
}

export function formatTimeAgo(dateObj: any): string {
  const time = parseCustomDate(dateObj);
  if (!time) return 'Invalid Date';

  const seconds = Math.max(0, Math.floor((Date.now() - time) / 1000));
  if (seconds < 60) {
    return 'just now';
  }

  const timeUnits: [string, number][] = [
    ['year', 365 * 24 * 60 * 60],
    ['month', (365 / 12) * 24 * 60 * 60],
    ['week', 7 * 24 * 60 * 60],
    ['day', 24 * 60 * 60],
    ['hour', 60 * 60],
    ['minute', 60],
  ];

  for (const [unit, unitSeconds] of timeUnits) {
    const count = Math.floor(seconds / unitSeconds);
    if (count >= 1) {
      return `${count} ${unit}${count > 1 ? 's' : ''} ago`;
    }
  }

  return new Date(time).toLocaleDateString();
}

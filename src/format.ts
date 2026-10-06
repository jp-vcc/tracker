const pad2 = (n: number): string => String(n).padStart(2, '0');

function whole(sec: number): number {
  return Number.isFinite(sec) ? Math.max(0, Math.floor(sec)) : 0;
}

/** `3h 05m`. Floors to the minute; sum seconds first, then format. */
export function formatHm(sec: number): string {
  const t = whole(sec);
  const h = Math.floor(t / 3600);
  const m = Math.floor((t % 3600) / 60);
  return `${h}h ${pad2(m)}m`;
}

/** `H:MM:SS` for the live timer. */
export function formatClock(sec: number): string {
  const t = whole(sec);
  const h = Math.floor(t / 3600);
  const m = Math.floor((t % 3600) / 60);
  return `${h}:${pad2(m)}:${pad2(t % 60)}`;
}

/** History rows: seconds for very short entries, otherwise hours and minutes. */
export function formatEntryDuration(sec: number): string {
  const t = whole(sec);
  return t < 60 ? `${t}s` : formatHm(t);
}

/** Parses hour and minute inputs. Empty counts as 0; anything but whole numbers is `null`. */
export function hmToSeconds(hours: string, minutes: string): number | null {
  const h = hours.trim() === '' ? '0' : hours.trim();
  const m = minutes.trim() === '' ? '0' : minutes.trim();
  if (!/^\d+$/.test(h) || !/^\d+$/.test(m)) return null;
  return Number(h) * 3600 + Number(m) * 60;
}

export function splitHm(sec: number): { hours: number; minutes: number } {
  const t = whole(sec);
  return { hours: Math.floor(t / 3600), minutes: Math.floor((t % 3600) / 60) };
}

/** Local calendar date `YYYY-MM-DD`. */
export function todayLocal(now: Date = new Date()): string {
  return `${now.getFullYear()}-${pad2(now.getMonth() + 1)}-${pad2(now.getDate())}`;
}

/** `1 task`, `2 tasks`. */
export function count(n: number, one: string, many: string): string {
  return `${n.toLocaleString('en-US')} ${n === 1 ? one : many}`;
}


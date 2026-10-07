import type { ActiveTimer, Entry, Project, Task } from './types';

/** Case-insensitive by name; the one comparator for every list and fallback. */
export function byName<T extends { name: string }>(a: T, b: T): number {
  const x = a.name.toLowerCase();
  const y = b.name.toLowerCase();
  if (x !== y) return x < y ? -1 : 1;
  return a.name < b.name ? -1 : a.name > b.name ? 1 : 0;
}

export function sortedProjects(projects: Project[]): Project[] {
  return [...projects].sort(byName);
}

/** Tasks of one project (none when `projectId` is null), sorted by name. */
export function sortedTasks(tasks: Task[], projectId: string | null): Task[] {
  if (projectId === null) return [];
  return tasks.filter((t) => t.projectId === projectId).sort(byName);
}

/** Seconds per task in one O(n) pass. Totals are derived, never stored. */
export function taskTotals(entries: Entry[]): Map<string, number> {
  const totals = new Map<string, number>();
  for (const e of entries) totals.set(e.taskId, (totals.get(e.taskId) ?? 0) + e.durationSec);
  return totals;
}

/** Seconds per project: the sum of its tasks' totals. */
export function projectTotals(tasks: Task[], totals: Map<string, number>): Map<string, number> {
  const out = new Map<string, number>();
  for (const t of tasks) out.set(t.projectId, (out.get(t.projectId) ?? 0) + (totals.get(t.id) ?? 0));
  return out;
}

/** Newest first: by date, then by creation time. */
export function newestFirst(a: Entry, b: Entry): number {
  if (a.date !== b.date) return a.date < b.date ? 1 : -1;
  return b.createdAt - a.createdAt;
}

export function entriesForTask(entries: Entry[], taskId: string): Entry[] {
  return entries.filter((e) => e.taskId === taskId).sort(newestFirst);
}

export function entriesForProject(entries: Entry[], tasks: Task[], projectId: string): Entry[] {
  const ids = new Set(tasks.filter((t) => t.projectId === projectId).map((t) => t.id));
  return entries.filter((e) => ids.has(e.taskId)).sort(newestFirst);
}

export interface DeleteImpact {
  tasks: number;
  entries: number;
  discardsTimer: boolean;
}

/** What deleting a project or task removes, for the confirmation text. */
export function deleteImpact(
  data: { tasks: Task[]; entries: Entry[]; activeTimer: ActiveTimer | null },
  kind: 'project' | 'task',
  id: string,
): DeleteImpact {
  const taskIds = new Set(
    kind === 'project' ? data.tasks.filter((t) => t.projectId === id).map((t) => t.id) : [id],
  );
  return {
    tasks: kind === 'project' ? taskIds.size : 1,
    entries: data.entries.filter((e) => taskIds.has(e.taskId)).length,
    discardsTimer: data.activeTimer !== null && taskIds.has(data.activeTimer.taskId),
  };
}

/** Whole seconds since the timer start; never negative (clock set backwards). */
export function elapsedSec(timer: ActiveTimer | null, nowMs: number): number {
  if (!timer) return 0;
  return Math.max(0, Math.floor((nowMs - timer.startedAt) / 1000));
}


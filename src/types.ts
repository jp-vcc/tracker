// Mirrors crates/tracker-core/src/model.rs. Keep the two in sync.

export type Source = 'timer' | 'manual';

export interface Project {
  id: string;
  name: string;
  createdAt: number;
}

export interface Task {
  id: string;
  projectId: string;
  name: string;
  createdAt: number;
}

export interface Entry {
  id: string;
  taskId: string;
  /** Local calendar date YYYY-MM-DD. Authoritative for "when". */
  date: string;
  /** UTC epoch ms of the timer start (audit only); null for manual entries. */
  startedAt: number | null;
  durationSec: number;
  note: string;
  source: Source;
  createdAt: number;
}

export interface ActiveTimer {
  taskId: string;
  startedAt: number;
}

export interface Snapshot {
  schemaVersion: number;
  projects: Project[];
  tasks: Task[];
  entries: Entry[];
  activeTimer: ActiveTimer | null;
}

export type BootResult =
  | { status: 'ok'; snapshot: Snapshot }
  | { status: 'corrupt'; backupPath: string; reason: string }
  | { status: 'newer'; version: number }
  | { status: 'io_error'; message: string };

export type RecoveryResult = Exclude<BootResult, { status: 'ok' }>;

/** Error shape serialised by the Rust side. */
export interface AppError {
  code: string;
  field?: string | null;
  message: string;
}

export interface NewEntry {
  taskId: string;
  date: string;
  durationSec: number;
  note: string;
}

export interface TaskPatch {
  name?: string;
  projectId?: string;
}

export interface EntryPatch {
  taskId?: string;
  date?: string;
  durationSec?: number;
  note?: string;
}


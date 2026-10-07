import { invoke } from '@tauri-apps/api/core';
import type { AppError, BootResult, EntryPatch, NewEntry, Snapshot, TaskPatch } from './types';

/** Typed error the UI can show next to a field (`field` set) or as a toast. */
export class ApiError extends Error {
  readonly code: string;
  readonly field: string | null;

  constructor(e: AppError) {
    super(e.message);
    this.name = 'ApiError';
    this.code = e.code;
    this.field = e.field ?? null;
  }
}

function isAppError(raw: unknown): raw is AppError {
  if (typeof raw !== 'object' || raw === null) return false;
  const r = raw as { code?: unknown; message?: unknown };
  return typeof r.code === 'string' && typeof r.message === 'string';
}

export function toApiError(raw: unknown): ApiError {
  if (raw instanceof ApiError) return raw;
  if (isAppError(raw)) return new ApiError(raw);
  const message =
    typeof raw === 'string' ? raw : raw instanceof Error ? raw.message : 'Something went wrong.';
  return new ApiError({ code: 'unexpected', message });
}

export type ErrorTarget =
  | { kind: 'field'; field: string; message: string }
  | { kind: 'toast'; message: string };

/** Where an error message belongs: next to a field, or in a toast. */
export function errorTarget(e: ApiError): ErrorTarget {
  return e.field
    ? { kind: 'field', field: e.field, message: e.message }
    : { kind: 'toast', message: e.message };
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (e) {
    throw toApiError(e);
  }
}

export const api = {
  bootstrap: () => call<BootResult>('bootstrap'),
  retryLoad: () => call<BootResult>('retry_load'),
  startFresh: () => call<Snapshot>('start_fresh'),

  createProject: (name: string) => call<Snapshot>('create_project', { name }),
  renameProject: (id: string, name: string) => call<Snapshot>('rename_project', { id, name }),
  deleteProject: (id: string) => call<Snapshot>('delete_project', { id }),

  createTask: (projectId: string, name: string) =>
    call<Snapshot>('create_task', { projectId, name }),
  updateTask: (id: string, patch: TaskPatch) => call<Snapshot>('update_task', { id, patch }),
  deleteTask: (id: string) => call<Snapshot>('delete_task', { id }),

  startTimer: (taskId: string) => call<Snapshot>('start_timer', { taskId }),
  stopTimer: () => call<Snapshot>('stop_timer'),
  discardTimer: () => call<Snapshot>('discard_timer'),

  addEntry: (entry: NewEntry) => call<Snapshot>('add_entry', { entry }),
  updateEntry: (id: string, patch: EntryPatch) => call<Snapshot>('update_entry', { id, patch }),
  deleteEntry: (id: string) => call<Snapshot>('delete_entry', { id }),
};


import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useReducer,
  useRef,
  type ReactNode,
} from 'react';
import { api, toApiError, type ApiError } from './api';
import { sortedProjects } from './selectors';
import type { BootResult, EntryPatch, NewEntry, Snapshot, TaskPatch } from './types';

export interface Selection {
  projectId: string | null;
  /** null means "All tasks" of the selected project. */
  taskId: string | null;
}

export interface UiState {
  boot: BootResult | null;
  snapshot: Snapshot | null;
  selection: Selection;
  toast: string | null;
  busy: boolean;
  /** Name of the task whose timer was just replaced by a new one. */
  switchedFrom: string | null;
}

export const initialState: UiState = {
  boot: null,
  snapshot: null,
  selection: { projectId: null, taskId: null },
  toast: null,
  busy: false,
  switchedFrom: null,
};

export type Action =
  | { type: 'booted'; result: BootResult }
  | { type: 'snapshot'; snapshot: Snapshot; select?: Selection }
  | { type: 'select'; selection: Selection }
  | { type: 'toast'; message: string | null }
  | { type: 'busy'; busy: boolean };

/**
 * Keeps the selection valid after any snapshot: a vanished project falls back to the
 * first remaining one in displayed order, a vanished (or moved) task to "All tasks".
 */
export function normalizeSelection(snapshot: Snapshot, sel: Selection): Selection {
  const projects = sortedProjects(snapshot.projects);
  const projectId =
    sel.projectId !== null && projects.some((p) => p.id === sel.projectId)
      ? sel.projectId
      : (projects[0]?.id ?? null);
  let taskId = sel.taskId;
  if (taskId !== null) {
    const t = snapshot.tasks.find((x) => x.id === taskId);
    if (!t || t.projectId !== projectId) taskId = null;
  }
  return { projectId, taskId };
}

export function reduce(state: UiState, action: Action): UiState {
  switch (action.type) {
    case 'booted': {
      if (action.result.status === 'ok') {
        const snapshot = action.result.snapshot;
        return {
          ...state,
          boot: action.result,
          snapshot,
          selection: normalizeSelection(snapshot, { projectId: null, taskId: null }),
          switchedFrom: null,
        };
      }
      return { ...state, boot: action.result, snapshot: null, switchedFrom: null };
    }
    case 'snapshot': {
      const prev = state.snapshot;
      const prevTimer = prev?.activeTimer ?? null;
      const nextTimer = action.snapshot.activeTimer;
      let switchedFrom: string | null = null;
      if (prev && prevTimer && nextTimer && prevTimer.taskId !== nextTimer.taskId) {
        switchedFrom = prev.tasks.find((t) => t.id === prevTimer.taskId)?.name ?? null;
      }
      return {
        ...state,
        snapshot: action.snapshot,
        selection: normalizeSelection(action.snapshot, action.select ?? state.selection),
        switchedFrom,
      };
    }
    case 'select':
      return { ...state, selection: action.selection };
    case 'toast':
      return { ...state, toast: action.message };
    case 'busy':
      return { ...state, busy: action.busy };
  }
}

export type MutationResult = { ok: true } | { ok: false; error: ApiError | null };

export interface Actions {
  selectProject(id: string): void;
  selectTask(id: string | null): void;
  createProject(name: string): Promise<MutationResult>;
  renameProject(id: string, name: string): Promise<MutationResult>;
  deleteProject(id: string): Promise<MutationResult>;
  createTask(projectId: string, name: string): Promise<MutationResult>;
  updateTask(id: string, patch: TaskPatch): Promise<MutationResult>;
  deleteTask(id: string): Promise<MutationResult>;
  startTimer(taskId: string): Promise<MutationResult>;
  stopTimer(): Promise<MutationResult>;
  discardTimer(): Promise<MutationResult>;
  addEntry(entry: NewEntry): Promise<MutationResult>;
  updateEntry(id: string, patch: EntryPatch): Promise<MutationResult>;
  deleteEntry(id: string): Promise<MutationResult>;
  dismissToast(): void;
  retryLoad(): Promise<void>;
  startFresh(): Promise<void>;
}

interface TrackerContextValue {
  state: UiState;
  actions: Actions;
}

const TrackerContext = createContext<TrackerContextValue | null>(null);

export function useTracker(): TrackerContextValue {
  const ctx = useContext(TrackerContext);
  if (!ctx) throw new Error('useTracker must be used inside <TrackerProvider>');
  return ctx;
}

interface MutateOptions {
  /** Error codes that need no toast (the state is already what the user wanted). */
  ignore?: string[];
  select?: (prev: Snapshot | null, next: Snapshot) => Selection | undefined;
}

export function TrackerProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(reduce, initialState);
  const busyRef = useRef(false);
  const snapshotRef = useRef<Snapshot | null>(null);
  snapshotRef.current = state.snapshot;

  useEffect(() => {
    let cancelled = false;
    api
      .bootstrap()
      .then((result) => {
        if (!cancelled) dispatch({ type: 'booted', result });
      })
      .catch((e: unknown) => {
        if (!cancelled) {
          dispatch({ type: 'booted', result: { status: 'io_error', message: toApiError(e).message } });
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // One command in flight at a time: the ref blocks a second click in the same tick,
  // `busy` in state disables the buttons.
  const mutate = useCallback(
    async (call: () => Promise<Snapshot>, opts: MutateOptions = {}): Promise<MutationResult> => {
      if (busyRef.current) return { ok: false, error: null };
      busyRef.current = true;
      dispatch({ type: 'busy', busy: true });
      try {
        const prev = snapshotRef.current;
        const next = await call();
        dispatch({ type: 'snapshot', snapshot: next, select: opts.select?.(prev, next) });
        return { ok: true };
      } catch (e) {
        const error = toApiError(e);
        const ignored = opts.ignore?.includes(error.code) ?? false;
        if (!ignored && !error.field) dispatch({ type: 'toast', message: error.message });
        return { ok: false, error };
      } finally {
        busyRef.current = false;
        dispatch({ type: 'busy', busy: false });
      }
    },
    [],
  );

  const actions = useMemo<Actions>(
    () => ({
      selectProject: (id) => dispatch({ type: 'select', selection: { projectId: id, taskId: null } }),
      selectTask: (id) =>
        dispatch({
          type: 'select',
          selection: { projectId: snapshotRef.current ? currentProjectId(snapshotRef.current, id) : null, taskId: id },
        }),
      createProject: (name) =>
        mutate(() => api.createProject(name), {
          select: (prev, next) => {
            const known = new Set<string>(prev?.projects.map((p) => p.id));
            const added = next.projects.find((p) => !known.has(p.id));
            return added ? { projectId: added.id, taskId: null } : undefined;
          },
        }),
      renameProject: (id, name) => mutate(() => api.renameProject(id, name)),
      deleteProject: (id) => mutate(() => api.deleteProject(id)),
      createTask: (projectId, name) => mutate(() => api.createTask(projectId, name)),
      updateTask: (id, patch) => mutate(() => api.updateTask(id, patch)),
      deleteTask: (id) => mutate(() => api.deleteTask(id)),
      startTimer: (taskId) => mutate(() => api.startTimer(taskId)),
      stopTimer: () => mutate(() => api.stopTimer(), { ignore: ['no_timer'] }),
      discardTimer: () => mutate(() => api.discardTimer(), { ignore: ['no_timer'] }),
      addEntry: (entry) => mutate(() => api.addEntry(entry)),
      updateEntry: (id, patch) => mutate(() => api.updateEntry(id, patch)),
      deleteEntry: (id) => mutate(() => api.deleteEntry(id)),
      dismissToast: () => dispatch({ type: 'toast', message: null }),
      retryLoad: async () => {
        try {
          dispatch({ type: 'booted', result: await api.retryLoad() });
        } catch (e) {
          dispatch({ type: 'toast', message: toApiError(e).message });
        }
      },
      startFresh: async () => {
        try {
          const snapshot = await api.startFresh();
          dispatch({ type: 'booted', result: { status: 'ok', snapshot } });
        } catch (e) {
          dispatch({ type: 'toast', message: toApiError(e).message });
        }
      },
    }),
    [mutate],
  );

  const value = useMemo(() => ({ state, actions }), [state, actions]);
  return <TrackerContext.Provider value={value}>{children}</TrackerContext.Provider>;
}

/** Project that owns the task, so selecting a task never leaves a stale project. */
function currentProjectId(snapshot: Snapshot, taskId: string | null): string | null {
  if (taskId === null) return null;
  return snapshot.tasks.find((t) => t.id === taskId)?.projectId ?? null;
}


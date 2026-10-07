import { useMemo, useState } from 'react';
import { ConfirmDialog } from './components/ConfirmDialog';
import { EntryDialog } from './components/EntryDialog';
import { EntryHistory } from './components/EntryHistory';
import { ProjectList } from './components/ProjectList';
import { RecoveryScreen } from './components/RecoveryScreen';
import { TaskList } from './components/TaskList';
import { TimerBar } from './components/TimerBar';
import { count, formatEntryDuration, formatHm } from './format';
import {
  deleteImpact,
  entriesForProject,
  entriesForTask,
  projectTotals,
  sortedProjects,
  sortedTasks,
  taskTotals,
} from './selectors';
import { TrackerProvider, useTracker } from './state';
import type { Snapshot } from './types';

type Dialog =
  | { kind: 'delete'; target: 'project' | 'task'; id: string }
  | { kind: 'delete-entry'; id: string }
  | { kind: 'entry'; entryId: string | null; taskId: string | null };

export default function App() {
  return (
    <TrackerProvider>
      <Shell />
    </TrackerProvider>
  );
}

function Shell() {
  const { state, actions } = useTracker();
  const toast = state.toast !== null && (
    <div role="alert" className="toast">
      <span>{state.toast}</span>
      <button type="button" onClick={actions.dismissToast}>
        Dismiss
      </button>
    </div>
  );

  let body;
  if (!state.boot) {
    body = (
      <p role="status" className="loading">
        Loading\u2026
      </p>
    );
  } else if (state.boot.status !== 'ok') {
    body = <RecoveryScreen result={state.boot} />;
  } else if (state.snapshot) {
    body = <Main snapshot={state.snapshot} />;
  }
  return (
    <>
      {body}
      {toast}
    </>
  );
}

function Main({ snapshot }: { snapshot: Snapshot }) {
  const { state, actions } = useTracker();
  const [dialog, setDialog] = useState<Dialog | null>(null);
  const sel = state.selection;

  const projects = useMemo(() => sortedProjects(snapshot.projects), [snapshot.projects]);
  const taskSecs = useMemo(() => taskTotals(snapshot.entries), [snapshot.entries]);
  const projectSecs = useMemo(
    () => projectTotals(snapshot.tasks, taskSecs),
    [snapshot.tasks, taskSecs],
  );
  const tasks = useMemo(
    () => sortedTasks(snapshot.tasks, sel.projectId),
    [snapshot.tasks, sel.projectId],
  );
  const project = projects.find((p) => p.id === sel.projectId) ?? null;
  const task = tasks.find((t) => t.id === sel.taskId) ?? null;

  const history = useMemo(() => {
    if (task) return entriesForTask(snapshot.entries, task.id);
    if (project) return entriesForProject(snapshot.entries, snapshot.tasks, project.id);
    return [];
  }, [snapshot.entries, snapshot.tasks, task, project]);

  const totalSecs = task ? (taskSecs.get(task.id) ?? 0) : project ? (projectSecs.get(project.id) ?? 0) : 0;
  const resetKey = `${sel.projectId ?? ''}/${sel.taskId ?? ''}`;
  const defaultTaskId = task?.id ?? tasks[0]?.id ?? snapshot.tasks[0]?.id ?? null;

  async function confirmDelete(d: Extract<Dialog, { kind: 'delete' | 'delete-entry' }>) {
    const res =
      d.kind === 'delete-entry'
        ? await actions.deleteEntry(d.id)
        : d.target === 'project'
          ? await actions.deleteProject(d.id)
          : await actions.deleteTask(d.id);
    if (res.ok) setDialog(null);
  }

  let dialogView = null;
  if (dialog?.kind === 'delete') {
    const name =
      dialog.target === 'project'
        ? snapshot.projects.find((p) => p.id === dialog.id)?.name
        : snapshot.tasks.find((t) => t.id === dialog.id)?.name;
    const impact = deleteImpact(snapshot, dialog.target, dialog.id);
    const removed =
      dialog.target === 'project'
        ? `${count(impact.tasks, 'task', 'tasks')} and ${count(impact.entries, 'entry', 'entries')} will be removed.`
        : `${count(impact.entries, 'entry', 'entries')} will be removed.`;
    dialogView = (
      <ConfirmDialog
        title={dialog.target === 'project' ? 'Delete project' : 'Delete task'}
        confirmLabel="Delete"
        onCancel={() => setDialog(null)}
        onConfirm={() => void confirmDelete(dialog)}
      >
        <p>{`Delete '${name ?? ''}'? ${removed}`}</p>
        {impact.discardsTimer && <p>The running timer will be discarded.</p>}
      </ConfirmDialog>
    );
  } else if (dialog?.kind === 'delete-entry') {
    const entry = snapshot.entries.find((e) => e.id === dialog.id);
    const taskName = snapshot.tasks.find((t) => t.id === entry?.taskId)?.name ?? '';
    dialogView = (
      <ConfirmDialog
        title="Delete time entry"
        confirmLabel="Delete"
        onCancel={() => setDialog(null)}
        onConfirm={() => void confirmDelete(dialog)}
      >
        <p>{`Delete this entry? ${formatEntryDuration(entry?.durationSec ?? 0)} on ${entry?.date ?? ''} for '${taskName}' will be removed.`}</p>
      </ConfirmDialog>
    );
  } else if (dialog?.kind === 'entry') {
    const entry = dialog.entryId ? (snapshot.entries.find((e) => e.id === dialog.entryId) ?? null) : null;
    dialogView = (
      <EntryDialog
        entry={entry}
        defaultTaskId={dialog.taskId}
        projects={snapshot.projects}
        tasks={snapshot.tasks}
        onClose={() => setDialog(null)}
      />
    );
  }

  return (
    <div className="app">
      <TimerBar snapshot={snapshot} />
      <main className="panes">
        <ProjectList
          projects={projects}
          projectSecs={projectSecs}
          onDelete={(id) => setDialog({ kind: 'delete', target: 'project', id })}
        />
        <TaskList
          project={project}
          projects={projects}
          tasks={tasks}
          taskSecs={taskSecs}
          projectSecs={project ? (projectSecs.get(project.id) ?? 0) : 0}
          activeTaskId={snapshot.activeTimer?.taskId ?? null}
          onDelete={(id) => setDialog({ kind: 'delete', target: 'task', id })}
          onAddTime={(taskId) => setDialog({ kind: 'entry', entryId: null, taskId })}
        />
        <section className="pane history" aria-label="History">
          <div className="history-head">
            <div>
              <h2>History</h2>
              {project && (
                <p className="subtitle">
                  {task ? `${task.name} in ${project.name}` : `All tasks in ${project.name}`}
                  {` \u2014 total ${formatHm(totalSecs)}`}
                </p>
              )}
            </div>
            <button
              type="button"
              disabled={snapshot.tasks.length === 0}
              title={snapshot.tasks.length === 0 ? 'Create a task first' : undefined}
              onClick={() => setDialog({ kind: 'entry', entryId: null, taskId: defaultTaskId })}
            >
              Add time
            </button>
          </div>
          <EntryHistory
            entries={history}
            tasks={snapshot.tasks}
            resetKey={resetKey}
            onEdit={(id) => setDialog({ kind: 'entry', entryId: id, taskId: null })}
            onDelete={(id) => setDialog({ kind: 'delete-entry', id })}
          />
        </section>
      </main>
      {dialogView}
    </div>
  );
}


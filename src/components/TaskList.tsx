import { useRef, useState } from 'react';
import { formatHm } from '../format';
import { useTracker } from '../state';
import type { Project, Task } from '../types';
import { AddNameForm } from './AddNameForm';
import { InlineName } from './InlineName';

interface Props {
  project: Project | null;
  projects: Project[];
  tasks: Task[];
  taskSecs: Map<string, number>;
  projectSecs: number;
  activeTaskId: string | null;
  onDelete: (id: string) => void;
  onAddTime: (taskId: string) => void;
}

export function TaskList({
  project,
  projects,
  tasks,
  taskSecs,
  projectSecs,
  activeTaskId,
  onDelete,
  onAddTime,
}: Props) {
  const { state, actions } = useTracker();
  const [editingId, setEditingId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  if (!project) {
    return (
      <section className="pane" aria-label="Tasks">
        <h2>Tasks</h2>
        <AddNameForm
          label="New task name"
          buttonLabel="Add task"
          disabled
          onAdd={() => Promise.resolve({ ok: false, error: null })}
        />
        <p className="empty">Create a project first, then add tasks to it.</p>
      </section>
    );
  }

  const allSelected = state.selection.taskId === null;
  return (
    <section className="pane" aria-label="Tasks">
      <h2>{`Tasks in ${project.name}`}</h2>
      <AddNameForm
        label="New task name"
        buttonLabel="Add task"
        inputRef={inputRef}
        onAdd={(name) => actions.createTask(project.id, name)}
      />
      <ul className="rows">
        <li className={allSelected ? 'row selected' : 'row'}>
          <button
            type="button"
            className="row-main"
            aria-current={allSelected ? 'true' : undefined}
            onClick={() => actions.selectTask(null)}
          >
            <span className="name">All tasks</span>
            <span className="total">{formatHm(projectSecs)}</span>
          </button>
        </li>
        {tasks.map((t) => {
          const selected = state.selection.taskId === t.id;
          const running = activeTaskId === t.id;
          return (
            <li key={t.id} className={selected ? 'row selected' : 'row'}>
              {editingId === t.id ? (
                <InlineName
                  initial={t.name}
                  label={`Rename task ${t.name}`}
                  onSave={(name) => actions.updateTask(t.id, { name })}
                  onDone={() => setEditingId(null)}
                />
              ) : (
                <button
                  type="button"
                  className="row-main"
                  aria-current={selected ? 'true' : undefined}
                  onClick={() => actions.selectTask(t.id)}
                  onDoubleClick={() => setEditingId(t.id)}
                  onKeyDown={(e) => {
                    if (e.key === 'F2') setEditingId(t.id);
                  }}
                >
                  <span className="name">{t.name}</span>
                  {running && <span className="badge">{'\u25B6 Running'}</span>}
                  <span className="total">{formatHm(taskSecs.get(t.id) ?? 0)}</span>
                </button>
              )}
              <div className="row-actions">
                <button
                  type="button"
                  className={running ? 'primary-stop' : 'primary'}
                  disabled={state.busy}
                  aria-label={running ? `Stop timer on ${t.name}` : `Start timer on ${t.name}`}
                  onClick={() => void (running ? actions.stopTimer() : actions.startTimer(t.id))}
                >
                  {running ? 'Stop' : 'Start'}
                </button>
                <button
                  type="button"
                  aria-label={`Add time to ${t.name}`}
                  onClick={() => onAddTime(t.id)}
                >
                  Add time
                </button>
                <button
                  type="button"
                  aria-label={`Rename task ${t.name}`}
                  onClick={() => setEditingId(t.id)}
                >
                  Rename
                </button>
                <button
                  type="button"
                  aria-label={`Delete task ${t.name}`}
                  onClick={() => onDelete(t.id)}
                >
                  Delete
                </button>
                <select
                  aria-label={`Move ${t.name} to another project`}
                  value={t.projectId}
                  onChange={(e) => void actions.updateTask(t.id, { projectId: e.target.value })}
                >
                  {projects.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name}
                    </option>
                  ))}
                </select>
              </div>
            </li>
          );
        })}
      </ul>
      {tasks.length === 0 && (
        <div className="empty">
          <p>This project has no tasks yet.</p>
          <button type="button" onClick={() => inputRef.current?.focus()}>
            Create task
          </button>
        </div>
      )}
    </section>
  );
}


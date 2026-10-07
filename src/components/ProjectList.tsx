import { useRef, useState } from 'react';
import { formatHm } from '../format';
import { useTracker } from '../state';
import type { Project } from '../types';
import { AddNameForm } from './AddNameForm';
import { InlineName } from './InlineName';

interface Props {
  projects: Project[];
  projectSecs: Map<string, number>;
  onDelete: (id: string) => void;
}

export function ProjectList({ projects, projectSecs, onDelete }: Props) {
  const { state, actions } = useTracker();
  const [editingId, setEditingId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  return (
    <section className="pane" aria-label="Projects">
      <h2>Projects</h2>
      <AddNameForm
        label="New project name"
        buttonLabel="Add project"
        inputRef={inputRef}
        onAdd={actions.createProject}
      />
      {projects.length === 0 ? (
        <div className="empty">
          <p>No projects yet. Create your first project to start tracking time.</p>
          <button type="button" onClick={() => inputRef.current?.focus()}>
            Create project
          </button>
        </div>
      ) : (
        <ul className="rows">
          {projects.map((p) => {
            const selected = state.selection.projectId === p.id;
            return (
              <li key={p.id} className={selected ? 'row selected' : 'row'}>
                {editingId === p.id ? (
                  <InlineName
                    initial={p.name}
                    label={`Rename project ${p.name}`}
                    onSave={(name) => actions.renameProject(p.id, name)}
                    onDone={() => setEditingId(null)}
                  />
                ) : (
                  <button
                    type="button"
                    className="row-main"
                    aria-current={selected ? 'true' : undefined}
                    onClick={() => actions.selectProject(p.id)}
                    onDoubleClick={() => setEditingId(p.id)}
                    onKeyDown={(e) => {
                      if (e.key === 'F2') setEditingId(p.id);
                    }}
                  >
                    <span className="name">{p.name}</span>
                    <span className="total">{formatHm(projectSecs.get(p.id) ?? 0)}</span>
                  </button>
                )}
                <div className="row-actions">
                  <button
                    type="button"
                    aria-label={`Rename project ${p.name}`}
                    onClick={() => setEditingId(p.id)}
                  >
                    Rename
                  </button>
                  <button
                    type="button"
                    aria-label={`Delete project ${p.name}`}
                    onClick={() => onDelete(p.id)}
                  >
                    Delete
                  </button>
                </div>
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}


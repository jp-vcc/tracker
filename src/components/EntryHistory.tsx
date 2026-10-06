import { useMemo, useState } from 'react';
import { formatEntryDuration } from '../format';
import type { Entry, Task } from '../types';

export const PAGE_SIZE = 200;

interface Props {
  /** Already sorted newest first. */
  entries: Entry[];
  tasks: Task[];
  /** Changing this resets the visible window to the first page. */
  resetKey: string;
  onEdit: (id: string) => void;
  onDelete: (id: string) => void;
}

/** History table. Renders at most `PAGE_SIZE` rows at a time, never the whole list. */
export function EntryHistory({ entries, tasks, resetKey, onEdit, onDelete }: Props) {
  const [win, setWin] = useState({ key: resetKey, limit: PAGE_SIZE });
  const limit = win.key === resetKey ? win.limit : PAGE_SIZE;
  const names = useMemo(() => new Map(tasks.map((t) => [t.id, t.name])), [tasks]);

  if (entries.length === 0) {
    return (
      <p className="empty">
        No time entries yet. Start a timer on a task, or choose Add time to log time you forgot to
        record.
      </p>
    );
  }

  const shown = Math.min(limit, entries.length);
  const visible = entries.slice(0, shown);
  return (
    <div className="history-table-wrap">
      <table className="history-table">
        <thead>
          <tr>
            <th scope="col">Date</th>
            <th scope="col">Task</th>
            <th scope="col">Duration</th>
            <th scope="col">Note</th>
            <th scope="col">Actions</th>
          </tr>
        </thead>
        <tbody>
          {visible.map((e) => {
            const taskName = names.get(e.taskId) ?? '';
            return (
              <tr key={e.id}>
                <td>{e.date}</td>
                <td>{taskName}</td>
                <td>
                  {formatEntryDuration(e.durationSec)}
                  {e.source === 'timer' ? ' (timer)' : ''}
                </td>
                <td className="note">{e.note}</td>
                <td className="row-actions">
                  <button
                    type="button"
                    aria-label={`Edit entry on ${e.date} for ${taskName}`}
                    onClick={() => onEdit(e.id)}
                  >
                    Edit
                  </button>
                  <button
                    type="button"
                    aria-label={`Delete entry on ${e.date} for ${taskName}`}
                    onClick={() => onDelete(e.id)}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {entries.length > shown && (
        <div className="history-more">
          <span>{`Showing ${shown.toLocaleString('en-US')} of ${entries.length.toLocaleString('en-US')}`}</span>
          <button type="button" onClick={() => setWin({ key: resetKey, limit: shown + PAGE_SIZE })}>
            Show more
          </button>
        </div>
      )}
    </div>
  );
}


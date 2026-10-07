import { useId, useMemo, useState, type FormEvent } from 'react';
import { hmToSeconds, splitHm, todayLocal } from '../format';
import { useDialog } from '../hooks/useDialog';
import { byName } from '../selectors';
import { useTracker } from '../state';
import type { Entry, Project, Task } from '../types';

const MAX_DURATION_SEC = 86_400;
const MAX_NOTE_CHARS = 500;
const DATE_RE = /^\d{4}-\d{2}-\d{2}$/;

interface Props {
  /** The entry being edited, or null to add a new one. */
  entry: Entry | null;
  defaultTaskId: string | null;
  projects: Project[];
  tasks: Task[];
  onClose: () => void;
}

type Errors = Partial<Record<'taskId' | 'duration' | 'date' | 'note', string>>;

/** Add or edit a time entry. Client checks give fast feedback; the backend stays authoritative. */
export function EntryDialog({ entry, defaultTaskId, projects, tasks, onClose }: Props) {
  const { state, actions } = useTracker();
  const { ref, onKeyDown } = useDialog<HTMLDivElement>(onClose);
  const titleId = useId();
  // Recomputed on every open (each open mounts a new dialog), so it cannot go stale across midnight.
  const [maxDate] = useState(() => todayLocal());

  const initial = splitHm(entry ? entry.durationSec : 0);
  const [taskId, setTaskId] = useState(entry?.taskId ?? defaultTaskId ?? tasks[0]?.id ?? '');
  const [hours, setHours] = useState(entry ? String(initial.hours) : '');
  const [minutes, setMinutes] = useState(entry ? String(initial.minutes) : '');
  const [durationTouched, setDurationTouched] = useState(false);
  const [date, setDate] = useState(entry?.date ?? maxDate);
  const [note, setNote] = useState(entry?.note ?? '');
  const [errors, setErrors] = useState<Errors>({});

  const options = useMemo(() => {
    const names = new Map(projects.map((p) => [p.id, p.name]));
    return tasks
      .map((t) => ({ id: t.id, project: names.get(t.projectId) ?? '', name: t.name }))
      .sort((a, b) => byName({ name: a.project }, { name: b.project }) || byName(a, b));
  }, [projects, tasks]);

  const noteLength = Array.from(note).length;

  async function submit(ev: FormEvent) {
    ev.preventDefault();
    const errs: Errors = {};
    if (!taskId) errs.taskId = 'Choose a task';

    let durationSec = entry ? entry.durationSec : 0;
    if (!entry || durationTouched) {
      const secs = hmToSeconds(hours, minutes);
      if (secs === null) errs.duration = 'Enter whole numbers for hours and minutes';
      else if (secs <= 0) errs.duration = 'Duration must be greater than zero';
      else if (secs > MAX_DURATION_SEC) errs.duration = 'Duration cannot be more than 24 hours';
      else durationSec = secs;
    }

    if (!DATE_RE.test(date)) errs.date = 'Enter a valid date';
    else if ((!entry || date !== entry.date) && date > maxDate) {
      errs.date = 'Date cannot be in the future';
    }

    if (Array.from(note.trim()).length > MAX_NOTE_CHARS) {
      errs.note = 'Note cannot be longer than 500 characters';
    }

    setErrors(errs);
    if (Object.keys(errs).length > 0) return;

    const body = { taskId, date, durationSec, note: note.trim() };
    const res = entry ? await actions.updateEntry(entry.id, body) : await actions.addEntry(body);
    if (res.ok) {
      onClose();
    } else if (res.error?.field) {
      setErrors({ [res.error.field]: res.error.message });
    }
  }

  return (
    <div className="overlay">
      <div
        ref={ref}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="dialog"
        onKeyDown={onKeyDown}
      >
        <h2 id={titleId}>{entry ? 'Edit time entry' : 'Add time'}</h2>
        <form onSubmit={(e) => void submit(e)} noValidate>
          <div className="field">
            <label htmlFor="entry-task">Task</label>
            <select
              id="entry-task"
              value={taskId}
              autoFocus
              onChange={(e) => setTaskId(e.target.value)}
            >
              {options.map((o) => (
                <option key={o.id} value={o.id}>
                  {`${o.project} \u203A ${o.name}`}
                </option>
              ))}
            </select>
            {errors.taskId && <p role="alert" className="field-error">{errors.taskId}</p>}
          </div>

          <div className="field">
            <span id="entry-duration-label">Duration</span>
            <div className="duration-row" role="group" aria-labelledby="entry-duration-label">
              <input
                type="number"
                min="0"
                inputMode="numeric"
                aria-label="Hours"
                value={hours}
                onChange={(e) => {
                  setHours(e.target.value);
                  setDurationTouched(true);
                }}
              />
              <span aria-hidden="true">h</span>
              <input
                type="number"
                min="0"
                inputMode="numeric"
                aria-label="Minutes"
                value={minutes}
                onChange={(e) => {
                  setMinutes(e.target.value);
                  setDurationTouched(true);
                }}
              />
              <span aria-hidden="true">m</span>
            </div>
            {errors.duration && <p role="alert" className="field-error">{errors.duration}</p>}
          </div>

          <div className="field">
            <label htmlFor="entry-date">Date</label>
            <input
              id="entry-date"
              type="date"
              max={maxDate}
              value={date}
              onChange={(e) => setDate(e.target.value)}
            />
            {errors.date && <p role="alert" className="field-error">{errors.date}</p>}
          </div>

          <div className="field">
            <label htmlFor="entry-note">Note (optional)</label>
            <textarea
              id="entry-note"
              rows={3}
              value={note}
              onChange={(e) => setNote(e.target.value)}
            />
            <span className="counter">{`${noteLength}/${MAX_NOTE_CHARS}`}</span>
            {errors.note && <p role="alert" className="field-error">{errors.note}</p>}
          </div>

          <div className="dialog-actions">
            <button type="button" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="primary" disabled={state.busy}>
              {entry ? 'Save changes' : 'Add time'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}


import { useState } from 'react';
import { formatClock, formatHm } from '../format';
import { useNow } from '../hooks/useNow';
import { elapsedSec } from '../selectors';
import { useTracker } from '../state';
import type { Snapshot } from '../types';
import { ConfirmDialog } from './ConfirmDialog';

/** Always-visible header: the running timer with Stop and Discard, or an idle hint. */
export function TimerBar({ snapshot }: { snapshot: Snapshot }) {
  const { state, actions } = useTracker();
  const timer = snapshot.activeTimer;
  const now = useNow(timer !== null);
  const [confirming, setConfirming] = useState(false);

  const task = timer ? snapshot.tasks.find((t) => t.id === timer.taskId) : undefined;
  if (!timer || !task) {
    return (
      <header className="timer-bar idle">
        <span>Timer idle. Press Start on a task to begin.</span>
      </header>
    );
  }
  const project = snapshot.projects.find((p) => p.id === task.projectId);
  const elapsed = elapsedSec(timer, now);

  return (
    <header className="timer-bar active">
      <span className="badge">{'\u25B6 Running'}</span>
      <span className="timer-task">
        <strong>{task.name}</strong>
        {project ? ` in ${project.name}` : ''}
      </span>
      <span className="elapsed" aria-label="Elapsed time">
        {formatClock(elapsed)}
      </span>
      {state.switchedFrom !== null && (
        <span className="switched">{`Switched from ${state.switchedFrom}`}</span>
      )}
      <span className="timer-actions">
        <button type="button" disabled={state.busy} onClick={() => void actions.stopTimer()}>
          Stop
        </button>
        <button type="button" disabled={state.busy} onClick={() => setConfirming(true)}>
          Discard
        </button>
      </span>
      {confirming && (
        <ConfirmDialog
          title="Discard timer"
          confirmLabel="Discard timer"
          onCancel={() => setConfirming(false)}
          onConfirm={() => {
            void actions.discardTimer().then((res) => {
              if (res.ok || res.error) setConfirming(false);
            });
          }}
        >
          <p>{`Discard the timer on '${task.name}'? ${formatHm(elapsed)} will not be saved.`}</p>
        </ConfirmDialog>
      )}
    </header>
  );
}


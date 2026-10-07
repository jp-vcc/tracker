import { useId, type ReactNode } from 'react';
import { useDialog } from '../hooks/useDialog';

interface Props {
  title: string;
  children: ReactNode;
  confirmLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
}

/** Destructive-action dialog: Cancel has focus, Escape cancels, Enter presses the focused button. */
export function ConfirmDialog({ title, children, confirmLabel, onConfirm, onCancel }: Props) {
  const titleId = useId();
  const { ref, onKeyDown } = useDialog<HTMLDivElement>(onCancel);
  return (
    <div className="overlay">
      <div
        ref={ref}
        role="alertdialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="dialog"
        onKeyDown={onKeyDown}
      >
        <h2 id={titleId}>{title}</h2>
        <div className="dialog-body">{children}</div>
        <div className="dialog-actions">
          <button type="button" autoFocus onClick={onCancel}>
            Cancel
          </button>
          <button type="button" className="danger" onClick={onConfirm}>
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}


import { useId, useState } from 'react';
import type { MutationResult } from '../state';

interface Props {
  initial: string;
  label: string;
  onSave: (name: string) => Promise<MutationResult>;
  onDone: () => void;
}

/** In-place rename input. Enter commits, Escape or leaving the field cancels. */
export function InlineName({ initial, label, onSave, onDone }: Props) {
  const [value, setValue] = useState(initial);
  const [error, setError] = useState<string | null>(null);
  const errorId = useId();

  async function commit() {
    const name = value.trim();
    if (name === initial) {
      onDone();
      return;
    }
    const res = await onSave(name);
    if (res.ok) onDone();
    else if (res.error?.field) setError(res.error.message);
  }

  return (
    <span className="inline-name">
      <input
        autoFocus
        aria-label={label}
        value={value}
        aria-invalid={error !== null}
        aria-describedby={error !== null ? errorId : undefined}
        onFocus={(e) => e.currentTarget.select()}
        onChange={(e) => {
          setValue(e.target.value);
          setError(null);
        }}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            void commit();
          } else if (e.key === 'Escape') {
            e.preventDefault();
            e.stopPropagation();
            onDone();
          }
        }}
        onBlur={onDone}
      />
      {error !== null && (
        <span id={errorId} role="alert" className="field-error">
          {error}
        </span>
      )}
    </span>
  );
}


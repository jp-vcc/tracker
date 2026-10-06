import { useId, useState, type FormEvent, type RefObject } from 'react';
import type { MutationResult } from '../state';

interface Props {
  label: string;
  buttonLabel: string;
  disabled?: boolean;
  inputRef?: RefObject<HTMLInputElement>;
  onAdd: (name: string) => Promise<MutationResult>;
}

/** Name input plus submit button for creating a project or a task. */
export function AddNameForm({ label, buttonLabel, disabled = false, inputRef, onAdd }: Props) {
  const [name, setName] = useState('');
  const [error, setError] = useState<string | null>(null);
  const errorId = useId();

  async function submit(e: FormEvent) {
    e.preventDefault();
    const res = await onAdd(name);
    if (res.ok) {
      setName('');
      setError(null);
    } else if (res.error?.field) {
      setError(res.error.message);
    }
  }

  return (
    <form className="add-form" onSubmit={(e) => void submit(e)}>
      <div className="add-row">
        <input
          ref={inputRef}
          aria-label={label}
          placeholder={label}
          value={name}
          disabled={disabled}
          aria-invalid={error !== null}
          aria-describedby={error !== null ? errorId : undefined}
          onChange={(e) => {
            setName(e.target.value);
            setError(null);
          }}
        />
        <button type="submit" disabled={disabled}>
          {buttonLabel}
        </button>
      </div>
      {error !== null && (
        <p id={errorId} role="alert" className="field-error">
          {error}
        </p>
      )}
    </form>
  );
}


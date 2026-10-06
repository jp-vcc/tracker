import { useEffect, useRef, useState, type KeyboardEvent as ReactKeyboardEvent } from 'react';

const FOCUSABLE = 'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled])';

/**
 * Shared modal behaviour: Escape cancels, Tab stays inside the dialog and focus
 * returns to the control that opened it when the dialog closes.
 */
export function useDialog<T extends HTMLElement>(onCancel: () => void) {
  const ref = useRef<T>(null);
  const [opener] = useState(() => document.activeElement as HTMLElement | null);

  useEffect(
    () => () => {
      if (opener && document.contains(opener) && typeof opener.focus === 'function') opener.focus();
    },
    [opener],
  );

  const onKeyDown = (e: ReactKeyboardEvent) => {
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      onCancel();
      return;
    }
    if (e.key !== 'Tab' || !ref.current) return;
    const items = Array.from(ref.current.querySelectorAll<HTMLElement>(FOCUSABLE));
    if (items.length === 0) return;
    const first = items[0];
    const last = items[items.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  };

  return { ref, onKeyDown };
}


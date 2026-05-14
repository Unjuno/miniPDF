import { useEffect, useRef } from 'react';

export const useKeyboardShortcuts = (
  shortcuts: Record<string, () => void>
) => {
  const shortcutsRef = useRef(shortcuts);
  const handlersRef = useRef<Map<string, () => void>>(new Map(Object.entries(shortcuts)));

  useEffect(() => {
    const keysChanged = Object.keys(shortcuts).some(
      key => shortcutsRef.current[key] !== shortcuts[key]
    ) || Object.keys(shortcutsRef.current).some(
      key => !(key in shortcuts)
    );

    if (keysChanged) {
      shortcutsRef.current = shortcuts;
      handlersRef.current = new Map(Object.entries(shortcuts));
    }
  }, [shortcuts]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey) {
        const key = e.key.toLowerCase();
        const shortcutKey = `Ctrl+${key}`;
        
        const handler = handlersRef.current.get(shortcutKey);
        if (handler) {
          e.preventDefault();
          handler();
          return;
        }

        if (key === '+' || (key === '=' && e.shiftKey)) {
          const plusKey = 'Ctrl++';
          const plusHandler = handlersRef.current.get(plusKey);
          if (plusHandler) {
            e.preventDefault();
            plusHandler();
            return;
          }
        }

        if (key === '-') {
          const minusKey = 'Ctrl+-';
          const minusHandler = handlersRef.current.get(minusKey);
          if (minusHandler) {
            e.preventDefault();
            minusHandler();
            return;
          }
        }

        if (key === '0') {
          const zeroKey = 'Ctrl+0';
          const zeroHandler = handlersRef.current.get(zeroKey);
          if (zeroHandler) {
            e.preventDefault();
            zeroHandler();
            return;
          }
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);
};

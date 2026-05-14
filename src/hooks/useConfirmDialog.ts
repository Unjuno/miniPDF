import { useState, useCallback, useRef } from 'react';

interface ConfirmDialogState {
  isOpen: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
}

export const useConfirmDialog = () => {
  const [dialogState, setDialogState] = useState<ConfirmDialogState>({
    isOpen: false,
    title: '',
    message: '',
    confirmText: 'OK',
    cancelText: 'キャンセル',
  });
  const resolveRef = useRef<((value: boolean) => void) | null>(null);

  const showConfirm = useCallback((
    message: string,
    title: string = '確認',
    confirmText?: string,
    cancelText?: string
  ): Promise<boolean> => {
    return new Promise((resolve) => {
      resolveRef.current = resolve;
      setDialogState({
        isOpen: true,
        title,
        message,
        confirmText,
        cancelText,
      });
    });
  }, []);

  const handleCancel = useCallback(() => {
    if (resolveRef.current) {
      resolveRef.current(false);
      resolveRef.current = null;
    }
    setDialogState(prev => ({ ...prev, isOpen: false }));
  }, []);

  const handleConfirm = useCallback(() => {
    if (resolveRef.current) {
      resolveRef.current(true);
      resolveRef.current = null;
    }
    setDialogState(prev => ({ ...prev, isOpen: false }));
  }, []);

  return {
    dialogState,
    showConfirm,
    handleCancel,
    handleConfirm,
  };
};


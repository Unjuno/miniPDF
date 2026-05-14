import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ErrorDisplay } from './ErrorDisplay';

describe('ErrorDisplay', () => {
  it('エラーがない場合は何も表示しない', () => {
    const { container } = render(
      <ErrorDisplay error={null} onDismiss={vi.fn()} />
    );
    expect(container.firstChild).toBeNull();
  });

  it('エラーメッセージを表示する', () => {
    render(<ErrorDisplay error="テストエラー" onDismiss={vi.fn()} />);
    const errorText = screen.getByText('テストエラー');
    expect(errorText).toBeTruthy();
  });

  it('エラーを閉じるボタンが表示される', () => {
    const onDismiss = vi.fn();
    render(<ErrorDisplay error="テストエラー" onDismiss={onDismiss} />);
    const closeButton = screen.getByRole('button');
    expect(closeButton).toBeTruthy();
  });

  it('閉じるボタンをクリックするとonDismissが呼ばれる', () => {
    const onDismiss = vi.fn();
    render(<ErrorDisplay error="テストエラー" onDismiss={onDismiss} />);
    const closeButton = screen.getByRole('button');
    closeButton.click();
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });
});


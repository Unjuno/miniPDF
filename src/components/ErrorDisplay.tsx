import React from 'react';
import './ErrorDisplay.css';

interface ErrorDisplayProps {
  error: string | null;
  onDismiss: () => void;
}

export const ErrorDisplay: React.FC<ErrorDisplayProps> = ({
  error,
  onDismiss,
}) => {
  if (!error) return null;

  // why: エラーメッセージを改行で分割して表示することで、複数のエラーを明確に区別
  // alt: そのまま表示（改行が無視される）
  // evidence: 改行で分割することで、各エラーを明確に区別できる
  const errorLines = error.split('\n');

  return (
    <div className="error-display">
      <div className="error-display-content">
        <div className="error-display-message">
          {errorLines.map((line, index) => (
            <p key={index}>{line}</p>
          ))}
        </div>
        <button className="error-display-button" onClick={onDismiss}>
          閉じる
        </button>
      </div>
    </div>
  );
};

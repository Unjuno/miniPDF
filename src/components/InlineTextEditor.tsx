import React, { useState, useEffect, useRef, useCallback } from 'react';
import { TextBlock } from '../types/pdf';
import { usePdfStore } from '../stores/pdfStore';
import { logger } from '../utils/logger';
import { useToast } from '../hooks/useToast';
import './InlineTextEditor.css';

interface InlineTextEditorProps {
  textBlock: TextBlock;
  scaleX: number;
  scaleY: number;
  pageHeight: number;
  onSave: () => void;
  onCancel: () => void;
}

export const InlineTextEditor: React.FC<InlineTextEditorProps> = ({
  textBlock,
  scaleX,
  scaleY,
  pageHeight,
  onSave,
  onCancel,
}) => {
  const { editTextBlock } = usePdfStore();
  const { error: showError } = useToast();
  const [text, setText] = useState(textBlock.text);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    setText(textBlock.text);
  }, [textBlock]);

  useEffect(() => {
    // why: 編集開始時にテキストエリアを自動フォーカスして即座に編集可能にする
    // alt: 手動でクリックが必要（操作が煩雑）
    // evidence: 自動フォーカスによりWordライクな即座の編集体験を提供
    if (textareaRef.current) {
      textareaRef.current.focus();
      textareaRef.current.select();
    }
  }, []);

  const screenY = pageHeight - textBlock.y - textBlock.height;

  // why: scaleX/scaleYが0または無効な値の場合をチェックしてNaNを防ぐ
  // alt: チェックなし（scaleX/scaleYが0の場合にNaNが発生する）
  // evidence: ゼロ除算によりNaNが発生し、レンダリング時にエラーになる
  const safeScaleX = scaleX === 0 || !Number.isFinite(scaleX) ? 1 : scaleX;
  const safeScaleY = scaleY === 0 || !Number.isFinite(scaleY) ? 1 : scaleY;

  const handleSave = useCallback(async () => {
    try {
      await editTextBlock(textBlock.id, text);
      onSave();
    } catch (error) {
      logger.error('Failed to edit text', error instanceof Error ? error : new Error(String(error)));
      showError('テキストの編集に失敗しました');
    }
  }, [text, textBlock.id, editTextBlock, onSave, showError]);

  const handleCancel = useCallback(() => {
    setText(textBlock.text);
    onCancel();
  }, [textBlock.text, onCancel]);

  const insertLineBreak = useCallback(() => {
    if (textareaRef.current) {
      const textarea = textareaRef.current;
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const newText = text.substring(0, start) + '\n' + text.substring(end);
      setText(newText);
      // カーソル位置を改行の後に設定
      setTimeout(() => {
        textarea.focus();
        textarea.setSelectionRange(start + 1, start + 1);
      }, 0);
    }
  }, [text]);

  const removeLineBreak = useCallback(() => {
    if (textareaRef.current) {
      const textarea = textareaRef.current;
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      
      // 選択範囲内の改行を削除
      let newText = text;
      if (start === end) {
        // カーソル位置の前後の改行を削除
        if (start > 0 && text[start - 1] === '\n') {
          newText = text.substring(0, start - 1) + text.substring(start);
          setTimeout(() => {
            textarea.focus();
            textarea.setSelectionRange(start - 1, start - 1);
          }, 0);
        } else if (start < text.length && text[start] === '\n') {
          newText = text.substring(0, start) + text.substring(start + 1);
          setTimeout(() => {
            textarea.focus();
            textarea.setSelectionRange(start, start);
          }, 0);
        }
      } else {
        // 選択範囲内の改行を削除
        const selectedText = text.substring(start, end);
        const textWithoutBreaks = selectedText.replace(/\n/g, ' ');
        newText = text.substring(0, start) + textWithoutBreaks + text.substring(end);
        setTimeout(() => {
          textarea.focus();
          textarea.setSelectionRange(start, start + textWithoutBreaks.length);
        }, 0);
      }
      setText(newText);
    }
  }, [text]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    // why: Enterキーで保存、Escapeキーでキャンセル（Wordライクな操作）
    // alt: ボタンクリックのみ（操作が煩雑）
    // evidence: キーボードショートカットにより編集効率が向上
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      handleSave();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      handleCancel();
    }
  }, [handleSave, handleCancel]);

  return (
    <div
      className="inline-text-editor"
      style={{
        position: 'absolute',
        left: `${textBlock.x * safeScaleX}px`,
        top: `${screenY * safeScaleY}px`,
        width: `${textBlock.width * safeScaleX}px`,
        minHeight: `${textBlock.height * safeScaleY}px`,
        zIndex: 1000,
      }}
    >
      <div className="inline-text-editor-toolbar">
        <button 
          type="button"
          onClick={insertLineBreak}
          className="inline-text-editor-toolbar-btn"
          title="改行を挿入"
        >
          ⏎
        </button>
        <button 
          type="button"
          onClick={removeLineBreak}
          className="inline-text-editor-toolbar-btn"
          title="改行を削除"
        >
          ⌫
        </button>
      </div>
      <textarea
        ref={textareaRef}
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={handleKeyDown}
        onBlur={handleSave}
        className="inline-text-editor-textarea"
        style={{
          width: '100%',
          minHeight: `${textBlock.height * safeScaleY}px`,
          fontSize: `${textBlock.fontSize * safeScaleY}px`,
          fontFamily: textBlock.fontFamily,
          lineHeight: textBlock.lineHeight,
          padding: '2px',
          border: '2px solid #007bff',
          borderRadius: '2px',
          backgroundColor: 'rgba(255, 255, 255, 0.95)',
          resize: 'both',
          overflow: 'auto',
        }}
        aria-label="テキストを編集"
      />
      <div className="inline-text-editor-hint">
        Ctrl+Enterで保存、Escapeでキャンセル
      </div>
    </div>
  );
};


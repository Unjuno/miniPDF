import React, { useState, useEffect, useRef } from 'react';
import { TextBlock } from '../types/pdf';
import { usePdfStore } from '../stores/pdfStore';
import { logger } from '../utils/logger';
import { useToast } from '../hooks/useToast';
import './TextEditor.css';

interface TextEditorProps {
  textBlock: TextBlock;
  onClose: () => void;
}

export const TextEditor: React.FC<TextEditorProps> = ({ textBlock, onClose }) => {
  const { editTextBlock } = usePdfStore();
  const { error: showError } = useToast();
  const [text, setText] = useState(textBlock.text);
  const [isEditing, setIsEditing] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    setText(textBlock.text);
  }, [textBlock]);

  const handleSave = async () => {
    try {
      await editTextBlock(textBlock.id, text);
      setIsEditing(false);
    } catch (error) {
      logger.error('Failed to edit text', error instanceof Error ? error : new Error(String(error)));
      showError('テキストの編集に失敗しました');
    }
  };

  const handleCancel = () => {
    setText(textBlock.text);
    setIsEditing(false);
  };

  const insertLineBreak = () => {
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
  };

  const removeLineBreak = () => {
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
  };

  if (!isEditing) {
    return (
      <div className="text-editor-view">
        <div className="text-editor-content" onClick={() => setIsEditing(true)}>
          {text || '(空のテキストブロック)'}
        </div>
        <div className="text-editor-actions">
          <button onClick={() => setIsEditing(true)}>編集</button>
          <button onClick={onClose}>閉じる</button>
        </div>
      </div>
    );
  }

  return (
    <div className="text-editor-edit">
      <label htmlFor="text-editor-textarea" className="text-editor-label">テキストを編集</label>
      <div className="text-editor-toolbar">
        <button 
          type="button"
          onClick={insertLineBreak}
          className="text-editor-toolbar-btn"
          title="改行を挿入 (カーソル位置)"
        >
          ⏎ 改行を挿入
        </button>
        <button 
          type="button"
          onClick={removeLineBreak}
          className="text-editor-toolbar-btn"
          title="改行を削除 (カーソル位置または選択範囲)"
        >
          ⌫ 改行を削除
        </button>
      </div>
      <textarea
        ref={textareaRef}
        id="text-editor-textarea"
        value={text}
        onChange={(e) => setText(e.target.value)}
        className="text-editor-textarea"
        rows={10}
        autoFocus
        aria-label="テキストを編集"
        placeholder="テキストを入力してください。改行はEnterキーまたは「改行を挿入」ボタンで追加できます。"
      />
      <div className="text-editor-hint">
        改行数: {text.split('\n').length - 1} | 文字数: {text.length}
      </div>
      <div className="text-editor-actions">
        <button onClick={handleSave} className="save-btn">保存</button>
        <button onClick={handleCancel} className="cancel-btn">キャンセル</button>
        <button onClick={onClose}>閉じる</button>
      </div>
    </div>
  );
};

interface TextInputProps {
  pageNumber: number;
  x: number;
  y: number;
  width: number;
  height: number;
  onSave: (text: string) => void;
  onCancel: () => void;
}

export const TextInput: React.FC<TextInputProps> = ({
  pageNumber,
  x,
  y,
  width,
  height,
  onSave,
  onCancel,
}) => {
  const { addTextBlock } = usePdfStore();
  const { error: showError, warning: showWarning } = useToast();
  const [text, setText] = useState('');
  const [fontSize, setFontSize] = useState(12.0);
  const [lineHeight, setLineHeight] = useState(1.2);
  const [fontFamily, setFontFamily] = useState('Arial');

  const handleSave = async () => {
    if (!text.trim()) {
      showWarning('テキストを入力してください');
      return;
    }
    try {
      // why: テキストブロックを追加してからダイアログを閉じる
      // alt: ダイアログを先に閉じる（追加が完了する前に閉じられる可能性がある）
      // evidence: awaitで追加が完了するのを待つことで、確実にテキストブロックが追加される
      await addTextBlock(pageNumber, x, y, width, height, text, fontSize, lineHeight, fontFamily);
      onSave(text);
    } catch (error) {
      logger.error('Failed to add text block', error instanceof Error ? error : new Error(String(error)));
      showError('テキストブロックの追加に失敗しました');
    }
  };

  return (
    <div className="text-input-dialog">
      <h3>新しいテキストブロックを追加</h3>
      <div className="form-group">
        <label htmlFor="text-input-textarea">テキスト:</label>
        <textarea
          id="text-input-textarea"
          value={text}
          onChange={(e) => setText(e.target.value)}
          rows={5}
          className="text-input-textarea"
          aria-label="テキストを入力"
        />
      </div>
      <div className="form-group">
        <label htmlFor="font-size-input">フォントサイズ:</label>
        <input
          id="font-size-input"
          type="number"
          value={fontSize}
          onChange={(e) => setFontSize(Number(e.target.value))}
          min="6"
          max="72"
          step="0.5"
          aria-label="フォントサイズを指定"
        />
      </div>
      <div className="form-group">
        <label htmlFor="font-family-select">フォント:</label>
        <select
          id="font-family-select"
          value={fontFamily}
          onChange={(e) => setFontFamily(e.target.value)}
          aria-label="フォントを選択"
        >
          <option value="Arial">Arial</option>
          <option value="Times New Roman">Times New Roman</option>
          <option value="Courier New">Courier New</option>
          <option value="Helvetica">Helvetica</option>
        </select>
      </div>
      <div className="form-actions">
        <button onClick={handleSave} className="save-btn">追加</button>
        <button onClick={onCancel} className="cancel-btn">キャンセル</button>
      </div>
    </div>
  );
};


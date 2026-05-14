import React, { useState, useRef } from 'react';
import { usePdfStore } from '../stores/pdfStore';
import { logger } from '../utils/logger';
import { useToast } from '../hooks/useToast';
import './ImageInserter.css';

interface ImageInserterProps {
  pageNumber: number;
  x: number;
  y: number;
  onCancel: () => void;
}

export const ImageInserter: React.FC<ImageInserterProps> = ({
  pageNumber,
  x,
  y,
  onCancel,
}) => {
  const { insertImage } = usePdfStore();
  const { error: showError, warning: showWarning } = useToast();
  const [width, setWidth] = useState(200.0);
  const [height, setHeight] = useState(200.0);
  const [imageFile, setImageFile] = useState<File | null>(null);
  const [preview, setPreview] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    if (!file.type.startsWith('image/')) {
      showWarning('画像ファイルを選択してください');
      return;
    }

    setImageFile(file);

    const reader = new FileReader();
    reader.onloadend = () => {
      setPreview(reader.result as string);
    };
    reader.readAsDataURL(file);
  };

  const handleInsert = async () => {
    if (!imageFile) {
      showWarning('画像ファイルを選択してください');
      return;
    }

    try {
      // why: FileReaderを使用して画像をBase64に変換してから追加
      // alt: 直接ファイルを送信（Base64変換が必要）
      // evidence: PDF生成時にBase64データが必要なため、事前に変換する必要がある
      const reader = new FileReader();
      reader.onloadend = async () => {
        try {
          const base64Data = reader.result as string;
          if (!base64Data) {
            showError('画像の読み込みに失敗しました');
            return;
          }
          // data:image/png;base64, のプレフィックスを除去
          const base64 = base64Data.split(',')[1];
          if (!base64) {
            showError('画像データの変換に失敗しました');
            return;
          }
          const format = imageFile.type.includes('png') ? 'png' : 'jpeg';

          // why: 画像を追加してからダイアログを閉じる
          // alt: ダイアログを先に閉じる（追加が完了する前に閉じられる可能性がある）
          // evidence: awaitで追加が完了するのを待つことで、確実に画像が追加される
          await insertImage(pageNumber, x, y, width, height, base64, format);
          onCancel();
        } catch (error) {
          logger.error('Failed to insert image', error instanceof Error ? error : new Error(String(error)));
          showError('画像の挿入に失敗しました');
        }
      };
      reader.onerror = () => {
        showError('画像の読み込みに失敗しました');
      };
      reader.readAsDataURL(imageFile);
    } catch (error) {
      logger.error('Failed to insert image', error instanceof Error ? error : new Error(String(error)));
      showError('画像の挿入に失敗しました');
    }
  };

  return (
    <div className="image-inserter-dialog">
      <h3>画像を挿入</h3>
      
      <div className="form-group">
        <label htmlFor="image-file-input">画像ファイル:</label>
        <input
          id="image-file-input"
          ref={fileInputRef}
          type="file"
          accept="image/png,image/jpeg,image/jpg"
          onChange={handleFileSelect}
          className="file-input"
          aria-label="画像ファイルを選択"
        />
        {preview && (
          <div className="image-preview">
            <img src={preview} alt="Preview" />
          </div>
        )}
      </div>

      <div className="form-group">
        <label htmlFor="image-width-input">幅 (pt):</label>
        <input
          id="image-width-input"
          type="number"
          value={width}
          onChange={(e) => setWidth(Number(e.target.value))}
          min="10"
          max="2000"
          step="1"
          aria-label="画像の幅を指定"
        />
      </div>

      <div className="form-group">
        <label htmlFor="image-height-input">高さ (pt):</label>
        <input
          id="image-height-input"
          type="number"
          value={height}
          onChange={(e) => setHeight(Number(e.target.value))}
          min="10"
          max="2000"
          step="1"
          aria-label="画像の高さを指定"
        />
      </div>

      <div className="form-actions">
        <button onClick={handleInsert} className="insert-btn" disabled={!imageFile}>
          挿入
        </button>
        <button onClick={onCancel} className="cancel-btn">
          キャンセル
        </button>
      </div>
    </div>
  );
};


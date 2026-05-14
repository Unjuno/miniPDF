import React, { useState } from 'react';
import './KeyboardShortcutsHelp.css';

interface Shortcut {
  key: string;
  description: string;
  category: string;
}

const shortcuts: Shortcut[] = [
  { key: 'Ctrl+O', description: 'ファイルを開く', category: 'ファイル操作' },
  { key: 'Ctrl+S', description: '保存', category: 'ファイル操作' },
  { key: 'Ctrl++', description: 'ズームイン', category: '表示' },
  { key: 'Ctrl+-', description: 'ズームアウト', category: '表示' },
  { key: 'Ctrl+0', description: 'ズームリセット', category: '表示' },
];

export const KeyboardShortcutsHelp: React.FC = () => {
  const [isOpen, setIsOpen] = useState(false);

  const groupedShortcuts = shortcuts.reduce((acc, shortcut) => {
    if (!acc[shortcut.category]) {
      acc[shortcut.category] = [];
    }
    acc[shortcut.category].push(shortcut);
    return acc;
  }, {} as Record<string, Shortcut[]>);

  return (
    <>
      <button
        className="help-button"
        onClick={() => setIsOpen(true)}
        title="キーボードショートカットヘルプ"
        aria-label="キーボードショートカットヘルプ"
      >
        ?
      </button>
      {isOpen && (
        <div className="help-overlay" onClick={() => setIsOpen(false)}>
          <div className="help-dialog" onClick={(e) => e.stopPropagation()}>
            <div className="help-header">
              <h2>キーボードショートカット</h2>
              <button
                className="help-close"
                onClick={() => setIsOpen(false)}
                aria-label="閉じる"
              >
                ×
              </button>
            </div>
            <div className="help-content">
              {Object.entries(groupedShortcuts).map(([category, items]) => (
                <div key={category} className="help-category">
                  <h3 className="help-category-title">{category}</h3>
                  <div className="help-shortcuts">
                    {items.map((item) => (
                      <div key={item.key} className="help-shortcut-item">
                        <kbd className="help-key">{item.key}</kbd>
                        <span className="help-description">{item.description}</span>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </>
  );
};


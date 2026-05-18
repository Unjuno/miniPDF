import React from 'react';
import './HtmlPreview.css';

interface HtmlPreviewProps {
  html: string | null;
}

export const HtmlPreview: React.FC<HtmlPreviewProps> = ({ html }) => {
  if (!html) {
    return <div className="html-preview-empty">Markdownを入力するとHTMLプレビューが表示されます</div>;
  }

  return (
    <div className="html-preview-shell">
      <div
        className="html-preview-document"
        dangerouslySetInnerHTML={{ __html: html }}
      />
    </div>
  );
};

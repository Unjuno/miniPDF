import React from 'react';
import { ImageElement as ImageElementType } from '../types/pdf';
import './ImageElement.css';

interface ImageElementProps {
  image: ImageElementType;
  isSelected: boolean;
  onSelect: () => void;
}

export const ImageElement: React.FC<ImageElementProps> = ({
  image,
  isSelected,
  onSelect,
}) => {
  return (
    <div
      className={`image-element ${isSelected ? 'selected' : ''}`}
      style={{
        '--element-left': `${image.x}px`,
        '--element-top': `${image.y}px`,
        '--element-width': `${image.width}px`,
        '--element-height': `${image.height}px`,
      } as React.CSSProperties}
      onClick={onSelect}
    >
      <img
        src={`data:image/${image.format};base64,${image.data}`}
        alt="PDF image"
        className="image-element-img"
      />
      {isSelected && (
        <div className="resize-handles">
          <div className="resize-handle resize-handle-nw"></div>
          <div className="resize-handle resize-handle-ne"></div>
          <div className="resize-handle resize-handle-sw"></div>
          <div className="resize-handle resize-handle-se"></div>
        </div>
      )}
    </div>
  );
};

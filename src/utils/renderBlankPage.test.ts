import { describe, expect, it, vi } from 'vitest';
import { renderBlankPage } from './renderBlankPage';

describe('renderBlankPage', () => {
  it('sizes the canvas and clears it', () => {
    const canvas = document.createElement('canvas');
    const fillRect = vi.fn();
    const context = { fillStyle: '', fillRect } as unknown as CanvasRenderingContext2D;
    const getContext = vi.fn().mockReturnValue(context);
    canvas.getContext = getContext as unknown as HTMLCanvasElement['getContext'];

    renderBlankPage(canvas, 100, 200, 1, 2);

    expect(canvas.width).toBe(200);
    expect(canvas.height).toBe(400);
    expect(canvas.style.width).toBe('100px');
    expect(canvas.style.height).toBe('200px');
    expect(context.fillStyle).toBe('#ffffff');
    expect(fillRect).toHaveBeenCalledWith(0, 0, 200, 400);
  });

  it('returns early when no context is available', () => {
    const canvas = document.createElement('canvas');
    canvas.getContext = vi.fn().mockReturnValue(null) as unknown as HTMLCanvasElement['getContext'];

    expect(() => renderBlankPage(canvas, 50, 60, 1)).not.toThrow();
  });

  it('handles null canvas safely', () => {
    expect(() => renderBlankPage(null as unknown as HTMLCanvasElement, 50, 60, 1)).not.toThrow();
  });
});

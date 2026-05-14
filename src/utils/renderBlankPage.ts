export const renderBlankPage = (
  canvas: HTMLCanvasElement | null,
  pageWidth: number,
  pageHeight: number,
  zoomLevel: number,
  devicePixelRatio: number = typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1
): void => {
  if (!canvas) return;

  const displayWidth = Math.max(1, Math.floor(pageWidth * zoomLevel));
  const displayHeight = Math.max(1, Math.floor(pageHeight * zoomLevel));
  const internalWidth = Math.max(1, Math.floor(pageWidth * zoomLevel * devicePixelRatio));
  const internalHeight = Math.max(1, Math.floor(pageHeight * zoomLevel * devicePixelRatio));

  if (canvas.width !== internalWidth || canvas.height !== internalHeight) {
    canvas.width = internalWidth;
    canvas.height = internalHeight;
  }

  canvas.style.width = `${displayWidth}px`;
  canvas.style.height = `${displayHeight}px`;

  const context = canvas.getContext('2d', { alpha: false });
  if (!context) {
    return;
  }

  context.fillStyle = '#ffffff';
  context.fillRect(0, 0, canvas.width, canvas.height);
};

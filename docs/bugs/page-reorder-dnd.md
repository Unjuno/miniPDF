# Bug Note: Page reorder drag-and-drop

Symptom:
- Dragging a page thumbnail to reorder pages does not trigger a reorder when the drag starts on the thumbnail image.
- Drag-and-drop may fail entirely in WebView2 if only the `text/plain` drag type is set.

Root cause:
- The thumbnail `img` element is natively draggable, so drag events were handled by the image instead of the parent page item, preventing the page item drag handler from setting the drag metadata.
- Some WebView2 drag-drop flows ignore `text/plain` unless the legacy `text` type is also set.

Fix:
- Disable native dragging on the thumbnail image so the parent page item drag handler always receives the drag start event.
- Set both `text/plain` and `text` drag data types so the drop handler can read the index reliably.

How to prevent regression:
- Keep drag handles/images marked as non-draggable when using parent-level HTML5 drag-and-drop.
- Maintain dual drag data types for compatibility and cover DnD interactions in manual testing.

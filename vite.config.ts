import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    outDir: 'dist',
    // why: 起動時間を最適化するため、コード分割とチャンクサイズを最適化
    // alt: デフォルト設定（単一バンドル）
    // evidence: コード分割により初期バンドルサイズが削減され、起動時間が短縮される
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          // why: コード分割を最適化して初期バンドルサイズを削減
          // alt: デフォルトの自動分割（最適化されていない可能性）
          // evidence: 手動分割により初期バンドルサイズが削減され、起動時間が短縮される
          if (id.includes('node_modules')) {
            if (id.includes('pdfjs-dist')) {
              return 'pdfjs';
            }
            if (id.includes('react') || id.includes('react-dom')) {
              return 'react-vendor';
            }
            if (id.includes('@tauri-apps')) {
              return 'tauri-vendor';
            }
            if (id.includes('zustand')) {
              return 'zustand';
            }
            return 'vendor';
          }
          // why: コンポーネント単位でコード分割して初期バンドルサイズを削減
          // alt: すべてのコンポーネントを1つのチャンクに（初期バンドルサイズが大きい）
          // evidence: コンポーネント単位で分割することで、必要なコンポーネントのみを読み込み、初期バンドルサイズを削減
          if (id.includes('src/components')) {
            // 重いコンポーネントを個別のチャンクに分割
            if (id.includes('PDFViewer')) {
              return 'pdf-viewer';
            }
            if (id.includes('TextEditor') || id.includes('TextInput')) {
              return 'text-editor';
            }
            if (id.includes('ImageInserter')) {
              return 'image-inserter';
            }
            // その他のコンポーネントは共通チャンクに
            return 'components';
          }
          // フックも個別のチャンクに分割
          if (id.includes('src/hooks')) {
            return 'hooks';
          }
        },
      },
    },
    // チャンクサイズの警告を無効化（大きなライブラリがあるため）
    chunkSizeWarningLimit: 1000,
    // 本番ビルドの最適化
    minify: 'esbuild', // terserより高速
    target: 'esnext', // 最新のJS機能を使用してバンドルサイズを削減
  },
  clearScreen: false,
  envPrefix: ['VITE_', 'TAURI_'],
  // 依存関係の事前バンドルを最適化
  optimizeDeps: {
    include: ['react', 'react-dom', 'zustand'],
    exclude: ['pdfjs-dist'], // PDF.jsは遅延読み込みするため除外
  },
});

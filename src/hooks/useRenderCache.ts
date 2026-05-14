import { useRef, useMemo } from 'react';

interface CacheEntry {
  imageData: ImageData;
  timestamp: number;
}

class LRURenderCache {
  private cache: Map<string, CacheEntry> = new Map();
  private maxSize: number;

  constructor(maxSize: number = 10) {
    this.maxSize = maxSize;
  }

  getKey(pageNum: number, zoomLevel: number): string {
    return `${pageNum}_${zoomLevel.toFixed(2)}`;
  }

  get(pageNum: number, zoomLevel: number): ImageData | null {
    const key = this.getKey(pageNum, zoomLevel);
    const entry = this.cache.get(key);
    
    if (entry) {
      this.cache.delete(key);
      this.cache.set(key, { ...entry, timestamp: Date.now() });
      return entry.imageData;
    }
    
    return null;
  }

  set(pageNum: number, zoomLevel: number, imageData: ImageData): void {
    const key = this.getKey(pageNum, zoomLevel);
    
    if (this.cache.has(key)) {
      this.cache.delete(key);
    }
    
    if (this.cache.size >= this.maxSize) {
      const firstKey = this.cache.keys().next().value;
      if (firstKey) {
        this.cache.delete(firstKey);
      }
    }
    
    this.cache.set(key, {
      imageData,
      timestamp: Date.now(),
    });
  }

  clear(): void {
    this.cache.clear();
  }

  size(): number {
    return this.cache.size;
  }
}

const globalCache = new LRURenderCache(10);

export function useRenderCache() {
  const cacheRef = useRef(globalCache);

  return useMemo(() => ({
    get: (pageNum: number, zoomLevel: number) => cacheRef.current.get(pageNum, zoomLevel),
    set: (pageNum: number, zoomLevel: number, imageData: ImageData) => 
      cacheRef.current.set(pageNum, zoomLevel, imageData),
    clear: () => cacheRef.current.clear(),
    size: () => cacheRef.current.size(),
  }), []);
}

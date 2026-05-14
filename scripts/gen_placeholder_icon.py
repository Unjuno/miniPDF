"""Emit a minimal valid RGB PNG (512x512) for `tauri icon` input. No third-party deps."""
from __future__ import annotations

import struct
import zlib
from pathlib import Path


def chunk(tag: bytes, data: bytes) -> bytes:
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)


def main() -> None:
    w = h = 512
    # RGB, filter 0 per scanline, flat blue-gray
    row = bytes([0]) + bytes([0x2A, 0x6B, 0x9E]) * w
    raw = row * h
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
    out = Path(__file__).resolve().parents[1] / "src-tauri" / "icons" / "icon.png"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(png)
    print(out, len(png))


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""生成 Peregrine 的 Windows 图标资源（assets/icon.ico）。

使用与 icon.rs 相同的占位图案：蓝色圆角背景 + 白色 "P" 字形，
生成 16x16、32x32、48x48 三档标准 ICO，供 Windows exe 嵌入与任务栏显示。
"""

import struct
from pathlib import Path

# "P" 字形位图（5 列 × 7 行），与 icon.rs 保持一致。
GLYPH_P = [
    [1, 1, 1, 1, 0],
    [1, 0, 0, 0, 1],
    [1, 0, 0, 0, 1],
    [1, 1, 1, 1, 0],
    [1, 0, 0, 0, 0],
    [1, 0, 0, 0, 0],
    [1, 0, 0, 0, 0],
]

BLUE = (37, 99, 235)
WHITE = (255, 255, 255)


def inside_rounded_rect(px: int, py: int, x: int, y: int, w: int, h: int, r: int) -> bool:
    r = max(0, min(r, w // 2, h // 2))
    if px < x or py < y or px >= x + w or py >= y + h:
        return False
    cx = max(x + r, min(px, x + w - 1 - r))
    cy = max(y + r, min(py, y + h - 1 - r))
    dx = px - cx
    dy = py - cy
    return dx * dx + dy * dy <= r * r


def render_icon(size: int) -> bytes:
    """渲染指定尺寸为 32-bit BGRA 像素数据（bottom-up）。"""
    n = size
    rgba = bytearray(n * n * 4)

    margin = round(n * 0.1)
    sq_min = margin
    sq_size = max(1, n - margin * 2)
    corner = round(sq_size * 0.22)

    pad = round(sq_size * 0.18)
    glyph_min_x = sq_min + pad
    glyph_min_y = sq_min + pad
    glyph_w = sq_size - pad * 2
    glyph_h = sq_size - pad * 2
    cell_w = glyph_w / 5.0
    cell_h = glyph_h / 7.0

    def set_pixel(x: int, y: int, b: int, g: int, r: int, a: int) -> None:
        idx = ((n - 1 - y) * n + x) * 4  # bottom-up
        rgba[idx] = b
        rgba[idx + 1] = g
        rgba[idx + 2] = r
        rgba[idx + 3] = a

    for y in range(n):
        for x in range(n):
            if not inside_rounded_rect(x, y, sq_min, sq_min, sq_size, sq_size, corner):
                continue

            color = (*BLUE, 255)

            gx = x - glyph_min_x
            gy = y - glyph_min_y
            if 0 <= gx < glyph_w and 0 <= gy < glyph_h:
                col = int(gx / cell_w)
                row = int(gy / cell_h)
                if 0 <= row < 7 and 0 <= col < 5 and GLYPH_P[row][col] == 1:
                    color = (*WHITE, 255)

            set_pixel(x, y, color[2], color[1], color[0], color[3])

    return bytes(rgba)


def build_bmp_dib(width: int, height: int, pixels: bytes) -> bytes:
    """生成 ICO 内部使用的 BITMAPINFOHEADER + XOR mask + AND mask。"""
    # BITMAPINFOHEADER (40 bytes)
    dib = struct.pack(
        "<IIIHHIIIIII",
        40,          # biSize
        width,       # biWidth
        height * 2,  # biHeight (XOR + AND)
        1,           # biPlanes
        32,          # biBitCount
        0,           # biCompression (BI_RGB)
        0,           # biSizeImage (可设为 0 for BI_RGB)
        0, 0,        # biXPelsPerMeter, biYPelsPerMeter
        0, 0,        # biClrUsed, biClrImportant
    )

    # AND mask：1 bit per pixel，按行对齐到 4 字节；全 0 表示不额外遮罩。
    row_bytes = ((width + 31) // 32) * 4
    and_mask = bytes(row_bytes * height)

    return dib + pixels + and_mask


def build_ico(sizes: list[int]) -> bytes:
    """组装多尺寸 ICO 文件。"""
    count = len(sizes)
    icondir = struct.pack("<HHH", 0, 1, count)

    entries = []
    images = []
    offset = 6 + 16 * count
    for size in sizes:
        pixels = render_icon(size)
        image = build_bmp_dib(size, size, pixels)
        image_size = len(image)
        # 目录条目：宽、高、颜色数、保留、颜色平面、位深、大小、偏移
        width_byte = size if size < 256 else 0
        height_byte = size if size < 256 else 0
        entries.append(
            struct.pack("<BBBBHHII", width_byte, height_byte, 0, 0, 1, 32, image_size, offset)
        )
        images.append(image)
        offset += image_size

    return icondir + b"".join(entries) + b"".join(images)


def main() -> None:
    out = Path(__file__).with_name("icon.ico")
    ico = build_ico([16, 32, 48])
    out.write_bytes(ico)
    print(f"generated {out} ({len(ico)} bytes)")


if __name__ == "__main__":
    main()

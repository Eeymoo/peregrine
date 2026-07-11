#!/usr/bin/env python3
"""生成 Peregrine 的高清 Windows 图标资源。

使用 4x4 超采样抗锯齿渲染蓝色圆角背景 + 白色 "P" 字形，
生成 16/32/48/64/128/256 六档 ICO，以及 512x512 PNG。
无外部依赖，仅使用标准库。
"""

import math
import struct
import zlib
from pathlib import Path

# "P" 字形位图（5 列 × 7 行）。
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


def inside_rounded_rect(px: float, py: float, x: float, y: float,
                         w: float, h: float, r: float) -> bool:
    """点 是否在圆角矩形内。"""
    r = max(0.0, min(r, w / 2, h / 2))
    if px < x or py < y or px >= x + w or py >= y + h:
        return False
    cx = max(x + r, min(px, x + w - 1 - r))
    cy = max(y + r, min(py, y + h - 1 - r))
    dx = px - cx
    dy = py - cy
    return dx * dx + dy * dy <= r * r


def glyph_alpha(px: float, py: float,
                gx0: float, gy0: float, gw: float, gh: float) -> float:
    """返回 "P" 字形在指定坐标的覆盖比例 [0, 1]，使用点采样。"""
    gx = px - gx0
    gy = py - gy0
    if gx < 0 or gx >= gw or gy < 0 or gy >= gh:
        return 0.0
    cell_w = gw / 5.0
    cell_h = gh / 7.0
    col = int(gx / cell_w)
    row = int(gy / cell_h)
    if 0 <= row < 7 and 0 <= col < 5 and GLYPH_P[row][col] == 1:
        return 1.0
    return 0.0


def render_icon(size: int, samples: int = 4) -> bytes:
    """渲染指定尺寸为 32-bit BGRA 像素数据（bottom-up），使用超采样抗锯齿。

    ``samples`` 为每像素的子采样数（samples x samples）。
    """
    n = size
    rgba = bytearray(n * n * 4)

    margin = n * 0.1
    sq_min = margin
    sq_size = max(1.0, n - margin * 2)
    corner = sq_size * 0.22

    pad = sq_size * 0.18
    glyph_min_x = sq_min + pad
    glyph_min_y = sq_min + pad
    glyph_w = sq_size - pad * 2
    glyph_h = sq_size - pad * 2

    inv_total = 1.0 / (samples * samples)

    for y in range(n):
        for x in range(n):
            r_acc = g_acc = b_acc = a_acc = 0.0

            for sy in range(samples):
                for sx in range(samples):
                    px = x + (sx + 0.5) / samples
                    py = y + (sy + 0.5) / samples

                    if not inside_rounded_rect(px, py, sq_min, sq_min,
                                               sq_size, sq_size, corner):
                        continue

                    # 判断此子像素是否在 P 字形内。
                    ga = glyph_alpha(px, py, glyph_min_x, glyph_min_y,
                                     glyph_w, glyph_h)
                    if ga > 0.5:
                        r_acc += WHITE[0]
                        g_acc += WHITE[1]
                        b_acc += WHITE[2]
                    else:
                        r_acc += BLUE[0]
                        g_acc += BLUE[1]
                        b_acc += BLUE[2]
                    a_acc += 255.0

            r = round(r_acc * inv_total)
            g = round(g_acc * inv_total)
            b = round(b_acc * inv_total)
            a = round(a_acc * inv_total)

            # bottom-up 行序
            idx = ((n - 1 - y) * n + x) * 4
            rgba[idx] = b       # B
            rgba[idx + 1] = g   # G
            rgba[idx + 2] = r   # R
            rgba[idx + 3] = a   # A

    return bytes(rgba)


def build_bmp_dib(width: int, height: int, pixels: bytes) -> bytes:
    """生成 ICO 内部使用的 BITMAPINFOHEADER + XOR mask + AND mask。"""
    dib = struct.pack(
        "<IIIHHIIIIII",
        40,          # biSize
        width,       # biWidth
        height * 2,  # biHeight (XOR + AND)
        1,           # biPlanes
        32,          # biBitCount
        0,           # biCompression
        0,           # biSizeImage
        0, 0,        # biXPelsPerMeter, biYPelsPerMeter
        0, 0,        # biClrUsed, biClrImportant
    )
    row_bytes = ((width + 31) // 32) * 4
    and_mask = bytes(row_bytes * height)
    return dib + pixels + and_mask


def build_ico(sizes: list[int], samples: int = 4) -> bytes:
    """组装多尺寸 ICO 文件。"""
    count = len(sizes)
    icondir = struct.pack("<HHH", 0, 1, count)

    entries = []
    images = []
    offset = 6 + 16 * count
    for size in sizes:
        pixels = render_icon(size, samples)
        image = build_bmp_dib(size, size, pixels)
        image_size = len(image)
        width_byte = size if size < 256 else 0
        height_byte = size if size < 256 else 0
        entries.append(
            struct.pack("<BBBBHHII", width_byte, height_byte, 0, 0,
                        1, 32, image_size, offset)
        )
        images.append(image)
        offset += image_size

    return icondir + b"".join(entries) + b"".join(images)


def build_png(size: int, samples: int = 4) -> bytes:
    """生成 RGBA PNG（纯 stdlib）。"""
    n = size
    # PNG 用 top-down RGBA。
    pixels = render_icon(n, samples)
    # render_icon 返回的是 bottom-up BGRA，需要转成 top-down RGBA。
    raw = bytearray()
    for y in range(n):
        row = bytearray()
        for x in range(n):
            src_idx = (y * n + x) * 4  # 已经是 top-down，但 render 是 bottom-up
            # 实际上 render_icon 做了 bottom-up 翻转（for ICO），PNG 需要反过来读行。
            src_idx = ((n - 1 - y) * n + x) * 4
            b = pixels[src_idx]
            g = pixels[src_idx + 1]
            r = pixels[src_idx + 2]
            a = pixels[src_idx + 3]
            row.extend([r, g, b, a])
        raw.append(0)  # filter byte: None
        raw.extend(row)

    def make_chunk(chunk_type: bytes, data: bytes) -> bytes:
        c = chunk_type + data
        crc = zlib.crc32(c) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + c + struct.pack(">I", crc)

    signature = b"\x89PNG\r\n\x1a\n"
    ihdr = struct.pack(">IIBBBBB", n, n, 8, 6, 0, 0, 0)  # 8-bit RGBA
    compressed = zlib.compress(bytes(raw), 9)

    return signature + make_chunk(b"IHDR", ihdr) + make_chunk(b"IDAT", compressed) + make_chunk(b"IEND", b"")


def main() -> None:
    sizes = [16, 32, 48, 64, 128, 256]

    # 生成 ICO（超采样：小尺寸用 8x 获得更好的抗锯齿）。
    ico_data = build_ico(sizes, samples=8)
    ico_path = Path(__file__).with_name("icon.ico")
    ico_path.write_bytes(ico_data)
    print(f"generated {ico_path} ({len(ico_data)} bytes)")

    # 生成高分辨率 PNG（用于 Tauri 图标）。
    png_data = build_png(512, samples=4)
    png_path = Path(__file__).with_name("icon.png")
    png_path.write_bytes(png_data)
    print(f"generated {png_path} ({len(png_data)} bytes)")


if __name__ == "__main__":
    main()

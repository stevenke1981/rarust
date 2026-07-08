from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[1]
ICON_DIR = ROOT / "rarust" / "assets" / "icons"
SIZES = [16, 24, 32, 48, 64, 128, 256]


PALETTE = {
    "bg": (18, 30, 48, 255),
    "panel": (36, 72, 116, 255),
    "panel2": (42, 132, 184, 255),
    "edge": (145, 205, 236, 255),
    "paper": (237, 246, 252, 255),
    "paper_shadow": (172, 202, 220, 255),
    "accent": (251, 191, 36, 255),
    "green": (42, 180, 112, 255),
    "red": (220, 84, 72, 255),
    "ink": (12, 20, 31, 255),
}


FUNCTIONS = {
    "open": "open",
    "create": "plus",
    "extract": "down",
    "test": "check",
    "refresh": "refresh",
    "close": "close",
    "password": "lock",
    "preview": "eye",
    "tab": "tab",
    "file": "file",
    "folder": "folder",
}


def scaled(size: int) -> tuple[Image.Image, ImageDraw.ImageDraw]:
    scale = 4
    img = Image.new("RGBA", (size * scale, size * scale), (0, 0, 0, 0))
    return img, ImageDraw.Draw(img)


def xy(size: int, *values: float) -> tuple[int, ...]:
    scale = size * 4 / 256
    return tuple(round(v * scale) for v in values)


def rounded(draw: ImageDraw.ImageDraw, size: int, box: tuple[float, float, float, float], radius: float, fill, outline=None, width=1):
    draw.rounded_rectangle(xy(size, *box), radius=round(radius * size * 4 / 256), fill=fill, outline=outline, width=max(1, round(width * size * 4 / 256)))


def draw_box(draw: ImageDraw.ImageDraw, size: int) -> None:
    rounded(draw, size, (36, 44, 220, 218), 36, PALETTE["bg"])
    rounded(draw, size, (54, 72, 202, 194), 18, PALETTE["panel"], PALETTE["edge"], 4)
    draw.polygon([xy(size, 54, 96)[0:2], xy(size, 128, 60)[0:2], xy(size, 202, 96)[0:2], xy(size, 128, 130)[0:2]], fill=PALETTE["panel2"])
    draw.line([xy(size, 54, 96)[0:2], xy(size, 128, 130)[0:2], xy(size, 202, 96)[0:2]], fill=PALETTE["edge"], width=max(2, round(size * 4 / 64)))
    rounded(draw, size, (84, 120, 172, 170), 10, PALETTE["paper"], PALETTE["paper_shadow"], 3)
    for y in [132, 146, 160]:
        draw.line([xy(size, 98, y)[0:2], xy(size, 158, y)[0:2]], fill=PALETTE["panel"], width=max(1, round(size * 4 / 96)))
    draw.arc(xy(size, 78, 80, 178, 188), 205, 335, fill=PALETTE["accent"], width=max(4, round(size * 4 / 28)))
    draw.polygon([xy(size, 173, 156)[0:2], xy(size, 198, 153)[0:2], xy(size, 184, 176)[0:2]], fill=PALETTE["accent"])


def draw_function(draw: ImageDraw.ImageDraw, size: int, kind: str) -> None:
    rounded(draw, size, (32, 32, 224, 224), 34, PALETTE["bg"])
    rounded(draw, size, (52, 58, 204, 198), 18, PALETTE["panel"], PALETTE["edge"], 4)

    if kind in {"file", "open", "create", "extract", "test", "refresh", "password", "preview"}:
        rounded(draw, size, (88, 66, 168, 176), 8, PALETTE["paper"], PALETTE["paper_shadow"], 3)
        draw.polygon([xy(size, 146, 66)[0:2], xy(size, 168, 88)[0:2], xy(size, 146, 88)[0:2]], fill=PALETTE["paper_shadow"])
        for y in [102, 120, 138]:
            draw.line([xy(size, 101, y)[0:2], xy(size, 154, y)[0:2]], fill=PALETTE["panel"], width=max(1, round(size * 4 / 100)))
    elif kind == "folder":
        rounded(draw, size, (66, 90, 190, 174), 10, PALETTE["accent"], PALETTE["edge"], 3)
        rounded(draw, size, (66, 76, 126, 104), 8, PALETTE["accent"], None, 1)
    elif kind == "tab":
        rounded(draw, size, (60, 84, 148, 174), 8, PALETTE["paper"], PALETTE["paper_shadow"], 3)
        rounded(draw, size, (96, 70, 184, 160), 8, PALETTE["panel2"], PALETTE["edge"], 3)

    if kind == "open":
        draw.arc(xy(size, 80, 104, 176, 190), 205, 330, fill=PALETTE["green"], width=max(5, round(size * 4 / 24)))
        draw.polygon([xy(size, 172, 158)[0:2], xy(size, 198, 154)[0:2], xy(size, 185, 178)[0:2]], fill=PALETTE["green"])
    elif kind == "plus" or kind == "create":
        draw.line([xy(size, 128, 94)[0:2], xy(size, 128, 180)[0:2]], fill=PALETTE["green"], width=max(7, round(size * 4 / 20)))
        draw.line([xy(size, 86, 137)[0:2], xy(size, 170, 137)[0:2]], fill=PALETTE["green"], width=max(7, round(size * 4 / 20)))
    elif kind == "down" or kind == "extract":
        draw.line([xy(size, 128, 82)[0:2], xy(size, 128, 154)[0:2]], fill=PALETTE["accent"], width=max(7, round(size * 4 / 20)))
        draw.polygon([xy(size, 96, 142)[0:2], xy(size, 160, 142)[0:2], xy(size, 128, 184)[0:2]], fill=PALETTE["accent"])
    elif kind == "check" or kind == "test":
        draw.line([xy(size, 82, 138)[0:2], xy(size, 116, 172)[0:2], xy(size, 182, 94)[0:2]], fill=PALETTE["green"], width=max(8, round(size * 4 / 20)), joint="curve")
    elif kind == "refresh":
        draw.arc(xy(size, 78, 76, 180, 178), 35, 310, fill=PALETTE["accent"], width=max(7, round(size * 4 / 22)))
        draw.polygon([xy(size, 166, 78)[0:2], xy(size, 196, 84)[0:2], xy(size, 176, 110)[0:2]], fill=PALETTE["accent"])
    elif kind == "close":
        draw.line([xy(size, 92, 92)[0:2], xy(size, 164, 164)[0:2]], fill=PALETTE["red"], width=max(9, round(size * 4 / 18)))
        draw.line([xy(size, 164, 92)[0:2], xy(size, 92, 164)[0:2]], fill=PALETTE["red"], width=max(9, round(size * 4 / 18)))
    elif kind == "lock" or kind == "password":
        rounded(draw, size, (86, 116, 170, 178), 10, PALETTE["accent"], PALETTE["edge"], 3)
        draw.arc(xy(size, 98, 70, 158, 134), 180, 360, fill=PALETTE["accent"], width=max(7, round(size * 4 / 22)))
        draw.ellipse(xy(size, 122, 140, 134, 152), fill=PALETTE["ink"])
    elif kind == "eye" or kind == "preview":
        draw.ellipse(xy(size, 76, 102, 180, 158), outline=PALETTE["accent"], width=max(7, round(size * 4 / 24)))
        draw.ellipse(xy(size, 112, 112, 144, 146), fill=PALETTE["accent"])


def save_icon(name: str, kind: str | None = None) -> None:
    ico_images = []
    for size in SIZES:
        img, draw = scaled(size)
        if kind is None:
            draw_box(draw, size)
        else:
            draw_function(draw, size, kind)
        img = img.resize((size, size), Image.Resampling.LANCZOS)
        ico_images.append(img)
        if size in {16, 32, 256}:
            img.save(ICON_DIR / f"{name}-{size}.png")
            (ICON_DIR / f"{name}-{size}.rgba").write_bytes(img.tobytes())
    ico_images[-1].save(
        ICON_DIR / f"{name}.ico",
        sizes=[(size, size) for size in SIZES],
        append_images=ico_images[:-1],
    )


def main() -> None:
    ICON_DIR.mkdir(parents=True, exist_ok=True)
    save_icon("app")
    for name, kind in FUNCTIONS.items():
        save_icon(name, kind)


if __name__ == "__main__":
    main()

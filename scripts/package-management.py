"""Build a self-contained management.html for the optional panel release updater."""
import argparse
import base64
from pathlib import Path


def package(root: Path) -> str:
    html = (root / "ui/ui.html").read_text()
    for name in ("llm", "configuration"):
        css = (root / f"ui/{name}.css").read_text()
        js = (root / f"ui/{name}.js").read_text().replace("</script", "<\\/script")
        html = html.replace(f'<link rel="stylesheet" href="/assets/{name}.css">', f"<style>{css}</style>")
        html = html.replace(f'<script src="/assets/{name}.js"></script>', f"<script>{js}</script>")
    for name in ("transit-mark.svg", "transit-logo.svg"):
        data = base64.b64encode((root / "logo" / name).read_bytes()).decode("ascii")
        html = html.replace(f"/assets/{name}?v=2", f"data:image/svg+xml;base64,{data}")
        html = html.replace(f"/assets/{name}", f"data:image/svg+xml;base64,{data}")
    notices = (root / "ui/THIRD_PARTY_NOTICES.md").read_text().replace("-->", "--&gt;")
    html = html.replace("</head>", f"<!--\n{notices}\n-->\n</head>")
    return html.replace('<meta charset="utf-8">', '<meta charset="utf-8">\n  <meta name="transit-management-api" content="1">')


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="Output management.html path")
    args = parser.parse_args()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(package(Path(__file__).resolve().parents[1]))
    print(f"Built {args.output}")

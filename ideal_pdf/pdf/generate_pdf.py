from setup.config import Config
from setup.kb_urls import CsvItem
from setup.logger import log
from .edit_html.read_html import read_html
from .edit_html.contents import TocItem

from pathlib import Path
import pdfkit
import re

OUTLINE_PATH = Path("outline")


def generate_full_pdf(kburls: list[CsvItem], dir_path: Path, config: Config):
    dir_path.mkdir(exist_ok=True)
    outline = default_outline(kburls)
    if config.pdf:
        generate_sub_pdf(kburls, dir_path, config, outline)
        if config.repeat_outline:
            outline = read_outline(kburls)
            generate_sub_pdf(kburls, dir_path, config, outline)
    else:
        html = read_html(kburls, outline, config)
        (dir_path / config.html_path).write_text(html, encoding="utf-8")

def generate_sub_pdf(
    kburls: list[CsvItem],
    dir_path: Path, config: Config,
    outline: list[TocItem]
):
    html = read_html(kburls, outline, config)
    (dir_path / config.html_path).write_text(html, encoding="utf-8")
    wkhtmltopdf(html, dir_path / config.pdf_path, config)
    log.info(f"Wrote PDF to {dir_path / config.pdf_path}")

def default_outline(kburls: list[CsvItem]) -> list[TocItem]:
    return [TocItem(header = row.header, page_num=0, link_id=row.id_path) for row in kburls]

def read_outline(kburls: list[CsvItem]) -> list[TocItem]:
    outline = OUTLINE_PATH.read_text(encoding="utf-8")
    outline_rows = map(str.strip, outline.splitlines())
    outline_rows = filter(lambda row: row.startswith('<item title="'), outline_rows)
    outline_rows = map(get_title_page, outline_rows)
    outline_rows = filter(lambda row: row[0][0].isnumeric(), outline_rows)
    return [create_toc_item(*row, csv_item) for row, csv_item in zip(outline_rows, kburls, strict=True)] 

def create_toc_item(header: str, page_num: int, csv_item: CsvItem) -> TocItem:
    return TocItem(header=header, page_num=page_num, link_id=csv_item.id_path)

def get_title_page(line: str) -> tuple[str, int]:
    title = re.search('title="([^"]*)"', line)
    page = re.search('page="([^"]*)"', line)
    assert title is not None and page is not None
    return title[1], int(page[1])

def generate_pdf(kburls: list[CsvItem], dir_path: Path, config: Config):
    dir_path.mkdir(exist_ok=True)
    html = read_html(kburls, [], config)
    (dir_path / config.html_path).write_text(html, encoding="utf-8")
    print(dir_path
     / config.pdf_path)
    if config.pdf:
        wkhtmltopdf(html, dir_path / config.pdf_path, config)
        log.info(f"Wrote PDF to {dir_path / config.pdf_path}")



def wkhtmltopdf(html: str, pdf_path: Path, config: Config):
    log.info("Starting wk")
    wk_config = pdfkit.configuration(wkhtmltopdf="")
    pdfkit.from_string(
        html,
        pdf_path,
        configuration=wk_config,
        options=config.wkhtml_settings,
        # toc={"xsl-style-sheet": "toc.xsl"},
    )

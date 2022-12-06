from .logger import log

from dataclasses import dataclass
from typing import Any, NamedTuple
from pathlib import Path

import toml
import argparse


class TocTypeConfig(NamedTuple):
    font_size: str
    padding_left: str
    margin: str

class TocConfig(NamedTuple):
    chapter: TocTypeConfig
    main: TocTypeConfig


# Public
class Config(NamedTuple):
    pdf: bool
    repeat_outline: bool
    languages: list[str]
    num_rows: int
    pdf_path: Path
    html_path: Path
    wkhtml_settings: dict[str, Any]
    toc_config: TocConfig

def read_config(filepath: str) -> Config:
    """Returns a simplified data structure containing the config settings"""
    if not Path(filepath).exists():
        log.error(f"Could not read {filepath}")
        exit(1)
    dict_config: dict[str, Any] = toml.load(filepath)
    try:
        config: Config = _parse_config(dict_config, _read_args())
    except KeyError as key:
        log.error(f"Failed to read '{key}' from config.toml")
        exit(1)

    return config

# Private
@dataclass
class _ArgConfig:
    quiet: bool
    fullrun: bool
    nofullrun: bool
    pdf: bool
    nopdf: bool
    repeat: bool
    norepeat: bool
    htmlpath: str
    pdfpath: str
    langs: list[str]
    num_rows: int

def _read_args() -> _ArgConfig:
    """Parses and return the information from system arguments"""
    parser = argparse.ArgumentParser()

    # Non Config    
    group = parser.add_mutually_exclusive_group()
    group.add_argument("-q", "--quiet", action="store_true", help="Quiet/Hide Logging")
    group.add_argument("-v", "--verbose", action="store_true", help="Verbose")

    # Languages
    parser.add_argument("-l", "--langs", type=str, nargs="+", help="Optional Languages eg: (en, it)")
    # Num rows
    parser.add_argument("-n", "--num_rows", "--numrows", type=int, help="Maximum Number of csv urls to use.")

    # Full Run Bool
    group = parser.add_mutually_exclusive_group()
    group.add_argument("-f", "--fullrun", action="store_true", help="Turns all the proper config on")
    group.add_argument("-nf", "--nofullrun", action="store_true", help="Turns fullrun off")

    # Pdf Bool
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--pdf", action="store_true", help="Turn Pdf On")
    group.add_argument("--nopdf", action="store_true", help="Turn Pdf Off")

    # Pdf Bool
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--repeat", action="store_true", help="Repeat With outline to gather pagenumbers")
    group.add_argument("--norepeat", action="store_true", help="Do not repeat With outline to gather pagenumbers")

    # Pdf Path
    parser.add_argument("-o", "--pdfpath", type=str, help="Path to write Final PDF")
    parser.add_argument("--htmlpath", "--html_path", type=str, help="Path to write HTML Output")

    return parser.parse_args(namespace=_ArgConfig) # type: ignore

def _parse_config(config: dict[str, Any], args: _ArgConfig) -> Config:
    """Merges system arguments and config arguments into one easy to read dataclass"""
    creation = config["creation"]
    filenames = config["filenames"]

    full_run = args.fullrun or (config["full_run"])
    if args.nofullrun: full_run = False

    pdf = (creation["pdf"] or config["full_run"] or args.pdf or args.fullrun) and (not args.nopdf)
    repeat_outline = (creation["repeat_outline"] or config["full_run"] or args.repeat or args.fullrun) and (not args.norepeat)
    # Sets the item to element if it exists prioritizing the first element
    (languages := args.langs or config["langs"])
    (pdf_path := args.pdfpath or filenames["pdf"])
    (html_path := args.htmlpath or filenames["html"])
    if full_run:
        num_rows = -1
    else:
        (num_rows := args.num_rows or creation["num_rows"])

    return Config(
        pdf=pdf,
        repeat_outline=repeat_outline,
        languages=languages,
        num_rows=num_rows,
        html_path=Path(html_path),
        pdf_path=Path(pdf_path),
        wkhtml_settings=config["wkhtmltopdf"],
        toc_config=read_toc_config(config["TOC"]),
    )

def read_toc_config(config: dict[str, Any]) -> TocConfig:
    main = TocTypeConfig(
        font_size=config["main_font_size"],
        padding_left=config["main_indent"],
        margin=config["main_margin"]
    )
    chapter = TocTypeConfig(
        font_size=config["chapter_font_size"],
        padding_left=config["chapter_indent"],
        margin=config["chapter_margin"]
    )
    return TocConfig(main=main, chapter=chapter)
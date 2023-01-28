from .paths import url_to_path, DIR_PATH
from .logger import log

from typing import Iterable
from pathlib import Path
from dataclasses import dataclass
import csv


@dataclass
class CsvItem:
    header: str
    url: str
    path: Path
    id_path: str
    slugs: list[str]
    include: int
    depth: int
    depth_str: str = ""

    @classmethod
    def from_dict(cls, row: dict[str, str]):
        url: str = row["URL"]
        path: Path = url_to_path(url)
        id_path = "/".join(path.parts).removeprefix("../kb_archive/html/")
        slugs: list[str]
        include: int
        depth: int
        header: str = row["Header"]
        
        try:
            if row["Include"] != "":
                include = int(row["Include"])
            else:
                include = 0
        except ValueError as s:
            log.error(f"Could not convert 'Include' field to integer for {url}: {s}")
            exit(1)


        if row["Depth"] == "":
            depth = 0
        elif row["Depth"].isnumeric():
            depth = int(row["Depth"])
        else:
            log.error(f"Invalid Depth Argument: {row['Depth']}")
            exit(1)

        slugs = [f"https://mariadb.com/kb/en/{slug}/" for slug in row["Duplicate slugs"].split(";") if slug.strip()]

        return cls(
            url=url,
            path=path,
            id_path=id_path,
            slugs=slugs,
            include=include,
            depth=depth,
            header=header,
        )

def read_csv(filepath: Path|str, num_rows: int) -> list[CsvItem]:
    if not Path(filepath).exists():
        log.error(f"Could not read: {filepath}")
        exit(1)
    with open(filepath, 'r', encoding="utf-8") as infile:
        content = csv.DictReader(infile)
        kb_urls = _parse_csv(content, num_rows)
    apply_depth(kb_urls)
    return kb_urls


def _parse_csv(content: Iterable[dict], num_rows) -> list[CsvItem]:
    rows = [row for row in content if row["Include"] not in ["", "0"]]
    rows = rows[:num_rows] if num_rows > 0 else rows
    return [CsvItem.from_dict(row) for row in rows if row["Include"] not in ["", "0"]]

def apply_depth(kb_urls: list[CsvItem]):
    depths = []
    for row in kb_urls:
        if row.depth >= len(depths):
            depths.extend([0] * (row.depth-len(depths)))
        elif row.depth < len(depths):
            for _ in range(len(depths)-row.depth):
                depths.pop()
        depths[row.depth-1] += 1
        row.depth_str = '.'.join([str(num) for num in depths])
        while row.depth_str.startswith("0."):
            row.depth_str = row.depth_str.removeprefix("0.")

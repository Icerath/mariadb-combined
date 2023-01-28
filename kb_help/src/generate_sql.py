from .version import Version
from .kb_item import KbItem
from .kb_archive import KbArchive
from . import debug
from .html2text import html_to_text

from typing import Iterator
from html import unescape
from pathlib import Path
import csv
from itertools import chain

CATEGORY_CSV = Path("input/help_cats.csv")
KB_URLS_PATH = Path("../kb_urls.csv")

def generate_sql(version: Version, concat_size: int) -> str:
    boilerplate = read_boilerplate()
    help_categories, category_info = read_category_info(version)
    kb_urls = read_kb_urls(category_info, version)
    help_relations, help_keywords = generate_keyword_sql(kb_urls)
    descriptions = generate_descriptions(kb_urls, version, concat_size)

    return merge_sql(boilerplate, help_categories, descriptions, help_relations, help_keywords)

def merge_sql(
    boilerplate: str, help_categories: list[str], descriptions: list[str],
    help_keywords: list[str], help_relations: list[str]
) -> str:
    lines = help_categories + [""] + descriptions + [""] + help_keywords + [""] + help_relations
    return "".join([boilerplate, "\n", "\n".join(lines)])

def read_boilerplate() -> str:
    return Path("input/starting_sql.sql").read_text(encoding="utf-8")

def read_category_info(version: Version) -> tuple[list[str], dict[str, int]]:
    """ Returns (raw sql string, mapping between category name and it's id) """
    csv_rows = read_category_csv_raw(version)
    # generates a unique ID for each category, with the category name '0' being first
    category_ids =  { '0': 0 } | {
        row["Name"]: cat_id
        for (cat_id, row) in enumerate(csv_rows, 1)
    }
    categories_str = [
        format_category_definition(row["Name"], cat_id, category_ids[row["Parent"]])
        for (cat_id, row) in enumerate(csv_rows, 1)
    ]

    return categories_str, category_ids

def read_category_csv_raw(version: Version) -> list[dict]:
    """Reads category_csv and filters categories added after the given version"""
    infile = CATEGORY_CSV.read_text(encoding="utf-8")
    unfiltered_csv = csv.DictReader(infile.splitlines())

    is_valid_version = lambda row: row["Include"] == "1" or (Version.from_str(row["Include"]) <= version)
    return list(filter(is_valid_version, unfiltered_csv))

def format_category_definition(name, cat_id: int, parent_id: int) -> str:
    return "insert into help_category (help_category_id,name,parent_category_id,url)" \
           f" values ({cat_id},'{name}',{parent_id},'');"


def read_kb_urls(category_ids: dict[str, int], version: Version):
    with open(KB_URLS_PATH, 'r', encoding="utf-8") as infile:
        reader = csv.DictReader(infile, strict=True)
        urls = set()
        rows = [KbItem(
            row["URL"],
            category_ids[row["HELP Cat"]],
            keywords_str=row["HELP Keywords"])
            for row in reader if is_valid_row(row, urls, version)
        ]
    return rows

def is_valid_row(row: dict[str, str], urls: set[str], version: Version) -> bool:
    if not row["URL"]:
        return False
    if not row["HELP Include"]:
        debug.warn("No Help Include for " + row["URL"])
        return False

    if row["HELP Include"] == '0' \
        or row["HELP Include"] != '1' and Version.from_str(row["HELP Include"]) > version:
        return False
    
    url = row["URL"]
    if url in urls:
        debug.warn(f"Duplicate url: '{url}'")
        return False

    urls.add(url)
    return True

def row_help_topics(kb_urls: list[KbItem]) -> Iterator[tuple[int, KbItem]]:
    # Starting at 3 to make room for HELP DATE AND HELP_VERSION
    return enumerate(kb_urls, 3)

def generate_keyword_sql(kb_urls: list[KbItem]) -> tuple[list[str], list[str]]:
    unique_keywords = set(chain(*[row.keywords for row in kb_urls]))
    
    topic_keywords_2d = [(topic_id, row.keywords) for (topic_id, row) in row_help_topics(kb_urls)]
    topic_keywords = []
    for (topic_id, keywords) in topic_keywords_2d:
        for keyword in keywords:
            topic_keywords.append((topic_id, keyword))

    keyword_ids: dict[str, int] = {
        keyword: keyword_id for (keyword_id, keyword)
        in enumerate(unique_keywords, 1)
    }
    help_keywords: list[str] = [
        insert_help_keyword(keyword_id, keyword)
        for (keyword, keyword_id) in keyword_ids.items()
    ]
    help_relations: list[str] = [
        insert_help_relations(topic_id, keyword_ids[keyword])
        for (topic_id, keyword) in topic_keywords
    ]

    return help_keywords, help_relations

def generate_descriptions(kb_urls: list[KbItem], version: Version, concat_size: int):
    topic_keywords = []
    help_topics = []
    archive = init_archive(kb_urls)
    for index, (help_topic_id, row) in enumerate(row_help_topics(kb_urls)):
        html = archive.read_html(row.url)

        topic_keywords.append((help_topic_id, row.keywords))
        description = html_to_text(html)
        page_name = read_page_name(html, row.url)
        help_topics.append(
            insert_help_topic(help_topic_id, row, page_name, description, concat_size)
        )
        progress = round(index / len(kb_urls) * 100)
        print(f"\r{progress}%", end="")

    return help_topics

def insert_help_keyword(keyword_id: int, keyword: str) -> str:
    return f"insert into help_keyword values ({keyword_id}, '{keyword}');"

def insert_help_relations(topic_id: int, keyword_id: int) -> str:
    return f"insert into help_relation values ({topic_id}, {keyword_id});"

def insert_help_topic(help_topic_id: int, row: KbItem, page_name: str, description: str, concat_size: int) -> str:
    """Creates a help topic row in sql"""
    parts = split_description_by_length(description, concat_size)
    description = parts.pop(0)
    output = "insert into help_topic (help_topic_id,help_category_id,name,description,example,url) values "
    output += f"({help_topic_id},{row.category},'{page_name}','{description}','','{row.url}');"

    concats = "".join([get_update_help_topic(desc, help_topic_id) for desc in parts])
    return output + concats

def split_description_by_length(description: str, line_length: int) -> list[str]:
    parts = []
    start = 0
    while len(description) >= line_length:
        index = description.rindex("\\n", 0, line_length)
        parts.append(description[start:index])
        start = index
    parts.append(description[start:])
        
    assert "".join(parts) == description
    return parts

def get_update_help_topic(description: str, help_topic_id: int) -> str:
    return "\nupdate help_topic set description = "\
        f"CONCAT(description, '{description}') WHERE help_topic_id = {help_topic_id};"

def read_page_name(html: str, url: str) -> str:
    if not ("<title>" in html and "</title>" in html):
        debug.error(f"Did not find title tag for '{url}'")

    index = html.index("<title>")
    end_index = html.index("</title>", index+1)

    title: str = html[index:end_index]\
        .removeprefix("<title>")\
        .removesuffix(" - MariaDB Knowledge Base")
    # Converts html escape sequences like '&amp'; to their text representations: '&'
    return unescape(title)

def init_archive(kb_urls: list[KbItem]):
    return KbArchive(map(lambda row: row.url, kb_urls))
from pathlib import Path
from typing import Iterator, Iterable

ARCHIVE_PATH = Path("../url_locations.txt")
HAS_INITIALIZED_ARCHIVE = False

class KbArchive:
    urls: dict[str, Path]

    def __init__(self, kb_urls: Iterable[str]):
        global HAS_INITIALIZED_ARCHIVE
        assert not HAS_INITIALIZED_ARCHIVE
        kb_urls = _clean_kb_urls(kb_urls)
        all_urls = _read_archive_urls_raw()
        filtered_urls = _filter_contained(all_urls, kb_urls)
        formatted_urls = { url: _format_raw_path(path) for (url, path) in filtered_urls }
        self.urls = formatted_urls
        HAS_INITIALIZED_ARCHIVE = True

    def read_html(self, url: str) -> str:
        path = self.get_path(url)
        return path.read_text(encoding="utf-8")
    
    def get_path(self, url: str) -> Path:
        url = url.strip().removesuffix('/')
        path = self.urls.get(url)
        assert path is not None, url
        return path
        
def _clean_kb_urls(kb_urls: Iterable[str]) -> set[str]:
    return { _clean_url(url) for url in kb_urls }

def _read_archive_urls_raw() -> Iterator[tuple[str, str]]:
    text = ARCHIVE_PATH.read_text(encoding="utf-8")
    for line in text.splitlines():
        url, path = line.split(' ', maxsplit=2)
        yield url, path

def _clean_url(url: str) -> str:
    return url.strip().removesuffix('/')

def _filter_contained(archive_urls: Iterable[tuple[str, str]], kb_urls: set[str]) -> Iterator[tuple[str, str]]:
    for url, path in archive_urls:
        if url in kb_urls:
            yield url, path

def _format_raw_path(raw_path: str) -> Path:
    base = "../kb_archive/" + raw_path.strip().removeprefix("../")
    return Path(base)
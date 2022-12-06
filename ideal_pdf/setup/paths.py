from pathlib import Path

DIR_PATH = Path("../KBArchive/HTML")
BASE_KB = "https://mariadb.com/kb/"

IGNORED_SUFFIXES: list[str] = ["+translate", "+flag", "+history", "/ask", "+search", "+change_order", "/post", "/remove"]
IGNORED_CONTAINED: list[str] = ["/+search/"]

def url_to_path(dir_path: Path, url: str) -> Path:
    url_suffix = url.strip().removeprefix(BASE_KB).strip('/')
    return (dir_path / url_suffix).with_suffix(".html")

def format_url(suffix: str) -> str|None:
    for symbol in ('#', '?'):
        if symbol in suffix:
            idx = suffix.index(symbol)
            suffix = suffix[:idx]
        assert symbol not in suffix
    url = suffix\
        .removeprefix("https://")\
        .removeprefix("mariadb.com")\
        .removeprefix('/')\
        .removeprefix("kb/")\
        .strip()

    should_ignore: bool = any(url.removesuffix('/').endswith(suffix) for suffix in IGNORED_SUFFIXES)
    if not url or should_ignore:
        return None
    return BASE_KB + url
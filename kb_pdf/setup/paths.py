from pathlib import Path

DIR_PATH_STR ="../kb_archive/HTML" 
DIR_PATH = Path(DIR_PATH_STR)
BASE_KB = "https://mariadb.com/kb/"
URL_LOCATIONS_PATH = Path("../url_locations.txt")


def load_url_locations() -> dict[str, Path]:
    lines = URL_LOCATIONS_PATH.read_text(encoding="utf-8").splitlines()
    lines = [line.split(' ', maxsplit=1) for line in lines]
    return { left: Path(right.replace("../html/", "../kb_archive/html/")) for left, right in lines }

def url_to_path(url: str) -> Path:
    url = url.strip().removesuffix('/')
    path = URL_LOCATIONS.get(url)
    assert path is not None, f"{url}"
    return path 

def format_url(suffix: str) -> str:
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

    return BASE_KB + url

URL_LOCATIONS = load_url_locations()
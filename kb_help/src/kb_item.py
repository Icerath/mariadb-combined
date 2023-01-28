from dataclasses import dataclass

@dataclass(init=False, slots=True)
class KbItem:
    url: str
    category: int
    keywords: list[str]

    def __init__(self, url: str, category: int, keywords_str: str):
        self.url = url
        self.category = category
        self.keywords = list(filter(bool, keywords_str.split(';')))

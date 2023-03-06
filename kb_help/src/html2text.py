LINE_LIMIT = 79

from . import debug
from bs4 import Tag, BeautifulSoup as Soup
from .html_tag_rules import *


def html_to_text(html: str, url: str) -> str:
    html = clean_html(html, url)
    soup = Soup(html, features="lxml")
    remove_junk(soup)
    apply_tag_rules(soup)
    remove_see_also(soup)
    text = soup.get_text()
    #modify the text
    #text = add_url(text, name)
    #text = modify_text(text)
    return text

def clean_html(html: str, url) -> str:
    if '<section id="content" class="limited_width col-md-8 clearfix">' not in html:
        debug.error(f"Invalid HTML for '{url}'")

    section = html.index('<section id="content" class="limited_width col-md-8 clearfix">')
    end_section = html.index('</section>')

    html = html[section: end_section + len('</section>')]
    return html

def remove_junk(soup: Soup):
        #helper method for easy removal
    def remove(soup: Soup, *args, **kwargs):
        _ = [tag.decompose() for tag in soup.find_all(*args, **kwargs) if tag != None]

    #remove irrelevant information
    remove(soup, "div", {"id": "content_disclaimer"}) #removes a disclaimer
    remove(soup, "div", {"id": "comments"}) #remove the comments
    remove(soup, "h2", text = "Comments") #remove the comments' header
    remove(soup, "div", {"id": "subscribe"}) #removes the subscribe thingy (I don't know what this removes)
    remove(soup, "div", {"class": "simple_section_nav"}) #removes extra links
    remove(soup, "div", {"class": "table_of_contents"}) #remove side contents bar

    #remove main header
    tag = soup.find("h1")
    if isinstance(tag, Tag): tag.decompose()

def apply_tag_rules(soup: Soup):
    tag: Soup
    for tag in soup.descendants: # type: ignore
        if tag.name in TAG_RULES:
            TAG_RULES[tag.name](tag)

def remove_see_also(soup: Soup):
    """Finds the header labled 'See Also', removes it and it's next sibling"""
    for n in range(2, 7):
        see_also = soup.find(f"h{n}", {"id": "see-also", "class": "anchored_heading"})
        if see_also is not None:
            ns = see_also.find_next_sibling()
            if ns is not None: ns.decompose() # type: ignore
            see_also.decompose() # type: ignore

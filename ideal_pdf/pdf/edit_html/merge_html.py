from setup.kb_urls import CsvItem
from setup.paths import BASE_KB
from setup.config import Config
from setup.logger import log

from .contents import create_contents, TocItem
import re


def merge_html(pages: list[str], kburls: list[CsvItem], outline: list[TocItem], config: Config) -> str:
    log.info("Merging HTML")
    html = "\n".join(pages)
    html = create_contents(outline, config.toc_config) + html
    html = absolute_links(html)
    html = internalise_links(html, kburls)
    html = START_BOILERPLATE + html + END_BOILERPLATE
    return html

def absolute_links(html: str) -> str:
    return html.replace('="/kb/', f'="{BASE_KB}')

def internalise_links(html: str, kburls: list[CsvItem]) -> str:
    urls_completed = set()

    for row in kburls:
        if row.include == 1:
            for url in row.slugs + [row.url]:
                urls_completed.add(url)
                html = html.replace(f'href="{url}#', f'href="#{row.id_path}')
                html = html.replace(f'href="{url}"', f'href="#{row.id_path}"')

    #remove duplicates hashes for previously external links carrying internal links
    pattern = r'(href ?= ?")(#[\w-]+)#([\w-]+)'
    html = re.sub(pattern, r"\1\2\3", html)

    return html


# region: -- Boilerplate
END_BOILERPLATE = "\n\n</body>\n</html>"
START_BOILERPLATE = (
"""
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <meta http-equiv="X-UA-Compatible" content="IE=edge">
        <title>MariaDB Server Documentation</title>
        <meta name="description" content="">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <meta http-equiv="Content-Type" content="text/html; charset=utf-8" />
        <link href="https://mariadb.com/kb/static/css/main.9a0d7dcebefd.css" rel="stylesheet" type="text/css" />
        <style>
            body {
                font-family: "Arial";
            }
            .pdfhorizontal_dotted_line {
                position: relative;
         
            }
            .pdfhorizontal_dotted_line span {
                display: inline-block;
                background: #fff;
                position: relative;
                z-index: 1;
            }
            .pdfhorizontal_dotted_line:after {
                content: '';
                position: absolute;
                margin-right: 0;
                top: 70%;
                left: 0;
                right: 0;
                z-index: -1;
                border-top: 2px dotted black;
            }
            a[href ^= "http"]:after {
                content: " " url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAKCAYAAACNMs+9AAAAVklEQVR4Xn3PgQkAMQhDUXfqTu7kTtkpd5RA8AInfArtQ2iRXFWT2QedAfttj2FsPIOE1eCOlEuoWWjgzYaB/IkeGOrxXhqB+uA9Bfcm0lAZuh+YIeAD+cAqSz4kCMUAAAAASUVORK5CYII=);    
            }
            a[class=""]:after {
                content: ""
            }       
        </style>
    </head>
<body class = "mpkb nodes products nodes_view jqui">\n\n
""")

#endregion: -- Boilerplate
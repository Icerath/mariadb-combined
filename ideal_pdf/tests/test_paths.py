from setup.paths import url_to_path, format_url, BASE_KB, DIR_PATH
from pathlib import Path

# test format_url

def test_format_url_bulk():
    inputs = Path("tests/test_inputs.txt").read_text(encoding="utf-8").split('\n')
    outputs = Path("tests/test_outputs.txt").read_text(encoding="utf-8").split('\n')

    for (input, output) in zip(inputs, outputs, strict=True):
        assert format_url(input) == output

# test url_to_path
def test_url_to_path_en():
    url = BASE_KB + "en/"
    expected = DIR_PATH / "en.html/"
    assert url_to_path(DIR_PATH, url) == expected

def test_url_to_path_select():
    url = BASE_KB + "en/select/"
    expected = DIR_PATH / "en/select.html/"
    assert url_to_path(DIR_PATH, url) == expected

def test_url_to_path_source():
    url = BASE_KB + "en/alter-user/+source/"
    expected = DIR_PATH / "en/alter-user/+source.html/"
    assert url_to_path(DIR_PATH, url) == expected

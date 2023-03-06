"""Version selecting and debug info, interface to run generation script"""
import time
import os
from pathlib import Path
from src.generate_sql import generate_sql
from src.version import Version
import src.debug as debug
import argparse

# Preparing system for colored text
os.system('')

SQL_FILENAME: str = "fill_help_tables.sql"
DEFAULT_CONCAT_SIZE = 15000

def read_args() -> tuple[list[Version], int]:
    parser = argparse.ArgumentParser()
    parser.add_argument("--length", "-l", type=int, default=DEFAULT_CONCAT_SIZE)
    parser.add_argument("--versions", "--version", "-v", nargs="+", required=True)
    args = parser.parse_args()

    versions = read_versions(args.versions)
    return versions, args.length

# Functions
def read_versions(args: list[str]) -> list[Version]:
    """Reads the version number while giving precise debug info"""
    versions = []
    for version_str in args:
        assert version_str.isnumeric(), version_str
        assert len(version_str) >= 3
        version = Version.from_str(version_str)
        assert version.major >= 10
        versions.append(version)
    return versions

def version_filepath(version: Version) -> Path:
    return Path("output") / f"fill_help_tables-{version.major}{version.minor}.sql"

def main():
    versions, concat_size = read_args()
    debug.success(f"Selected Versions: {versions}")

    Path("output").mkdir(exist_ok=True)
    for version in versions:
        debug.success(f"Generating Version: {version}")
        new_sql = generate_sql(version, concat_size-400) #makes room for line info around description
        version_filepath(version).write_text(new_sql)

if __name__ == "__main__":
    start = time.perf_counter()
    main()
    taken = time.perf_counter() - start
    debug.time_info(f"Took {taken:.2f}s")
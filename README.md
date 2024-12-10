
![project logo](image.png)

# Teledisk Analyzer

A command-line tool to analyze Teledisk disk image files.

## Overview

This tool walks through the directories given on the command line, searching for `.td0` files, including those within Zip archives and tarballs. It prints track and sector information and attempts to locate a CP/M format directory.

## Features

- **File Support**: Analyzes `.td0` files and extracts information from them.
- **Archive Handling**: Supports scanning within Zip and tarball archives.
- **Track and Sector Info**: Prints detailed information about tracks and sectors.
- **CP/M Directory Detection**: Attempts to locate and analyze CP/M formatted directories.

## Planned Features

- **Support for Additional Formats**: Extend the tool to identy FAT and other disk image formats.
- **Improved Command-Line Interface**: Enhance the command-line interface with more options.

## Installation

To install this tool, clone the repository and build it using Cargo:

```bash
git clone https://github.com/yourusername/your-repo-name.git
cd your-repo-name
cargo build --release
```

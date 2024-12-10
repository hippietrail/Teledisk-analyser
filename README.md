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

- **Enhanced Error Handling**: Implement more robust error handling for file reading and parsing to improve user experience and debugging.
- **Support for Additional Formats**: Extend the tool to support additional disk image formats beyond `.td0`, making it more versatile.
- **Improved Command-Line Interface**: Enhance the command-line interface with more options and better help messages for users.
- **Graphical User Interface (GUI)**: Consider developing a GUI for users who prefer a visual approach to analyzing disk images.
- **Unit Tests and Documentation**: Increase test coverage and improve inline documentation for better maintainability and understanding of the codebase.

## Future Development Roadmap

Based on TODOs and code analysis, the following features are planned for future development:

- **Enhanced Error Handling**: Implement more robust error handling for file reading and parsing to improve user experience and debugging.
- **Support for Additional Formats**: Extend the tool to support additional disk image formats beyond `.td0`, making it more versatile.
- **Improved Command-Line Interface**: Enhance the command-line interface with more options and better help messages for users.
- **Graphical User Interface (GUI)**: Consider developing a GUI for users who prefer a visual approach to analyzing disk images.
- **Unit Tests and Documentation**: Increase test coverage and improve inline documentation for better maintainability and understanding of the codebase.

## Installation

To install this tool, clone the repository and build it using Cargo:

```bash
git clone https://github.com/yourusername/your-repo-name.git
cd your-repo-name
cargo build --release
```
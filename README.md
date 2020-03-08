# plurigrep

## Brief description

A tool for finding groups of lines in text, where regular expressions from a given list match on nearby lines

## Installation

    stack install

--- this builds and copies the executable into `~/.local/bin/`

## Usage

    plurigrep - find groups of neighboring lines in text, in which matches occur for all REGEXes from a given set

    Usage: cat someText.txt | plurigrep [-r|--radius RADIUS] [--no-color] [REGEX...]
      This prints matching lines 

    Available options:
      -r,--radius RADIUS       Radius (default 5)
      --no-color               Turn off color highlighting
      -h,--help                Show this help text

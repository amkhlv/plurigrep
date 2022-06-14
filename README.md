# plurigrep

## Brief description

A tool for finding groups of lines in text, where regular expressions from a given list match on nearby lines

## Installation

    cargo install --path .

--- this builds and copies the executable into `~/.cargo/bin/`

## Usage

    plurigrep [OPTIONS] [REGEXEN]...

    ARGS:
        <REGEXEN>...    
    
    OPTIONS:
            --completion         Generate bash completion
            --debug              
        -h, --help               Print help information
        -m, --margin <MARGIN>    [default: 4]
            --nocolor            
            --nosep              
        -r, --radius <RADIUS>    [default: 8]
            --sep <SEPARATOR>    [default:
                                 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━]
        -V, --version            Print version information


## Encoding problems

Many text files are incorrectly encoded. In this case:

    cat someText.txt | iconv -c | plurigrep ...

(see `man iconv`)

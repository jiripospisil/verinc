use std::{
    env,
    fs::{read_to_string, write},
    process::exit,
};

use verinc::{Position, Version};

fn usage() {
    eprintln!(
        "Usage: verinc [flags] <file>

Increments X.Y.Z version in the given file.

Options:
 -h, --help        print this help
 -l, --list        list versions found in the file, use the index with --position
 -p, --position    position of the version to increment or \"all\" (defaults to 0)
 -s, --stdout      do not modify file in-place but print to stdout
 --major           increment major version
 --minor           increment minor version
 --patch           increment patch version (default)

Examples
 # Increment patch version of the first version found in-place
 verinc foo.txt

 # List all versions found together with their index
 verinc --list foo.txt

 # Increment major version of the third version found and print to stdout
 verinc --major --stdout --position 2 file
"
    );
}

fn list_versions(content: &str) {
    for (idx, ver) in verinc::list_versions(content).iter().enumerate() {
        println!("{idx}: {ver}");
    }
}

fn inc(position: Position, version: Version, content: &str) -> String {
    verinc::inc(content, position, version)
}

fn error(msg: &str) {
    eprintln!("{}", msg);
    exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.is_empty() {
        return usage();
    }

    let mut list = false;
    let mut stdout = false;
    let mut position = Position::Nth(0);
    let mut version = Version::Patch;

    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        if arg == "-h" || arg == "--help" {
            return usage();
        }

        if arg == "-s" || arg == "--stdout" {
            stdout = true;
            continue;
        }

        if arg == "-l" || arg == "--list" {
            list = true;
            continue;
        }

        if arg == "-p" || arg == "--position" {
            if let Some(pos) = iter.next() {
                if pos == "all" {
                    position = Position::All;
                } else {
                    position = match pos.parse::<u32>() {
                        Ok(num) => Position::Nth(num),
                        Err(_) => return error("Error: Invalid position!"),
                    }
                }
                continue;
            } else {
                return error("Error: Missing position!");
            }
        }

        if arg == "--major" {
            version = Version::Major;
            continue;
        }

        if arg == "--minor" {
            version = Version::Minor;
            continue;
        }

        if arg == "--patch" {
            version = Version::Patch;
            continue;
        }

        if arg.starts_with('-') {
            return usage();
        }

        if let Ok(content) = read_to_string(arg) {
            if list {
                return list_versions(&content);
            } else {
                let result = inc(position, version, &content);

                if stdout {
                    return println!("{}", result);
                }

                return write(arg, result.as_bytes()).unwrap();
            }
        } else {
            return error(&format!("Error: Cannot open file '{}'!", arg));
        }
    }

    usage();
}

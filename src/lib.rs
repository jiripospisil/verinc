//! Verinc is a small command line utility (and a library) which increments version
//! numbers in the given file. The primary use case for this is maintenance of my
//! Arch Linux packages.

use std::io::{stdout, IsTerminal};

use regex::{Regex, Replacer};

#[derive(Debug)]
pub enum Position {
    All,
    Nth(u32),
}

#[derive(Debug)]
pub enum Version {
    Major,
    Minor,
    Patch,
}

#[derive(Debug)]
struct Replace {
    idx: u32,
    position: Position,
    version: Version,
}

impl Replace {
    fn new(position: Position, version: Version) -> Self {
        Replace {
            idx: 0,
            position,
            version,
        }
    }
}

impl Replacer for Replace {
    fn replace_append(&mut self, caps: &regex::Captures<'_>, dst: &mut String) {
        let mut major = caps["major"].parse::<u32>().unwrap();
        let mut minor = caps["minor"].parse::<u32>().unwrap();
        let mut patch = caps["patch"].parse::<u32>().unwrap();

        if matches!(self.position, Position::Nth(n) if n == self.idx)
            || matches!(self.position, Position::All)
        {
            let old_major = major;
            let old_minor = minor;
            let old_path = patch;

            match self.version {
                Version::Major => {
                    major += 1;
                    minor = 0;
                    patch = 0;
                }
                Version::Minor => {
                    minor += 1;
                    patch = 0;
                }
                Version::Patch => patch += 1,
            }

            if stdout().is_terminal() {
                println!("{old_major}.{old_minor}.{old_path} -> {major}.{minor}.{patch}");
            }
        }

        self.idx += 1;

        dst.push_str(&format!("{}.{}.{}", major, minor, patch));
    }
}

const REGEX: &str = r"(?<major>0|[1-9]\d*)\.(?<minor>0|[1-9]\d*)\.(?<patch>0|[1-9]\d*)";

/// Finds a version in `hay` at `position` and increments one of its components according
/// to `version`.
pub fn inc(hay: &str, position: Position, version: Version) -> String {
    Regex::new(REGEX)
        .unwrap()
        .replace_all(hay, Replace::new(position, version))
        .to_string()
}

/// Returns a list of all recognized versions in `hay`.
pub fn list_versions(hay: &str) -> Vec<&str> {
    Regex::new(REGEX)
        .unwrap()
        .find_iter(hay)
        .map(|m| m.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_versions() {
        assert_eq!(
            inc("foo bar baz", Position::Nth(1), Version::Patch),
            "foo bar baz"
        );
    }

    #[test]
    fn patch() {
        assert_eq!(inc("1.0.0", Position::Nth(0), Version::Patch), "1.0.1");
        assert_eq!(inc("1.0.0", Position::All, Version::Patch), "1.0.1");

        assert_eq!(
            inc("1.0.0 foo 1.0.0", Position::Nth(0), Version::Patch),
            "1.0.1 foo 1.0.0"
        );
        assert_eq!(
            inc("1.0.0 1.0.0", Position::All, Version::Patch),
            "1.0.1 1.0.1"
        );

        assert_eq!(
            inc("1.0.0 1.0.0", Position::Nth(1), Version::Patch),
            "1.0.0 1.0.1"
        );
    }

    #[test]
    fn minor() {
        assert_eq!(inc("1.0.0", Position::Nth(0), Version::Minor), "1.1.0");
        assert_eq!(inc("1.0.1", Position::Nth(0), Version::Minor), "1.1.0");
        assert_eq!(inc("1.0.0", Position::All, Version::Minor), "1.1.0");
        assert_eq!(inc("1.0.1", Position::All, Version::Minor), "1.1.0");

        assert_eq!(
            inc("1.0.0 1.0.0", Position::Nth(0), Version::Minor),
            "1.1.0 1.0.0"
        );
        assert_eq!(
            inc("1.0.0 1.0.0", Position::All, Version::Minor),
            "1.1.0 1.1.0"
        );
        assert_eq!(
            inc("1.0.1 1.0.2", Position::Nth(0), Version::Minor),
            "1.1.0 1.0.2"
        );
        assert_eq!(
            inc("1.0.2 1.0.1", Position::All, Version::Minor),
            "1.1.0 1.1.0"
        );

        assert_eq!(
            inc("1.0.0 1.2.1", Position::Nth(1), Version::Minor),
            "1.0.0 1.3.0"
        );
    }

    #[test]
    fn major() {
        assert_eq!(inc("1.0.0", Position::Nth(0), Version::Major), "2.0.0");
        assert_eq!(inc("1.0.1", Position::Nth(0), Version::Major), "2.0.0");
        assert_eq!(inc("1.0.0", Position::All, Version::Major), "2.0.0");
        assert_eq!(inc("1.0.1", Position::All, Version::Major), "2.0.0");

        assert_eq!(
            inc("1.0.0 1.0.0", Position::Nth(0), Version::Major),
            "2.0.0 1.0.0"
        );
        assert_eq!(
            inc("1.0.0 1.0.0", Position::All, Version::Major),
            "2.0.0 2.0.0"
        );
        assert_eq!(
            inc("3.0.1 1.0.2", Position::Nth(0), Version::Major),
            "4.0.0 1.0.2"
        );
        assert_eq!(
            inc("3.0.2 1.0.1", Position::All, Version::Major),
            "4.0.0 2.0.0"
        );

        assert_eq!(
            inc("1.0.0 1.2.1", Position::Nth(1), Version::Major),
            "1.0.0 2.0.0"
        );
    }

    #[test]
    fn leading_zeros() {
        assert_eq!(
            inc("1.01.0 12.13.14", Position::Nth(0), Version::Major),
            "1.01.0 13.0.0"
        );
    }

    #[test]
    fn multiline() {
        assert_eq!(
            inc(
                "1.1.0\nhello\nworld\n12.13.14",
                Position::Nth(1),
                Version::Minor
            ),
            "1.1.0\nhello\nworld\n12.14.0"
        );
    }
}

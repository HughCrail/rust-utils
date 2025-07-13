use anyhow::{Result, anyhow};
use chrono::{Datelike, NaiveDateTime, NaiveTime};
use clap::{Parser, Subcommand};
use home::home_dir;
use regex::Regex;
use std::{
    env,
    fs::{canonicalize, metadata},
    path::PathBuf,
};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(alias = "j")]
    Journal,
    #[command(alias = "n")]
    New {
        title: String,
        extension: String,
        #[arg(long, short)]
        signature: Option<String>,
        #[arg(long, short)]
        keywords: Vec<String>,
    },
}

/// Represents the component type for trimming.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Component {
    Title,
    Keyword,
    Identifier,
    Signature,
}

const IDENTIFIER_DATE_FMT: &str = "%Y%m%dT%H%M%S";

fn main() -> Result<()> {
    let cli = Cli::parse();
    let dir_path = get_valid_notes_dir()?;
    match cli.command {
        Command::Journal => {
            let now = chrono::Local::now();
            let start_week = now.date_naive().week(chrono::Weekday::Mon).first_day();
            let id =
                NaiveDateTime::new(start_week, NaiveTime::default()).format(IDENTIFIER_DATE_FMT);
            let iso_week = now.iso_week();

            let fname = Note::new(
                id.to_string(),
                format!("Week {} of {}", iso_week.week(), iso_week.year()),
                None,
                vec!["journal".to_string()],
                "md".to_string(),
            )
            .format_file_name();
            let path = dir_path.join(&fname).to_string_lossy().into_owned();
            println!("{path}");
        }
        Command::New {
            title,
            signature,
            keywords,
            extension,
        } => {
            let fname = Note::new_now(title, signature, keywords, extension).format_file_name();
            println!("{fname}");
        }
    }
    Ok(())
}

#[derive(Debug)]
struct Note {
    identifier: String,
    title: String,
    signature: Option<String>,
    keywords: Vec<String>,
    extension: String,
}

impl Note {
    fn new_now(
        title: String,
        signature: Option<String>,
        keywords: Vec<String>,
        extension: String,
    ) -> Self {
        let now = chrono::Local::now().naive_local().format("%Y%m%dT%H%M%S");
        Note::new(now.to_string(), title, signature, keywords, extension)
    }
    fn new(
        identifier: String,
        title: String,
        signature: Option<String>,
        keywords: Vec<String>,
        extension: String,
    ) -> Self {
        Note {
            identifier,
            title,
            signature,
            keywords,
            extension,
        }
    }

    fn format_file_name(&self) -> String {
        let mut fname = sluggify_and_apply_rules(&self.identifier, Component::Identifier);
        let slugified_signature = self
            .signature
            .as_ref()
            .map(|s| sluggify_and_apply_rules(s, Component::Signature));
        if let Some(sig) = slugified_signature {
            fname.push_str(&format!("=={sig}"));
        }
        let slugified_title = sluggify_and_apply_rules(&self.title, Component::Title);
        fname.push_str(&format!("--{slugified_title}"));
        if !self.keywords.is_empty() {
            let slugified_keywords = self
                .keywords
                .iter()
                .map(|k| sluggify_and_apply_rules(k, Component::Keyword))
                .collect::<Vec<_>>()
                .join("_");
            fname.push_str(&format!("__{slugified_keywords}"));
        }
        fname.push_str(&format!(".{}", self.extension));
        fname
    }
}

fn get_notes_dir() -> Result<PathBuf> {
    env::var("NOTES_DIR").map(PathBuf::from).or_else(|_| {
        home_dir()
            .map(|home| home.join("notes"))
            .ok_or_else(|| anyhow!("Failed to get notes directory. Please set the NOTES_DIR environment variable or ensure the home directory is accessible."))
    })
}

fn get_valid_notes_dir() -> Result<PathBuf> {
    let dir = get_notes_dir()?;
    let dirmeta = metadata(&dir).map_err(|_| {
        anyhow!(
            "The specified notes directory does not exist or is not accessible: {:?}",
            dir
        )
    })?;
    if !dirmeta.is_dir() {
        Err(anyhow!(
            "The specified notes directory is not a valid directory: {:?}",
            dirmeta
        ))
    } else {
        canonicalize(&dir)
            .map_err(|_| anyhow!("Failed to canonicalize the notes directory path: {:?}", dir))
    }
}

fn sluggify_and_apply_rules(s: &str, component: Component) -> String {
    let slug = match component {
        Component::Title => sluggify_title(s),
        Component::Keyword => sluggify_keyword(s),
        Component::Identifier => valid_identifier(s),
        Component::Signature => sluggify_signature(s),
    };
    let slug = remove_dot_characters(&slug);
    let slug = replace_consecutive_token_characters(&slug, component);
    trim_right_token_characters(&slug, component)
}

// ][{}!@#$%^&*()+'\"?,.|;:~`‘’“”/
const ILLEGAL_CHARS: [char; 29] = [
    ']', '[', '{', '}', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '+', '\'', '"', '?', ',',
    '.', '|', ';', ':', '~', '`', '‘', '’', '“', '”',
];

fn sluggify_title(title: &str) -> String {
    slug_hyphenate(&title.replace(ILLEGAL_CHARS, "").replace('=', "")).to_lowercase()
}

fn sluggify_signature(title: &str) -> String {
    slug_put_equals(&title.replace(ILLEGAL_CHARS, "").replace('-', "")).to_lowercase()
}

fn sluggify_keyword(keyword: &str) -> String {
    keyword
        .replace(ILLEGAL_CHARS, "")
        .replace([' ', '-', '=', '_'], "")
        .to_lowercase()
}

/// Removes all non-ASCII characters from `s` and replaces them with spaces.
/// This is useful as a helper function to construct slug-like names.
pub fn slug_keep_only_ascii(s: &str) -> String {
    s.chars() // Iterate over each character in the string slice
        .map(|c| {
            if c.is_ascii_graphic() || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect() // Collect the characters into a new String
}

/// Replaces spaces and underscores with hyphens in `s`.
/// Also replaces multiple hyphens with a single one and removes any
/// leading and trailing hyphens.
pub fn slug_hyphenate(s: &str) -> String {
    // Replace spaces and underscores with hyphens
    let s1 = s.replace([' ', '_'], "-");

    // Replace multiple hyphens with a single one
    let re_multi_hyphen = Regex::new(r"-{2,}").unwrap();
    let s2 = re_multi_hyphen.replace_all(&s1, "-").to_string();

    // Remove leading and trailing hyphens
    let re_leading_trailing_hyphen = Regex::new(r"^-|-$").unwrap();
    re_leading_trailing_hyphen.replace_all(&s2, "").to_string()
}

/// Replaces spaces and underscores with equals signs in `s`.
/// Also replaces multiple equals signs with a single one and removes any
/// leading and trailing equals signs.
pub fn slug_put_equals(s: &str) -> String {
    // Replace spaces and underscores with equals signs
    let s1 = s.replace([' ', '_'], "=");

    // Replace multiple equals signs with a single one
    let re_multi_equals = Regex::new(r"={2,}").unwrap();
    let s2 = re_multi_equals.replace_all(&s1, "=").to_string();

    // Remove leading and trailing equals signs
    let re_leading_trailing_equals = Regex::new(r"^-|=$").unwrap();

    re_leading_trailing_equals.replace_all(&s2, "").to_string()
}

/// Ensures that `identifier` is valid.
/// It must not contain square brackets, parentheses, "query-filenames:",
/// or "query-contents:".
pub fn valid_identifier(identifier: &str) -> String {
    // Remove "query-filenames:"
    let s1 = identifier.replace("query-filenames:", "");

    // Remove "query-contents:"
    let s2 = s1.replace("query-contents:", "");

    // Remove square brackets and parentheses
    // The regex `[][()]+` matches one or more occurrences of `[`, `]`, `(`, or `)`.
    let re_brackets_parentheses = Regex::new(r"[\[]()]+").unwrap();

    re_brackets_parentheses.replace_all(&s2, "").to_string()
}

/// Removes dot characters from `str`.
pub fn remove_dot_characters(s: &str) -> String {
    s.replace(".", "") // Replace all occurrences of '.' with an empty string
}

/// Removes `=`, `-`, `_`, and `@` from the end of `s`.
/// The removal is done only if necessary according to `component`.
pub fn trim_right_token_characters(s: &str, component: Component) -> String {
    let pattern = if component == Component::Title {
        // For 'title', remove '=', '@', '_', '+'
        // The `+` makes it match one or more of these characters
        Regex::new(r"[=@_]+$").unwrap()
    } else {
        // For other components, remove '=', '@', '-', '_'
        // The `+` makes it match one or more of these characters
        Regex::new(r"[=@_-]+$").unwrap()
    };

    // `replace_all` with an empty string effectively removes the matched characters
    pattern.replace_all(s, "").to_string()
}

/// Replaces consecutive characters with a single one in `s`.
/// Hyphens, underscores, equal signs, and at signs are replaced with
/// a single one in `s`, if necessary according to `component`.
pub fn replace_consecutive_token_characters(s: &str, component: Component) -> String {
    // Regex for multiple underscores: __ -> _
    let re_multi_underscore = Regex::new(r"_{2,}").unwrap();
    let s1 = re_multi_underscore.replace_all(s, "_").to_string();

    // Regex for multiple equals signs: == -> =
    let re_multi_equals = Regex::new(r"={2,}").unwrap();
    let s2 = re_multi_equals.replace_all(&s1, "=").to_string();

    // Regex for multiple at signs: @@ -> @
    let re_multi_at = Regex::new(r"@{2,}").unwrap();
    let s3 = re_multi_at.replace_all(&s2, "@").to_string();

    // Conditional handling for hyphens based on component
    if component == Component::Title {
        s3 // Hyphens (--) are allowed in titles, so no replacement
    } else {
        // Regex for multiple hyphens: -- -> -
        let re_multi_hyphen = Regex::new(r"-{2,}").unwrap();
        re_multi_hyphen.replace_all(&s3, "-").to_string()
    }
}

use std::ffi::OsStr;
use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;

use pulldown_cmark::HeadingLevel;

use clap::Parser;
use eyre::{eyre, Result};

use serde::{Deserialize, Serialize};

use threatmd::{MarkdownIter, MarkdownParser};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The directory storing the markdown threats
    threat_dir: PathBuf,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct MetaField {
    sid: String,
    severity: String,
    target: Vec<String>,
    likelihood: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Threat {
    #[serde(rename = "SID")]
    sid: String,
    severity: String,
    target: Vec<String>,
    description: String,
    details: String,
    example: String,
    mitigations: String,
    condition: String,
    references: String,
    prerequisites: String,
    #[serde(rename = "Likelihood Of Attack")]
    likelihood: String,
}

fn parse_md(markdown_input: &str) -> Result<Threat> {
    let p = MarkdownParser::new(markdown_input);
    let mut parser = p.iter();

    let metadata = parser.metadata()?;

    let metadata: MetaField = serde_yaml::from_str(&metadata)?;

    let description = parser.heading(HeadingLevel::H1)?;

    let details = p.to_string(parser.multi(MarkdownIter::text));

    parser.named_heading(HeadingLevel::H2, "Example")?;
    let example = p.to_string(parser.multi(MarkdownIter::text));

    parser.named_heading(HeadingLevel::H2, "Mitigations")?;
    let mitigations = p.to_string(parser.multi(MarkdownIter::text));

    parser.named_heading(HeadingLevel::H2, "Condition")?;
    let condition = p.get_text(parser.lang_block("python")?);

    parser.named_heading(HeadingLevel::H2, "Prerequisites")?;
    let prerequisites = p.to_string(parser.multi(MarkdownIter::text));
    parser.named_heading(HeadingLevel::H2, "References")?;
    let refernces = parser.item_list()?;
    Ok(Threat {
        sid: metadata.sid,
        severity: metadata.severity,
        target: metadata.target,
        likelihood: metadata.likelihood,
        description,
        details,
        example,
        mitigations,
        condition,
        prerequisites,
        references: refernces.join(", "),
    })
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut threats = vec![];

    let mut stdout = io::stdout().lock();

    if !cli.threat_dir.is_dir() {
        return Err(eyre!("The given path was not a directory"));
    }

    for entry in cli
        .threat_dir
        .read_dir()
        .expect("expected to the directory to be readable.")
    {
        if let Ok(entry) = entry {
            let entry = entry.path();
            if Some(OsStr::new("md")) == entry.extension() {
                let md_input = read_to_string(&entry)?;
                threats.push(parse_md(&md_input)?);
            }
        }
    }

    serde_json::to_writer_pretty(&mut stdout, &threats)?;

    Ok(())
}

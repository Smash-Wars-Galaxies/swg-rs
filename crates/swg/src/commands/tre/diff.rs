use clap::{Args, ValueEnum};
use itertools::Itertools;
use miette::{miette, Context, IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use similar::{ChangeTag, TextDiff};
use std::{
    cmp::Ordering,
    collections::HashSet,
    fmt::Display,
    fs::File,
    io::{Cursor, Read, Seek},
    path::PathBuf,
};
use swg_stf::{read::StringTableReader, types::StringTable};
use swg_tre::TreArchive;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    #[default]
    Symantic,
    Full,
}

#[derive(Debug, Eq, PartialEq)]
enum Change {
    Added(String, String),
    Removed(String, String),
    Comparison(String, String, String),
    Context(Vec<String>),
    Modified(String, String, Vec<Change>, Vec<Change>),
}

impl Change {
    pub fn with_children(&mut self, children: Vec<Change>) -> Result<()> {
        match self {
            Change::Modified(_, _, vec, _) => {
                children.into_iter().for_each(|c| vec.push(c));
                vec.sort();
                Ok(())
            }
            _ => Err(miette!("tried to add children to an addition or removal")),
        }
    }

    pub fn with_related(&mut self, related: Vec<Change>) -> Result<()> {
        match self {
            Change::Modified(_, _, _, vec) => {
                related.into_iter().for_each(|c| vec.push(c));
                vec.sort();
                Ok(())
            }
            _ => Err(miette!("tried to add related to an addition or removal")),
        }
    }
}

impl Ord for Change {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Change {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Change::Added(key, value) => match other {
                Change::Added(other_key, other_value) => {
                    key.partial_cmp(other_key).and_then(|ord| {
                        value
                            .partial_cmp(other_value)
                            .map(|ord_val| ord.then(ord_val))
                    })
                }
                _ => None,
            },
            Change::Removed(key, value) => match other {
                Change::Removed(other_key, other_value) => {
                    key.partial_cmp(other_key).and_then(|ord| {
                        value
                            .partial_cmp(other_value)
                            .map(|ord_val| ord.then(ord_val))
                    })
                }
                _ => None,
            },
            Change::Comparison(key, _, _) => match other {
                Change::Comparison(other_key, _, _) => key.partial_cmp(other_key),
                _ => None,
            },
            Change::Context(_) => None,
            Change::Modified(key, value, children, _) => match other {
                Change::Modified(other_key, other_value, other_children, _) => {
                    key.partial_cmp(other_key).and_then(|ord| {
                        value
                            .partial_cmp(other_value)
                            .map(|ord_val| ord.then(ord_val))
                            .and_then(|ord| {
                                children
                                    .partial_cmp(other_children)
                                    .map(|ord_children| ord.then(ord_children))
                            })
                    })
                }
                _ => None,
            },
        }
    }
}

impl Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Change::Added(_, v) => {
                writeln!(f, "✅ {}", v.green())
            }
            Change::Removed(_, v) => {
                writeln!(f, "❌ {}", v.red())
            }
            Change::Comparison(key, old, new) => {
                writeln!(f, "* {}: {} vs {}", key, old.red(), new.green())
            }
            Change::Context(values) => {
                writeln!(
                    f,
                    "{}",
                    values
                        .iter()
                        .map(|l| String::from_iter((0..1).map(|_| ' ').chain(l.chars())))
                        .join("\n")
                )
            }
            Change::Modified(_, v, children, related) => {
                let mut txt_final = related.iter().map(|c| format!("{}", c)).join("");

                let mut section = String::new();
                let mut current_key = String::new();
                for c in children {
                    let key = match c {
                        Change::Added(key, _) => format!("* {} added:\n", key),
                        Change::Removed(key, _) => format!("* {} removed:\n", key),
                        Change::Modified(key, _, _, _) => format!("* {} modified:\n", key),
                        _ => current_key.clone(),
                    };

                    if current_key != key {
                        if !section.is_empty() {
                            txt_final.push_str(
                                &section.split("\n").map(|l| "  ".to_string() + l).join("\n"),
                            );
                            txt_final.push('\n');
                        }
                        section.clear();

                        txt_final.push_str(&key);
                        current_key = key
                    }

                    section.push_str(&format!("{}\n", c));
                }

                txt_final.push_str(&section.split("\n").map(|l| "  ".to_string() + l).join("\n"));

                writeln!(f, "🔃 {}", v.blue())?;
                writeln!(
                    f,
                    "{}",
                    txt_final
                        .split("\n")
                        .filter(|l| l.trim().len() > 1)
                        .map(|l| "  ".to_string() + l)
                        .join("\n")
                )
            }
        }
    }
}

#[derive(Args)]
pub struct DiffArgs {
    /// An input TRE file
    #[arg(short, long, value_name = "FILE")]
    left: PathBuf,

    /// An input TRE file
    #[arg(short, long, value_name = "FILE")]
    right: PathBuf,

    /// Comparison mode
    #[arg(short, long, value_enum, default_value_t=Mode::Symantic)]
    mode: Mode,
}

impl DiffArgs {
    fn handle_stf_file(&self, left: &StringTable, right: &StringTable) -> Result<Vec<Change>> {
        let mut result = Vec::new();

        // Find Added Entries
        right
            .keys()
            .filter(|k| !left.contains_key(k.as_str()))
            .map(|k| Change::Added("entries".into(), k.to_string()))
            .for_each(|c| result.push(c));

        // Find Removed Entries
        left.keys()
            .filter(|k| !right.contains_key(k.as_str()))
            .map(|k| Change::Removed("entries".into(), k.to_string()))
            .for_each(|c| result.push(c));

        // Find Differences
        left.keys()
            .filter(|k| right.contains_key(k.as_str()))
            .sorted()
            .filter_map(|k| {
                let old = left
                    .get_key_value(k)
                    .and_then(|(_, v)| v.to_string().ok())
                    .unwrap_or("".into());

                let new = right
                    .get_key_value(k)
                    .and_then(|(_, v)| v.to_string().ok())
                    .unwrap_or("".into());

                let diff = TextDiff::from_lines(&old, &new);
                if diff.ratio() < 1.0 {
                    let mut comparison = Vec::new();
                    if self.mode == Mode::Full {
                        for op in diff.ops().iter() {
                            for change in diff.iter_inline_changes(op) {
                                let mut context = String::new();
                                for (emphasized, value) in change.iter_strings_lossy() {
                                    if emphasized {
                                        if change.tag() == ChangeTag::Insert {
                                            context.push_str(&format!(
                                                "{}",
                                                value.green().underline()
                                            ));
                                        } else {
                                            context
                                                .push_str(&format!("{}", value.red().underline()));
                                        }
                                    } else {
                                        context.push_str(&format!("{}", value.dimmed()));
                                    }
                                }
                                comparison.push(context);
                            }
                        }
                    }
                    Some(Change::Modified(
                        "entries".into(),
                        k.into(),
                        vec![],
                        vec![Change::Context(comparison)],
                    ))
                } else {
                    None
                }
            })
            .for_each(|c| result.push(c));

        Ok(result)
    }

    fn handle_file<'a>(
        &self,
        name: &'a str,
        left: &'a Vec<u8>,
        right: &'a Vec<u8>,
    ) -> Result<Option<Change>> {
        let mut result: Option<Change> = None;

        if left.len() != right.len() {
            result
                .get_or_insert(Change::Modified(
                    "files".into(),
                    name.into(),
                    Vec::new(),
                    Vec::new(),
                ))
                .with_related(vec![Change::Comparison(
                    "size".into(),
                    left.len().to_string(),
                    right.len().to_string(),
                )])?;
        }

        if name.ends_with(".stf") {
            let stf_left = StringTableReader::decode(Cursor::new(left))?;
            let stf_right = StringTableReader::decode(Cursor::new(right))?;
            let changes = self.handle_stf_file(&stf_left, &stf_right)?;
            if !changes.is_empty() {
                result
                    .get_or_insert(Change::Modified(
                        "files".into(),
                        name.into(),
                        Vec::new(),
                        Vec::new(),
                    ))
                    .with_children(changes)?;
            }
        }

        Ok(result)
    }

    fn handle_tre<'a, R: Read + Seek>(
        &self,
        name: &'a str,
        left: &'a mut TreArchive<R>,
        right: &'a mut TreArchive<R>,
    ) -> Result<Option<Change>> {
        let mut result: Option<Change> = None;

        if left.len() != right.len() {
            result
                .get_or_insert(Change::Modified(
                    "tre".into(),
                    name.into(),
                    Vec::new(),
                    Vec::new(),
                ))
                .with_related(vec![Change::Comparison(
                    "entries".into(),
                    left.len().to_string(),
                    right.len().to_string(),
                )])?;
        }

        if self.mode == Mode::Full {
            if left.get_record_compression() != right.get_record_compression() {
                result
                    .get_or_insert(Change::Modified(
                        "tre".into(),
                        name.into(),
                        Vec::new(),
                        Vec::new(),
                    ))
                    .with_related(vec![Change::Comparison(
                        "record compression".into(),
                        left.get_record_compression().to_string(),
                        right.get_record_compression().to_string(),
                    )])?;
            }

            if left.get_record_block_size() != right.get_record_block_size() {
                result
                    .get_or_insert(Change::Modified(
                        "tre".into(),
                        name.into(),
                        Vec::new(),
                        Vec::new(),
                    ))
                    .with_related(vec![Change::Comparison(
                        "record block size".into(),
                        left.get_record_block_size().to_string(),
                        right.get_record_block_size().to_string(),
                    )])?;
            }

            if left.get_name_block_size() != right.get_name_block_size() {
                result
                    .get_or_insert(Change::Modified(
                        "tre".into(),
                        name.into(),
                        Vec::new(),
                        Vec::new(),
                    ))
                    .with_related(vec![Change::Comparison(
                        "name block size".into(),
                        left.get_name_block_size().to_string(),
                        right.get_name_block_size().to_string(),
                    )])?;
            }
        }

        let left_names = left
            .file_names()
            .map(|s| s.to_owned())
            .collect::<HashSet<_>>();
        let right_names = right
            .file_names()
            .map(|s| s.to_owned())
            .collect::<HashSet<_>>();

        let mut all_names = left_names.union(&right_names).collect::<Vec<_>>();
        all_names.sort();

        let files_added: Vec<Change> = all_names
            .iter()
            .copied()
            .filter(|f| right_names.contains(f.as_str()) && !left_names.contains(f.as_str()))
            .map(|k| Change::Added("files".into(), k.to_string()))
            .collect();

        if !files_added.is_empty() {
            result
                .get_or_insert(Change::Modified(
                    "tre".into(),
                    name.into(),
                    Vec::new(),
                    Vec::new(),
                ))
                .with_children(files_added)?;
        }

        let files_removed: Vec<Change> = all_names
            .iter()
            .copied()
            .filter(|f| !right_names.contains(f.as_str()) && left_names.contains(f.as_str()))
            .map(|k| Change::Removed("files".into(), k.to_string()))
            .collect();

        if !files_removed.is_empty() {
            result
                .get_or_insert(Change::Modified(
                    "tre".into(),
                    name.into(),
                    Vec::new(),
                    Vec::new(),
                ))
                .with_children(files_removed)?;
        }

        let files_shared = all_names
            .iter()
            .copied()
            .filter(|f| right_names.contains(f.as_str()) && left_names.contains(f.as_str()))
            .collect::<Vec<_>>();

        if !files_shared.is_empty() {
            for file in &files_shared {
                let mut data_left = Vec::new();
                let mut file_left = left.by_name(file)?;
                std::io::copy(&mut file_left, &mut data_left).into_diagnostic()?;

                let mut data_right = Vec::new();
                let mut file_right = right.by_name(file)?;
                std::io::copy(&mut file_right, &mut data_right).into_diagnostic()?;

                let file_modified = self.handle_file(file, &data_left, &data_right)?;
                if let Some(c) = file_modified {
                    result
                        .get_or_insert(Change::Modified(
                            "tre".into(),
                            name.into(),
                            Vec::new(),
                            Vec::new(),
                        ))
                        .with_children(vec![c])?;
                }
            }
        }

        Ok(result)
    }

    pub fn handle(&self) -> Result<()> {
        let l = File::open(&self.left)
            .into_diagnostic()
            .context(format!("path: {}", &self.left.display()))?;

        let mut left = TreArchive::new(&l)?;

        let r = File::open(&self.right)
            .into_diagnostic()
            .context(format!("path: {}", &self.right.display()))?;

        let mut right = TreArchive::new(&r)?;

        let difference = self.handle_tre(&self.left.to_string_lossy(), &mut left, &mut right)?;

        if let Some(d) = difference {
            println!("{}", d);
        }

        Ok(())
    }
}

use clap::{Args, ValueEnum};
use miette::{Context, IntoDiagnostic, Result};
use owo_colors::{
    colors::{Green, Red},
    OwoColorize, Style,
};
use similar::{ChangeTag, TextDiff};
use std::{collections::HashSet, fs::File, io::Cursor, path::PathBuf};
use swg_stf::read::StringTableReader;
use swg_tre::TreArchive;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    #[default]
    Symantic,
    Contents,
    Full,
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
    pub fn handle(&self) -> Result<()> {
        let l = File::open(&self.left)
            .into_diagnostic()
            .context(format!("path: {}", &self.left.display()))?;

        let mut left = TreArchive::new(&l)?;

        let r = File::open(&self.right)
            .into_diagnostic()
            .context(format!("path: {}", &self.right.display()))?;
        let mut right = TreArchive::new(&r)?;

        if left.len() != right.len() {
            println!(
                "* entries: {} | {}",
                left.len().fg::<Red>(),
                right.len().fg::<Red>()
            );
        }

        if self.mode == Mode::Full {
            if left.get_record_compression() != right.get_record_compression() {
                println!(
                    "* record compression: {} | {}",
                    left.get_record_compression().fg::<Red>(),
                    right.get_record_compression().fg::<Red>()
                );
            }

            if left.get_record_block_size() != right.get_record_block_size() {
                println!(
                    "* record block size: {} | {}",
                    left.get_record_block_size().fg::<Red>(),
                    right.get_record_block_size().fg::<Red>()
                );
            }

            if left.get_name_block_size() != right.get_name_block_size() {
                println!(
                    "* name block size: {} | {}",
                    left.get_name_block_size().fg::<Red>(),
                    right.get_name_block_size().fg::<Red>()
                );
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

        let files_added = all_names
            .iter()
            .copied()
            .filter(|f| right_names.contains(f.as_str()) && !left_names.contains(f.as_str()))
            .collect::<Vec<_>>();

        let files_removed = all_names
            .iter()
            .copied()
            .filter(|f| !right_names.contains(f.as_str()) && left_names.contains(f.as_str()))
            .collect::<Vec<_>>();

        if !files_added.is_empty() {
            println!("* files added:");
            for f in &files_added {
                println!("\t* {}", f.fg::<Green>());
            }
        }

        if !files_removed.is_empty() {
            println!("* files removed:");
            for f in &files_removed {
                println!("\t* {}", f.fg::<Red>());
            }
        }

        let files_shared = all_names
            .iter()
            .copied()
            .filter(|f| right_names.contains(f.as_str()) && left_names.contains(f.as_str()))
            .collect::<Vec<_>>();

        if !files_shared.is_empty() {
            let mut files_modified = String::new();
            for file in &files_shared {
                let mut data_left = Vec::new();
                let mut file_left = left.by_name(file)?;
                std::io::copy(&mut file_left, &mut data_left).into_diagnostic()?;

                let mut data_right = Vec::new();
                let mut file_right = right.by_name(file)?;
                std::io::copy(&mut file_right, &mut data_right).into_diagnostic()?;

                let mut file_modified = String::new();

                if data_left.len() != data_right.len() {
                    file_modified.push_str(&format!(
                        "\t\t* size: {} | {}\n",
                        data_left.len().fg::<Red>(),
                        data_right.len().fg::<Red>()
                    ));
                }

                if self.mode != Mode::Symantic && file.ends_with(".stf") {
                    let stf_left = StringTableReader::decode(Cursor::new(data_left))?;
                    let stf_right = StringTableReader::decode(Cursor::new(data_right))?;

                    println!("{:#?}", serde_json::to_string(&stf_left).unwrap());

                    let entries_added = stf_right
                        .iter()
                        .filter(|(k, _)| !stf_left.contains_key(k.as_str()))
                        .collect::<Vec<_>>();
                    if !entries_added.is_empty() {
                        file_modified.push_str("\t\t* entries added:\n");
                        for (k, v) in &entries_added {
                            file_modified.push_str(&format!(
                                "\t\t\t* {}: {}\n",
                                k.fg::<Green>(),
                                v.display().fg::<Green>()
                            ));
                        }
                    }

                    let entries_removed = stf_left
                        .iter()
                        .filter(|(k, _)| !stf_right.contains_key(k.as_str()))
                        .collect::<Vec<_>>();
                    if !entries_removed.is_empty() {
                        file_modified.push_str("\t\t* entries removed:\n");
                        for (k, v) in &entries_removed {
                            file_modified.push_str(&format!(
                                "\t\t\t* {}: {}\n",
                                k.fg::<Red>(),
                                v.display().fg::<Red>()
                            ));
                        }
                    }

                    let entries_shared = stf_left
                        .keys()
                        .filter(|k| stf_right.contains_key(k.as_str()))
                        .collect::<Vec<_>>();
                    if !entries_shared.is_empty() {
                        let mut entries_modified = String::new();

                        for key in entries_shared {
                            let old = stf_left
                                .get_key_value(key)
                                .and_then(|(_, v)| v.to_string().ok())
                                .unwrap_or("".into());

                            let new = stf_right
                                .get_key_value(key)
                                .and_then(|(_, v)| v.to_string().ok())
                                .unwrap_or("".into());

                            let diff = TextDiff::from_lines(&old, &new);
                            if diff.ratio() < 1.0 {
                                entries_modified.push_str(&format!("\t\t\t* {}\n", key));
                            }

                            if self.mode == Mode::Full {
                                for group in diff.grouped_ops(0).iter() {
                                    for op in group {
                                        entries_modified.push_str("\t\t\t  ");
                                        for change in diff.iter_inline_changes(op) {
                                            let (_, s) = match change.tag() {
                                                ChangeTag::Delete => ("-", Style::new().red()),
                                                ChangeTag::Insert => ("+", Style::new().green()),
                                                ChangeTag::Equal => (" ", Style::new().dimmed()),
                                            };
                                            for (emphasized, value) in change.iter_strings_lossy() {
                                                if emphasized {
                                                    entries_modified.push_str(&format!(
                                                        "{}",
                                                        value.style(s).underline()
                                                    ));
                                                } else {
                                                    entries_modified
                                                        .push_str(&format!("{}", value.style(s)));
                                                }
                                            }
                                            entries_modified.push_str("\n\t\t\t  ");
                                        }
                                    }
                                    entries_modified.push('\n');
                                }
                            }
                        }

                        if !entries_modified.is_empty() {
                            file_modified.push_str("\t\t* entries modified\n");
                            file_modified.push_str(&entries_modified);
                        }
                    }
                }

                if !file_modified.is_empty() {
                    files_modified.push_str(&format!("\t* {}\n", file));
                    files_modified.push_str(&file_modified);
                }
            }

            if !files_modified.is_empty() {
                println!(" * files modified:");
                println!("{}", files_modified);
            }
        }

        Ok(())
    }
}

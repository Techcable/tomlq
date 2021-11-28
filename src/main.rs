use std::{env, process::{self, Command}};
use std::path::{PathBuf, Path};
use std::io::{self, Read, Write};
use std::ffi::{OsString};

use itertools::Itertools;
use thiserror::Error;

#[derive(Debug, Copy, Clone)]
enum TargetMode {
    File,
    LiteralStrings,
    Stdin
}
#[derive(Debug, Copy, Clone)]
enum TargetFormat {
    Toml,
    Yaml,
}
impl TargetFormat {
    fn parse_into_json<R: Read>(&self, mut input: R) -> anyhow::Result<serde_json::Value> {
        match *self {
            TargetFormat::Toml => {
                let mut buf = String::new();
                input.read_to_string(&mut buf)?;
                Ok(toml::from_str::<serde_json::Value>(&buf)?)
            },
            TargetFormat::Yaml => {
                Ok(serde_yaml::from_reader::<_, serde_json::Value>(input)?)
            }
        }
    }
    #[inline]
    fn name(&self) -> &'static str {
        match *self {
            TargetFormat::Toml => "toml",
            TargetFormat::Yaml => "yaml"
        }
    }
    fn infer_from_ext(p: &Path) -> Result<TargetFormat, AmbiguousExtension> {
        let ext = p.extension().ok_or_else(|| AmbiguousExtension::Missing { path: p.to_owned() })?;
        if ext == "toml" {
            Ok(TargetFormat::Toml)
        } else if ext == "yaml" {
            Ok(TargetFormat::Yaml)
        } else {
            Err(AmbiguousExtension::Unknown { ext: ext.to_owned() })
        }
    }
}
#[derive(Debug, Error)]
enum AmbiguousExtension {
    #[error("File has no extension: {}", path.display())]
    Missing {
        path: PathBuf
    },
    #[error("Unknown file extension: {}", ext.to_string_lossy())]
    Unknown {
        ext: OsString
    }
}


fn main() -> anyhow::Result<()> {
    let mut args = env::args().peekable();
    args.next().unwrap();

    let mut flags =args.peeking_take_while(|s| s.starts_with("-") && s != "--").collect_vec();
    if args.peek().map(String::as_ref) == Some("--") {
        args.next().unwrap();
    }
    let mode = if flags.iter().find(|flag| *flag == "--args").is_some() {
        Some(TargetMode::LiteralStrings)
    } else if flags.iter().find(|flag| *flag == "--jsonargs").is_some() {
        eprintln!("ERROR: Option `--jsonargs` not supported");
        process::exit(1);
    } else {
        None
    };
    let mut format = None;
    if let Some(idx) = flags.iter().position(|flag| flag == "--toml") {
        flags.remove(idx);
        format = Some(TargetFormat::Toml);
    }
    if let Some(idx) = flags.iter().position(|flag| flag == "--yaml") {
        flags.remove(idx);
        if let Some(existing) = format {
            eprintln!("ERROR: Conflicting options --yaml and --{}", existing.name());
            process::exit(1);
        }
        format = Some(TargetFormat::Yaml);
    }
    let command = args.next().unwrap_or_else(|| {
        eprintln!("ERROR: Must specify the `jq` command to execute");
        process::exit(1);
    });
    let targets = args.collect_vec();
    let mode = if targets.is_empty() {
        if let Some(explicit) = mode {
            eprintln!("ERROR: Cannot speicfy explicit {:?} mode without specifying any targets!", explicit);
            process::exit(1);
        } else {
            TargetMode::Stdin
        }
    } else if let Some(explicit) = mode {
        explicit
    } else {
        TargetMode::File
    };
    let format = match format {
        Some(explicit) => explicit,
        None => {
            if matches!(mode, TargetMode::File) {
                TargetFormat::infer_from_ext(Path::new(&targets[0]))?
            } else {
                eprintln!("ERROR: Must specify explicit --yaml or --toml mode for {:?} mode", mode);
                process::exit(1);
            }
        }
    };
    let converted = convert_to_json(mode, format, &targets)?;
    let mut builder = Command::new("jq");
    builder.args(&flags);
    builder.arg(&*command);
    builder.stdin(process::Stdio::null()); // NOTE: May override later
    match converted {
        ConvertedInput::TempFiles { ref files } => {
            builder.args(files.iter().map(|f| f.to_path_buf()));
        },
        ConvertedInput::LiteralStrings { ref literals } => {
            builder.args(literals.iter());
        },
        ConvertedInput::Stdin { .. } => {
            builder.stdin(process::Stdio::piped());
            // NOTE: We'll finish this later
        }
    }
    let mut child = builder.spawn()?;
    if let ConvertedInput::Stdin { ref text } = converted {
        let mut stdin = child.stdin.take().unwrap();
        eprintln!("Text: {}", text);
        stdin.write_all(text.as_bytes())?;
        drop(stdin); // close it
    }
    let status = child.wait()?;
    drop(converted); // Drop the tempfiles
    if let Some(code) = status.code() {
        process::exit(code);
    }
    Ok(())
}


fn convert_to_json(mode: TargetMode, format: TargetFormat, targets: &[String]) -> anyhow::Result<ConvertedInput> {
    match mode {
        TargetMode::File => {
            let paths = targets.iter().map(|s| PathBuf::from(&**s)).collect::<Vec<_>>();
            let mut tempfiles = Vec::new();
            for p in &paths {
                let input = std::io::BufReader::new(std::fs::File::open(p)?);
                let json = format.parse_into_json(input)?;
                let mut tempfile = tempfile::Builder::new()
                    .prefix("tomlq")
                    .suffix(".json")
                    .tempfile()?;
                serde_json::to_writer(tempfile.as_file_mut(), &json)?;
                tempfiles.push(tempfile.into_temp_path());
            }
            Ok(ConvertedInput::TempFiles { files: tempfiles })
        },
        TargetMode::LiteralStrings => {
            let mut translated = Vec::new();
            for target in targets {
                let val = format.parse_into_json(target.as_bytes())?;
                translated.push(serde_json::to_string(&val)?);
            }
            Ok(ConvertedInput::LiteralStrings { literals: translated })
        },
        TargetMode::Stdin => {
            let input = io::stdin();
            let val = format.parse_into_json(input)?;
            Ok(ConvertedInput::Stdin { text: serde_json::to_string(&val)? })
        }
    }
}

enum ConvertedInput {
    TempFiles {
        files: Vec<tempfile::TempPath>
    },
    LiteralStrings {
        literals: Vec<String>
    },
    Stdin {
        text: String
    }
}

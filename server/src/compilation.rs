use std::{
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use landlock::{Compatible, RulesetError};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{info, warn};
use crate::config;

pub async fn compile(
    source_dir: impl AsRef<Path>,
    document_path: &PathBuf,
    compiler: Option<Compiler>,
) -> CompilationResult {
    let output_dir = tempfile::tempdir().unwrap();

    let output_path = output_dir.path().join("dilladocs_output.pdf");

    // Determine compiler
    let compiler = compiler.unwrap_or_else(|| {
        if document_path.ends_with("typ") {
            return Compiler::Typst;
        } else if document_path.ends_with("tex") {
            return Compiler::PdfLatex;
        }
        warn!("Couldn't determine file format for {:?}", &document_path);
        return Compiler::PdfLatex;
    });

    // Execute
    fs::create_dir_all("typst_package_cache").await.unwrap();
    let log = compiler.compile(
        document_path,
        &source_dir,
        &output_path,
        "typst_package_cache",
    ); // TODO: make cache configurable

    // Format result
    let output: Result<Vec<u8>, _> = fs::read(output_path).await;
    let output = match output {
        Err(err) => {
            warn!("Error reading compilation output: {}", err);
            for file in output_dir.path().read_dir().unwrap() {
                println!("- {:?}", file)
            }
            None
        }
        Ok(data) => {
            info!("Read {} bytes", data.len());
            Some(data)
        }
    };

    CompilationResult {
        log: log.unwrap_or("Execution failed".into()),
        document: output,
    }
}

impl Compiler {
    /// Compiles [input_file] in [input_dir] to [output_file] returning the compilation log.
    ///
    /// - input_file must be subpath of input_dir
    /// - on error output_file may not exist
    pub fn compile(
        &self,
        input_file: impl AsRef<std::path::Path>,
        input_dir: impl AsRef<std::path::Path>,
        output_file: impl AsRef<std::path::Path>,
        typst_package_cache: impl AsRef<std::path::Path>,
    ) -> Option<String> {
        let config = config::Config::get();
        let input_file = input_file
            .as_ref()
            .canonicalize()
            .expect("compile input doesn't exist");
        let input_dir = input_dir
            .as_ref()
            .canonicalize()
            .expect("input dir doesn't exist");
        let typst_package_cache = typst_package_cache
            .as_ref()
            .canonicalize()
            .expect("typst cache dir doesn't exist");
        assert!(input_file.starts_with(&input_dir));
        let output_file = output_file.as_ref();
        let output_dir = output_file.parent()?;
        assert!(output_file.is_absolute());
        assert!(output_file.starts_with(&output_dir));
        assert!(!output_file.exists());
        info!("Writing output to: {:?}", &output_file);

        // Setup command
        // TODO: implement other compilers, cleanup lifetime madness
        let typst_package_cache = typst_package_cache
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();
        let log: String = match self {
            Compiler::PdfLatex | Compiler::XeLatex | Compiler::LuaLatex => {
                let output_file = output_file.file_stem()?;
                let mut raw_command = Command::new(self.binary_location());
                let mut command = raw_command.env_clear().current_dir(&input_dir).args([
                    "-output-directory",
                    output_dir.to_str().unwrap(),
                    "-jobname",
                    output_file.to_str().unwrap(),
                    //"-output-format=pdf",
                    &input_file.as_os_str().to_str().unwrap(),
                ]);
                /*
                if config.sandbox_compilations {
                    let output_dir = output_dir.to_path_buf();
                    unsafe {
                        command = command.pre_exec(move || {
                            restrict(
                                &input_dir,
                                &output_dir,
                                &typst_package_cache,
                                self.binary_location(),
                                true,
                            )
                                .unwrap();
                            Ok(())
                        })
                    }
                }
                */
                command
                    .output()
                    .map(|output| String::from_utf8(output.stdout).unwrap())
                    .unwrap_or("Output capture failed".into())
            }
            Compiler::Typst => {
                let mut raw_command = Command::new("/usr/bin/typst");
                let mut command = raw_command.env_clear().current_dir(&input_dir).args([
                    "compile",
                    &input_file.as_os_str().to_str().unwrap(),
                    &output_file.as_os_str().to_str().unwrap(),
                    "--package-cache-path",
                    &typst_package_cache,
                ]);
                if config.sandbox_compilations {
                    let output_dir = output_dir.to_path_buf();
                    unsafe {
                        command = command.pre_exec(move || {
                            restrict(
                                &input_dir,
                                &output_dir,
                                &typst_package_cache,
                                "/usr/bin/typst",
                                false,
                            )
                            .unwrap();
                            Ok(())
                        })
                    }
                }
                command
                    .output()
                    .map(|output| {
                        let mut res = String::from_utf8(output.stdout).unwrap();
                        res.push_str(&String::from_utf8(output.stderr).unwrap());
                        res
                    })
                    .unwrap_or("Output capture failed".into())
            }
        };

        Some(log)
    }

    fn binary_location(&self) -> &'static str {
        match self {
            Compiler::Typst => "/usr/bin/typst",
            Compiler::XeLatex => "/usr/bin/xelatex",
            Compiler::LuaLatex => "/usr/bin/lualatex",
            Compiler::PdfLatex => "/usr/bin/pdflatex",
        }
    }
}

fn restrict(
    input_dir: &PathBuf,
    output_dir: &PathBuf,
    cache_dir: &str,
    binary: &str,
    no_net: bool,
) -> Result<(), RulesetError> {
    use landlock::{
        ABI, Access, AccessFs, AccessNet, Ruleset, RulesetAttr, RulesetCreatedAttr,
        path_beneath_rules,
    };

    let abi = ABI::V4;
    let mut ruleset = Ruleset::default()
        .set_compatibility(landlock::CompatLevel::HardRequirement)
        .handle_access(AccessFs::from_all(abi))?;
    if no_net {
        ruleset = ruleset.handle_access(AccessNet::from_all(abi))?;
    }
    let _ruleset = ruleset
        .create()?
        .set_no_new_privs(true)
        // typst packages need that for now: .add_rule(AccessNet::ConnectTcp)?
        .add_rules(path_beneath_rules(
            &[
                &PathBuf::from_str("/usr/share/").unwrap(),
                &PathBuf::from_str("/etc/").unwrap(),
                &PathBuf::from_str("/usr/lib/").unwrap(),
                &PathBuf::from_str("/var/lib/texfm/").unwrap(),
                &input_dir,
            ],
            AccessFs::from_read(abi),
        ))?
        .add_rules(path_beneath_rules(
            &[output_dir],
            AccessFs::from_write(abi) | AccessFs::from_read(abi),
        ))?
        .add_rules(path_beneath_rules(
            &["/dev/null", binary, cache_dir],
            AccessFs::from_all(abi),
        ))?
        .restrict_self();

    //info!("Applied ruleset: {:?}", ruleset);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Compiler {
    PdfLatex,
    XeLatex,
    LuaLatex,
    Typst,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompilationResult {
    pub log: String,
    pub document: Option<Vec<u8>>,
}

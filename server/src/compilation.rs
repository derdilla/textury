#[cfg(not(feature = "fireframe"))]
use std::path::PathBuf;

#[cfg(not(feature = "fireframe"))]
pub async fn compile_insecure(
    source_dir: impl AsRef<std::path::Path>,
    document_path: &PathBuf,
    compiler: Compiler,
) -> CompilationResult {
    use tokio::fs;
    use tracing::{info, warn};

    let output_dir = tempfile::tempdir().unwrap();

    let output_path = output_dir.path().join("textury_output.pdf");

    // Execute
    fs::create_dir_all("typst_package_cache").await.unwrap();
    let log = compiler.compile_insecure(
        document_path,
        &source_dir,
        &output_path,
        "typst_package_cache",
    );

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
        log: log.unwrap_or("Error: Execution failed".into()),
        document: output,
    }
}

impl Compiler {
    /// Command specifically for running in the sandbox
    #[cfg(feature = "fireframe")]
    pub fn build_remote_script(&self, relative_input: &str) -> Option<String> {
        let compiler = self.alpine_binary_location();
        let command = match self {
            Compiler::PdfLatex | Compiler::XeLatex | Compiler::LuaLatex => {
                build_latex_command(compiler, &format!("$PROJECT_DIR/{}", relative_input), "$OUTPUT_DIR", "textury_out")
            }
            Compiler::Typst => {
                build_typst_command(compiler, &format!("$PROJECT_DIR/{}", relative_input), "$OUTPUT_FILE", "typst_package_cache")
            }
        };
        Some(format!(r#"
            #!/bin/sh

            OUTPUT_DIR=$(mktemp -d)
            OUTPUT_FILE="$OUTPUT_DIR/textury_out.pdf"

            {} 2>&1
            mv $OUTPUT_FILE $PROJECT_DIR/textury_out.pdf
        "#, command))
    }

    /// Compiles [input_file] in [input_dir] to [output_file] returning the compilation log.
    ///
    /// - input_file must be subpath of input_dir
    /// - on error output_file may not exist
    #[cfg(not(feature = "fireframe"))]
    fn compile_insecure(
        &self,
        input_file: impl AsRef<std::path::Path>,
        input_dir: impl AsRef<std::path::Path>,
        output_file: impl AsRef<std::path::Path>,
        typst_package_cache: impl AsRef<std::path::Path>,
    ) -> Option<String> {
        use std::process::Command;
        use tracing::{info};

        // Basic safety checks
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
        assert!(output_file.starts_with(output_dir));
        assert!(!output_file.exists());
        info!("Writing output to: {:?}", &output_file);

        // Run compilation
        let typst_package_cache = typst_package_cache
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();
        let script = match self {
            Compiler::PdfLatex | Compiler::XeLatex | Compiler::LuaLatex => {
                let output_dir = output_file.parent()?;
                let output_file = output_file.file_stem()?;
                build_latex_command(self.binary_location(),
                                    input_file.to_str()?,
                                    output_dir.to_str().unwrap(),
                                    output_file.to_str().unwrap())
            }
            Compiler::Typst => build_typst_command(self.binary_location(),
                                                   input_file.to_str()?,
                                                   output_file.to_str()?,
                                                   &typst_package_cache)
        };
        let log = Command::new("sh")
            .env_clear()
            .current_dir(&input_dir)
            .arg("-c")
            .arg(script)
            .output()
            .map(|output| {
                let mut res = String::from_utf8_lossy(&output.stdout).to_string();
                res.push_str(String::from_utf8_lossy(&output.stderr).as_ref());
                res
            })
            .unwrap_or("Output capture failed".into());

        Some(log)
    }

    #[cfg(not(feature = "fireframe"))]
    #[cfg(not(any(feature = "alpine", feature = "macos")))]
    fn binary_location(&self) -> &'static str {
        match self {
            Compiler::Typst => "/usr/bin/typst",
            Compiler::XeLatex => "/usr/bin/xelatex",
            Compiler::LuaLatex => "/usr/bin/lualatex",
            Compiler::PdfLatex => "/usr/bin/pdflatex",
        }
    }

    #[cfg(not(feature = "fireframe"))]
    #[cfg(feature = "alpine")]
    fn binary_location(&self) -> &'static str { self.alpine_binary_location() }

    #[cfg(any(feature = "fireframe", feature = "alpine"))]
    fn alpine_binary_location(&self) -> &'static str {
        match self {
            Compiler::Typst => "/usr/bin/typst",
            Compiler::XeLatex => "/opt/texlive/bin/x86_64-linuxmusl/xelatex",
            Compiler::LuaLatex => "/opt/texlive/bin/x86_64-linuxmusl/lualatex",
            Compiler::PdfLatex => "/opt/texlive/bin/x86_64-linuxmusl/pdflatex",
        }
    }
}

fn build_latex_command(binary: &str, input_file: &str, output_dir: &str, output_basename: &str) -> String {
    //"-output-format=pdf",
    format!("{} -output-directory {} -jobname {} -interaction=nonstopmode {}", binary, output_dir, output_basename, input_file)
}

fn build_typst_command(binary: &str, input_file: &str, output_file: &str, package_cache: &str) -> String {
    format!("{binary} compile {input_file} {output_file} --package-cache-path {package_cache}")
}

#[derive(Debug)]
pub enum Compiler {
    PdfLatex,
    XeLatex,
    LuaLatex,
    Typst,
}

#[derive(Debug)]
pub struct CompilationResult {
    pub log: String,
    pub document: Option<Vec<u8>>,
}

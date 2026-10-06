impl Compiler {
    /// Command specifically for running in the sandbox
    pub fn build_remote_script(&self, relative_input: &str) -> Option<String> {
        let compiler = self.binary_name();
        let command = match self {
            Compiler::PdfLatex | Compiler::XeLatex | Compiler::LuaLatex => {
                build_latex_command(compiler, &format!("$PROJECT_DIR/{}", relative_input), "$OUTPUT_DIR", "textury_out")
            }
            Compiler::Typst => {
                build_typst_command(compiler, &format!("$PROJECT_DIR/{}", relative_input), "$OUTPUT_FILE")
            }
        };
        Some(format!(r#"
            #!/bin/sh

            OUTPUT_DIR=$(mktemp -d)
            OUTPUT_FILE="$OUTPUT_DIR/textury_out.pdf"

            # TODO: make this configurable (#87)
            ulimit -t 600
            ulimit -d 1000000

            {} 2>&1
            mv $OUTPUT_FILE $PROJECT_DIR/textury_out.pdf
        "#, command))
    }

    fn binary_name(&self) -> &'static str {
        match self {
            Compiler::Typst => "typst",
            Compiler::XeLatex => "xelatex",
            Compiler::LuaLatex => "lualatex",
            Compiler::PdfLatex => "pdflatex",
        }
    }
}

fn build_latex_command(binary: &str, input_file: &str, output_dir: &str, output_basename: &str) -> String {
    //"-output-format=pdf",
    format!("{} -output-directory {} -jobname {} -interaction=nonstopmode {}", binary, output_dir, output_basename, input_file)
}

fn build_typst_command(binary: &str, input_file: &str, output_file: &str) -> String {
    format!("{binary} compile {input_file} {output_file}")
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
    pub document: Option<Vec<u8>>,
}

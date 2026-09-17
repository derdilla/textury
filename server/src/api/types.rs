use crate::{app, compilation};

include!(concat!(env!("OUT_DIR"), "/api_types.rs"));

impl Into<UserId> for app::UserID {
    fn into(self) -> UserId {
        UserId(self.0)
    }
}

impl Into<ProjectId> for app::ProjectID {
    fn into(self) -> ProjectId {
        ProjectId(self.0)
    }
}

impl Into<compilation::Compiler> for Compiler {
    fn into(self) -> compilation::Compiler {
        match self {
            Compiler::PdfLatex => compilation::Compiler::PdfLatex,
            Compiler::XeLatex => compilation::Compiler::XeLatex,
            Compiler::LuaLatex => compilation::Compiler::LuaLatex,
            Compiler::Typst => compilation::Compiler::Typst,
        }
    }
}

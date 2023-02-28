mod csstree;
mod sass_spec;
mod suite;

use std::{
    env,
    path::{Path, PathBuf},
};

pub use csstree::{TestCsstreeCase, TestCsstreeSuite};
pub use sass_spec::{TestSassSpecCase, TestSassSpecSuite};
pub use suite::{Case, Suite};

pub fn get_project_root() -> PathBuf {
    Path::new(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(2)
    .unwrap()
    .to_path_buf()
}

#[derive(Default, Debug)]
pub struct CliArgs {
    pub filter: Option<String>,
    pub detail: bool,
    pub diff: bool,
}

impl CliArgs {
    pub fn should_print_detail(&self) -> bool {
        self.filter.is_some() || self.detail
    }
}

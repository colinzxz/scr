use std::path::{Path, PathBuf};

use crate::{
    get_project_root,
    suite::{Case, Suite, TestResult},
};

const PATH: &str = "tasks/coverage/sass-spec/spec";
const HRX_SECTION_SEPARATOR: &str = r"
<===>
================================================================================
";
const FILE_PREFIX: &str = "<===>";

pub struct TestSassSpecCase {
    path: PathBuf,
    virtual_path: Option<PathBuf>,
    code: String,
    result: TestResult,
}

impl TestSassSpecCase {}

impl Case for TestSassSpecCase {
    fn new(path: PathBuf, code: String, virtual_path: Option<PathBuf>) -> Self {
        Self { path, virtual_path, code, result: TestResult::ToBeRun }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn virtual_path(&self) -> &Option<PathBuf> {
        &self.virtual_path
    }

    fn test_result(&self) -> &TestResult {
        &self.result
    }

    fn should_fail(&self) -> bool {
        let path = self.path().to_string_lossy();

        let virtual_path_contain_error = if let Some(virtual_path) = &self.virtual_path {
            virtual_path.to_string_lossy().contains("error")
        } else {
            false
        };

        path.contains("error") || virtual_path_contain_error
    }

    fn run(&mut self) {
        self.result = self.execute();
    }
}

pub struct TestSassSpecSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> TestSassSpecSuite<T> {
    fn create_case_from_hrx(&self, cases: &mut Vec<T>, path: PathBuf, source: String) {
        source.split(HRX_SECTION_SEPARATOR).filter(|code| !code.is_empty()).for_each(|c| {
            let mut code = String::new();
            let mut is_case_file = false;
            let mut virtual_path: Option<PathBuf> = None;

            c.lines().for_each(|line| {
                if line.starts_with(FILE_PREFIX) {
                    if is_case_file {
                        cases.push(T::new(path.clone(), code.clone(), virtual_path.clone()));
                    }

                    is_case_file = false;
                    virtual_path = None;
                    code = String::new();

                    let file_path = Path::new(line.strip_prefix(FILE_PREFIX).unwrap().trim());
                    if file_path
                        .file_name()
                        .map_or(false, |name| name == "input.scss" || name == "input.sass")
                    {
                        is_case_file = true;
                        virtual_path = Some(file_path.to_path_buf());
                    }
                } else if is_case_file {
                    code += line;
                }
            });

            if is_case_file {
                cases.push(T::new(path.clone(), code.clone(), virtual_path.clone()));
            }
        });
    }
}

impl<T: Case> TestSassSpecSuite<T> {
    pub fn new() -> Self {
        Self { test_root: get_project_root().join(PATH), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for TestSassSpecSuite<T> {
    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }

    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn skip_test_path(&self, path: &Path) -> bool {
        path.to_string_lossy().contains("libsass")
            || path.extension().map_or(true, |ext| ext == "md" || ext == "yml")
    }

    fn save_test_cases(&mut self, cases: Vec<T>) {
        self.test_cases = cases
    }

    fn file_to_cases(&self, path: PathBuf, code: String) -> Vec<T> {
        let is_hrx = path.extension().map_or(false, |ext| ext == "hrx");
        let mut cases = vec![];

        if is_hrx {
            self.create_case_from_hrx(&mut cases, path, code)
        } else {
            cases.push(T::new(path, code, None));
        }

        cases
    }
}

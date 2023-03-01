use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::{
    get_project_root,
    suite::{Case, Suite, TestResult},
};

const CASES_PATH: &str = "tasks/coverage/postcss-parser-tests/cases";

pub struct TestPostcssCase {
    path: PathBuf,
    code: String,
    virtual_path: Option<String>,
    should_fail: bool,
    result: TestResult,
}

impl Case for TestPostcssCase {
    fn new(path: PathBuf, code: String, virtual_path: Option<String>) -> Self {
        Self { path, code, should_fail: false, virtual_path, result: TestResult::ToBeRun }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn virtual_path(&self) -> &Option<String> {
        &self.virtual_path
    }

    fn test_result(&self) -> &TestResult {
        &self.result
    }

    fn should_fail(&self) -> bool {
        self.should_fail
    }

    fn run(&mut self) {
        self.result = self.execute();
    }
}

pub struct TestPostcssSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> TestPostcssSuite<T> {
    pub fn new() -> Self {
        Self { test_root: get_project_root().join(CASES_PATH), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for TestPostcssSuite<T> {
    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }

    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn skip_test_path(&self, path: &Path) -> bool {
        path.extension().map_or(true, |ext| ext == "json")
    }

    fn save_test_cases(&mut self, cases: Vec<T>) {
        self.test_cases = cases
    }
}

use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::{
    get_project_root,
    suite::{Case, Suite, TestResult},
};

const FIXTURES_PATH: &str = "tasks/coverage/csstree/fixtures/ast";

#[derive(Debug, Default, Clone, Deserialize)]
pub struct CsstreeCase {
    pub source: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct CsstreeErrorCase {
    pub source: Option<String>,
    pub error: Option<String>,
}

/// block.json
#[derive(Debug, Default, Clone, Deserialize)]
#[serde()]
pub struct CsstreeAstJson {
    error: Option<Vec<CsstreeErrorCase>>,
}

pub struct TestCsstreeCase {
    path: PathBuf,
    code: String,
    virtual_path: Option<String>,
    result: TestResult,
}

impl TestCsstreeCase {}

impl Case for TestCsstreeCase {
    fn new(path: PathBuf, code: String, virtual_path: Option<String>) -> Self {
        Self { path, code, virtual_path, result: TestResult::ToBeRun }
    }

    fn should_fail(&self) -> bool {
        if let Some(p) = self.virtual_path() { p.starts_with("error #") } else { false }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }

    fn virtual_path(&self) -> &Option<String> {
        &self.virtual_path
    }

    fn test_result(&self) -> &TestResult {
        &self.result
    }

    fn run(&mut self) {
        self.result = self.execute();
    }
}

pub struct TestCsstreeSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> TestCsstreeSuite<T> {
    pub fn new() -> Self {
        Self { test_root: get_project_root().join(FIXTURES_PATH), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for TestCsstreeSuite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    #[allow(unused_variables)]
    fn skip_test_path(&self, path: &Path) -> bool {
        false
    }

    fn save_test_cases(&mut self, cases: Vec<T>) {
        self.test_cases = cases
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }

    fn file_to_cases(&self, path: PathBuf, source: String) -> Vec<T> {
        let json: Value = serde_json::from_str(source.as_str()).unwrap();
        let json = json.as_object().unwrap();
        let mut cases: Vec<T> = vec![];
        // println!("Json: {json:#?}");

        json.keys().into_iter().for_each(|key| {
            if key == "error" {
                if let Some(errors) = json.get(key).unwrap().as_array() {
                    let mut index = 0;
                    errors.iter().for_each(|error| {
                        index += 1;
                        let error = error.as_object().unwrap();
                        cases.push(T::new(
                            path.clone(),
                            error["source"].to_string(),
                            Some(format!("error #{index}").to_string()),
                        ));
                    });
                }
            } else {
                if let Some(case) = json.get(key).unwrap().as_object() {
                    cases.push(T::new(
                        path.clone(),
                        case["source"].to_string(),
                        Some(key.to_string()),
                    ));
                };
            }
        });

        cases
    }
}

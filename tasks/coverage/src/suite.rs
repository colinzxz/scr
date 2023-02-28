use std::{
    fs::{self, File},
    io::{self, stdout, Read, Write},
    panic::{catch_unwind, UnwindSafe},
    path::{Path, PathBuf},
};

use console::Style;
use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;
use similar::{ChangeTag, TextDiff};
use walkdir::WalkDir;

use crate::{get_project_root, CliArgs};

#[derive(Debug)]
pub enum TestResult {
    ToBeRun,
    Passed,
    IncorrectPassed,
    PassError(String),
    Mismatch(String, String),
    CorrectError(String),
}

pub trait Case: Sized + UnwindSafe {
    fn new(path: PathBuf, code: String, virtual_path: Option<PathBuf>) -> Self;
    fn code(&self) -> &str;
    fn path(&self) -> &Path;
    fn virtual_path(&self) -> &Option<PathBuf>;
    fn skip_test_case(&self) -> bool {
        false
    }
    fn should_fail(&self) -> bool {
        false
    }
    fn test_result(&self) -> &TestResult;

    fn run(&mut self);

    fn execute(&self) -> TestResult {
        // let source_text = self.code();
        // println!("source_text: {source_text:#?}");
        self.parser_rusult_to_test_result(Err(String::new()))
    }

    fn parser_rusult_to_test_result(&self, result: Result<String, String>) -> TestResult {
        let should_fail = self.should_fail();
        match result {
            Err(err) if should_fail => TestResult::CorrectError(err),
            Err(err) if !should_fail => TestResult::PassError(err),
            Ok(_) if should_fail => TestResult::IncorrectPassed,
            Ok(_) if !should_fail => TestResult::Passed,
            _ => unreachable!(),
        }
    }

    fn test_passed(&self) -> bool {
        let result = self.test_result();
        assert!(!matches!(result, TestResult::ToBeRun), "test should be run ");
        matches!(result, TestResult::Passed | TestResult::CorrectError(_))
    }

    fn test_parsed(&self) -> bool {
        let result = self.test_result();
        assert!(!matches!(result, TestResult::ToBeRun), "test should be run");
        matches!(result, TestResult::Passed | TestResult::Mismatch(_, _))
    }

    fn print<W: Write>(&self, args: &CliArgs, writer: &mut W) -> io::Result<()> {
        match self.test_result() {
            TestResult::PassError(error) => {
                writer.write_all(format!("Except to Parse: {:?}", self.path()).as_bytes())?;
                if self.virtual_path().is_some() {
                    writer.write_all(
                        format!(" -> {:?}\n", self.virtual_path().clone().unwrap()).as_bytes(),
                    )?;
                } else {
                    writer.write_all("\n".as_bytes())?;
                }
                writer.write_all(error.as_bytes())?;
            }
            TestResult::Mismatch(ast_string, except_string) => {
                if args.diff {
                    writer.write_all(format!("Mismatch: {:?}\n", self.path()).as_bytes())?;
                    self.print_diff(writer, &ast_string, &except_string)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn print_diff<W: Write>(
        &self,
        writer: &mut W,
        origin_str: &str,
        except_str: &str,
    ) -> io::Result<()> {
        let text_diff = TextDiff::from_lines(origin_str, except_str);
        for change in text_diff.iter_all_changes() {
            let (sign, style) = match change.tag() {
                ChangeTag::Delete => ("-", Style::new().red()),
                ChangeTag::Insert => ("+", Style::new().green()),
                ChangeTag::Equal => continue,
            };

            writer.write_all(
                format!("{}{}", style.apply_to(sign).bold(), style.apply_to(change)).as_bytes(),
            )?;
        }
        Ok(())
    }
}

#[allow(unused)]
pub struct CoverageReport<'a, T> {
    all_positives: usize,
    all_negatives: usize,
    parsed_positives: usize,
    passed_positives: usize,
    passed_negatives: usize,
    failed_positives: Vec<&'a T>,
    failed_negatives: Vec<&'a T>,
}

/// A Test Suite is responsible for reading code from a repository
pub trait Suite<T: Case> {
    fn get_test_root(&self) -> &Path;
    fn skip_test_path(&self, path: &Path) -> bool;
    fn save_test_cases(&mut self, cases: Vec<T>);
    fn get_test_cases(&self) -> &Vec<T>;

    fn file_to_cases(&self, path: PathBuf, code: String) -> Vec<T> {
        vec![T::new(path, code, None)]
    }

    fn run(&mut self, name: &str, args: &CliArgs) {
        self.read_test_cases(args);

        let report = self.coverage_report();

        let mut output = stdout();

        self.print_coverage(name, args, &report, &mut output).unwrap();

        if args.filter.is_none() {
            self.snapshot_errors(name, &report).unwrap();
        }
    }

    fn read_test_cases(&mut self, args: &CliArgs) {
        let filter = args.filter.as_deref();
        let test_root = self.get_test_root();

        // get all tests
        let paths = WalkDir::new(test_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_owned())
            .filter(|path| !self.skip_test_path(path))
            .filter(|path| filter.map_or(true, |query| path.to_string_lossy().contains(query)))
            .collect::<Vec<_>>();

        let cases = paths
            .into_iter()
            .map(|path| {
                let code = fs::read_to_string(&path).unwrap_or_else(|_| {
                    let file = File::open(&path).unwrap();
                    let mut content = String::new();
                    DecodeReaderBytesBuilder::new()
                        .encoding(Some(UTF_16LE))
                        .build(file)
                        .read_to_string(&mut content)
                        .unwrap();
                    content
                });
                let path = path.strip_prefix(test_root).unwrap().to_owned();
                self.file_to_cases(path, code)
            })
            .flat_map(|case| case)
            .filter(|case| !case.skip_test_case())
            .filter_map(|mut case| {
                let path = case.path().to_path_buf();
                catch_unwind(move || {
                    case.run();
                    Some(case)
                })
                .unwrap_or_else(|_| {
                    println!("panic: {path:?}");
                    None
                })
            })
            .collect::<Vec<_>>();

        self.save_test_cases(cases);
    }

    fn coverage_report(&self) -> CoverageReport<T> {
        let cases = self.get_test_cases();

        let (negatives, positives): (Vec<_>, Vec<_>) =
            cases.into_iter().partition(|case| case.should_fail());

        let all_positives = positives.len();
        let all_negatives = negatives.len();

        let not_parsed_positives = positives.iter().filter(|case| !case.test_parsed()).count();

        let parsed_positives = all_positives - not_parsed_positives;

        let failed_positives =
            positives.into_iter().filter(|case| !case.test_passed()).collect::<Vec<_>>();

        let passed_positives = all_positives - failed_positives.len();

        let failed_negatives =
            negatives.into_iter().filter(|case| !case.test_passed()).collect::<Vec<_>>();

        let passed_negatives = all_negatives - failed_negatives.len();

        CoverageReport {
            all_positives,
            all_negatives,
            parsed_positives,
            passed_positives,
            passed_negatives,
            failed_positives,
            failed_negatives,
        }
    }

    fn print_coverage<W: Write>(
        &self,
        name: &str,
        args: &CliArgs,
        report: &CoverageReport<T>,
        writer: &mut W,
    ) -> io::Result<()> {
        let CoverageReport {
            all_positives,
            // all_negatives,
            parsed_positives,
            //  pass_positives,
            //  pass_negatives,
            ..
        } = report;

        let parse_diff = (*parsed_positives as f64) / (*all_positives as f64) * 100.0;

        writer.write_all(format!("{name} Summary:\n").as_bytes())?;
        writer.write_all(
            format!("AST Parsed: {parsed_positives}/{all_positives} ({parse_diff:.2}%)\n")
                .as_bytes(),
        )?;

        if args.should_print_detail() {
            for case in &report.failed_positives {
                case.print(args, writer)?;
            }

            for case in &report.failed_negatives {
                case.print(args, writer)?;
            }
        }

        writer.flush()?;

        Ok(())
    }

    fn snapshot_errors(&self, name: &str, report: &CoverageReport<T>) -> io::Result<()> {
        let path = get_project_root().join(format!("tasks/coverage/{}.snap", name.to_lowercase()));
        let mut file = File::create(path).unwrap();

        let mut cases = self
            .get_test_cases()
            .iter()
            .filter(|case| matches!(case.test_result(), TestResult::CorrectError(_)))
            .collect::<Vec<_>>();

        cases.sort_by_key(|case| case.path().to_string_lossy().to_string());

        let args = CliArgs { detail: true, ..CliArgs::default() };

        self.print_coverage(name, &args, report, &mut file)?;

        let mut out = String::new();

        for case in &cases {
            if let TestResult::CorrectError(error) = &case.test_result() {
                out.push_str(error);
            }
        }

        file.write_all(out.as_bytes())?;
        file.flush()?;

        Ok(())
    }
}

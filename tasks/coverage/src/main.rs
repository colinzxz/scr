use pico_args::Arguments;
use scr_coverage::{CliArgs, Suite, TestSassSpecCase, TestSassSpecSuite};

fn main() {
    let mut args = Arguments::from_env();
    let command = args.subcommand().expect("subcommand");

    let args = CliArgs {
        filter: args.opt_value_from_str("--filter").unwrap(),
        detail: args.contains("--detail"),
        diff: args.contains("--diff"),
    };

    let task = command.as_deref().unwrap_or("default");

    let run_sass_spec = || {
        let mut suite = TestSassSpecSuite::<TestSassSpecCase>::new();
        suite.run("SassSpec", &args);
    };

    match task {
        "sass" => run_sass_spec(),
        _ => {
            run_sass_spec();
        }
    }
}

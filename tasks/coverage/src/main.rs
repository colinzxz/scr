use pico_args::Arguments;
use scr_coverage::{
    CliArgs, Suite, TestCsstreeCase, TestCsstreeSuite, TestPostcssCase, TestPostcssSuite,
    TestSassSpecCase, TestSassSpecSuite,
};

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
        TestSassSpecSuite::<TestSassSpecCase>::new().run("SassSpec", &args);
    };

    let run_csstree = || {
        TestCsstreeSuite::<TestCsstreeCase>::new().run("Csstree", &args);
    };

    let run_postcss = || {
        TestPostcssSuite::<TestPostcssCase>::new().run("Postcss", &args);
    };

    match task {
        "sass" => run_sass_spec(),
        "csstree" => run_csstree(),
        "postcss" => run_postcss(),
        _ => {
            run_sass_spec();
            run_csstree();
            run_postcss();
        }
    };
}

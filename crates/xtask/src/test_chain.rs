//! The per-tier test commands and the `:ci` chain from pragmatic-tdd's runner contract.

use std::process::{Command, ExitCode};
use std::str::FromStr;

use anyhow::bail;

/// A test tier. Only the tiers this repo has exist here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// `#[cfg(test)]` modules in lib and bin targets; test doubles only.
    Unit,
}

impl FromStr for Tier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "unit" => Ok(Tier::Unit),
            other => bail!("unknown tier `{other}`; this repo has: unit"),
        }
    }
}

impl Tier {
    fn profile(self) -> &'static str {
        match self {
            Tier::Unit => "unit",
        }
    }
}

/// Runs one tier. With `ci`, wraps it in [`ci_chain`].
pub fn run_tier(tier: Tier, ci: bool, passthrough: &[String]) -> ExitCode {
    let code = if ci {
        ci_chain(
            || setup(tier),
            || tests(tier, passthrough),
            || teardown(tier),
        )
    } else {
        tests(tier, passthrough)
    };
    ExitCode::from(u8::try_from(code).unwrap_or(1))
}

/// setup -> tests -> teardown. A failed setup skips the tests (and teardown, as in
/// `setup && { test; code=$?; teardown; exit $code; }`). Teardown always runs once the
/// tests have started, even when they fail, and the chain exits with the tests' code.
pub fn ci_chain(
    setup: impl FnOnce() -> i32,
    tests: impl FnOnce() -> i32,
    teardown: impl FnOnce() -> i32,
) -> i32 {
    let setup_code = setup();
    if setup_code != 0 {
        return setup_code;
    }
    let test_code = tests();
    let teardown_code = teardown();
    if teardown_code != 0 {
        eprintln!("xtask: teardown exited with {teardown_code}");
    }
    test_code
}

fn setup(tier: Tier) -> i32 {
    match tier {
        // The unit tier has no real dependency to stand up.
        Tier::Unit => 0,
    }
}

fn teardown(tier: Tier) -> i32 {
    match tier {
        Tier::Unit => 0,
    }
}

fn tests(tier: Tier, passthrough: &[String]) -> i32 {
    let mut nextest = Command::new(cargo());
    nextest
        .args([
            "nextest",
            "run",
            "--workspace",
            "--locked",
            "--profile",
            tier.profile(),
        ])
        .args(passthrough);
    let code = status(nextest);
    if code != 0 || tier != Tier::Unit {
        return code;
    }
    // nextest doesn't run doctests; they need no dependency, so they belong to the unit tier.
    let mut doctests = Command::new(cargo());
    doctests.args(["test", "--doc", "--workspace", "--locked"]);
    status(doctests)
}

fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".into())
}

fn status(mut command: Command) -> i32 {
    match command.status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(err) => {
            eprintln!("xtask: could not run {command:?}: {err}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    mod ci_chain {
        use super::*;

        fn run(setup: i32, tests: i32, teardown: i32) -> (i32, Vec<&'static str>) {
            let calls = RefCell::new(Vec::new());
            let code = ci_chain(
                || {
                    calls.borrow_mut().push("setup");
                    setup
                },
                || {
                    calls.borrow_mut().push("tests");
                    tests
                },
                || {
                    calls.borrow_mut().push("teardown");
                    teardown
                },
            );
            (code, calls.into_inner())
        }

        #[test]
        fn should_run_every_step_and_exit_zero_when_all_pass() {
            assert_eq!(run(0, 0, 0), (0, vec!["setup", "tests", "teardown"]));
        }

        #[test]
        fn should_run_teardown_and_exit_with_the_tests_code_when_tests_fail() {
            assert_eq!(run(0, 101, 0), (101, vec!["setup", "tests", "teardown"]));
        }

        #[test]
        fn should_exit_with_the_tests_code_when_only_teardown_fails() {
            assert_eq!(run(0, 0, 2), (0, vec!["setup", "tests", "teardown"]));
        }

        #[test]
        fn should_exit_with_the_tests_code_when_tests_and_teardown_fail() {
            assert_eq!(run(0, 101, 2), (101, vec!["setup", "tests", "teardown"]));
        }

        #[test]
        fn should_skip_the_tests_when_setup_fails() {
            assert_eq!(run(3, 0, 0), (3, vec!["setup"]));
        }
    }

    mod tier {
        use super::*;

        #[test]
        fn should_parse_unit() {
            assert_eq!("unit".parse::<Tier>().unwrap(), Tier::Unit);
        }

        #[test]
        fn should_reject_a_tier_this_repo_does_not_have() {
            assert!("e2e".parse::<Tier>().is_err());
        }
    }
}

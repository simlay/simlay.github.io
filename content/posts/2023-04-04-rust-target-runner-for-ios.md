+++
title = "Using rust's target runner for iOS simulators"
date = 2023-04-03
description = "This is a demonstration on using cargo run for iOS simulator targets"
draft = true
+++

# Introduction

I've become a pretty big fan for rust and cargo. I've found it quite convenient
to run `cargo run` or `cargo test`. This doesn't work quite as nicely if you're
cross compiling. Fortunately, there's the [`target.<tripple>.runner` cargo
feature](https://doc.rust-lang.org/cargo/reference/config.html#targettriplerunner).
Unfortunately, I've found it hard to find example uses of this. An [advanced
search on GitHub for `runner = ` in
`config.toml`](https://github.com/search?p=3&q=runner+%3D+language%3ATOML+filename%3Aconfig.toml&type=Code)
is how I've found other examples.

In this post, I demonstrate an iOS simulator target runner script. The way the
app's `Info.plist` is setup is liable to change and break in the future but
this works with `macOS 13.3`, `Xcode 14.1` targeted at the `iOS 16.1`
simulator.

# Prior work

I discovered [`cargo dinghy`](https://github.com/sonos/dinghy) a few years ago
and [added support for iOS simulator tests in
CI](https://github.com/sonos/dinghy/pull/96). My complaints about using dinghy
here are:
* CI maybe compiling dinghy and then compile
the project.
* Recalling `cargo dinghy --platform auto-ios-aarch64-sim test`


# Setup

To get started, you need to specify the [target runner for
cargo](https://doc.rust-lang.org/cargo/reference/config.html#targettriplerunner)

{{ source_code(
        path="code/2023-04-03-rust-target-runner-for-ios/.cargo/config.toml",
        source_type="rust")
 }}

This [cargo
config](https://doc.rust-lang.org/cargo/reference/config.html#hierarchical-structure)
tells cargo what script/executable to use when running `cargo run --target
aarch64-apple-ios-sim -- arg1 arg2`. The arguments to this script/executable
are the path to the target binary and then the arguments to said binary.

Here's the `ios-sim-runner.sh` script.
{{ source_code(
    path="code/2023-04-03-rust-target-runner-for-ios/ios-sim-runner.sh",
    source_type="sh")
 }}

# Usage

Put the `ios-sim-runner.sh` in your path or local to the project and the
`.cargo/config.toml` in desired cargo configuration and then you'll be able to
run:

`cargo test --target x86_64-apple-ios` or `cargo test --target
aarch64-apple-ios-sim` and you'll see something like the following:
```
   Compiling rust-target-runner-for-ios v0.1.0 (/Users/simlay/projects/simlay.github.io/code/2023-04-03-rust-target-runner-for-ios)
    Finished test [unoptimized + debuginfo] target(s) in 0.42s
     Running unittests src/main.rs (target/x86_64-apple-ios/debug/deps/rust_target_runner_for_ios-3d1ec4c5f726a109)

running 2 tests
test tests::it_works ... ok
test tests::it_no_work ... FAILED

failures:

---- tests::it_no_work stdout ----
thread 'tests::it_no_work' panicked at 'assertion failed: `(left != right)`
  left: `4`,
 right: `4`', src/main.rs:26:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::it_no_work

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `--bin rust-target-runner-for-ios`
```


# Description

Given that this script will certainly break in the future due to changes in either

# Future Work

In the future I'd like to use
[`ios-deploy`](https://github.com/ios-control/ios-deploy) to [deploy/run/test
over wifi](https://stackoverflow.com/a/23827549). The hard part about this is
the requirements and uses of [`codesign`ing the
app](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Procedures/Procedures.html)

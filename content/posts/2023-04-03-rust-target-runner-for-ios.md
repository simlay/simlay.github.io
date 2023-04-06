+++
title = "Using rust's target runner for iOS simulators"
date = 2023-04-03
description = "This is an example of using cargo run for iOS simulator targets"
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

The source code for this post is available in the [`code` subdirectory of this repo](https://github.com/simlay/simlay.github.io/tree/master/code) if you want to use it.

# Prior work

I discovered [`cargo dinghy`](https://github.com/sonos/dinghy) a few years ago
and [added support for iOS simulator tests in
CI](https://github.com/sonos/dinghy/pull/96). My complaints about using dinghy
here are:
* CI compiling dinghy (if you don't have it cached) and then compiling the
project. I've found this to result in longer CI times.
* Recalling the arguments to dinghy such as `cargo dinghy --platform auto-ios-aarch64-sim test`


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

With a simple rust `main.rs` or `lib.rs`:
{{ source_code(
    path="code/2023-04-03-rust-target-runner-for-ios/src/main.rs",
    source_type="rust")
 }}

Put the `ios-sim-runner.sh` in your path or local to the project and the
`.cargo/config.toml` in desired cargo configuration and then you'll be able to
run:

`cargo test --target x86_64-apple-ios` or `cargo test --target
aarch64-apple-ios-sim` and you'll see something like the following:
```
    Finished test [unoptimized + debuginfo] target(s) in 0.00s
     Running unittests src/main.rs (target/aarch64-apple-ios-sim/debug/deps/rust_target_runner_for_ios-21a08454287c0bcd)

running 2 tests
test tests::this_test_passes ... ok
test tests::this_test_fails ... FAILED

failures:

---- tests::this_test_fails stdout ----
thread 'tests::this_test_fails' panicked at 'assertion failed: `(left == right)`
  left: `4`,
 right: `5`', src/main.rs:26:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::this_test_fails

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--bin rust-target-runner-for-ios`
```

One can do `cargo test --target aarch64-apple-ios-sim -- this_test_passes` or
`cargo run --target aarch64-apple-ios-sim`


# Description

Given that this script will certainly break in the future due to changes with
either `simctl` or the `Info.plist` specifications so I'll tell you about the
sections.

## App Bundle

{{ source_code(
    path="code/2023-04-03-rust-target-runner-for-ios/ios-sim-runner.sh",
    source_type="sh",
    start_line=6,
    end_line=46
    )
 }}

This is the section to bundle the executable that's the first argument into an
iOS simulator app.

## Device ID

{{ source_code(
    path="code/2023-04-03-rust-target-runner-for-ios/ios-sim-runner.sh",
    source_type="sh",
    start_line=48,
    end_line=77
    )
 }}

This whole section is about getting the `DEVICE_ID` as this is needed for the
`get_app_container`. Otherwise, one can just use `booted` for most cases of
device id.

This will start an iOS simulator if one is not started.

## Start the app

{{ source_code(
    path="code/2023-04-03-rust-target-runner-for-ios/ios-sim-runner.sh",
    source_type="sh",
    start_line=79,
    end_line=92
    )
 }}

Here we install the app bundle and start the app in waiting/debugger mode as
we'll later use the `PID` to retrieve the exit status of the app and propagate
the status code up.

## LLDB

{{ source_code(
    path="code/2023-04-03-rust-target-runner-for-ios/ios-sim-runner.sh",
    source_type="sh",
    start_line=93,
    end_line=101
    )
 }}
{{ source_code(
    path="code/2023-04-03-rust-target-runner-for-ios/ios-sim-runner.sh",
    source_type="sh",
    start_line=101,
    end_line=124
    )
 }}
{{ source_code(
    path="code/2023-04-03-rust-target-runner-for-ios/ios-sim-runner.sh",
    source_type="sh",
    start_line=124
    )
 }}

Here we:
* create a very simple [`lldb` script](https://lldb.llvm.org/man/lldb.html#cmdoption-lldb-source)
* run said lldb script
* read the `stdout`, parse it, and retrieve the exit status.
* exit with the status code.

# Future Work

In the future I'd like to use
[`ios-deploy`](https://github.com/ios-control/ios-deploy) to [deploy/run/test
over wifi](https://stackoverflow.com/a/23827549). The hard part about this is
the requirements and uses of [`codesign`ing the
app](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Procedures/Procedures.html)

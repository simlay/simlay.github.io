+++
title = "Writing iOS XCTests in Rust"
date = 2026-01-13
description = "Use objc2-xc-ui-test in an iOS app"
+++

# Introduction


In this post, I'll briefly show how to bundle a rust binary into an iOS app as
well as the complexities of bundling a XCTest app written in rust. I can't say
that this should be used in production as I only figured out how to do this a
few months (~November 2025) so this might be a brittle setup.

The code here is available in [`code` subdirectory of this repo](https://github.com/simlay/simlay.github.io/tree/master/code) if you want to use it.


# Background

While `XCTest` has been around since [Xcode
5](https://developer.apple.com/documentation/xctest), `XCUIAutomation` wasn't
it's own framework nor in the public framework list until [Xcode
16.3](https://developer.apple.com/documentation/xcode-release-notes/xcode-16_3-release-notes#Testing-and-Automation).
When it was added as a public API (rather than private). This change enabled
[objc2](https://github.com/madsmtm/objc2) to generate bindings for
[`objc2-xc-test`](https://crates.io/crates/objc2-xc-test) and
[`objc2-xc-ui-automation`](https://crates.io/crates/objc2-xc-ui-automation)
headers and generating some pretty ergonomic bindings for apple frameworks.
These two rust crates call into the Objective-C APIs for `XCTest` and
`XCUIAutomation`.

Note: These bindings are generated based on [targeting macOS for
now](https://github.com/madsmtm/objc2/issues/408). [Various work arounds for
some calls have been added](https://github.com/madsmtm/objc2/pull/809).


# Rust iOS app bundling hacks

In my previous post about [using a target runner for
ios](/posts/rust-target-runner-for-ios/), the bulk of the exercise was about
just bundling a rust executable into an iOS app. In short, an iOS app is a
directory suffixed with `.app`, an `Info.plist` and a binary.

The simplest manifest(`Info.plist`) I've found is:

{{ source_code(
    path="code/2026-01-13-use-objc2-xc-ui-automation/RustWrapper.app/Info.plist",
    source_type="xml"
    )
}}

The key parts here are `CFBundleExecutable` as `use-objc2-xc-ui-automation`
which is the executable and `CFBundleIdentifier` as `com.simlay.net.Dinghy`
which is the identifier when launching the app via `simctl launch booted
com.simlay.net.Dinghy`.

Given that bundling is a post-`cargo build` step, I wrap this in a `Makefile`:

{{ source_code(
    path="code/2026-01-13-use-objc2-xc-ui-automation/Makefile",
    source_type="Makefile",
    start_line=1,
    end_line=16
    )
}}

Here, `make run` will run `cargo build`, copy the executable, put it in the
expected location, install and launch it in the iPhone 16e simulator. (The
        simulator is expected to be booted up).

This should output:
```
$ make run
cargo build --target aarch64-apple-ios-sim --all --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
cp ./target/aarch64-apple-ios-sim/debug/use-objc2-xc-ui-automation ./RustWrapper.app/
xcrun simctl install 'iPhone 16e' ./RustWrapper.app/
xcrun simctl launch --console --terminate-running-process 'iPhone 16e' com.simlay.net.Dinghy
com.simlay.net.Dinghy: 74660
Hello, world!
```

# Bundling up an XCTest app.

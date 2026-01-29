+++
title = "Writing iOS XCTests in Rust"
date = 2026-01-28
description = "Use objc2-xc-ui-test, objc2-xc-test to test and iOS app as well as get code coverage reports."
+++

# Introduction

In this post, I'll briefly show how to bundle a rust binary into an iOS app as
well as the complexities of bundling a XCTest app written in rust. I can't say
that this should be used in production. I only figured out how to do this a
few months ago (~November 2025) so this might be a brittle setup.

I'll also briefly touch on how to get code coverage reports via the XCTest and
App itself.

The code here is available in [`code` subdirectory of this
repo](https://github.com/simlay/simlay.github.io/tree/master/code/2026-01-use-objc2-xc-ui-automation)
if you want to use it.

Running the stuff from this post requires:
* xcode installed
* the iphone SDK tooling installed
* rust installed along with the `aarch64-apple-ios-sim` target
* Starting the `iPhone 16e` simulator.

This post is best viewed on desktop.


# Background

While `XCTest` has been around since [Xcode
5](https://developer.apple.com/documentation/xctest), `XCUIAutomation` wasn't
it's own framework nor in the public framework list until [Xcode
16.3](https://developer.apple.com/documentation/xcode-release-notes/xcode-16_3-release-notes#Testing-and-Automation).
When it was added as a public API (rather than private), this change enabled
[objc2](https://github.com/madsmtm/objc2) to generate bindings for
[`objc2-xc-test`](https://crates.io/crates/objc2-xc-test) and
[`objc2-xc-ui-automation`](https://crates.io/crates/objc2-xc-ui-automation).
These two rust crates call into the Objective-C APIs for `XCTest` and
`XCUIAutomation`.

I don't believe it's very well documented how an XCTest app is ran. What I can
say is that it's packaged similar to a regular iOS app but the executable is
either derived from or is the `XCTRunner` which I've shown here. I hypothesize
that this executable looks for things that derive from the Objective-C `XCTest`
class. Later in this post you'll see a `#[ctor::ctor]` around a function that
just calls `let _ = TestCase::class();`


This is more or less the architecture. RustApp is the app we're testing
against and RustUITests is the app doing the testing/automating.

```
|-----------------|                                         |-----------------|
|                 |             app.launch()                |                 |
|  RustUITests    |                 ->                      |     RustApp     |
|                 |             app.state()                 |                 |
|-----------------|                                         |-----------------|
```

Then to do an action it's something like:
```
|-----------------|                                         |-----------------|
|                 |  let input = app.textFields().element() |     No text     |
|  RustUITests    |                ->                       |     RustApp     |
|                 |            input.tap()                  |   Now has text  |
|-----------------|     input.typeText(foo_ns_string);      |-----------------|
```

In theory, one could be writing these UI Tests against a react-native app rather
than a Rust `objc2-ui-kit` app.


Note: These bindings are generated based on [targeting macOS for
now](https://github.com/madsmtm/objc2/issues/408). [Various work arounds for
some calls have been added](https://github.com/madsmtm/objc2/pull/809). For
this reason, the examples here are using a `[patch.crate-io]` for the objc2
crates. This also enables us to a nicer/not-yet-released `declare_class!` macro
from `objc2`


# Rust iOS app bundling hacks

In my previous post about [using a target runner for
ios](/posts/rust-target-runner-for-ios/), the bulk of the exercise was about
just bundling a rust executable into an iOS app. In short, an iOS app is a
directory suffixed with `.app`, an `Info.plist` and a binary.

The simplest manifest(`Info.plist`) I've found is:

{{ source_code(
    path="code/2026-01-use-objc2-xc-ui-automation/RustApp.app/Info.plist",
    source_type="xml"
    )
}}

The key parts here are `CFBundleExecutable` as `use-objc2-xc-ui-automation`
which is the executable and `CFBundleIdentifier` as `com.simlay.net.Dinghy`
which is the identifier when launching the app via `simctl launch booted
com.simlay.net.Dinghy`.

Given that bundling is a post-`cargo build` step, I wrap this in a `Makefile`:

{{ source_code(
    path="code/2026-01-use-objc2-xc-ui-automation/Makefile",
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
cp ./target/aarch64-apple-ios-sim/debug/use-objc2-xc-ui-automation ./RustApp.app/
xcrun simctl install 'iPhone 16e' ./RustApp.app/
xcrun simctl launch --console --terminate-running-process 'iPhone 16e' com.simlay.net.Dinghy
com.simlay.net.Dinghy: 74660
Hello, world!
```

It's a little out of scope for this post but the smallest "app" I've found for
this to be a single view that's just a `UITextField`.

{{ source_code(
    path="code/2026-01-use-objc2-xc-ui-automation/src/main.rs",
    source_type="rust",
    start_line=1,
    end_line=69
    )
}}

The thing missing in this file is the definition of `set_llvm_profile_write`
which is covered in the code coverage section below.

# Using `objc2-xc-test` and `objc2-xc-ui-automation`

To use the XCTest and XCUIAutomation from rust you declare the class and use
`ctor` to register the class. This is a `#![no_main]` rust binary. The actual test here will:
* Start the app (possibly with a `LLVM_PROFILE_FILE` environment variable set).
* It will tap on the one and only text field
* Type some text into the `UITextfield`.
* Take a screenshot
* Ask siri a question from plain text
* Tap the home button - This will exit siri.
* Tap the home button again - This will background the app (triggering an
        `__llvm_profile_write_file` for code coverage).


{{ source_code(
    path="code/2026-01-use-objc2-xc-ui-automation/ui_tests/src/main.rs",
    source_type="rust"
    )
}}

# Bundling the XCTests

Bundling the XCTest into an "iOS app" is a lot more difficult. We need:
* `build.rs`
* `ui_tests.xctestconfiguration` - this is generated from
`ui_tests.xctestconfiguration.base` with some `sed` text replacements.
* `RustUITests.app` and a `Info.plist`
* `DinghyUITests.xctest/Info.plist`.


Where's what the directory structure looks like with all these:
```sh
$ tree
.
├── Cargo.lock
├── Cargo.toml
├── Makefile
├── RustUITests.app
│   ├── Frameworks
│   ├── Info.plist
│   ├── PlugIns
│   │   └── DinghyUITests.xctest
│   │       ├── Info.plist
│   │       └── ui_tests
│   └── XCTRunner
├── RustApp.app
│   ├── Info.plist
│   └── use-objc2-xc-ui-automation
├── src
│   └── main.rs
├── stdout.txt
├── ui_tests
│   ├── build.rs
│   ├── Cargo.toml
│   ├── src
│   │   └── main.rs
│   ├── ui_tests.xctestconfiguration
│   └── ui_tests.xctestconfiguration.base
└── ui_tests.png
```

The makefile that wraps these things is a bit chaotic. But the general steps are:
* Build the `ui_tests` binary.
* Copy it into the `RustUITests.app/Plugins/DinghyUITests.xctest/` directory.
* Copy a number of frameworks from the
`IPhoneSimulator.platform/Developer/Library/Frameworks` from the xcode SDKs
into the `RustUITests.app/Frameworks` directory
* Install the app to be tested in the iOS simulator
* Install the UITest app into the iOS simulator
* Get both of their app containers (`xcrun simctl get_container`)
* Do a `sed` replacement on the container path for the xctest.configuration
* Launch the `RustUITests` app with `SIMCTL_CHILD_LLVM_PROFILE_FILE=`
and `SIMCTL_CHILD_DINGHY_LLVM_PROFILE_FILE` to enable code coverage.

{{ source_code(
    path="code/2026-01-use-objc2-xc-ui-automation/Makefile",
    source_type="make",
    start_line=25,
    end_line=67
    )
}}

Throw this all together and `make ui-tests-run` should bundle up both apps,
      install them into the simulator, and run the XCTests. Generally, after
      `cargo build` finishes, this still takes about 10 seconds to bundle,
      install and run. More if there is an error in the XCTests.

At the end of the `ui-tests-run` rule, it calls `make ui-tests-cp-screenshot`.
This copies the created screenshot from the app's data directory to the local
one and compresses the image. I've compressed the image quite but you can get
the gist:

![](../../posts/2026-01-use-objc2-xc-ui-automation/ui_tests.png)

## Test Coverage reports

This is a bit of an aside but if we're going this far, we might as well get
test coverage from the App and the UI Test App. To do this, you need `"-C",
     "instrument-coverage"` as a rust flag and set the environment variable of
     `LLVM_PROFILE_FILE` for both the XCTest app and the app to be tested. This
     is why there is are environment variables of:
     `SIMCTL_CHILD_DINGHY_LLVM_PROFILE_FILE` and
     `SIMCTL_CHILD_LLVM_PROFILE_FILE` which both point to the data path of
     their respective app containers.

I have spent a lot of time trying to get the [incremental instrumentation
(`%c`)](https://clang.llvm.org/docs/SourceBasedCodeCoverage.html#running-the-instrumented-program)
but have been unsuccessful. The work around for this is to add a notification
handler for `UIApplicationDidEnterBackgroundNotification` to call
`__llvm_profile_write_file`. This is pretty hacky but when the app is
backgrounded, it writes the profile file to `LLVM_PROFILE_FILE`.


{{ source_code(
    path="code/2026-01-use-objc2-xc-ui-automation/src/main.rs",
    source_type="rust",
    start_line=71
    )
}}

With that in mind, we can make the `ui-tests-cov` just depend on `ui-tests-run`.

{{ source_code(
    path="code/2026-01-use-objc2-xc-ui-automation/Makefile",
    source_type="make",
    start_line=69,
    end_line=89
    )
}}

Assuming you've got the `genhtml` installed, this will generate a coverage
report html in `target/cov/index.html`.


## build.rs

The `build.rs` is something you'll probably copy from somewhere else (just like
        I did). Notably you need to a to have it link against the testing
frameworks. Also because this requires the `ui_tests` to be in it's own crate
in the workspace.

{{ source_code(
    path="code/2026-01-use-objc2-xc-ui-automation/ui_tests/build.rs",
    source_type="rust"
    )
}}

# Caveats

Getting the exit status of any running iOS app is not a simple task.
[`cargo-dinghy`](https://github.com/sonos/dinghy/) gets the exit status by
spawning the application in debug mode, attaching and running the process via
the debugger. This has been described as "more or less cursed". For this
reason, I suggest the hack of copying the screenshot(s) out of the XCTest app's
data directory. If the `cp` fails, the tests have failed.

I got the `xctestconfiguration` by creating an Xcode project and a little bit
of reverse engineering throwing in the App's ID and the container path.


# Closing thoughts

This is a pretty brittle setup and I'm not sure I suggest it in production. One
can't get the exit status easily, getting this to run on a device is still
quite unclear - How do I get the data container on device?.

Once setup, I find the TUI interface for development a lot nicer in general.
The `xcodebuild` tooling is just a bit worse than a set of make lines. It
should be noted that `Xcode` isn't needed to be open for this at all.

The other cool thing about this setup is that one can run XCTests against an
app that's not in the same project. I actually spent a lot of time trying to
write automation tests against Safari in the simulator (it did not work).

Shout out to [mads](https://github.com/madsmtm/) for doing great work on all
the things in `objc2`.

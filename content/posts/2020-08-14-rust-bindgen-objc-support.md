+++
title = "Using bindgen to generate Rust bindings for Objective-c"
date = 2020-08-14
+++

# Introduction

This is a tutorial how to build
[uikit-sys](https://github.com/simlay/uikit-sys) using [bindgen](https://github.com/rust-lang/rust-bindgen) and how to avoid some of the
pitfalls of generating objective-c bindings.

I got into this rather niche topic because I used to work on a react-native app
and found the build system to be too fragile and wanted to have some of the
stability that rust offers. I spent a little bit of time looking at
[SSheldon/rust-uikit](https://github.com/SSheldon/rust-uikit) but it became
clear that this would rather manual and still rather brittle. So, I stumbled upon the mild objective-c support in rust-bindgen and
have been [adding more features since](https://github.com/rust-lang/rust-bindgen/pulls?q=is%3Apr+author%3Asimlay).

Usage of the uikit-sys crate will be saved for another post.

# Setting up the project

First off, you'll need to `cargo new --lib uikit-sys`. In your `Cargo.toml`, you'll need:
```toml
[dependencies]
objc = "*"
block = "*"

[build-dependencies]
bindgen = "*"
```
But you should probably set the versions so that this is a more stable package.

Your `src/lib.rs` really only needs:

```rust
include!(concat!(env!("OUT_DIR"), "/uikit.rs"));
```
but you probably don't neeed `rustc` telling you all about the
non-{snake,camel,upper-case-glabals} you should do:

```rust
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/uikit.rs"));
```

If you're unfamiliar, `include!(concat!(env!("OUT_DIR"), "file-name.rs"))` is a
pretty [common pattern for cargo build
scripts](https://doc.rust-lang.org/cargo/reference/build-script-examples.html).

# Adding a build.rs
Now, on to the `build.rs`:
```rust
use std::env;
use std::path::Path;
fn main() {

    let target = std::env::var("TARGET").unwrap();
    let target = if target == "aarch64-apple-ios" {
        "arm64-apple-ios"
    } else {
        &target
    };

    let sdk_path = "/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS13.6.sdk";

    println!("cargo:rustc-link-lib=framework=UIKit");
    let builder = bindgen::Builder::default()
        .rustfmt_bindings(true)
        .header_contents("UIKit.h", "#include<UIKit/UIKit.h>")

        .clang_args(&[&format!("--target={}", target)])
        .clang_args(&["-isysroot", sdk_path])

        .block_extern_crate(true)
        .generate_block(true)
        .clang_args(&["-fblocks"])

        .objc_extern_crate(true)
        .clang_args(&["-x", "objective-c"])

        .blacklist_item("timezone")
        .blacklist_item("IUIStepper")
        .blacklist_function("dividerImageForLeftSegmentState_rightSegmentState_")
        .blacklist_item("objc_object");

    let bindings = builder.generate().expect("unable to generate bindings");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    bindings
        .write_to_file(Path::new(&out_dir).join("uikit.rs"))
        .expect("could not write bindings");
}
```

This is an abridged version and it doesn't handle different iOS platforms due to the `sdk_path`. So, let's break this down.

## Getting the right target.
First off, Cargo has a few [environment
variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts)
such as `TARGET` and `OUT_DIR` and that's why
```rust
let target = std::env::var("TARGET").unwrap();
let out_dir = env::var_os("OUT_DIR").unwrap();
```
are needed.

Then we need to turn `arch64-apple-ios` into `arm64-apple-ios` because the
clang triple doesn't match. Once
[rust-lang/rust-bindgen#1211](https://github.com/rust-lang/rust-bindgen/issues/1211)
is resolved, this bit won't be needed:
```rust
let target = if target == "aarch64-apple-ios" {
    "arm64-apple-ios"
} else {
    &target
};
```

## Getting the sysroot

Following this, you'll notice that I've hard coded the `sdk_path`:
```rust
let sdk_path = "/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS13.6.sdk";
```
I'm taking a shortcut here for the purposes of brevity but this is effectively
the command `xcrun --sdk iphoneos --show-sdk-path`. Doing this right, you
should look at the `TARGET` triple, if it's prefixed with `x86_64`, use
`iphonesimulator` rather than `iphoneos`. This should be done with
[`std::process::Command`](https://doc.rust-lang.org/std/process/struct.Command.html)
in your build script because this path changes with the Xcode/iPhone SDK version.
[This is I did it for
uikit-sys](https://github.com/simlay/uikit-sys/blob/1ee18440547de342aa2530d04d0dda82313fcc55/build.rs#L3-L25).

This is then passed to the `clang_args` of bindgen to set the [sysroot for the
precompiled
headers](https://clang.llvm.org/docs/UsersManual.html#relocatable-pch-files).


## Plugging it into bindgen

So, this is the meat of this project and is really quite dense. I've added some
spacing to give you an idea of the groupings.
```rust
println!("cargo:rustc-link-lib=framework=UIKit");
let builder = bindgen::Builder::default()
    .rustfmt_bindings(true)
    .header_contents("UIKit.h", "#include<UIKit/UIKit.h>")

    .clang_args(&[&format!("--target={}", target)])
    .clang_args(&["-isysroot", sdk_path])

    .block_extern_crate(true)
    .generate_block(true)
    .clang_args(&["-fblocks"])

    .objc_extern_crate(true)
    .clang_args(&["-x", "objective-c"])

    .blacklist_item("timezone")
    .blacklist_item("IUIStepper")
    .blacklist_function("dividerImageForLeftSegmentState_rightSegmentState_")
    .blacklist_item("objc_object");

let bindings = builder.generate().expect("unable to generate bindings");
```

In my actual of uikit-sys, I've got this section littered with comments.

* The `cargo:rustc-link-lib=framework=UIKit` is how [cargo tell's rustc to link
against](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-lib)
the UIKit framework.
* `rustfmt_bindings(true)` is pretty obvious and also optional. I'd strongly
recommend this because you will almost certainly be jumping around the
generated bindings to figure out what types you will need when trying to use
this.

* `.header_contents("UIKit.h", "#include<UIKit/UIKit.h>")` is pretty obvious. I
think you might be able to use `.header("UIKit.h")` but I've had issues with it.

* These two were mentioned in the previous sections about target and sdk path.
```rust
    .clang_args(&[&format!("--target={}", target)])
    .clang_args(&["-isysroot", sdk_path])
```

* `.block_extern_crate(true).generate_block(true).clang_args(&["-fblocks"])` is
about adding generation of
[Blocks](https://en.wikipedia.org/wiki/Blocks_%28C_language_extension%29) that
are commonly used with objective-c frameworks.

* `.objc_extern_crate(true).clang_args(&["-x", "objective-c"])` is to tell
clang to return the objective-c AST to bindgen and actually produce the
bindings we want for this project.

* The `objc_extern_crate(true)` and `block_extern_crate(true)` tell bindgen
that prfix the generation with `extern crate objc` and ` extern
crate block`.


### Block lists
So, this is one of the more annoying issues with generating bindings that one can pretty
much only learn by trial and error. If you uncomment any of these you will
almost certainly have an error:

```rust
    .blacklist_item("timezone")
    .blacklist_item("IUIStepper")
    .blacklist_function("dividerImageForLeftSegmentState_rightSegmentState_")
    .blacklist_item("objc_object");
```

Here's pretty much the issue with each:
* time.h as has a variable called `timezone` that conflicts with some of the
objective-c calls from `NSCalendar.h` in the Foundation framework.
* The issues with `IUIStepper` and
`dividerImageForLeftSegmentState_rightSegmentState_` are documented at
[rust-lang/rust-bindgen#1705](https://github.com/rust-lang/rust-bindgen/issues/1705).
* `objc_object` is a bit odd and is because [`Object`doesn't implement
`Copy`](http://sasheldon.com/rust-objc/objc/runtime/struct.Object.html). If you look at the generation, it's not used anywhere else so it's safe to say that you don't really need it.

This is a much smaller list than I had when I first started this project due to
some of the fixes I added to bindgen.

# Let's build it!

To build this whole thing, we now run `cargo build --target
aarch64-apple-ios` and wait for an unfortunately long time. The long compile
time is because UIKit relies on a number of other Objective-c frameworks.

You can find the code for this exact example at [github.com/simlay/uikit-sys-blog-post](https://github.com/simlay/uikit-sys-blog-post).

# So, what have we built?

This is now a uikit-sys crate and has a lot of things in it. Mine has 134133
lines but it will depend on your iOS SDK.

To reiterate what's in the [Objective-c section of the bindgen
guide](https://rust-lang.github.io/rust-bindgen/objc.html), for a given
objective-c class `Foo`, bindgen will produce a struct `Foo` and a trait
`IFoo`. `Foo` will impl `IFoo`. If the objective-c class `Foo` inherits from
`Bar`, the struct `Foo` will also implement `IBar` along with any thing that
the `Bar` class also inherits. The same idea applies to
[protocols](https://www.tutorialspoint.com/objective_c/objective_c_protocols.htm)
and
[categories](https://www.tutorialspoint.com/objective_c/objective_c_categories.htm).

The trait `IFoo` will actually have the getters, setters and methods that
match the Objective-c class `Foo`.

This class `Foo` is a `repr(transparent)` which means that it acts as the
object it returns. As of
[rust-lang/rust-bindgen#1847](https://github.com/rust-lang/rust-bindgen/pull/1847),
this means that if a objective-c method returns a `Baz` class, the rust bindgen
will also return a `Baz` struct.

I could add many more words here but I will leave that for another time.

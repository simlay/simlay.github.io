+++
title = "An attempt in adding memory management to generated Objectiv-c bindings"
date = 2020-08-23
description = "This is to show you that you really can't use the Objective-c memory management tooling with rust-bindgen :("
draft = true
+++

# Introduction

In my recent posts on [using bindgen to generate UIKit
bindings](../posts/rust-bindgen-objc-support/) and [how to use the uikit-sys
bindings](../posts/using-uikit-sys/), I talk about how to get started with
writing bindings but I explicitly avoid actually deallocating the objective-c
memory. Up to this point in messing around with UIKit and objective-c
frameworks, I've passed the buck on memory management. That's not to say it's
not of importance, it's that the user of uikit-sys will probably have to handle
the memory collection.

In this post we discuss why using MRR won't work.

# Using Manual Retain-Release (MRR)

Apple has neat [memory
management](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/MemoryMgmt/Articles/MemoryMgmt.html)
tools like Manual Retain-Release (MRR) and [Automatic Reference Counting
(ARC)](https://en.wikipedia.org/wiki/Automatic_Reference_Counting). Others have
written more (and probably better posts) about this but in general, MRR is
where when you `` a given allocation, you do a `retain` and when you
`Drop` (maybe it goes out of scope) that instance, you do a `release`. If the
`retentionCount` goes below zero, the object is automatically deallocated. So,
it would be quite nice if our generated rust objects could manage their own memory. Let's give it a try.

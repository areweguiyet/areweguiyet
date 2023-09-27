+++
title = "Bindings"
description = "Bindings to an existing, non-Rust GUI framework"
criteria = "TODO"

[extra]

# TODO: Unsure how this should be categorised?
[extra.crates.gemgui]
name = "GemGui"
technology = "web"
platform = ["desktop", "android"]

# TODO: Unsure how this should be categorised?
[extra.crates.imgui]
technology = "custom"
platform = ["desktop", "mobile"]

[extra.crates.gtk]
name = "GTK 3"
technology = "gtk"
platform = ["desktop"]

[extra.crates.gtk4]
name = "GTK 4"
technology = "gtk"
platform = ["desktop"]

[extra.crates.flutter_rust_bridge]
technology = "custom"
platform = ["desktop", "mobile", "web"]

[extra.crates.winsafe]
name = "WinSafe"
technology = "native"
platform = ["windows"]

[extra.crates.iui]
technology = "native"
platform = ["desktop"]

[extra.crates.lvgl]
technology = "custom"
platform = ["embedded"]

[extra.crates.fltk]
description = "The FLTK crate is a crossplatform lightweight gui library which can be linked to statically to produce small, self-contained and fast binaries"
technology = "custom"
platform = ["desktop"]

[extra.crates.qt_widgets]
description = "Ritual Qt bindings"
docs = "https://rust-qt.github.io/qt/"
technology = "qt"
platform = ["desktop", "mobile", "embedded", "web"] # Unverified

[extra.crates.rust-qt-binding-generator]
technology = "qt"
platform = ["desktop", "mobile", "embedded", "web"] # Unverified

[extra.crates.qmetaobject]
description = "A framework empowering everyone to create Qt/QML applications with Rust. It does so by building QMetaObjects at compile time, registering QML types (optionally via exposing QQmlExtensionPlugins) and providing idiomatic wrappers."
technology = "qt"
platform = ["desktop", "mobile", "embedded", "web"] # Unverified

[extra.crates.cxx-qt]
name = "CXX-Qt"
description = "CXX-Qt is a library that automatically generates code to transfer data between Rust and C++ through common interfaces such as QObjects that can be exposed directly into QML. It uses the cxx crate for safe interaction between Rust and C++."
docs = "https://kdab.github.io/cxx-qt/book/"
technology = "qt"
platform = ["desktop", "mobile", "embedded", "web"] # Unverified

[extra.crates.sciter-rs]
name = "Sciter"
technology = "web"
platform = ["desktop", "mobile", "embedded"]

[extra.crates.core-foundation]
technology = "native"
platform = ["macos", "ios"]

[extra.crates.cacao]
technology = "native"
platform = ["macos", "ios"]

[extra.crates.windows]
technology = "native"
platform = ["windows"]

+++

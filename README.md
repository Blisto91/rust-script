[![CI](https://github.com/fornwall/rust-script/workflows/CI/badge.svg)](https://github.com/fornwall/rust-script/actions?query=workflow%3ACI)
[![Crates.io](https://img.shields.io/crates/v/rust-script.svg)](https://crates.io/crates/rust-script)

# rust-script

`rust-script` is a command-line tool to run Rust "scripts" which can make use of Cargo's package ecosystem.  It can also evaluate expressions and run filters.

Some of `rust-script`'s features include:

- Reading Cargo manifests embedded in Rust scripts.
- Caching compiled artefacts (including dependencies) to amortise build times.
- Supporting executable Rust scripts via UNIX hashbangs and Windows file associations.
- Evaluating expressions on the command-line.
- Using expressions as stream filters (*i.e.* for use in command pipelines).
- Running unit tests and benchmarks from scripts.
- Custom templates for command-line expressions and filters.

**Note**: `rust-script` *does not* work when Cargo is instructed to use a target architecture different to the default host architecture.

Table of contents:

- [Installation](#installation)
  - [Features](#features)
  - [Manually Compiling and Installing](#compiling)
  - [Self-Executing Scripts](#hashbang)
- [Usage](#usage)
  - [Scripts](#scripts)
  - [Expressions](#expressions)
  - [Stream Filters](#filters)
  - [Environment Variables](#env-vars)
  - [Templates](#templates)
- [Known Issues](#issues)
- [License](#license)
  - [Contribution](#contribution)

<a name="installation"></a>
## Installation

The recommended method for installing `rust-script` is by using Cargo's `install` subcommand:

```sh
cargo install rust-script
```

If you have already installed `rust-script`, you can update to the latest version by using:

```sh
cargo install --force rust-script
```

<a name="features"></a>
### Cargo Features

The following features are defined:

- `suppress-cargo-output` (default): if building the script takes less than 2 seconds and succeeds, `rust-script` will suppress Cargo's output.  Note that this disabled coloured Cargo output on Windows.

<a name="compiling"></a>
### Manually Compiling and Installing

`rust-script` requires Rust 1.11 or higher to build.  Rust 1.4+ was supported prior to version 0.2.

Once built, you should place the resulting executable somewhere on your `PATH`. At that point, you should be able to invoke it by using `rust-script`.

<a name="hashbang"></a>
### Self-Executing Scripts

On UNIX systems, you can use `#!/usr/bin/env rust-script` as a hashbang line in a Rust script.  If the script file is executable, this will allow you to execute a script file directly.

If you are using Windows, you can associate the `.crs` extension (which is simply a renamed `.rs` file) with `rust-script`.  This allows you to execute Rust scripts simply by naming them like any other executable or script.

TODO: This can be done using the `rust-script --file-association` command. This command can also remove the file association.  If you pass `--amend-pathext` to the `file-assocation install` command, it will also allow you to execute `.crs` scripts *without* having to specify the file extension, in the same way that `.exe` and `.bat` files can be used.

If you want to make a script usable across platforms, it is recommended that you use *both* a hashbang line *and* give the file a `.crs` file extension.

<a name="usage"></a>
## Usage

Generally, you can get an overview of the available options using the `--help` flag.

<a name="scripts"></a>
### Scripts

The primary use for `rust-script` is for running Rust source files as scripts.  For example:

```shell
$ echo 'fn main() { println!("Hello, World!"); }' > hello.rs
$ rust-script hello.rs
Hello, World!
$ rust-script hello # you can leave off the file extension
Hello, World!
```

The output of Cargo will be hidden unless compilation fails, or takes longer than a few seconds.

`rust-script` will also look for embedded dependency and manifest information in the script.  For example, all of the following are equivalent:

- `now.crs` (code block manifest with UNIX hashbang and `.crs` extension):

    ```rust
    #!/usr/bin/env rust-script
    //! This is a regular crate doc comment, but it also contains a partial
    //! Cargo manifest.  Note the use of a *fenced* code block, and the
    //! `cargo` "language".
    //!
    //! ```cargo
    //! [dependencies]
    //! time = "0.1.25"
    //! ```
    extern crate time;
    fn main() {
        println!("{}", time::now().rfc822z());
    }
    ```

- `now.rs` (dependency-only, short-hand manifest):

    ```rust
    // cargo-deps: time="0.1.25"
    // You can also leave off the version number, in which case, it's assumed
    // to be "*".  Also, the `cargo-deps` comment *must* be a single-line
    // comment, and it *must* be the first thing in the file, after the
    // hashbang.
    extern crate time;
    fn main() {
        println!("{}", time::now().rfc822z());
    }
    ```

    > **Note**: you can write multiple dependencies by separating them with commas.  *E.g.* `time="0.1.25", libc="0.2.5"`.

On running either of these, `rust-script` will generate a Cargo package, build it, and run the result.  The output may look something like:

```shell
$ rust-script now
    Updating registry `https://github.com/rust-lang/crates.io-index`
   Compiling winapi-build v0.1.1
   Compiling winapi v0.2.8
   Compiling libc v0.2.30
   Compiling kernel32-sys v0.2.2
   Compiling time v0.1.38
   Compiling now v0.1.0 (file:///C:/Users/drk/AppData/Local/Cargo/script-cache/file-now-37cb982cd51cc8b1)
    Finished release [optimized] target(s) in 49.7 secs
Sun, 17 Sep 2017 20:38:58 +1000
```

Subsequent runs, provided the script has not changed, will likely just run the cached executable directly:

```shell
$ rust-script now
Sun, 17 Sep 2017 20:39:40 +1000
```

Useful command-line arguments:

- `--bench`: Compile and run benchmarks.  Requires a nightly toolchain.
- `--debug`: Build a debug executable, not an optimised one.
- `--features <features>`: Cargo features to pass when building and running.
- `--force`: Force the script to be rebuilt.  Useful if you want to force a recompile with a different toolchain.
- `--gen-pkg-only`: Generate the Cargo package, but don't compile or run it.  Effectively "unpacks" the script into a Cargo package.
- `--test`: Compile and run tests.

<a name="expressions"></a>
### Expressions

`rust-script` can also run pieces of Rust code directly from the command line.  This is done by providing the `--expr` option; this causes `rust-script` to interpret the `<script>` argument as source code *instead* of as a file path.  For example, code can be executed from the command line in a number of ways:

- `rust-script --dep time --expr "extern crate time; time::OffsetDateTime::now_utc().format(time::Format::Rfc3339).to_string()"`
- `rust-script --dep time=0.1.38 --expr "extern crate time; time::now().rfc822z().to_string()"` - uses a specific version of `time`
- `rust-script -D time -e "..."` - guess and inject `extern crate time`; this only works when the package and crate names of a dependency match.
- `rust-script -d time -x time -e "..."` - injects `extern crate time`; works when the names do *not* match.

The code given is embedded into a block expression, evaluated, and printed out using the `Debug` formatter (*i.e.* `{:?}`).

Useful command-line arguments:

- `-d`/`--dep`: add a dependency to the generated `Cargo.toml` manifest.
- `-x`/`--extern`: inject `extern crate` into generated script.
- `-D`/`--dep-extern`: do both of the above.
- `-t`/`--template`: Specify a custom template for this expression (see section on templates).

<a name="filters"></a>
### Stream Filters

You can use `rust-script` to write a quick stream filter, by specifying a closure to be called for each line read from stdin, like so:

```text
$ cat now.crs | rust-script --loop \
    "let mut n=0; move |l| {n+=1; println!(\"{:>6}: {}\",n,l.trim_right())}"
   Compiling loop v0.1.0 (file:///C:/Users/drk/AppData/Local/Cargo/script-cache/loop-58079283761aab8433b1)
     1: // cargo-deps: time="0.1.25"
     2: extern crate time;
     3: fn main() {
     4:     println!("{}", time::now().rfc822z());
     5: }
```

You can achieve a similar effect to the above by using the `--count` flag, which causes the line number to be passed as a second argument to your closure:

```text
$ cat now.crs | rust-script --count --loop \
    "|l,n| println!(\"{:>6}: {}\", n, l.trim_right())"
   Compiling loop v0.1.0 (file:///C:/Users/drk/AppData/Local/Cargo/script-cache/loop-58079283761aab8433b1)
     1: // cargo-deps: time="0.1.25"
     2: extern crate time;
     3: fn main() {
     4:     println!("{}", time::now().rfc822z());
     5: }
```

Note that, like with expressions, you can specify a custom template for stream filters.

<a name="env-vars"></a>
### Environment Variables

The following environment variables are provided to scripts by `rust-script`:

- `CARGO_SCRIPT_BASE_PATH`: the base path used by `rust-script` to resolve relative dependency paths.  Note that this is *not* necessarily the same as either the working directory, or the directory in which the script is being compiled.

- `CARGO_SCRIPT_PKG_NAME`: the generated package name of the script.

- `CARGO_SCRIPT_SAFE_NAME`: the file name of the script (sans file extension) being run.  For scripts, this is derived from the script's filename.  May also be `"expr"` or `"loop"` for those invocations.

- `CARGO_SCRIPT_SCRIPT_PATH`: absolute path to the script being run, assuming one exists.  Set to the empty string for expressions.

<a name="templates"></a>
### Templates

You can use templates to avoid having to re-specify common code and dependencies.  You can view a list of your templates by running `rust-script templates list` (note the hyphen), or show the folder in which they should be stored by running `rust-script templates show`.  You can dump the contents of a template using `rust-script templates dump NAME`.

Templates are Rust source files with two placeholders: `#{prelude}` for the auto-generated prelude (which should be placed at the top of the template), and `#{script}` for the contents of the script itself.

For example, a minimal expression template that adds a dependency and imports some additional symbols might be:

```rust
// cargo-deps: itertools="0.6.2"
#![allow(unused_imports)]
#{prelude}
extern crate itertools;
use std::io::prelude::*;
use std::mem;
use itertools::Itertools;

fn main() {
    let result = {
        #{script}
    };
    println!("{:?}", result);
}
```

If stored in the templates folder as `grabbag.rs`, you can use it by passing the name `grabbag` via the `--template` option, like so:

```text
$ rust-script -t grabbag -e "mem::size_of::<Box<Read>>()"
16
```

In addition, there are three built-in templates: `expr`, `loop`, and `loop-count`.  These are used for the `--expr`, `--loop`, and `--loop --count` invocation forms.  They can be overridden by placing templates with the same name in the template folder.  If you have *not* overridden them, you can dump the contents of these built-in templates using the `templates dump` command noted above.

<a name="issues"></a>
## Known Issues

### [Issue #50](https://github.com/DanielKeep/cargo-script/issues/50)

There is a problem on Windows where `rust-script` can hang when asking Cargo for the path to a package's compiled executable.  `rust-script` currently works around this by using an older heuristic to guess this path on affected versions.  This can, however, lead to `rust-script` being unable to correctly locate a compiled executable.

If this is a problem, `rust-script` can be instructed to use the accurate-but-buggy approach by setting the `CARGO_SCRIPT_IGNORE_ISSUE_50` environment variable to any non-empty string.

<a name="license"></a>
## License

Licensed under either of

* MIT license (see [LICENSE](LICENSE) or <http://opensource.org/licenses/MIT>)
* Apache License, Version 2.0 (see [LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>)

at your option.

<a name="contribution"></a>
### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.

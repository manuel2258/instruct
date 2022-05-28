# Instruct

[![codecov](https://codecov.io/gh/manuel2258/instruct/branch/master/graph/badge.svg?token=JORKMY1BBV)](https://codecov.io/gh/manuel2258/instruct)

A language to write general purpose 'makefile like' tasks which are powerful and reuseable.

## Status

This project is somewhat working!

- [x] Parsing
    - [] 
- [x] Static analysis
- [x] Interpreter
- [/] Task project definition and config files
- [] Dependency system
- [] Runner system
    - [x] cmd's
    - [] ssh
    - [] docker
    - [] python
- [] Testing system

## Goals

My goal for this project is to provide an alternative to makefiles / bash scripts, especially for the devops world.  
Most of the time automation code is simply dumped into a few bash scripts or directly into pipeline config files, which makes them hard to maintain / reuse.  

Therefor this language thrives to provide following goals:
- Clean syntax that is easy to read and understand
- Reuseability of tasks
- Simple package / dependency system
- Native support for multiple executors:
    - Simple shell
    - Docker
    - Python

## Example

A currently fully working small example.

```
module as variables;

collection as interpolate: {
    let (final_stdout: stdout) from task as stdout: {
        let (pre_var: var) from block as pre: {
            let (var: stdout) from run with (trim_stdout): echo pre;
            run: echo dyn_${var};
        };
        let (stdout) from run with (trim_stdout) as main: echo interpolated '${pre_var}' used in main;
        block as post: {
            let (stdout) from run with (stdin: stdout): sed s/main/post/g;
            run: echo ${stdout};
        };
    };

    task as call: {
        let (final_stdout) from call as main: variables.interpolate.stdout;
        run as post: echo ${final_stdout} used after call;
    };

    task as exit-code: {
        let (status1: status) from run as pre: cat Cargo.toml;
        let (status2: status) from run as main: cat random_file.json;
        run as post: echo "cat Cargo.toml: $${status1}, cat random_file.json: $${status2}";
    };
};
```

It can be executed using cargo:

```sh
cargo run variables.interpolate.stdout
cargo run variables.interpolate.exit-code
```
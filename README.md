# Task Lang

A language to write general purpose makefile like tasks which are powerfull and reuseable

## Status

This project is still not even close to working!

- [/] Parsing
- [/] Static analysis
- [/] Interpretter
- [] Task project definition and config files
- [] Dependency System

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

The sytax is still heavily WIP, however a small look into how a snipped could look:

```
module as simple;

task as sample_task: {
    let (pre_var: var) from block with (runner: sh) as pre: {
        let (var: stdout) from run with (silent): echo "pre";
        run: echo dyn_$var;
    };
    run as exec: ls -al;
    call as post: test.sample_task;
};

collection as test: {
    task as sample_task: {
        run as exec: ls -al;
    };
};
```

Parts of this can already be executed using cargo!  
```sh
cargo run ./examples/example.task simple.sample_task.exec
```
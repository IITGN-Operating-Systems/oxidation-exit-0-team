# CS330 Lab assignments

This repository contains lab assignments for CS330 "Operating Systems".

## Why Rust?

Historically, C has been mainly used for OS development because of its portability,
minimal runtime, direct hardware/memory access, and (decent) usability.  
Rust provides all of these features with addition of memory safety guarantee,
strong type system, and modern language abstractions
which help programmers to make less mistakes when writing code.

## Why are we using older Rust toolchains?

The features we require for low level, OS independent Rust are still experimental.
Many of the said features which needed 3rd party dependencies are in a phase of
moving over to official Rust `core`.  
And therefore the handover has caused abandonment of these 3rd party projects.
To use these dependencies, we have to use older nightly versions of Rust.

## Can we use latest Rust nightly?

You are free to port the code over to newer versions. We will try to help you if you get stuck.  
But sadly, this endeavour will not be considered for deadline extension. ¯\\\_(ツ)\_/¯

## Acknowledgement

We built our labs based on the materials developed for
`Georgia Tech CS3210` and `CS140e: An Experimental Course on Operating Systems` by Sergio Benitez.  
We want to port it to use newer toolchains such as Rust 2021 (or hopefully 2024) edition and 
Raspberry 5 if possible.

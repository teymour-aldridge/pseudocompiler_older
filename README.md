# Pseudocompiler

## Preamble

OCR (a UK exam board) like many educational organisations ask students to write code on paper. A
strange practice, given that nobody has written code without access to a translator (either an
interpreter or compiler) since punch tape went out of fashion (and even then the code written was 
still run). Teachers are allowed to pick from a variety of languages (suggestions include "Python, 
C family of languages (for example C#, C++,etc.), Java, JavaScript, Visual Basic/.Net, PHP, Delphi, 
BASIC") to teach in. These are used for a "programming project" which students complete.

When it comes to the actual exam, however, students are asked to write in "pseudocode."
Unfortunately rather than accepting anything which looks like a computer program, OCR have provided
a pseudocode "guide" (which is actually enough to build a formal grammar). Given that they provide a
formal grammar, I thought it might be interesting to build a compiler for their specification. They
don't go so far as to describe how their "pseudocode" handles memory management (and a few other
issues) so I've improvised on that front. It seemed like a reasonable guess that you wouldn't want
to unveil whole classes of difficult to catch undefined behaviour related issues on students just
learning to program, so I've assumed that garbage collection is the way to go.

This compiler can output either LLVM IR or directly produce Javascript.

This is an educational tool. It's probably **not** the correct language to build production software
in, but it would be interesting to see somebody try :D

It's a real shame that the exam board teach a programming paradigm which focuses on imperative
procedural programming at GCSE and object-orientated programming at A-level rather than exploring
more interesting stuff such as ownership models, functional programming (logic programming might be
nice too). Going for functional programming might make it easier to mix the language with
mathematics education.

Anyway, what do I know?

I do have a long list of complaints which I plan on sending to OCR.

## Getting started
Currently the best way to use this is to build it from source. 

You'll need a copy of `rustc` (the Rust compiler) which you can install from https://rustup.rs

```
git clone https://github.com/teymour-aldridge/pseudocompiler
cd pseudocompiler
cargo install .
```

That will take a while (it has to compile millions of lines of C++ followed by the Rust code for the
compiler).

Then just run `pseudocompiler [options] [file]`

Which will compile your program.

To compile and run your program run `pseudocompiler --run [other options] [file]`

# TrolledLang
TrolledLang is a programming language primarily made just for fun and as a learning experience. It tries to be a beginner friendly language while also not being garbage compiled, although I doubt that this goal is something that I will achieve.

## Installation
Download the source and compile it with cargo.

## Usage
To parse a single line of code in the terminal directly, use
``cargo run run "expresion in here"``.

To parse a file use ``cargo run file filename.tlang``.

To open a shell, use ``cargo run run``. Type ``exit`` or ``quit`` in the shell to exit it.

## Syntax
I haven't created all the syntax yet, but I have created some of it.
The compiler isn't finished yet either, so for the moment only assignments, code blocks and literals are supported.
### Assignments
```
x = 3;
y = 23;
```
Variable declarations have identical syntax to assignments. Types are inferred.

### Function definitions
```
def add = func [a: Int, b: Int] -> Int (
    a + b
);

def print_add = func [a: Int, b: Int] (
    print[add[a, b]];
);

def print_add_4_5 = func (
    print_add[4, 5];
);
```
``def`` can be used to set a namespace element, such as a function for example. All namespace elements are constant and can be accessed from anywhere within the code.

``func`` is used to define a function. The syntax is designed such that you can create functions as soon as you learn about ``def`` and code blocks, ``()``. This is to smooth out the learning curve and add more and finer steps.

### Special case function calls
```
print "Hello world!";

def single_arg = func [arg1: Int] ( x = arg1 + 53; );
single_arg 23;
```
Functions with single arguments don't require ``[]`` to be called, as long as you have a single argument and that argument is a literal or a block of code;

This is because of the same reasoning as the function definitions. You don't have to learn about ``[]`` to write a hello world program. This _might_ make it easier to learn, but I don't know to be honest.
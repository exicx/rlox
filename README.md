# rlox

Interpreter implementation of the (jlox) Lox programming language, designed by Robert Nystrom.

## License

This work (the original components of it) is licensed under the Apache 2.0 license.
The project is based on the book "Crafting Interpreters", by Nystrom, Robert.

## EBNF

```ebnf
program -> declaration* EOF ;

declaration -> funDecl | varDecl | statement ;

funDecl -> "fun" function ;
function -> IDENTIFIER "(" parameters? ")" block ;
varDecl -> "var" IDENTIFIER ( "=" expression )? ";" ;

statement -> exprStmt
    | forStmt
    | ifStmt
    | printStmt
    | returnStmt
    | whileStmt
    | block ;

exprStmt -> expression ";" ;
forStmt -> "for" "("
           ( varDecl | exprStmt | ";" )
           expression? ";"
           expression? ")" statement ;
ifStmt -> "if" "(" expression ")" statement ( "else" statement )? ;
printStmt -> "print" expression ";" ;
returnStmt -> "return" expression? ";" ;
whileStmt -> "while" "(" expression ")" statement ;
block -> "{" declaration* "}" ;

expression -> assignment ;
assignment -> IDENTIFIER "=" assignment | logic_or ;
logic_or -> logic_and ( "or" logic_and )* ;
logic_and -> equality ( "and" equality )* ;
equality -> comparison ( ( "!=" | "==" ) comparison )* ;
comparison -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term -> factor ( ( "-" | "+" ) factor )* ;
factor -> unary ( ( "/" | "*" ) unary )* ;
unary -> ( "!" | "-" ) unary | call ;
call -> primary ( "(" arguments? ")" )* ;
primary -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER ;

arguments -> expression ( "," expression )* ;
parameters -> IDENTIFIER ( "," IDENTIFIER )* ;
```

# Understanding Interpreters (for me)

Lexers/Scanners consume a byte stream and produce a series of tokens.
This process is also called Lexical Analysis.

The Parser consumes the stream of tokens to produce the abstract syntax tree (AST).
This process is also called Syntax Analysis.

The interpreter consumes the AST and evaluates expressions and statements. It defines the global variable table
and handles environments for the stack, functions, closures, and classes.
This process is also called Semantic Analysis.

jlox does not create a bytecode or virtual machine. We directly interpret over the AST.
(we *kind of* skip semantic analysis because we directly consume the AST)

clox will define a bytecode specification and run that bytecode through a virtual machine.

# Mai

Mai is a new programming language that compiles into webassemly using LLVM.

Example:

```rust
gm safe_sub(x, y) {
    if (x > y) {
        return x - y;
    } else {
        return 0;
    }
}
```

## Running

Requirements:

- Rust with the wasm32-unknown target
- LLVM version 15 toolchain
- wabt

Trying it out:

```
cargo run
```

Output:

```
Input file path: "main.mai"
Raw input contents:
"mai safe_sub(x, y) {\n    if (x > y) {\n        return x - y;\n    } else {\n        return 0;\n    }\n}\n"

Lexed tokens:
[Fun, Ident("safe_sub"), LParen, Ident("x"), Comma, Ident("y"), RParen, LBrace, If, LParen, Ident("x"), Greater, Ident("y"), RParen, LBrace, Return, Ident("x"), Minus, Ident("y"), Semicolon, RBrace, Else, LBrace, Return, Number("0"), Semicolon, RBrace, RBrace]

Parsed expression:
[Function { name: Ident("safe_sub"), params: [Ident("x"), Ident("y")], body: [If { cond: BinaryExpr { op: Greater, left: Variable { name: Ident("x") }, right: Variable { name: Ident("y") } }, then_branch: Block([Return { keyword: Return, value: Some(BinaryExpr { op: Minus, left: Variable { name: Ident("x") }, right: Variable { name: Ident("y") } }) }]), else_branch: Some(Block([Return { keyword: Return, value: Some(Literal { value: "0" }) }])) }] }]

Compiled wasm to wat:
(module
  (type (;0;) (func))
  (type (;1;) (func (param f64 f64) (result f64)))
  (func $__wasm_call_ctors (type 0))
  (func $safe_sub (type 1) (param f64 f64) (result f64)
    f64.const 0x0p+0 (;=0;)
    local.get 0
    local.get 1
    f64.sub
    local.get 1
    local.get 0
    f64.ge
    select)
  (memory (;0;) 2)
  (global $__stack_pointer (mut i32) (i32.const 66560))
  (global (;1;) i32 (i32.const 1024))
  (global (;2;) i32 (i32.const 1024))
  (global (;3;) i32 (i32.const 1024))
  (global (;4;) i32 (i32.const 66560))
  (global (;5;) i32 (i32.const 0))
  (global (;6;) i32 (i32.const 1))
  (export "memory" (memory 0))
  (export "__wasm_call_ctors" (func $__wasm_call_ctors))
  (export "safe_sub" (func $safe_sub))
  (export "__dso_handle" (global 1))
  (export "__data_end" (global 2))
  (export "__global_base" (global 3))
  (export "__heap_base" (global 4))
  (export "__memory_base" (global 5))
  (export "__table_base" (global 6)))

Trying inputs 3.0 and 4.0 into safe_sub
F64(0.0)
Trying inputs 3.0 and 1.0 into safe_sub
F64(2.0)
```

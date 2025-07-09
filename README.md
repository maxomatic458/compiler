This repository contains the code for a compiler for a custom programming language,
which i developed as part of a "Besondere Lernleistung" (academic paper for school).

The paper is written in german and can be found in the `bll/` directory.

The compiler is written in Rust and uses LLVM IR as target language.
If a installation of clang (>= 16) is available the compiler will output a executable binary.

A web demo of the compiler is available at https://compiler-demo.pages.dev/
## Usage
```
./compiler <input-file>
```

### Arguments
- `-e`, `--emit-llvm`: Emit LLVM IR instead of compiling to binary
- `-o`, `--output <file>`: Output file for the binary (default: `./out.exe`)
- `-h`, `--help`: Print help message
- `-v`, `--version`: Print version information

- `-d`, `--dont-write-output`: Don't write output to a file (intended for debugging)

## Language

### Syntax and Semantics
The syntax is inspired by Python and Rust.

```py
import "std/io.mx"

class Foo { # class
    inner: int64,
}

def new() for Foo -> Foo { # static method
    return Foo { inner: 0 };
}

def get_inner(self) for Foo -> int64 { # method
    return self.inner;
}

def main() -> int64 { # function + entry point

    let mut foo = Foo::new(); # variable declaration

    foo.inner = 0; # field reassignment

    while (foo.inner) < 10 { # while loop
        foo.inner = foo.inner + 1;
    }

    println("The inner value is:");
    println(foo.get_inner().to_string());

    return 0;
}
```

### Other language features
* No memory safety (the user is responsible for managing memory)
* Use functions of the c standard library (e.g for I/O, memory allocation)
* Generics
* Operator overloading
* Rather helpful error messages

More examples can be found in `example/` and `src/tests/`.
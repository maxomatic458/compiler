#![cfg(test)]
use crate::compiler::Compiler;
use rstest::rstest;
use serial_test::file_serial;
use std::io::Write;

// const TEMP_FILE: &str = "tmp.exe";
#[cfg(target_os = "windows")]
const TEMP_FILE: &str = "tmp.exe";
#[cfg(not(target_os = "windows"))]
const TEMP_FILE: &str = "tmp";

#[rstest]
#[file_serial]
#[case(
    "def main() -> int64 {
        return 1;
    }",
    Ok(1)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        return -(10 + 5);
    }",
    Ok(-15)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        return - 5 * 10;
    }",
    Ok(-50)
)]
#[file_serial]
#[case(
    "def my_function() -> int64 {
        return 2;
    }

    def main() -> int64 {
        return {
            let a = my_function();
            return a * a;
        }
    }",
    Ok(4)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        return {
            let a = 1;
            let b = 2;
            let c = 3;
            return a + b + c;
        }
    }",
    Ok(6)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        let b = 10;
        if b == 10 {
            return 1;
        }
        return 0;
    }",
    Ok(1)
)]
#[file_serial]
#[case(
    "class Foo<T> {
        data: T,
    }

    def main() -> int64 {
        let foo = Foo<int64> {
            data: 10,
        };

        return foo.data + {
            return foo.data * 2
        };
    }",
    Ok(30)
)]
#[file_serial]
#[case(
    "def fib(n: int64) -> int64 {
        if n == 0 {
            return 0;
        }
        if n == 1 {
            return 1;
        }
        return fib(n - 1) + fib(n - 2);
    }

    def main() -> int64 {
        return fib(10);
    }",
    Ok(55)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        return 10 + true;
    }", Err(())
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        let num = 17;

        if num == 0 {
            return 0;
        } else if num == 1 {
            return 1;
        } else if num == 2 {
            return 2;
        } else if num == 3 {
            return 3;
        } else if num == 4 {
            return 4;
        } else if num == 5 {
            return 5;
        } else if num == 6 {
            return 6;
        } else if num == 7 {
            return 7;
        } else if num == 8 {
            return 8;
        } else if num == 9 {
            return 9;
        } else if num == 10 {
            return 10;
        } else if num == 11 {
            return 11;
        } else if num == 12 {
            return 12;
        } else if num == 13 {
            return 13;
        } else if num == 14 {
            return 14;
        } else if num == 15 {
            return 15;
        } else if num == 16 {
            return 16;
        } else if num == 17 {
            return 5;
        } else if num == 18 {
            return 18;
        } else if num == 19 {
            return 19;
        } else if num == 20 {
            return 20;
        } else if num == 21 {
            return 21;
        } else if num == 22 {
            return 22;
        } else if num == 23 {
            return 23;
        } else if num == 24 {
            return 24;
        } else if num == 25 {
            return 25;
        } else if num == 26 {
            return 26;
        } else if num == 27 {
            return 27;
        } else if num == 28 {
            return 28;
        } else if num == 29 {
            return 29;
        } else if num == 30 {
            return 30;
        } else if num == 31 {
            return 31;
        } else if num == 32 {
            return 32;
        } else if num == 33 {
            return 33;
        } else if num == 34 {
            return 34;
        } else if num == 35 {
            return 35;
        } else if num == 36 {
            return 36;
        }
        return 0;
    }",
    Ok(5)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        let a = 4;

        if a == 1 {
            return 0;
        } else if a == 2 {
            return 0;
        } else {
            return 1;
        }
    }",
    Ok(1)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        let a = 4;

        if a == 1 {
            return 0;
        } else if a == 2 {
            return 0;
        }

    }", Err(())
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        let a = 4;

        if a == 1 {
            return 0;
        }

        return 4;
    }",
    Ok(4)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        let a = 4;
        if a == 1 {
            
        } else if a == 4 {
            return 4;
        }
        return 0;
    }",
    Ok(4)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        let boolean = true;

        if boolean {
            return 1;
        } else {
            return 0;
        }
        let a = 10;
    }",
    Ok(1)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        let foo = 100;
        let mut bar = 0;

        while bar < foo {
            bar = bar + 1;
        }

        return bar + foo;

    }",
    Ok(200)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        let list_1 = [1, 2, 3];
        let list_2 = [4, 5, 6];
        let num = list_1[0] + list_2[0];

        return num;
    }",
    Ok(5)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        let mut list = [1, 2, 3];
        list[0] = 5;

        return list[0] + list[1] + list[2];
    }",
    Ok(10)
)]
#[file_serial]
#[case(
    "class Foo {
        data: int64,
    }

    def main() -> int64 {
        let foo = Foo {
            data: 10,
        };

        return foo.data;
    }",
    Ok(10)
)]
#[file_serial]
#[case(
    "class Foo {
        data: int64,
    }

    def main() -> int64 {
        let foo = Foo {
            data: 10,
        };

        return foo.data + {
            return foo.data * 2
        };
    }",
    Ok(30)
)]
#[file_serial]
#[case(
    "class Bar {
        data: int64,
    }

    class Foo {
        bar: Bar,
    }

    def main() -> int64 {
        let mut foo = Foo {
            bar: Bar {
                data: 10,
            },
        };

        foo.bar.data = 20;

        return foo.bar.data;
    }",
    Ok(20)
)]
#[file_serial]
#[case(
    "class Foo {
        data: int64,
    }

    def new(data: int64) for Foo -> Foo {
        return Foo {
            data: data,
        };
    }

    def main() -> int64 {
        let foo = Foo::new(10);

        return foo.data;
    }",
    Ok(10)
)]
#[file_serial]
#[case(
    "class Foo {
        data: int64,
    }

    def new(data: int64) for Foo -> Foo {
        return Foo {
            data: data,
        };
    }

    def get_data(self) for Foo -> int64 {
        return self.data;
    }

    def main() -> int64 {
        let foo = Foo::new(10);

        return foo.get_data();
    }",
    Ok(10)
)]
#[file_serial]
#[case(
    "class Foo {
        data: int64,
    }

    def new(data: int64) for Foo -> Foo {
        return Foo {
            data: data,
        };
    }

    def get_data(self) for Foo -> int64 {
        return self.data;
    }

    def inc_data(self) for Foo {
        self.data = self.data + 1;
    }

    def dec_data(self) for Foo {
        self.data = self.data - 1;
    }

    def main() -> int64 {
        let to_increment = 20;
        let to_decrement = 10;

        let foo = Foo::new(0);

        let mut c = 0;

        while c < to_increment {
            foo.inc_data();
            c = c + 1;
        }
        c = 0;

        while c < to_decrement {
            foo.dec_data();
            c = c + 1;
        }

        return foo.get_data();
    }",
    Ok(10)
)]
#[file_serial]
#[case(
    "def foo() {
        if true {

        } else if true {

        } else {

        }
    }

    def main() -> int64 {
        return 0;
    }",
    Ok(0)
)]
#[file_serial]
#[case(
    "extern def calloc(num: int64, size: int64) -> int64

    def _calloc<T>(num: int64, size: int64) -> *T {
        return calloc(num, size) as *T;
    }

    class Foo {
        data: *int64,
    }

    def main() -> int64 {
        let mut ptr = _calloc<int64>(1, 4);
        let foo = Foo {
            data: ptr,
        };
        return 0;
    }",
    Ok(0)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        let bool1 = true;
        let bool2 = false;

        if bool1 && bool2 {
            return 0;
        }

        return 1;
    }",
    Ok(1)
)]
#[file_serial]
#[case(
    "class Foo<T> {
        inner: T,
    }

    def inner<T>(self) for Foo -> T {
        return self.inner;
    }

    def main() -> int64 {
        let a = Foo<Foo<int64>> {
            inner: Foo<int64> {
                inner: 10,
            }
        };

        let inner = a.inner<Foo<int64>>().inner<int64>();

        return inner;

    }",
    Ok(10)
)]
#[file_serial]
#[case(
    "def is_three(self) for int64 -> bool {
        return self == 3;
    }
    
    def main() -> int64 {
        let a = 3;
        let b = 4;

        if a.is_three() {
            return 1;
        }

        if b.is_three() {
            return 0;
        }

        return 0;
    }",
    Ok(1)
)]
// der test führt zu stackoverflow rekursion in trait vielleicht fixen
// #[file_serial]
// #[case(
//     "def eq(self, other: int) for int -> bool {
//         return self == other;
//     }

//     def main() -> int {
//         let a = 3;
//         let b = 4;
//         let mut res = 0;

//         if a == 3 {
//             res = res + 1;
//         }

//         if b == 3 {
//             res = res + 1;
//         }

//         if a == 4 {
//             res = res + 1;
//         }

//         return res;
//     }",
//     Ok(1)
// )]
#[file_serial]
#[case(
    "def is_prime(self) for int64 -> bool {
        if self < 2 {
            return false;
        }

        let mut i = 2;

        while i < self {
            if self % i == 0 {
                return false;
            }
            i = i + 1;
        }

        return true;
    } 

    def main() -> int64 {
        let a = 10;
        let b = 11;

        if a.is_prime() {
            return 0;
        }

        if b.is_prime() {
            return 1;
        }

        return 0;
    }",
    Ok(1)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        if true {
            if true {
                return 1;
            }
        }
    
        return 1;
    }",
    Ok(1)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        if true {
            if true {
                return 1;
            } else {
                return 0;
            }
        }
    
        return 1;
    }",
    Ok(1)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        if true {
            if true {
                if true {
                    return 0;
                }
            } else {

            }
        }
        return 1;
    }",
    Ok(0)
)]
#[file_serial]
#[case(
    "def add(self, other: int64) for int64 -> int64 {
        return 4;
    }

    def main() -> int64 {
        let a = 10;
        let b = 20;

        return a + b;
    }",
    Ok(4)
)]
#[file_serial]
#[case(
    "class Foo {
        data: int64,
    }
    
    def add(self, other: Foo) for Foo -> Foo {
        return Foo { data: self.data + other.data };
    }

    def main() -> int64 {
        let a = Foo { data: 10 };
        let b = Foo { data: 20 };

        let c = a + b;

        return c.data;
    }",
    Ok(30)
)]
#[file_serial]
#[case(
    "def eq(self, other: int64) for int64 -> bool {
        return true;
    }

    def main() -> int64 {
        let a = 10;
        let b = 20;

        if a == b {
            return 1;
        }

        return 0,
    }",
    Ok(1)
)]
#[case(
    "def eq(self, other: int64) for int64 -> int64 {
        return true;
    }

    def main() -> int64 {
        let a = 10;
        let b = 20;

        if a == b {
            return 1;
        }

        return 0,
    }",

    Err(())
)]
#[file_serial]
#[case(
    "class Foo {
        data: int64,
    }
    
    def eq(self, other: int64) for int64 -> Foo {
        return Foo { data: 10 };
    } 

    def main() -> int64 {
        let a = 10;
        let b = 20;

        let c = a == b;

        return c.data;
    }",
    Ok(10)
)]
#[file_serial]
#[case(
    "class Foo {
        data: int64,
    }

    def add(self, other: int64) for Foo -> Foo {
        return Foo { data: self.data + other };
    }

    def main() -> int64 {
        let a = Foo { data: 10 };
        let b = 20;

        let c = a + b;

        return c.data;
    }",
    Ok(30)
)]
#[file_serial]
#[case(
    "class Foo {
        data: int64,
    }

    def add(self, other: int64) for Foo -> Foo {
        return Foo { data: self.data + other };
    }

    def add(self, other: Foo) for Foo -> Foo {
        return Foo { data: self.data + other.data };
    }

    def main() -> int64 {
        let a = Foo { data: 10 };
        let b = 20;

        let c = a + b;

        let a = Foo { data: 10 };
        let b = Foo { data: 20 };

        let d = a + b;

        return c.data + d.data;
    }",
    Ok(60)
)]
#[file_serial]
#[case(
    "
        def main() -> int64 {
            let mut x = &15;
            ~x = 20;

            let mut y = 10;
            let mut z = &y;

            ~z = 30;

            let a = (~x) + (~z);

            return a;
        }
    ",
    Ok(50)
)]
// #[file_serial]
// #[case(
//     "class Foo {
//         data: int,
//     }

//     def idx(self, idx: int) for Foo -> int {
//         return self.data;
//     }

//     def idx(self, idx: int) for int -> int {
//         return self;
//     }

//     def main() -> int {
//         let a = Foo { data: 10 };
//         let b = 20;

//         let c = a[0];
//         let d = b[0];

//         return (c + d)[-420];
//     }",
//     Ok(30)
// )]
// #[file_serial]
// #[case(
//     "class Foo {
//         data: int,
//     }

//     class Bar {
//         data: int,
//     }

//     def idx(self, idx: Bar) for Foo -> int {
//         return self.data;
//     }

//     def main() -> int {
//         let a = Foo { data: 10 };
//         let b = Bar { data: 20 };

//         return a[b];
//     }",
//     Ok(10)
// )]
#[file_serial]
#[case(
    "class Vector {
        x: int64,
        y: int64,
    }
    
    def add(self, other: Vector) for Vector -> Vector {
        return Vector {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }

    def eq(self, other: Vector) for Vector -> bool {
        return self.x == other.x && self.y == other.y;
    }

    def main() -> int64 {
        let a = Vector { x: 10, y: 20 };
        let b = Vector { x: 20, y: 30 };

        let c = a + b;

        if c == Vector { x: 30, y: 50 } {
            return 1;
        }

        return 0;   
    }",
    Ok(1)
)]
#[file_serial]
#[case(
    "class SmallList {
        first: int64,
        second: int64,
        third: int64,
    }

    def idx(self, idx: int64) for SmallList -> *int64 {
        if idx == 0 {
            return &self.first;
        } else if idx == 1 {
            return &self.second;
        } else {
            return &self.third;
        }
    }

    def sum(self) for SmallList -> int64 {
        return self[0] + self[1] + self[2];
    }

    def main() -> int64 {
        let list = SmallList { first: 10, second: 20, third: 30 };

        return list.sum();
    }",
    Ok(60)
)]
#[file_serial]
#[case(
    "class SmallList {
        first: int64,
        second: int64,
        third: int64,
    }

    def idx(self, idx: int64) for SmallList -> *int64 {
        if idx == 0 { 
            return &self.first;
        } else if idx == 1 {
            return &self.second;
        } else {
            return &self.third;
        }
    }

    def sum(self) for SmallList -> int64 {
        return self[0] + self[1] + self[2];
    }

    def main() -> int64 {
        let mut list = SmallList { first: 10, second: 20, third: 30 };

        list[0] = 20;

        return list.sum(); 
        
    }",
    Ok(70)
)]
#[file_serial]
#[case(
    "def idx(self, idx: int64) for int64 -> *int64 {
        return &(self * idx);
    }
    
    def main() -> int64 {
        let a = 10;
        let b = 20;

        let c = a[2];
        let d = b[3]; 

        return c + d;
    }",
    Ok(80)
)]
#[file_serial]
#[case(
    "def inner<T>(x: T) -> T {
        return x;
    }
    
    def outer<T>(x: T) -> T {
        return inner<T>(x);
    }
    
    
    def main() -> int64 {
        return outer<int64>(42);
    }",
    Ok(42)
)]
#[file_serial]
#[case(
    "class Foo<T> {
        inner: T;
    }
    
    def idx<T>(self, idx: int64) for Foo<T> -> *T {
        return &self.inner;
    }
    
    
    def main() -> int64 {
    
        let foo = Foo { inner: 15 };
        let inner = foo[0];
    
    
        return inner;
    }",
    Ok(15)
)]
#[file_serial]
#[case(
    "class Foo<T> {
        inner: T;
    }
    
    def foo(foo: *Foo<int64>) -> int64 {
        return 0;
    }

    def main() -> int64 {
        let foo = Foo { inner: 15 };
        return foo(&foo);
    }
    ",
    Ok(0)
)]
#[file_serial]
#[case(
    "class Foo<T> {
        inner: T;
    }
    
    def foo(foo: *Foo<int64>) -> int64 {
        return 0;
    }

    def main() -> int64 {
        let foo = Foo { inner: 15 };
        return foo(foo);
    }
    ",
    Err(())
)]
#[file_serial]
#[case(
    "class Foo<T> {
        inner: T,
    }
    
    def test(self) for Foo<int64> -> int64 {
        return 0;
    }
    
    def inner<T>(self) for Foo<T> -> T {
        return self.inner;
    }
    
    def main() -> int64 {
        let f = Foo<int64>{ inner: 42 };
    
        f.test();
    
        let ret_val = f.inner<int64>();
        return ret_val;
    }",
    Ok(42)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        return 4.0 as int64;
    }",
    Ok(4)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        let a = 1.0;
        let b = 3.90;

        return (a + b) as int64;
    }",
    Ok(4)
)]
#[file_serial]
#[case(
    "def main() -> int64 {
        let a = 120 as float;
        let b = 3.90;

        return (a + b) as int64;
    }",
    Ok(123)
)]
#[file_serial]
#[case(
    "# comment
    def main() -> int64 {
        # return 1;
        return 11;
    }",
    Ok(11)
)]
#[file_serial]
#[case(
    "def foo() {}
    def main() -> int64 {
        let x = foo();
        return 0;
    }",
    Err(())
)]
#[file_serial]
#[case(
    "
    extern def calloc(num: int64, size: int64) -> int64

    def _calloc<T>(num: int64) -> *T {
        return calloc(num, size_of(T)) as *T
    }

    class List<T> {
        data: *T,
        len: int64,
        cap: int64,
    }

    def with_capacity<T>(cap: int64) for List -> List<T> {
        return List {
            data: _calloc<T>(cap * size_of(T)),
            len: 0,
            cap: cap,
        }
    }

    def swap<T>(self, i: int64, j: int64) for List<T> {
        let foo = 0;
        # let tmp = self[i];
        #self[i] = self[j];
        #self[j] = tmp;
    }

    def main() -> int64 {
        let my_list: List<int64> = List::with_capacity<int64>(10);

        my_list.swap(0, 1);

        return 0;
    }",
    Ok(0)
)]

fn end_to_end_test(#[case] source_code: &'static str, #[case] expected: Result<i32, ()>) {
    let result = compile_and_run(source_code);

    assert_eq!(result, expected);
}

fn compile_and_run(source_code: &str) -> Result<i32, ()> {
    let ir = match Compiler::compile(source_code, None) {
        Ok(out) => out,
        Err(_) => {
            return Err(());
        }
    };

    let mut out = std::process::Command::new("clang")
        .args(["-x", "ir", "-", "-o", TEMP_FILE])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    let mut stdin = out.stdin.as_ref().unwrap();
    stdin.write_all(ir.as_bytes()).unwrap();

    let status = out.wait().unwrap();

    if status.code().unwrap() != 0 {
        return Err(());
    }

    while !std::path::Path::new(&TEMP_FILE).exists() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let mut out = std::process::Command::new(TEMP_FILE)
        .spawn()
        .expect("failed to execute process");

    let status = out.wait().unwrap();
    let code = status.code().unwrap();

    Ok(code)
}

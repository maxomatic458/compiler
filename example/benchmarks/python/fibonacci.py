from time import time

def fib(n: int) -> int:
    if n < 2:
        return n
    return fib(n-1) + fib(n-2)


if __name__ == '__main__':
    start = time()
    print(fib(35))
    print((time() - start) * 1000)
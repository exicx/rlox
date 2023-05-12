def fib(limit):
  a = 0
  temp = 0
  b = 1

  while (a < limit):
    temp = a
    a = b
    b = temp + b

def bench():
  j = 0
  while (j < 1000000):
    fib(100000)
    j = j + 1

bench()

import sys
import pyrscel

if len(sys.argv) == 3:
    print(pyrscel.eval(*sys.argv[1:]))
else:
    for i in range(10000):
        prog = f"{i} + 3"
        assert pyrscel.eval(prog, "{}") == (i + 3)

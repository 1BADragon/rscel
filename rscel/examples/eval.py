import sys
import rscel

if len(sys.argv) == 2:
    print(rscel.eval(sys.argv[1], {}))
elif len(sys.argv) == 3:
    print(rscel.eval(*sys.argv[1:]))
else:
    for i in range(10000):
        prog = "foo + 3"
        assert rscel.eval(prog, {"foo": i}) == (i + 3)

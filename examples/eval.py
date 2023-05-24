import sys
import rscel

if len(sys.argv) == 3:
    print(rscel.eval(*sys.argv[1:]))
else:
    for i in range(10000):
        prog = "foo + 3"
        assert rscel.eval(prog, f'{{"foo": {i}}}') == (i + 3)

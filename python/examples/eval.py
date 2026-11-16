import sys
import time

import rscel

if len(sys.argv) == 2:
    print(rscel.eval(sys.argv[1], {}))
elif len(sys.argv) == 3:
    print(rscel.eval(*sys.argv[1:]))
else:
    start_time = time.time()
    for i in range(10000):
        prog = "foo + 3"
        assert rscel.eval(prog, {"foo": i}) == (i + 3)
    print(f"{time.time() - start_time}")

    start_time = time.time()
    for i in range(10000):
        c = rscel.CelContext()
        c.add_program_str("entry", "foo + 3")
        b = rscel.BindContext()
        b.bind("foo", i)
        assert c.exec("entry", b) == (i + 3)
    print(f"{time.time() - start_time}")

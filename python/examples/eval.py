import sys
import rscel
import time

if len(sys.argv) == 2:
    print(rscel.eval(sys.argv[1], {}))
elif len(sys.argv) == 3:
    print(rscel.eval(*sys.argv[1:]))
else:
    start_time = time.time()
    for i in range(10000):
        prog = "foo + 3"
        assert rscel.eval(prog, {"foo": i}) == (i + 3)
    print(f'{time.time() - start_time}')

    start_time = time.time()
    for i in range(10000):
        c = rscel.CelContext()
        c.add_program_str('entry', "foo + 3")
        b = rscel.BindContext()
        b.bind('foo', i)
        assert c.exec('entry', b) == (i + 3)
    print(f'{time.time() - start_time}')

    p = rscel.CelProgram()
    p.add_source("foo + 3")
    s = p.serialize_to_bincode()
    start_time = time.time()
    for i in range(10000):
        c = rscel.CelContext()
        p = rscel.CelProgram()
        p.add_serialized_bincode(s)

        c.add_program('entry', p)

        b = rscel.BindContext()
        b.bind('foo', i)
        assert c.exec('entry', b) == (i + 3)
    print(f'{time.time() - start_time}')

import gzip, struct, sys

TAG_END=0; TAG_BYTE=1; TAG_SHORT=2; TAG_INT=3; TAG_LONG=4; TAG_FLOAT=5; TAG_DOUBLE=6
TAG_BYTE_ARRAY=7; TAG_STRING=8; TAG_LIST=9; TAG_COMPOUND=10; TAG_INT_ARRAY=11; TAG_LONG_ARRAY=12

def read_payload(f, t):
    if t==TAG_BYTE: return struct.unpack('>b', f.read(1))[0]
    if t==TAG_SHORT: return struct.unpack('>h', f.read(2))[0]
    if t==TAG_INT: return struct.unpack('>i', f.read(4))[0]
    if t==TAG_LONG: return struct.unpack('>q', f.read(8))[0]
    if t==TAG_FLOAT: return struct.unpack('>f', f.read(4))[0]
    if t==TAG_DOUBLE: return struct.unpack('>d', f.read(8))[0]
    if t==TAG_STRING:
        n=struct.unpack('>h', f.read(2))[0]; return f.read(n).decode('utf-8')
    if t==TAG_BYTE_ARRAY:
        n=struct.unpack('>i', f.read(4))[0]; return list(f.read(n))
    if t==TAG_INT_ARRAY:
        n=struct.unpack('>i', f.read(4))[0]; return struct.unpack(f'>{n}i', f.read(4*n))
    if t==TAG_LONG_ARRAY:
        n=struct.unpack('>i', f.read(4))[0]; return struct.unpack(f'>{n}q', f.read(8*n))
    if t==TAG_LIST:
        ct=struct.unpack('>b', f.read(1))[0]; n=struct.unpack('>i', f.read(4))[0]
        return [read_payload(f, ct) for _ in range(n)]
    if t==TAG_COMPOUND:
        d={}
        while True:
            t2=struct.unpack('>b', f.read(1))[0]
            if t2==TAG_END: break
            n=struct.unpack('>h', f.read(2))[0]; name=f.read(n).decode('utf-8')
            d[name]=read_payload(f, t2)
        return d
    raise ValueError(f'unknown tag {t}')

def parse(path):
    with gzip.open(path,'rb') as f:
        t=struct.unpack('>b', f.read(1))[0]
        n=struct.unpack('>h', f.read(2))[0]; name=f.read(n).decode('utf-8')
        return read_payload(f, t)

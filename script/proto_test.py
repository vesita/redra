import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), 'proto'))

import declare_pb2


def main():
    trailer = trailer_pb2.Trailer()
    trailer.scale = 4
    trailer.size = 0x1000001
    
    code = trailer.SerializeToString()
    
    print(len(code))
    
if __name__ == '__main__':
    main()
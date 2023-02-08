import socket
import sys
import time

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
addr = ('localhost', 26099)
sock.bind(addr)


print("pinging...", file=sys.stderr)
while True:
        buf, raddr = sock.recvfrom(4096)
        print("receive: " + buf.decode("utf-8"))
        buf = "this is a ping to port 6200!".encode('utf-8')
        sock.sendto(buf, ("127.0.0.1", 6200))
        buf = "this is a ping to reply!".encode('utf-8')
        sock.sendto(buf, raddr)
        time.sleep(1)

import logging
import socket

logging.basicConfig(format="%(asctime)s %(message)s", level=logging.INFO)

serversocket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
serversocket.bind(('10.42.0.1', 4000))
serversocket.listen(5)
logging.info("Listening on 10.42.0.1:4000")

try:
    while True:
        (clientsocket, address) = serversocket.accept()

        request = clientsocket.recv(1000)
        temp = int(request.decode()) / 1000
        msg = f"Received: {temp} F"
        logging.info(msg)
        clientsocket.send(msg.encode())
        clientsocket.close()
except KeyboardInterrupt:
    print("shutting down")
    serversocket.close()

import socket
import json

HOST = '0.0.0.0'
PORT = 1555
OUTPUT_FILE = 'metrics.out'

def listen_udp_and_write():
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s, open(OUTPUT_FILE, 'a') as f:
        s.bind((HOST, PORT))
        print(f"Listening on UDP {HOST}:{PORT}, writing to {OUTPUT_FILE}")

        while True:
            data, addr = s.recvfrom(65535)  # Max UDP packet size
            try:
                obj = json.loads(data.decode())
                json_line = json.dumps(obj)
                f.write(json_line + '\n')
                f.flush()  # Ensure it's written immediately
                print(f"Received and wrote JSON from {addr}")
            except json.JSONDecodeError as e:
                print(f"Invalid JSON from {addr}: {e}")

if __name__ == "__main__":
    listen_udp_and_write()


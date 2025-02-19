import http.server
import socketserver

PORT = 8001

Handler = http.server.SimpleHTTPRequestHandler

http = socketserver.TCPServer(("", PORT), Handler)

print(f"Serving at port {PORT}")

try:
    http.serve_forever()
finally:
    http.server_close()
import ws
import ws.ws as wsmain

def main():
    wsmain.start_websocket_server()

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("WebSocket server terminated by SIGINT (Ctrl+C)")
        exit(0)
    except Exception as e:
        exit(1)
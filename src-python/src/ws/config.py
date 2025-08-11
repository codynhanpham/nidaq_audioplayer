"""
WebSocket Server Configuration

Configuration settings for the WebSocket audio server.

Most of the settings here are unused for now
"""

# Server settings
WEBSOCKET_HOST = "localhost"
WEBSOCKET_PORT = 21749

# Audio settings
DEFAULT_VOLUME = 1.0
SUPPORTED_AUDIO_FORMATS = ['.wav', '.mp3', '.flac', '.ogg', '.m4a', '.aiff']

# Response settings
INCLUDE_TIMESTAMP = True
INCLUDE_SERVER_INFO = True

# Logging settings
LOG_LEVEL = "INFO"
LOG_FORMAT = "%(asctime)s - %(name)s - %(levelname)s - %(message)s"

# Security settings (for future enhancement)
ENABLE_CORS = True
ALLOWED_ORIGINS = ["*"]  # Be more restrictive in production

# Performance settings
MAX_MESSAGE_SIZE = 1024 * 1024  # 1MB
CONNECTION_TIMEOUT = 300  # 5 minutes

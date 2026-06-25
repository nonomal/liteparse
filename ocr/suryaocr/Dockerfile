FROM ghcr.io/astral-sh/uv:python3.12-trixie

# System libraries needed by torch / Pillow / Surya
RUN apt-get update && apt-get install -y \
    libgomp1 \
    libglib2.0-0 \
    libsm6 \
    libxext6 \
    libxrender-dev \
    libgl1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy project metadata, lockfile, and server code
COPY pyproject.toml uv.lock README.md server.py test_server.py ./

# Install dependencies
RUN uv sync --frozen

# Expose port
EXPOSE 8830

# Run server
CMD ["uv", "run", "python3", "server.py"]

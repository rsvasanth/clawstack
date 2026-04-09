FROM python:3.12-slim

WORKDIR /app
COPY python/ /app/python/
COPY pyproject.toml /app/

RUN pip install uv && uv sync --frozen

EXPOSE 8000
ENTRYPOINT ["uv", "run", "fastapi", "dev", "learning/main.py"]

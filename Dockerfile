FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY __init__.py __main__.py schema.py db.py diff.py ./
ENTRYPOINT ["python", "__main__.py"]

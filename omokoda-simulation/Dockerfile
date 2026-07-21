# Ọmọ Kọ́dà — Ògún executor / simulation dev container (Python).
# executor.py is a library the Steward calls; this image provides its runtime
# so you can `docker compose exec executor python -m pytest` or import it.

FROM python:3.12-slim
WORKDIR /app

COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

# py_compile as a build-time smoke check; then idle so the dev env stays available.
RUN python -m py_compile executor.py simulation.py 2>/dev/null || python -m py_compile executor.py
CMD ["sleep", "infinity"]

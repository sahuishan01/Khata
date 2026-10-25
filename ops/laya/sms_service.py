"""Private, resident Laya decision service. Never stores or logs SMS bodies."""
import asyncio
import os
from contextlib import asynccontextmanager
from pathlib import Path

from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
from starlette.concurrency import run_in_threadpool

from sms_policy import MODEL_VERSION, contains_otp, cpu_idle_enough, decide


async def cpu_ready():
    def counters():
        return list(map(int, Path('/proc/stat').read_text().splitlines()[0].split()[1:9]))
    try:
        before = counters()
        await asyncio.sleep(.2)
        return cpu_idle_enough(before, counters())
    except (OSError, ValueError, IndexError):
        return False


@asynccontextmanager
async def lifespan(app):
    import torch
    import laya
    torch.set_num_threads(2)
    torch.set_num_interop_threads(1)
    app.state.agent = laya.load(os.environ["KHATA_LAYA_MODEL_DIR"], device="cpu", backend="eager")
    app.state.slots = asyncio.Semaphore(1)
    yield


app = FastAPI(lifespan=lifespan, docs_url=None, redoc_url=None, openapi_url=None)


@app.get("/health")
async def health():
    return {"status": "ok", "model_version": MODEL_VERSION}


@app.post("/decide")
async def classify(request: Request):
    # Read a bounded body without FastAPI reflecting raw input in error responses.
    raw = bytearray()
    async for part in request.stream():
        raw.extend(part)
        if len(raw) > 70000:
            return JSONResponse({"error": "SMS request too large"}, status_code=413)
    try:
        import json
        payload = json.loads(raw)
        body = payload["body"]
        if not isinstance(body, str) or not 0 < len(body) <= 16000:
            raise ValueError()
    except (ValueError, KeyError, TypeError):
        return JSONResponse({"error": "Invalid SMS request"}, status_code=400)
    if contains_otp(body):
        return JSONResponse({"error": "Authentication messages are not processed"}, status_code=400)
    if app.state.slots.locked() or not await cpu_ready():
        return JSONResponse({"error": "Classifier busy"}, status_code=503)
    async with app.state.slots:
        try:
            return await run_in_threadpool(decide, app.state.agent, body)
        except Exception:
            # No request bodies, model exception text or validation input in logs.
            return JSONResponse({"error": "Classifier unavailable"}, status_code=503)

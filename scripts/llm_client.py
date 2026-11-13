#!/usr/bin/env python3
"""
llm_client.py - single entry point for every model call in this project.

Design rules (from the project brief):
  * LLM output is untrusted -> never used without a validator.
  * Every call is logged: model name, prompt sha256, token counts, wall time.
  * Low temperature for translation and repair.
  * Free/local only: the paid Anthropic API is out of budget, so the default
    backend is the local llama.cpp server (CUDA, all layers on GPU).

Backends
  local  : llama.cpp OpenAI-compatible server (http://127.0.0.1:PORT/v1)
  opencode : the `opencode run` CLI (used only for the free hosted models)

Call log: logs/llm_calls.jsonl (one JSON object per call)
Cache   : artifacts/llm_cache/<sha256>.json  (prompt+params -> response)
"""
from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request
from dataclasses import dataclass, field, asdict
from typing import Any

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE_DIR = os.path.join(ROOT, "artifacts", "llm_cache")
CALL_LOG = os.path.join(ROOT, "logs", "llm_calls.jsonl")
DEFAULT_URL = "http://127.0.0.1:8080/v1/chat/completions"
DEFAULT_API_KEY = "local"


# --------------------------------------------------------------------------- #
# config
# --------------------------------------------------------------------------- #
@dataclass
class LLMConfig:
    backend: str = "local"           # "local" | "opencode"
    model: str = "local"             # "local" -> llama.cpp server model id
    url: str = DEFAULT_URL
    api_key: str = DEFAULT_API_KEY
    temperature: float = 0.0
    max_tokens: int = 2048
    timeout: int = 900
    use_cache: bool = True
    # opencode backend
    opencode_bin: str = os.path.expanduser("~/.opencode/bin/opencode")

    def fingerprint(self) -> dict:
        return {
            "backend": self.backend,
            "model": self.model,
            "temperature": self.temperature,
            "max_tokens": self.max_tokens,
        }


# --------------------------------------------------------------------------- #
# prompt hashing + logging
# --------------------------------------------------------------------------- #
def prompt_hash(payload: Any) -> str:
    blob = json.dumps(payload, sort_keys=True, ensure_ascii=False, default=str)
    return hashlib.sha256(blob.encode("utf-8")).hexdigest()


def _log_call(rec: dict) -> None:
    os.makedirs(os.path.dirname(CALL_LOG), exist_ok=True)
    with open(CALL_LOG, "a") as fh:
        fh.write(json.dumps(rec, ensure_ascii=False) + "\n")


def _cache_path(key: str) -> str:
    return os.path.join(CACHE_DIR, key + ".json")


def cache_get(key: str) -> dict | None:
    p = _cache_path(key)
    if os.path.isfile(p):
        try:
            with open(p) as fh:
                return json.load(fh)
        except Exception:
            return None
    return None


def cache_put(key: str, value: dict) -> None:
    os.makedirs(CACHE_DIR, exist_ok=True)
    p = _cache_path(key)
    tmp = p + ".tmp"
    with open(tmp, "w") as fh:
        json.dump(value, fh, ensure_ascii=False, indent=2)
    os.makedirs(os.path.dirname(p), exist_ok=True)
    os.replace(tmp, p)


# --------------------------------------------------------------------------- #
# backends
# --------------------------------------------------------------------------- #
def _call_local(cfg: LLMConfig, messages: list[dict]) -> dict:
    body = {
        "model": cfg.model,
        "messages": messages,
        "temperature": cfg.temperature,
        "max_tokens": cfg.max_tokens,
        "cache_prompt": False,
        "stream": False,
    }
    req = urllib.request.Request(
        cfg.url,
        data=json.dumps(body).encode("utf-8"),
        headers={
            "Content-Type": "application/json",
            "Authorization": "Bearer " + cfg.api_key,
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=cfg.timeout) as r:
            raw = json.load(r)
    except urllib.error.HTTPError as e:
        err_body = e.read().decode("utf-8", errors="ignore")
        raise RuntimeError(f"LLM API request failed (HTTP {e.code}): {err_body}") from e
    text = raw["choices"][0]["message"]["content"]
    usage = raw.get("usage") or {}
    timings = raw.get("timings") or {}
    return {
        "text": text,
        "usage": {
            "prompt_tokens": usage.get("prompt_tokens", 0),
            "completion_tokens": usage.get("completion_tokens", 0),
            "total_tokens": usage.get("total_tokens", 0),
        },
        "timings": {
            "prompt_per_second": timings.get("prompt_per_second"),
            "predicted_per_second": timings.get("predicted_per_second"),
        },
        "raw_model": raw.get("model"),
    }


_FENCE_RE = re.compile(r"```(?:[A-Za-z0-9_+-]*)\n(.*?)```", re.S)


def _call_opencode(cfg: LLMConfig, messages: list[dict]) -> dict:
    """Fallback backend: the `opencode run` CLI against the free hosted models."""
    prompt = "\n\n".join(m["content"] for m in messages)
    cmd = [cfg.opencode_bin, "run", "--model", cfg.model, prompt]
    t0 = time.time()
    out = subprocess.run(cmd, capture_output=True, text=True, timeout=cfg.timeout)
    dt = time.time() - t0
    if out.returncode != 0:
        raise RuntimeError(f"opencode CLI failed: {out.stderr[-2000:]}")
    text = out.stdout
    # rough token accounting for the CLI backend (no usage metadata available)
    est = len(text) // 4 + len(prompt) // 4
    return {
        "text": text,
        "usage": {
            "prompt_tokens": len(prompt) // 4,
            "completion_tokens": len(text) // 4,
            "total_tokens": est,
            "note": "estimated (CLI backend returns no usage metadata)",
        },
        "timings": {"predicted_per_second": (len(text) // 4) / dt if dt else None},
        "raw_model": cfg.model,
    }


# --------------------------------------------------------------------------- #
# public API
# --------------------------------------------------------------------------- #
@dataclass
class LLMResult:
    text: str
    usage: dict
    timings: dict
    model: str
    prompt_sha256: str
    cached: bool
    wall_seconds: float
    raw: dict = field(default_factory=dict)

    def code_blocks(self) -> list[str]:
        """All fenced code blocks in the response, in order."""
        return [m.group(1) for m in _FENCE_RE.finditer(self.text)]

    def first_code_block(self) -> str | None:
        b = self.code_blocks()
        return b[0] if b else None


def call(messages: list[dict], cfg: LLMConfig | None = None, tag: str = "") -> LLMResult:
    cfg = cfg or LLMConfig()
    payload = {"messages": messages, **cfg.fingerprint()}
    ph = prompt_hash(payload)
    key = ph

    if cfg.use_cache:
        hit = cache_get(key)
        if hit is not None:
            _log_call({
                "ts": time.strftime("%Y-%m-%dT%H:%M:%S"),
                "tag": tag,
                "cached": True,
                "prompt_sha256": ph,
                "model": cfg.model,
                "backend": cfg.backend,
                "usage": hit.get("usage"),
                "wall_seconds": hit.get("wall_seconds"),
            })
            return LLMResult(
                text=hit["text"], usage=hit.get("usage", {}), timings=hit.get("timings", {}),
                model=hit.get("model", cfg.model), prompt_sha256=ph, cached=True,
                wall_seconds=hit.get("wall_seconds", 0.0), raw=hit.get("raw", {}),
            )

    t0 = time.time()
    if cfg.backend == "local":
        r = _call_local(cfg, messages)
    elif cfg.backend == "opencode":
        r = _call_opencode(cfg, messages)
    else:
        raise ValueError(f"unknown backend {cfg.backend}")
    dt = time.time() - t0

    rec = LLMResult(
        text=r["text"], usage=r["usage"], timings=r["timings"],
        model=r.get("raw_model", cfg.model), prompt_sha256=ph, cached=False,
        wall_seconds=dt, raw={"raw_model": r.get("raw_model")},
    )
    if cfg.use_cache:
        cache_put(key, {
            "text": rec.text, "usage": rec.usage, "timings": rec.timings,
            "model": rec.model, "wall_seconds": dt, "raw": rec.raw,
        })
    _log_call({
        "ts": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "tag": tag,
        "cached": False,
        "prompt_sha256": ph,
        "model": rec.model,
        "backend": cfg.backend,
        "usage": rec.usage,
        "timings": rec.timings,
        "wall_seconds": round(dt, 3),
        "temp": cfg.temperature,
        "max_tokens": cfg.max_tokens,
    })
    return rec


def ask(prompt: str, system: str = "", cfg: LLMConfig | None = None, tag: str = "") -> LLMResult:
    msgs = []
    if system:
        msgs.append({"role": "system", "content": system})
    msgs.append({"role": "user", "content": prompt})
    return call(msgs, cfg=cfg, tag=tag)


if __name__ == "__main__":
    cfg = LLMConfig()
    if len(sys.argv) > 1 and sys.argv[1] == "--ping":
        r = ask("Reply with exactly: PIPELINE_OK", cfg=cfg, tag="ping")
        print(r.text.strip())
        print("usage:", r.usage, "model:", r.model)

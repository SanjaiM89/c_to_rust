#!/usr/bin/env python3
"""
FrontEnd Server for c_to_rust Transpilation Engine
Provides real-time SSE streaming, Web VS Code API, NVIDIA GPU & System Telemetry.
Zero external dependencies (uses standard library http.server).
"""

import os
import sys
import json
import time
import shutil
import urllib.parse
import urllib.request
import subprocess
import threading
import queue
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(BASE_DIR)

# Global state for current conversion session
conversion_lock = threading.Lock()
current_session = {
    "is_running": False,
    "source_url": "",
    "crate_name": "",
    "crate_dir": "",
    "zip_path": "",
    "latest_c_code": "",
    "latest_rust_code": "",
    "stages": [],
    "logs": []
}

# Active SSE subscriber queues
sse_clients = []
sse_lock = threading.Lock()

def broadcast_event(event_type: str, data: dict):
    payload = f"event: {event_type}\ndata: {json.dumps(data)}\n\n"
    with sse_lock:
        stale = []
        for q in sse_clients:
            try:
                q.put_nowait(payload)
            except Exception:
                stale.append(q)
        for q in stale:
            if q in sse_clients:
                sse_clients.remove(q)

def get_gpu_telemetry():
    telemetry = {
        "available": False,
        "name": "N/A",
        "utilization": 0.0,
        "memory_used": 0.0,
        "memory_total": 0.0,
        "temperature": 0.0,
        "power_draw": 0.0
    }
    try:
        out = subprocess.check_output(
            ["nvidia-smi", "--query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu,power.draw,name", "--format=csv,noheader,nounits"],
            text=True, timeout=2
        ).strip()
        parts = [p.strip() for p in out.split(",")]
        if len(parts) >= 6:
            telemetry = {
                "available": True,
                "utilization": float(parts[0]),
                "memory_used": float(parts[1]),
                "memory_total": float(parts[2]),
                "temperature": float(parts[3]),
                "power_draw": float(parts[4]),
                "name": parts[5]
            }
    except Exception as e:
        telemetry["error"] = str(e)
    return telemetry

_last_cpu_idle = 0.0
_last_cpu_total = 0.0

def get_system_telemetry():
    global _last_cpu_idle, _last_cpu_total

    # RAM from /proc/meminfo
    ram = {"total_gb": 0.0, "used_gb": 0.0, "percent": 0.0}
    try:
        with open("/proc/meminfo") as f:
            lines = f.readlines()
        mem = {}
        for l in lines:
            parts = l.split(":")
            if len(parts) == 2:
                mem[parts[0].strip()] = int(parts[1].split()[0])
        total = mem.get("MemTotal", 0) / (1024 * 1024)
        avail = mem.get("MemAvailable", 0) / (1024 * 1024)
        used = total - avail
        pct = (used / total * 100) if total else 0
        ram = {"total_gb": round(total, 2), "used_gb": round(used, 2), "percent": round(pct, 1)}
    except Exception:
        pass

    # CPU from /proc/stat
    cpu_pct = 0.0
    try:
        with open("/proc/stat") as f:
            fields = [float(x) for x in f.readline().split()[1:8]]
        idle = fields[3] + fields[4]
        total = sum(fields)
        if _last_cpu_total > 0:
            diff_total = total - _last_cpu_total
            diff_idle = idle - _last_cpu_idle
            if diff_total > 0:
                cpu_pct = round((1.0 - diff_idle / diff_total) * 100.0, 1)
        _last_cpu_idle = idle
        _last_cpu_total = total
    except Exception:
        pass

    # Disk usage
    disk = {"total_gb": 0.0, "used_gb": 0.0, "percent": 0.0}
    try:
        st = os.statvfs(PROJECT_ROOT)
        total_gb = (st.f_blocks * st.f_frsize) / (1024**3)
        free_gb = (st.f_bavail * st.f_frsize) / (1024**3)
        used_gb = total_gb - free_gb
        pct = (used_gb / total_gb * 100) if total_gb else 0
        disk = {"total_gb": round(total_gb, 1), "used_gb": round(used_gb, 1), "percent": round(pct, 1)}
    except Exception:
        pass

    # Local LLM health
    llm = {"online": False, "model": "offline", "port": 8080}
    try:
        req = urllib.request.Request("http://127.0.0.1:8080/v1/models", headers={"Authorization": "Bearer local"})
        with urllib.request.urlopen(req, timeout=1) as resp:
            data = json.load(resp)
            if "data" in data and len(data["data"]) > 0:
                llm = {"online": True, "model": data["data"][0].get("id", "Qwen3-4B-Instruct"), "port": 8080}
    except Exception:
        pass

    return {
        "gpu": get_gpu_telemetry(),
        "ram": ram,
        "cpu": {"percent": max(0.0, min(100.0, cpu_pct)), "cores": os.cpu_count() or 4},
        "disk": disk,
        "llm": llm
    }

def run_conversion_worker(source_url: str):
    global current_session
    with conversion_lock:
        current_session["is_running"] = True
        current_session["source_url"] = source_url

    broadcast_event("status", {"state": "started", "source": source_url})
    broadcast_event("stage", {"id": 1, "name": "Repository Ingestion", "desc": f"Cloning {source_url}..."})
    broadcast_event("command", {"cmd": f"git clone {source_url}", "status": "running"})

    script_path = os.path.join(PROJECT_ROOT, "scripts", "convert_repo.py")
    out_dir = PROJECT_ROOT
    env = os.environ.copy()
    env["PATH"] = f"{os.path.expanduser('~/.cargo/bin')}:{env.get('PATH', '')}"

    cmd = [sys.executable, "-u", script_path, "--out", out_dir, source_url]
    
    try:
        proc = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
            cwd=PROJECT_ROOT,
            env=env
        )

        in_c_code = False
        in_rust_code = False
        c_buffer = []
        rust_buffer = []

        for line in iter(proc.stdout.readline, ""):
            line_str = line.rstrip()

            # Broadcast raw log
            broadcast_event("log", {"text": line_str})

            # Detect commands & stages
            if "Cloning git repository" in line_str:
                broadcast_event("stage", {"id": 1, "name": "Cloning", "desc": "Repository cloned successfully"})
                broadcast_event("command", {"cmd": f"git clone {source_url}", "status": "completed"})
            elif "Found C library files" in line_str:
                broadcast_event("stage", {"id": 2, "name": "AST Analysis", "desc": "Decomposing C headers and source units"})
            elif "Translating" in line_str:
                broadcast_event("stage", {"id": 3, "name": "LLM Synthesis", "desc": "Synthesizing safe Rust crate..."})
            elif "Verifying compilation (`cargo check`)" in line_str:
                broadcast_event("stage", {"id": 4, "name": "Verification", "desc": "Running compiler borrow & type checker"})
                broadcast_event("command", {"cmd": "cargo check --message-format=short", "status": "running"})
            elif "Compiler repair attempt" in line_str:
                broadcast_event("stage", {"id": 4, "name": "Automated Repair", "desc": line_str})
            elif "`cargo check` PASSED" in line_str:
                broadcast_event("command", {"cmd": "cargo check", "status": "passed"})
            elif "Running test suite (`cargo test`)" in line_str:
                broadcast_event("command", {"cmd": "cargo test", "status": "running"})
            elif "`cargo test` PASSED" in line_str:
                broadcast_event("command", {"cmd": "cargo test", "status": "passed"})
            elif "Packaging Rust crate" in line_str:
                broadcast_event("stage", {"id": 5, "name": "Packaging", "desc": "Compressing crate into .zip package"})

            # Detect C Code streaming block
            if "📄 [C INPUT CODE" in line_str:
                in_c_code = True
                c_buffer = []
                continue
            elif in_c_code and ("===" in line_str or "🦀 [GENERATED RUST" in line_str):
                in_c_code = False
                c_code_text = "\n".join(c_buffer)
                current_session["latest_c_code"] = c_code_text
                broadcast_event("c_code", {"code": c_code_text})

            if in_c_code:
                c_buffer.append(line_str)
                if len(c_buffer) % 5 == 0:
                    broadcast_event("c_code_stream", {"chunk": line_str})

            # Detect Rust Code streaming block
            if "🦀 [GENERATED RUST CODE" in line_str or "🔧 [REPAIRED RUST" in line_str:
                in_rust_code = True
                rust_buffer = []
                continue
            elif in_rust_code and ("===" in line_str and len(rust_buffer) > 3):
                in_rust_code = False
                rust_code_text = "\n".join(rust_buffer)
                current_session["latest_rust_code"] = rust_code_text
                broadcast_event("rust_code", {"code": rust_code_text})

            if in_rust_code:
                rust_buffer.append(line_str)
                if len(rust_buffer) % 5 == 0:
                    broadcast_event("rust_code_stream", {"chunk": line_str})

            # Check for output zip package
            if "Package:" in line_str and ".zip" in line_str:
                zip_path = line_str.split("Package:")[1].split("(")[0].strip()
                current_session["zip_path"] = zip_path
                # derive crate name
                crate_name = os.path.basename(zip_path).replace(".zip", "")
                current_session["crate_name"] = crate_name
                current_session["crate_dir"] = os.path.join(PROJECT_ROOT, "target", "converted", crate_name)

        proc.wait()

        if proc.returncode == 0:
            broadcast_event("stage", {"id": 5, "name": "Completed", "desc": "Crate generated & verified successfully!"})
            broadcast_event("complete", {
                "success": True,
                "crate_name": current_session.get("crate_name", "converted_crate"),
                "zip_path": current_session.get("zip_path", ""),
                "download_url": "/api/download"
            })
        else:
            broadcast_event("error", {"message": f"Process exited with returncode {proc.returncode}"})

    except Exception as e:
        broadcast_event("error", {"message": str(e)})
    finally:
        with conversion_lock:
            current_session["is_running"] = False
        broadcast_event("status", {"state": "idle"})

def get_active_crate_dir():
    # If a conversion happened, return that crate dir
    if current_session.get("crate_dir") and os.path.isdir(current_session["crate_dir"]):
        return current_session["crate_dir"]
    # Check inih-rs
    inih_dir = os.path.join(os.path.dirname(PROJECT_ROOT), "inih_to_rust", "inih-rs")
    if os.path.isdir(inih_dir):
        return inih_dir
    # Check artifacts/final
    final_dir = os.path.join(PROJECT_ROOT, "artifacts", "final")
    if os.path.isdir(final_dir):
        return final_dir
    return PROJECT_ROOT

class TranspilerHTTPHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=BASE_DIR, **kwargs)

    def do_GET(self):
        url_parsed = urllib.parse.urlparse(self.path)
        path = url_parsed.path

        # SSE endpoint
        if path == "/api/events":
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Cache-Control", "no-cache")
            self.send_header("Connection", "keep-alive")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()

            client_queue = queue.Queue(maxsize=500)
            with sse_lock:
                sse_clients.append(client_queue)

            # Send initial state
            initial_data = {
                "is_running": current_session["is_running"],
                "crate_name": current_session["crate_name"],
                "has_zip": bool(current_session["zip_path"])
            }
            self.wfile.write(f"event: init\ndata: {json.dumps(initial_data)}\n\n".encode("utf-8"))
            self.wfile.flush()

            try:
                while True:
                    try:
                        msg = client_queue.get(timeout=10)
                        self.wfile.write(msg.encode("utf-8"))
                        self.wfile.flush()
                    except queue.Empty:
                        # Keep-alive heartbeat
                        self.wfile.write(b": heartbeat\n\n")
                        self.wfile.flush()
            except (BrokenPipeError, ConnectionResetError):
                pass
            finally:
                with sse_lock:
                    if client_queue in sse_clients:
                        sse_clients.remove(client_queue)
            return

        # System Telemetry API
        if path == "/api/system_stats":
            stats = get_system_telemetry()
            self.send_json_response(stats)
            return

        # File tree for Web VS Code IDE
        if path == "/api/files":
            crate_dir = get_active_crate_dir()
            file_tree = []
            for root, dirs, files in os.walk(crate_dir):
                # Filter out build artifacts
                dirs[:] = [d for d in dirs if d not in ["target", ".git", "__pycache__"]]
                for f in files:
                    if f.endswith((".rs", ".toml", ".c", ".h", ".md", ".json", ".lock")):
                        full_path = os.path.join(root, f)
                        rel_path = os.path.relpath(full_path, crate_dir)
                        ext = os.path.splitext(f)[1].lstrip(".")
                        lang = "rust" if ext == "rs" else "c" if ext in ["c", "h"] else "toml" if ext == "toml" else "markdown" if ext == "md" else "json"
                        file_tree.append({
                            "path": rel_path,
                            "name": f,
                            "size": os.path.getsize(full_path),
                            "language": lang
                        })
            file_tree.sort(key=lambda x: (not x["path"].startswith("src/"), x["path"]))
            self.send_json_response({"root": os.path.basename(crate_dir), "files": file_tree})
            return

        # File content API for Web VS Code IDE
        if path == "/api/file":
            query = urllib.parse.parse_qs(url_parsed.query)
            target_rel = query.get("path", ["src/lib.rs"])[0]
            crate_dir = get_active_crate_dir()
            full_path = os.path.abspath(os.path.join(crate_dir, target_rel))
            
            # Prevent directory traversal
            if not full_path.startswith(os.path.abspath(crate_dir)):
                self.send_error(403, "Access denied")
                return

            if not os.path.isfile(full_path):
                self.send_error(404, "File not found")
                return

            try:
                with open(full_path, "r", encoding="utf-8", errors="replace") as f:
                    content = f.read()
                self.send_json_response({
                    "path": target_rel,
                    "content": content,
                    "language": "rust" if target_rel.endswith(".rs") else "c" if target_rel.endswith((".c", ".h")) else "toml"
                })
            except Exception as e:
                self.send_error(500, str(e))
            return

        # Download ZIP package
        if path == "/api/download":
            zip_file = current_session.get("zip_path")
            if not zip_file or not os.path.isfile(zip_file):
                # Fallback to inih_rs.zip if exists
                fallback_zip = os.path.join(PROJECT_ROOT, "inih_rs.zip")
                if os.path.isfile(fallback_zip):
                    zip_file = fallback_zip
                else:
                    self.send_error(404, "No converted package available yet")
                    return

            filename = os.path.basename(zip_file)
            size = os.path.getsize(zip_file)
            self.send_response(200)
            self.send_header("Content-Type", "application/zip")
            self.send_header("Content-Disposition", f'attachment; filename="{filename}"')
            self.send_header("Content-Length", str(size))
            self.end_headers()
            with open(zip_file, "rb") as f:
                shutil.copyfileobj(f, self.wfile)
            return

        # Standard static file serving
        return super().do_GET()

    def do_POST(self):
        url_parsed = urllib.parse.urlparse(self.path)
        path = url_parsed.path

        if path == "/api/convert":
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length).decode("utf-8")
            try:
                data = json.loads(body)
                url = data.get("url", "").strip()
                if not url:
                    self.send_json_response({"error": "No URL provided"}, status=400)
                    return

                if current_session["is_running"]:
                    self.send_json_response({"error": "A translation is already running"}, status=409)
                    return

                # Spawn worker thread
                t = threading.Thread(target=run_conversion_worker, args=(url,), daemon=True)
                t.start()

                self.send_json_response({"status": "started", "url": url})
            except Exception as e:
                self.send_json_response({"error": str(e)}, status=500)
            return

        self.send_error(404, "Endpoint not found")

    def send_json_response(self, data, status=200):
        body = json.dumps(data).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(body)

def main():
    port = int(os.environ.get("PORT", 3000))
    server_address = ("0.0.0.0", port)
    httpd = ThreadingHTTPServer(server_address, TranspilerHTTPHandler)
    print(f"=================================================================")
    print(f"🚀 c_to_rust FrontEnd Server Running at: http://localhost:{port}")
    print(f"   • Dashboard:   http://localhost:{port}/")
    print(f"   • Web VS Code: http://localhost:{port}/ide.html")
    print(f"   • Telemetry:   http://localhost:{port}/telemetry.html")
    print(f"=================================================================")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down server...")
        httpd.server_close()

if __name__ == "__main__":
    main()

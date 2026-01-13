"""Safe command execution with timeouts and output truncation."""

import subprocess
import time
from typing import Dict, Any


def safe_exec(cmd: str, timeout: int = 15) -> Dict[str, Any]:
    """
    Execute command safely with timeout and output limits.

    Args:
        cmd: Command to execute
        timeout: Timeout in seconds (max 15s default)

    Returns:
        Dict with keys: code, stdout, stderr, duration, truncated
    """
    start_time = time.time()

    try:
        result = subprocess.run(
            cmd,
            shell=True,
            capture_output=True,
            text=True,
            timeout=timeout,
        )

        duration = time.time() - start_time

        # Truncate output
        stdout = result.stdout
        stderr = result.stderr
        truncated = False

        if len(stdout) > 4000:
            stdout = stdout[:4000] + "\n... (truncated)"
            truncated = True

        if len(stderr) > 2000:
            stderr = stderr[:2000] + "\n... (truncated)"
            truncated = True

        return {
            "code": result.returncode,
            "stdout": stdout,
            "stderr": stderr,
            "duration": round(duration, 2),
            "truncated": truncated,
        }

    except subprocess.TimeoutExpired:
        duration = time.time() - start_time
        return {
            "code": -1,
            "stdout": "",
            "stderr": f"Command timed out after {timeout}s",
            "duration": round(duration, 2),
            "truncated": False,
            "timeout": True,
        }

    except Exception as e:
        duration = time.time() - start_time
        return {
            "code": -1,
            "stdout": "",
            "stderr": f"Execution error: {str(e)}",
            "duration": round(duration, 2),
            "truncated": False,
            "error": str(e),
        }


# CLI demo
if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        print("Usage: python -m runtime.exec '<command>'")
        sys.exit(1)

    cmd = " ".join(sys.argv[1:])
    print(f"Executing: {cmd}\n")

    result = safe_exec(cmd)

    print(f"Exit code: {result['code']}")
    print(f"Duration: {result['duration']}s")
    print(f"Truncated: {result.get('truncated', False)}")
    print(f"\nStdout:\n{result['stdout']}")
    if result['stderr']:
        print(f"\nStderr:\n{result['stderr']}")

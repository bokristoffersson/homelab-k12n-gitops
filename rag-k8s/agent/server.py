"""Server for rag-k8s agent with persistent model loading."""

import json
import sys
from pathlib import Path
from typing import Dict, Any, Optional
from flask import Flask, request, jsonify
from flask_cors import CORS

sys.path.insert(0, str(Path(__file__).parent.parent))

from agent.tool import K8sExecTool
from agent.plan import MLXPlanner
from agent.config import get_model_id
from mlx_lm import load

app = Flask(__name__)
CORS(app)  # Enable CORS for Claude to call

# Global model instance (loaded once at startup)
_planner: Optional[MLXPlanner] = None
_tool: Optional[K8sExecTool] = None


def load_model():
    """Load model once at startup."""
    global _planner, _tool
    if _planner is None:
        print(f"Loading model: {get_model_id()}")
        model, tokenizer = load(get_model_id())
        _planner = MLXPlanner(model_id=get_model_id(), model=model, tokenizer=tokenizer)
        _tool = K8sExecTool()
        _tool.planner = _planner  # Use the pre-loaded planner
        print("Model loaded and ready!")
    return _planner, _tool


@app.route("/health", methods=["GET"])
def health():
    """Health check endpoint."""
    return jsonify({"status": "healthy", "model_loaded": _planner is not None})


@app.route("/k8s-exec", methods=["POST"])
def k8s_exec_endpoint():
    """
    Execute K8s operation.
    
    Request body should conform to K8S_EXEC_SCHEMA:
    {
        "intent": "restart",
        "resource": "deployment",
        "namespace": "prod",
        "name": "api",
        "selector": null,
        "constraints": {
            "dryRun": true
        }
    }
    """
    try:
        payload = request.get_json()
        if not payload:
            return jsonify({"error": "No JSON payload provided"}), 400
        
        # Ensure model is loaded
        planner, tool = load_model()
        
        # Execute operation
        result = tool.execute(payload)
        
        return jsonify(result)
    
    except Exception as e:
        return jsonify({
            "error": "Internal server error",
            "details": str(e)
        }), 500


@app.route("/plan", methods=["POST"])
def plan_endpoint():
    """
    Generate a command plan without executing.
    
    Request body:
    {
        "intent": "restart",
        "resource": "deployment",
        "namespace": "prod",
        "name": "api",
        "selector": null,
        "context": "optional context string"
    }
    """
    try:
        payload = request.get_json()
        if not payload:
            return jsonify({"error": "No JSON payload provided"}), 400
        
        # Ensure model is loaded
        planner, _ = load_model()
        
        # Generate plan
        plan = planner.plan_command(
            intent=payload.get("intent"),
            resource=payload.get("resource"),
            namespace=payload.get("namespace"),
            name=payload.get("name"),
            selector=payload.get("selector"),
            context=payload.get("context", "")
        )
        
        return jsonify(plan)
    
    except Exception as e:
        return jsonify({
            "error": "Internal server error",
            "details": str(e)
        }), 500


if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="RAG-K8S Agent Server")
    parser.add_argument("--host", default="127.0.0.1", help="Host to bind to")
    parser.add_argument("--port", type=int, default=8000, help="Port to bind to")
    parser.add_argument("--preload", action="store_true", help="Preload model at startup")
    args = parser.parse_args()
    
    if args.preload:
        print("Preloading model...")
        load_model()
        print("Model preloaded. Starting server...")
    
    print(f"Starting RAG-K8S agent server on http://{args.host}:{args.port}")
    print(f"Model: {get_model_id()}")
    print("\nEndpoints:")
    print("  GET  /health - Health check")
    print("  POST /k8s-exec - Execute K8s operation")
    print("  POST /plan - Generate command plan")
    
    app.run(host=args.host, port=args.port, debug=False)

"""
Start the node with the PRIVATE_KEY and RSA_PRIVATE_KEY_BASE64 environment variable set to the base64 encoded RSA private key.
python3 -m alith.inference.server
"""

import argparse
import uvicorn


def run(
    host: str = "localhost",
    port: int = 8000,
    engine_type: str = "llamacpp",
    *,
    model: str,
    settlement: bool = False
):
    """Run an inference server with the given address and engine type."""

    if engine_type == "llamacpp":
        from llama_cpp.server.app import create_app
        from llama_cpp.server.settings import ModelSettings, ServerSettings

        server_settings = ServerSettings(host=host, port=port)
        model_settings = [ModelSettings(model=model)]
        app = create_app(server_settings=server_settings, model_settings=model_settings)
        if settlement:
            from .settlement import TokenBillingMiddleware
            from .query import DataQueryMiddleware
            from ..lazai.node.middleware import HeaderValidationMiddleware
            from ..lazai.request import INFERENCE_TYPE

            app.add_middleware(HeaderValidationMiddleware, type=INFERENCE_TYPE)
            app.add_middleware(DataQueryMiddleware)
            app.add_middleware(TokenBillingMiddleware)

        return uvicorn.run(
            app,
            host=server_settings.host,
            port=int(server_settings.port),
            ssl_keyfile=server_settings.ssl_keyfile,
            ssl_certfile=server_settings.ssl_certfile,
        )
    elif engine_type == "openai":
        from .proxy_server import app

        if settlement:
            from .settlement import TokenBillingMiddleware
            from .query import DataQueryMiddleware
            from ..lazai.node.middleware import HeaderValidationMiddleware
            from ..lazai.request import INFERENCE_TYPE

            app.add_middleware(HeaderValidationMiddleware, type=INFERENCE_TYPE)
            app.add_middleware(DataQueryMiddleware)
            app.add_middleware(TokenBillingMiddleware)
        return uvicorn.run(
            app,
            host=host,
            port=port,
        )
    else:
        raise NotImplementedError()


if __name__ == "__main__":
    description = "Alith inference server. Host your own or remote LLMs!🚀"
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument(
        "--host",
        type=str,
        help="Server host",
        default="localhost",
    )
    parser.add_argument(
        "--port",
        type=int,
        help="Server port",
        default=8000,
    )
    parser.add_argument(
        "--model",
        type=str,
        help="Model name or path",
        default="/root/models/qwen2.5-1.5b-instruct-q5_k_m.gguf",
    )
    parser.add_argument(
        "--engine",
        type=str,
        help="Engine type",
        default="llamacpp",
    )
    args = parser.parse_args()

    run(
        host=args.host,
        port=args.port,
        engine_type=args.engine,
        model=args.model,
        settlement=True,
    )

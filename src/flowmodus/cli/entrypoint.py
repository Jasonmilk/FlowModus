import os
import sys
import asyncio
import signal

from flowmodus.lifecycle import Sidecar, SidecarConfig
from flowmodus.data_plane.proxy import Proxy


async def _run(sidecar: Sidecar, host: str, port: int, debug: bool) -> None:
    """Start proxy and wait until shutdown."""
    proxy = Proxy(sidecar, debug=debug)
    await proxy.start(host=host, port=port)
    print(f"FlowModus ready on http://{host}:{port}", flush=True)

    stop_event = asyncio.Event()
    loop = asyncio.get_running_loop()
    for sig in (signal.SIGINT, signal.SIGTERM):
        loop.add_signal_handler(sig, stop_event.set)
    await stop_event.wait()

    print("Shutting down...", flush=True)
    await sidecar.shutdown(signal.SIGTERM)
    await proxy.stop()


def main() -> None:
    """CLI entry point. All defaults from environment variables."""
    debug = "--debug" in sys.argv

    config = SidecarConfig()
    sidecar = Sidecar(config)

    print(f"Loading registry...", flush=True)
    asyncio.run(sidecar.start())
    print(f"Registry loaded: {len(sidecar.registry_snapshot)} suppliers", flush=True)

    host = config.proxy_host
    port = config.proxy_port

    asyncio.run(_run(sidecar, host, port, debug))


if __name__ == "__main__":
    main()

import os
import asyncio
import signal

from flowmodus.lifecycle import Sidecar, SidecarConfig
from flowmodus.data_plane.proxy import Proxy


async def _run(sidecar: Sidecar, host: str, port: int) -> None:
    """Start proxy and wait until shutdown."""
    proxy = Proxy(sidecar)
    await proxy.start(host=host, port=port)

    # Wait indefinitely until a signal arrives
    stop_event = asyncio.Event()
    loop = asyncio.get_running_loop()
    for sig in (signal.SIGINT, signal.SIGTERM):
        loop.add_signal_handler(sig, stop_event.set)
    await stop_event.wait()

    await sidecar.shutdown(signal.SIGTERM)
    await proxy.stop()


def main() -> None:
    """CLI entry point – all defaults are read from the environment."""
    config = SidecarConfig()
    sidecar = Sidecar(config)

    host = config.proxy_host
    port = config.proxy_port

    asyncio.run(_run(sidecar, host, port))


if __name__ == "__main__":
    main()

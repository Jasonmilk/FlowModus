import asyncio
import argparse
from flowmodus.lifecycle import Sidecar, SidecarConfig


def main():
    """CLI entry point."""
    parser = argparse.ArgumentParser(description="FlowModus Sidecar")
    parser.add_argument("--host", default="127.0.0.1", help="Listen host")
    parser.add_argument("--port", type=int, default=8080, help="Listen port")
    parser.add_argument("--data-sharing", action="store_true", help="Enable data sharing")
    args = parser.parse_args()
    
    # Create config
    config = SidecarConfig(
        data_sharing_enabled=args.data_sharing
    )
    
    # Create and start sidecar
    sidecar = Sidecar(config)
    
    # Run
    asyncio.run(_run(sidecar, args.host, args.port))


async def _run(sidecar: Sidecar, host: str, port: int):
    await sidecar.start()
    await sidecar.proxy.start(host, port)
    
    # Wait forever
    while sidecar.state != SidecarState.DEAD:
        await asyncio.sleep(1.0)


if __name__ == "__main__":
    main()

import logging

from eqty_sdk._rust import context, signer, statements

logger = logging.getLogger()


def enable_logging(enable: bool):
    import os

    import eqty_sdk._rust as core

    if enable:
        logging.basicConfig(level=logging.DEBUG)
        os.environ["RUST_LOG"] = "eqty_sdk=debug"
    else:
        logging.basicConfig(level=logging.CRITICAL + 1)

    core.enable_rust_logging(enable)
    logger.info("Logging Enabled")
    return logger


def core_init(temp_dir: str):
    logger.info("Resetting core-py context")
    try:
        context.reset()
    except Exception as e:
        logger.info("Context reset failed (expected if not initialized): %s", e)

    logger.info("Initializing core-py context")
    context.init(temp_dir)
    logger.info("Creating signer from private key")
    try:
        signer.create_signer_from_private_key(
            "eHb22WNFvUXihogn8fubQjW7hHEqwY3fEKt745V4xXg=", "ed25519", "unit-test"
        )
    except ValueError as e:
        logger.info("signer already created %s", e)

    logger.info("Setting active signer")
    signer.set_active_signer("unit-test")


def create_simple_graph():
    logger.info("Creating compute statement")
    statements.create_computation_statement(
        ["urn:cid:input1"],
        ["urn:cid:output1"],
        computation=None,
        operated_by=None,
        executed_on=None,
        timestamp="2025-09-08T10:26:00Z",
    )

import logging
import os
from enum import Enum
from typing import cast

from eqty_sdk._rust import signer as eqty_core_signer
from eqty_sdk.errors import UsageError

logger = logging.getLogger("eqty.sdk.config.did")


class SIGNER_ALGORITHMS(Enum):
    SECP256K1 = "secp256k1"
    SECP256R1 = "secp256r1"
    ED25519 = "ed25519"


class Signer:
    name: str = ""
    did_key: str = ""

    def __init__(self, name: str, did_key: str):
        self.name = name
        self.did_key = did_key

    @staticmethod
    def new(algorithm: SIGNER_ALGORITHMS = SIGNER_ALGORITHMS.ED25519) -> "Signer":
        try:
            signer = eqty_core_signer.create_new_signer(algorithm.value)
            logger.info(f"Created signer '{signer.name}'")
            return cast(Signer, signer)
        except Exception as e:
            logger.error(e)
            raise

    @staticmethod
    def vcomp_notary(
        url: str = "http://docker.eqtylab.internal:8066",
    ) -> "Signer":
        try:
            signer = eqty_core_signer.create_vcomp_signer(url)
            logger.info(f"Created VComp signer '{signer.name}'")
            return cast(Signer, signer)
        except RuntimeError as e:
            logger.error(f"Failed to configure VComp Notary. {e}")
            raise

    @staticmethod
    def auth_service(
        url: str,
    ) -> "Signer":
        try:
            # Get the api key from the env var
            api_key = os.getenv("EQTY_API_KEY")
            if not api_key:
                raise UsageError(
                    "The env var 'EQTY_API_KEY' must be set to use Signer.auth_service()"
                )
            signer = eqty_core_signer.create_auth_service_signer(url, api_key)
            return cast(Signer, signer)
        except RuntimeError as e:
            logger.error(f"Failed to configure Auth Service Signer. {e}")
            raise

    @staticmethod
    def yubihsm2(
        auth_key_id: int,
        signing_key_id: int,
        password: str,
    ) -> "Signer":
        try:
            signer = eqty_core_signer.create_yubihsm2_signer(auth_key_id, signing_key_id, password)
            return cast(Signer, signer)
        except RuntimeError as e:
            logger.error(f"Failed to configure YUBI HSM signer. {e}")
            raise

    @staticmethod
    def from_private_key(algorithm: SIGNER_ALGORITHMS, private_key: str) -> "Signer":
        try:
            signer = eqty_core_signer.create_signer_from_private_key(private_key, algorithm.value)
            logger.info(f"Created signer '{signer.name}' from a private key")
            return cast(Signer, signer)
        except RuntimeError as e:
            logger.error(e)
            raise


def set_active_signer(signer: Signer) -> None:
    eqty_core_signer.set_active_signer(signer.name)

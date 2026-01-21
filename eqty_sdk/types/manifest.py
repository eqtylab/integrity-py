import logging
import os
from pathlib import Path
from typing import Union

from eqty_sdk._rust import manifest as eqty_core_manifest
from eqty_sdk.config import Config
from eqty_sdk.errors import UsageError
from eqty_sdk.feature_flags import FEATURE_FLAGS, FeatureFlags
from eqty_sdk.statements.common import Statements

logger = logging.getLogger("eqty.sdk.manifest")


class Manifest:
    def __init__(self, manifest: str):
        self.manifest_str = manifest

    @classmethod
    def from_statements(cls, statements: Statements, include_context: bool = True) -> "Manifest":
        """Creates a Manifest for the provided statements."""
        if FeatureFlags.is_enabled(FEATURE_FLAGS.GRAPH_IDS):
            logger.info(f"Generating manifest from graph {len(statements.graphs)}")
            manifest_str = eqty_core_manifest.generate_v4(
                statements.graphs, Config().blob_dir(), include_context
            )
        else:
            logger.info(f"Generating manifest from {len(statements.statements)} statements")
            manifest_str = eqty_core_manifest.generate(
                statements.statements, Config().blob_dir(), statements.attributes, include_context
            )

        instance = cls(manifest_str)
        return instance

    def export(self, file: Path) -> None:
        logger.debug(f"Saving manifest to {file}")
        with open(file, "w") as manifest_file:
            manifest_file.write(self.manifest_str)

    @classmethod
    def import_manifest(cls, manifest: Union[str, Path], **kwargs) -> None:
        if isinstance(manifest, Path):
            with open(manifest, "r") as f:
                manifest_str = f.read()
        else:
            manifest_str = manifest

        logger.debug("Importing manifest")
        blobs = eqty_core_manifest.import_manifest(manifest_str, kwargs)

        # Save each blob to the blob directory
        blob_dir = Config().blob_dir()
        blob_dir.mkdir(parents=True, exist_ok=True)  # Ensure directory exists

        for blob_key, blob_content in blobs.items():
            blob_file_path = blob_dir / blob_key
            with open(blob_file_path, "wb") as f:
                f.write(blob_content)

    @staticmethod
    def merge(a: str, b: str) -> str:
        """Combines the manifest strings `a` and `b` into a single manifest.
        `a` and `b` must both be valid JSON strings of manifests.

        Returns:
            The combined manifests as a single JSON string

        """
        logger.debug("Merging manfiests")
        manifest = eqty_core_manifest.merge(a, b)
        return str(manifest)

    def register(self) -> None:
        """Registers the manifest with the Integrity Service."""
        api_key = os.getenv("EQTY_API_KEY")
        if not api_key:
            raise UsageError("The env var 'EQTY_API_KEY' must be set")
        eqty_core_manifest.register(self.manifest_str, api_key)

import logging
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

import toml

from eqty_sdk._rust import (
    context as eqty_core_context,
)
from eqty_sdk.context import Context

from .cid_ignore import CidIgnore

logger = logging.getLogger("eqty.sdk.config")


# Settings that get saved to disk
@dataclass
class Settings:
    def __init__(
        self,
        url: Optional[str] = None,
        cid_ignore: Optional[CidIgnore] = None,
        store_all_blobs: bool = False,
        generate_model_signing_signatures: bool = False,
    ):
        cid_config = cid_ignore or CidIgnore()
        self.url = url
        self.cid_ignore = cid_config
        self.store_all_blobs = store_all_blobs
        self.generate_model_signing_signatures = generate_model_signing_signatures

    def to_toml(self):
        cid_ignore = self.cid_ignore or CidIgnore()
        return toml.dumps(
            {
                "url": self.url,
                "store_all_blobs": self.store_all_blobs,
                "cid_ignore": cid_ignore.model_dump(),
                "generate_model_signing_signatures": self.generate_model_signing_signatures,
            }
        )


# Class for handling sdk configuration and settings
class Config:
    _instance = None  # Class-level attribute to hold the singleton instance

    @property
    def store_all_blobs(self) -> bool:
        """Returns if all cid'ed blobs should be stored."""
        return self._settings.store_all_blobs

    @property
    def config_dir(self) -> Path:
        """Returns the config dir that was set during initalization."""
        return self._config_dir

    @property
    def cid_ignore(self) -> CidIgnore:
        """Returns the current cid ignore settings."""
        return self._settings.cid_ignore

    @property
    def generate_model_signing_signatures(self) -> bool:
        """Returns whether model signing signatures are generated for directories."""
        return self._settings.generate_model_signing_signatures

    @property
    def root_context(self) -> Context:
        """Returns the root context object."""
        if not self._root_context:
            # lazy create a default context
            logger.info("Root context not set. Creating")
            self.set_default_context(Context.new())

        assert self._root_context
        return self._root_context

    def __new__(cls):
        """Constructor."""
        if not cls._instance:
            cls._instance = super(Config, cls).__new__(cls)
        return cls._instance

    def __init__(self) -> None:
        """Singleton wrapper. Returns the already initialized config, or creates a new singleton."""
        if hasattr(self, "_initialized") and self._initialized:
            return

        # create a default context, that can be overwritten only once
        self._root_context: Optional[Context] = None

        # Set all the property defaults when the singleton is created
        self._settings = Settings()

        self._config_file_name = "config.toml"
        self._config_dir = Path.cwd() / ".eqty_sdk"
        self._config_path = os.path.join(self._config_dir, self._config_file_name)
        logger.debug(f"sdk config path: {self._config_path}")

    def init(self, custom_dir: Optional[str] = None):
        """Public Config initializer. Loads config if it exists, or creates a default."""
        self.__load_config_file__(custom_dir)

        self._initialized: bool = True

    def save(self) -> None:
        """Write configuration to disk."""
        with open(self._config_path, "w") as f:
            settings = self._settings.to_toml()
            f.write(settings)

        logger.debug(f"Configuration saved to {self._config_path}")

    def load(self) -> None:
        """Loads the configuration from disk."""
        logger.debug(f"Loading configuration from {self._config_path}")
        with open(self._config_path, "r") as f:
            settings_toml = toml.load(f)
            # Convert cid_ignore dict to CidIgnore object if it exists
            if "cid_ignore" in settings_toml and isinstance(settings_toml["cid_ignore"], dict):
                settings_toml["cid_ignore"] = CidIgnore(**settings_toml["cid_ignore"])
            self._settings = Settings(**settings_toml)

        if self._settings.url:
            eqty_core_context.set_integrity_service_url(self._settings.url)

        cid_rules = self._settings.cid_ignore
        eqty_core_context.set_cid_ignore_rules(
            cid_rules.include_hidden_files,
            cid_rules.gitignore,
            cid_rules.include_symlinks,
        )
        eqty_core_context.set_generate_model_signing_signatures(
            self._settings.generate_model_signing_signatures
        )

    def blob_dir(self) -> Path:
        folder = Path(self.config_dir, "blobs")
        folder.mkdir(parents=True, exist_ok=True)
        return folder

    def set_cid_ignore(
        self,
        include_hidden_files: Optional[bool] = None,
        gitignore: Optional[bool] = None,
        include_symlinks: Optional[bool] = None,
    ) -> "Config":
        """Sets the rules for files/folders to ignore when CIDing a directory."""
        self._settings.cid_ignore = CidIgnore(
            include_hidden_files=include_hidden_files or False,
            gitignore=gitignore or False,
            include_symlinks=include_symlinks or False,
        )
        self.save()

        eqty_core_context.set_cid_ignore_rules(include_hidden_files, gitignore, include_symlinks)
        return self

    def set_store_all_blobs(self, store: bool) -> "Config":
        """Sets a flag to store everything that gets CID'ed.
        NOTE: The `store` argument for a specific asset will override this flag.
        """
        self._settings.store_all_blobs = store
        self.save()
        return self

    def set_integrity_service_url(
        self,
        url: str,
    ) -> "Config":
        """Sets the URL used to connect to the integrity service."""
        self._settings.url = url
        eqty_core_context.set_integrity_service_url(self._settings.url)
        self.save()
        return self

    def set_generate_model_signing_signatures(self, issue: bool) -> "Config":
        """Sets whether to generate model signing signatures for directories."""
        self._settings.generate_model_signing_signatures = issue
        eqty_core_context.set_generate_model_signing_signatures(issue)
        self.save()
        return self

    def set_default_context(self, ctx: Context) -> "Config":
        """Sets the default context for grouping of statements.

        Args:
            ctx: The Context

        """
        if self._root_context:
            logger.error("The default context can only be set once.")
            return self

        self._root_context = ctx
        return self

    def __load_config_file__(self, custom_dir: Optional[str] = None) -> None:
        """Loads the config file from custom_dir, or creates a default config file if it doesn't exist."""
        if custom_dir:
            self._config_dir = Path(custom_dir)
            self._config_path = os.path.join(custom_dir, self._config_file_name)

        os.makedirs(os.path.dirname(self._config_path), exist_ok=True)
        logger.info(f"Initializing context at {self._config_dir}")
        eqty_core_context.init(self._config_dir)

        if os.path.exists(self._config_path):
            self.load()
        else:
            # create default file
            self.save()

import json
import logging
from typing import Optional

from eqty_sdk._rust import (
    Graph as Context,
    signer as eqty_core_signer,
    statements as eqty_core_statements,
)
from eqty_sdk.config import Config
from eqty_sdk.statements import add_did_statement, add_metadata_statement
from eqty_sdk.types.signer import Signer

logger = logging.getLogger("eqty.sdk.Did")


class Did:
    def __init__(self, ctx: Context, did: str, signer: Optional[Signer], **kwargs):
        self.ctx = ctx
        self.statement_ids = []

        is_vcomp_signer = (
            True
            if signer and eqty_core_signer.get_signer_type(signer.name) == "vcomp_notary"
            else False
        )

        if is_vcomp_signer:
            signer_name = signer.name if signer else ""  # should never be None here

            # Register DID statements + VCs provided by the vcomp signer
            statements = eqty_core_signer.get_signer_statements(signer_name)
            blobs = eqty_core_signer.get_signer_blobs(signer_name)

            # save statements locally
            for stmt_str in statements:
                stmt = json.loads(stmt_str)
                eqty_core_statements.register_statement(stmt_str)
                self.statement_ids.append(stmt.get("@id"))

            # save blobs locally
            blob_dir = Config().blob_dir()
            blob_dir.mkdir(parents=True, exist_ok=True)  # Ensure directory exists

            for cid, data in blobs.items():
                blob_file_path = blob_dir / cid
                with open(blob_file_path, "wb") as f:
                    f.write(data)
        else:
            # Register DID statement statement for metadata provided by the user
            did_statement_ids = add_did_statement(did)
            self.statement_ids.extend(did_statement_ids)

        metadata_statement_ids = add_metadata_statement(did, json.dumps(kwargs))
        self.statement_ids.extend(metadata_statement_ids)

    @staticmethod
    def from_signer(signer: Signer, **kwargs) -> "Did":
        return Did(Context.new(), signer.did_key, signer, **kwargs)

    @staticmethod
    def from_did_string(did: str, **kwargs) -> "Did":
        return Did(Context.new(), did, None, **kwargs)

    @staticmethod
    def with_context(ctx: Context):
        class _Factory:
            def from_signer(self, signer: Signer, **kwargs) -> "Did":
                return Did(ctx, signer.did_key, signer, **kwargs)

            def from_did_string(self, did: str, **kwargs) -> "Did":
                return Did(ctx, did, None, **kwargs)

        return _Factory()

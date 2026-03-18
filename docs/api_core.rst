Core
====

Initialization
--------------

.. function:: eqty_sdk.init(default_context=None, *, custom_dir=None) -> Config

   Initializes the sdk config. Must be called before setting individual config values.

   :param default_context: Optional default context applied to subsequent operations.
   :param custom_dir: Optional path to the SDK app directory. If omitted, uses
      ``.eqty_sdk`` under the current working directory.
   :returns: The initialized SDK configuration.
   :rtype: eqty_sdk.Config

Global Functions
----------------
.. autofunction:: eqty_sdk.purge_blob_store
.. autofunction:: eqty_sdk.purge_statement_store
.. autofunction:: eqty_sdk.set_active_signer

DID
------

.. autoclass:: eqty_sdk.DID
   :members:

CID
------

.. autoclass:: eqty_sdk.CID
   :members:

UUID
------

.. autoclass:: eqty_sdk.UUID
   :members:

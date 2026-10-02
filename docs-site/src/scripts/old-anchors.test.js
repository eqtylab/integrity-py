import assert from "node:assert/strict";
import { test } from "node:test";
import { resolveOldAnchor } from "./old-anchors.js";

// Headings as the built pages carry them: the old site's anchor is the name the heading shows.
test("finds the heading that shows the anchor's full name", () => {
  const signer = [
    { id: "eqty_sdk_rustsignername", text: "eqty_sdk._rust.Signer.name" },
    { id: "eqty_sdk_rustsignernew", text: "eqty_sdk._rust.Signer.new" },
  ];
  assert.equal(resolveOldAnchor("eqty_sdk._rust.Signer.new", signer), "eqty_sdk_rustsignernew");
});

test("finds a heading that shows only the end of the name", () => {
  const assets = [
    { id: "dataset", text: "Dataset" },
    { id: "custom", text: "Custom" },
  ];
  assert.equal(resolveOldAnchor("eqty_sdk.asset.Custom", assets), "custom");
});

test("prefers the full name to a heading that shows only its end", () => {
  const signer = [
    { id: "signer", text: "Signer" },
    { id: "eqty_sdk_rustsigner", text: "eqty_sdk._rust.Signer" },
  ];
  assert.equal(resolveOldAnchor("eqty_sdk._rust.Signer", signer), "eqty_sdk_rustsigner");
});

test("lands on the parent when the page does not show the member", () => {
  const assets = [
    { id: "eqty_sdkasset", text: "eqty_sdk.asset" },
    { id: "eqty_sdkassetasset", text: "eqty_sdk.asset.Asset" },
  ];
  assert.equal(
    resolveOldAnchor("eqty_sdk.asset.Asset._create_eqty_statements", assets),
    "eqty_sdkassetasset",
  );
});

test("takes the first of two headings with the same name", () => {
  const functions = [
    { id: "set_active_signer", text: "set_active_signer" },
    { id: "set_active_signer-1", text: "set_active_signer" },
  ];
  assert.equal(resolveOldAnchor("eqty_sdk.set_active_signer", functions), "set_active_signer");
});

test("reads past the whitespace around a heading's text", () => {
  const page = [{ id: "eqty_sdkinit", text: "\n  eqty_sdk.init  " }];
  assert.equal(resolveOldAnchor("eqty_sdk.init", page), "eqty_sdkinit");
});

test("leaves anchors alone that are not dotted names", () => {
  const page = [{ id: "signer", text: "Signer" }];
  assert.equal(resolveOldAnchor("signer_1", page), null);
  assert.equal(resolveOldAnchor("", page), null);
});

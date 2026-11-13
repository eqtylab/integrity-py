/*
  Old links into the API reference name a heading by its object's full path, such as
  #eqty_sdk._rust.Signer.new, the id the MkDocs site gave it. This site's ids are slugs
  (eqty_sdk_rustsignernew), and a redirect keeps the anchor, so the browser finds no such
  section and opens the page at the top. This finds the heading that shows that name instead.
  scripts/check_old_links.py runs resolveOldAnchor against every old anchor.
*/

// An API object's full path, the form of every old API heading's id.
const DOTTED = /^[A-Za-z_]\w*(\.\w+)+$/;

/*
  The id of the heading an old API anchor names, or null. A heading shows the full name or only
  its end (Custom for eqty_sdk.asset.Custom); the longest match wins, then the nearest parent's,
  for a member the page does not show.
*/
export function resolveOldAnchor(anchor, headings) {
  if (!DOTTED.test(anchor)) return null;
  const parts = anchor.split(".");
  for (let end = parts.length; end > 0; end--) {
    for (let start = 0; start < end; start++) {
      const name = parts.slice(start, end).join(".");
      const heading = headings.find((h) => h.text.trim() === name);
      if (heading) return heading.id;
    }
  }
  return null;
}

function scrollToOldAnchor() {
  let anchor;
  try {
    anchor = decodeURIComponent(location.hash.slice(1));
  } catch {
    return;
  }
  if (!anchor || document.getElementById(anchor)) return;
  const headings = [...document.querySelectorAll("main :is(h1, h2, h3, h4, h5, h6)[id]")];
  const id = resolveOldAnchor(
    anchor,
    headings.map((h) => ({ id: h.id, text: h.textContent })),
  );
  if (id) document.getElementById(id).scrollIntoView();
}

// Node imports this file for the check, where there is no page.
if (typeof document !== "undefined") {
  scrollToOldAnchor();
  addEventListener("hashchange", scrollToOldAnchor);
}

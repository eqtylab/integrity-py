(function () {
  const STORAGE_KEY = "eqty-docs-theme";

  function preferredTheme() {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }

  function currentTheme() {
    return localStorage.getItem(STORAGE_KEY) || preferredTheme();
  }

  function applyTheme(theme) {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem(STORAGE_KEY, theme);
    const button = document.querySelector(".eqty-theme-toggle");
    if (button) {
      button.setAttribute("aria-label", `Switch to ${theme === "dark" ? "light" : "dark"} mode`);
      button.setAttribute("title", `Switch to ${theme === "dark" ? "light" : "dark"} mode`);
    }
  }

  function toggleTheme() {
    applyTheme(currentTheme() === "dark" ? "light" : "dark");
  }

  function ensureButton() {
    if (document.querySelector(".eqty-theme-toggle")) return;

    const button = document.createElement("button");
    button.className = "eqty-theme-toggle";
    button.type = "button";
    button.innerHTML =
      '<span class="eqty-theme-toggle-track"><span class="eqty-theme-toggle-icon" aria-hidden="true">☀</span><span class="eqty-theme-toggle-icon" aria-hidden="true">☾</span><span class="eqty-theme-toggle-thumb" aria-hidden="true"></span></span>';
    button.addEventListener("click", toggleTheme);
    document.body.appendChild(button);
    applyTheme(currentTheme());
  }

  document.addEventListener("DOMContentLoaded", function () {
    applyTheme(currentTheme());
    ensureButton();
  });
})();

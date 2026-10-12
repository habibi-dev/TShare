(function (global) {
  let mounted = false;
  let lastFocus = null;

  function t(key, fallback) {
    if (global.TShareI18n && typeof global.TShareI18n.t === "function") {
      const v = global.TShareI18n.t(key);
      if (v && v !== key) return v;
    }
    return fallback;
  }

  function mount() {
    if (mounted || document.getElementById("tshareModal")) {
      mounted = true;
      return;
    }
    const root = document.createElement("div");
    root.id = "tshareModal";
    root.className = "tshare-modal";
    root.hidden = true;
    root.setAttribute("aria-hidden", "true");
    root.innerHTML =
      '<div class="tshare-modal__backdrop" data-tshare-modal-close></div>' +
      '<div class="tshare-modal__dialog" role="alertdialog" aria-modal="true" aria-labelledby="tshareModalTitle">' +
      '<div class="tshare-modal__icon" id="tshareModalIcon" aria-hidden="true"></div>' +
      '<p class="tshare-modal__title" id="tshareModalTitle"></p>' +
      '<p class="tshare-modal__message" id="tshareModalMessage"></p>' +
      '<button type="button" class="tshare-modal__btn" id="tshareModalOk"></button>' +
      "</div>";
    document.body.appendChild(root);

    const ok = document.getElementById("tshareModalOk");
    ok.addEventListener("click", () => hide());
    root.querySelector("[data-tshare-modal-close]").addEventListener("click", () => hide());
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && !root.hidden) hide();
    });

    document.addEventListener("tshare:locale", () => refreshLabels());
    refreshLabels();
    mounted = true;
  }

  function refreshLabels() {
    const ok = document.getElementById("tshareModalOk");
    if (ok) ok.textContent = t("modal.ok", "OK");
  }

  function iconSvg(variant) {
    if (variant === "success") {
      return (
        '<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" width="28" height="28">' +
        '<path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"/></svg>'
      );
    }
    return (
      '<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" width="28" height="28">' +
      '<path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z"/></svg>'
    );
  }

  function show(message, options) {
    mount();
    const opts = options || {};
    const variant = opts.variant === "success" ? "success" : "error";
    const root = document.getElementById("tshareModal");
    const titleEl = document.getElementById("tshareModalTitle");
    const msgEl = document.getElementById("tshareModalMessage");
    const iconEl = document.getElementById("tshareModalIcon");
    const dialog = root.querySelector(".tshare-modal__dialog");

    root.classList.toggle("tshare-modal--success", variant === "success");
    root.classList.toggle("tshare-modal--error", variant === "error");
    iconEl.innerHTML = iconSvg(variant);
    titleEl.textContent =
      variant === "success" ? t("modal.titleSuccess", "Done") : t("modal.titleError", "Notice");
    msgEl.textContent = String(message || "");
    refreshLabels();

    lastFocus = document.activeElement;
    root.hidden = false;
    root.setAttribute("aria-hidden", "false");
    document.body.classList.add("tshare-modal-open");
    const okBtn = document.getElementById("tshareModalOk");
    if (okBtn) okBtn.focus();
  }

  function hide() {
    const root = document.getElementById("tshareModal");
    if (!root || root.hidden) return;
    root.hidden = true;
    root.setAttribute("aria-hidden", "true");
    document.body.classList.remove("tshare-modal-open");
    if (lastFocus && typeof lastFocus.focus === "function") lastFocus.focus();
  }

  function error(message) {
    show(message, { variant: "error" });
  }

  function success(message) {
    show(message, { variant: "success" });
  }

  document.addEventListener("DOMContentLoaded", mount);

  global.TShareModal = { show, error, success, hide };
})(window);

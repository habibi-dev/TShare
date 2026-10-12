(function (global) {
  const STORAGE_KEY = "tshare_locale";
  const THEME_KEY = "tshare_theme";

  const ICON_MOON =
    '<svg class="chrome-icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" aria-hidden="true"><path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21.752 15.002A9.7 9.7 0 0 1 18 15.75 9.75 9.75 0 0 1 8.25 6c0-1.33.266-2.597.748-3.752A9.75 9.75 0 0 0 3 11.25 9.75 9.75 0 0 0 12.75 21a9.75 9.75 0 0 0 9.002-5.998"></path></svg>';
  const ICON_SUN =
    '<svg class="chrome-icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" aria-hidden="true"><path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 18c3.31 0 6-2.69 6-6s-2.69-6-6-6-6 2.69-6 6 2.69 6 6 6M12 2v2m0 16v2M4 12H2m20 0h-2m-.93-7.07-1.41 1.41M6.34 17.66l-1.41 1.41m14.14 0-1.41-1.41M6.34 6.34 4.93 4.93"></path></svg>';

  const messages = {
    en: {
      meta: {
        title: "TShare.ir | Secure temporary text & file sharing",
        description: "Share notes and files with a short link. No sign-up required.",
      },
      nav: { links: "Your links" },
      header: { tagline: "Fast · Private · Ephemeral" },
      footer: { github: "GitHub", powered: "Powered by Rust 🦀" },
      home: {
        title: "Secure, ephemeral text & file sharing",
        subtitle:
          "Share notes, short text, and essential files with a short link.<br/>No sign-up — addresses, contacts, images, docs, or quick instructions.",
        share: "Create share",
        view: "Open link",
        showIdPlaceholder: "Enter your 6-character ID",
        showSubmit: "View share",
        back: "Back",
        notePlaceholder: "123 Main St, Suite 4 — City, ZIP 12345",
        fileAttach: "Attach file (optional)",
        filePrompt: "Click, drag & drop, or Ctrl+V to attach",
        fileClear: "Remove",
        fileHint: "Max {maxMb} MB — allowed: {extensions}",
        oneTime: "Burn after read",
        restrictIp: "Restrict to IP",
        requirePassword: "Password required",
        passwordPlaceholder: "Enter password",
        expiryLabel: "Expires in:",
        maxViewsLabel: "Max views:",
        createSubmit: "Create share",
        stats: "So far, {text} text and {file} file shares ({total} total).",
        statsTransfer: "Uploaded {up} · Downloaded {down}",
        expiry: {
          "5": "5 min", "10": "10 min", "30": "30 min", "60": "1 hour",
          "180": "3 hours", "360": "6 hours", "720": "12 hours", "1440": "24 hours",
        },
        views: { v1: "1 view", v5: "5 views", v10: "10 views", unlimited: "Unlimited" },
      },
      history: {
        title: "Your share history",
        subtitle: "Links created in this browser.",
        back: "Back to home",
      },
      show: {
        title: "Your share ID",
        subtitle: "Use this ID to share or open your content.",
        copy: "Copy text",
        share: "Share",
        delete: "Delete this share",
        backHome: "Back to home",
        download: "Download",
        fileLabel: "Attachment",
        fileHint: "Click to download",
        views: "Viewed {n} time(s) so far.",
        idCopied: "ID copied",
      },
      password: { view: "View", passwordPlaceholder: "Enter password" },
      error: { backHome: "Back to home" },
      theme: { light: "Light mode", dark: "Dark mode" },
      aria: { langToggle: "Switch language", themeToggle: "Toggle color theme" },
      modal: { ok: "OK", titleError: "Something went wrong", titleSuccess: "Success" },
      toast: {
        copyTextSuccess: "Text copied successfully.",
        copyTextError: "Could not copy text. Please try again.",
        copyIdSuccess: "ID copied successfully.",
        copyUrlSuccess: "Page URL copied successfully.",
        copyUrlError: "Could not copy the URL. Please try again.",
      },
      errors: {
        showIdLength: "The ID must be exactly 6 characters.",
        showIdEmpty: "Please enter your share ID.",
        charLimit: "The maximum allowed length is 10,000 characters.",
        needContent: "Please enter text or attach a file.",
        fileTooBig: "File size must not exceed {maxMb} MB.",
        fileType: "This file type is not allowed.",
        expiryInvalid: "Invalid expiry value.",
        viewsInvalid: "Invalid max views value.",
        passwordRequired: "Please enter a password.",
        shareCreate: "Could not create the share.",
        shareCreateRetry: "Could not create the share. Please try again.",
        deleteFailed: "Could not delete the share.",
        deleteNoPermission:
          "Cannot delete this share. The link token was not found in this browser.",
      },
    },
    fa: {
      meta: {
        title: "TShare.ir | اشتراک امن و موقت متن و فایل",
        description: "اشتراک متن و فایل با لینک کوتاه — بدون ثبت‌نام.",
      },
      nav: { links: "لینک‌های شما" },
      header: { tagline: "سریع · خصوصی · موقت" },
      footer: { github: "گیت‌هاب", powered: "Powered by Rust 🦀" },
      home: {
        title: "اشتراک امن و موقت متن و فایل",
        subtitle:
          "ابزار ساده برای اشتراک‌گذاری یادداشت‌ها، متن‌های کوتاه و فایل‌های ضروری.<br/>بدون ثبت‌نام، لینک ایجاد کنید و به اشتراک بگذارید؛ مثل آدرس، شماره تماس، تصویر، سند یا دستورالعمل‌های کاری.",
        share: "اشتراک‌گذاری",
        view: "مشاهده",
        showIdPlaceholder: "شناسه خود را وارد کنید",
        showSubmit: "مشاهده اشتراک‌گذاری",
        back: "بازگشت",
        vpnNotice: "کاربر گرامی، حسب نظر مقام قضایی لطفاً از ارسال فایل‌ها و کانفیگ‌های مربوط به VPN خودداری کنید. 😉",
        notePlaceholder: "تهران، خیابان ولیعصر، نرسیده به میدان ونک، پلاک ۱۲۳ - کد پستی: 123456789",
        fileAttach: "پیوست فایل (اختیاری)",
        filePrompt: "کلیک، کشیدن به صفحه، یا Ctrl+V برای پیوست",
        fileClear: "حذف فایل",
        fileHint: "حداکثر {maxMb} مگابایت — فرمت‌های مجاز: {extensions}",
        oneTime: "یک‌بار مصرف",
        restrictIp: "محدود به IP خاص",
        requirePassword: "نیاز به رمز عبور",
        passwordPlaceholder: "رمز عبور را وارد کنید",
        expiryLabel: "مدت انقضا:",
        maxViewsLabel: "حداکثر تعداد بازدید:",
        createSubmit: "ایجاد اشتراک‌گذاری",
        stats: "تا این لحظه، {text} متن و {file} فایل به اشتراک گذاشته شده ({total} اشتراک‌گذاری).",
        statsTransfer: "آپلود {up} · دانلود {down}",
        expiry: {
          "5": "۵ دقیقه", "10": "۱۰ دقیقه", "30": "۳۰ دقیقه", "60": "۱ ساعت",
          "180": "۳ ساعت", "360": "۶ ساعت", "720": "۱۲ ساعت", "1440": "۲۴ ساعت",
        },
        views: { v1: "۱ بار", v5: "۵ بار", v10: "۱۰ بار", unlimited: "نامحدود" },
      },
      history: {
        title: "تاریخچه اشتراک‌گذاری شما",
        subtitle: "در زیر لیستی از لینک‌هایی که در این مرورگر ساخته‌اید را می‌بینید.",
        back: "بازگشت به خانه",
      },
      show: {
        title: "شناسه شما برای اشتراک گذاری",
        subtitle: "شناسه زیر را برای اشتراک‌گذاری متن یا فایل خود استفاده کنید.",
        copy: "کپی متن",
        share: "اشتراک گذاری",
        delete: "حذف این اشتراک گذاری",
        backHome: "بازگشت به خانه",
        download: "دانلود",
        fileLabel: "پیوست فایل",
        fileHint: "برای دانلود کلیک کنید",
        views: "تا این لحظه، {n} بار مشاهده شده است.",
        idCopied: "شناسه کپی شد",
      },
      password: { view: "مشاهده", passwordPlaceholder: "رمز عبور را وارد کنید" },
      error: { backHome: "بازگشت به خانه" },
      theme: { light: "حالت روشن", dark: "حالت تاریک" },
      aria: { langToggle: "تغییر زبان", themeToggle: "تغییر تم رنگی" },
      modal: { ok: "باشه", titleError: "خطا", titleSuccess: "انجام شد" },
      toast: {
        copyTextSuccess: "متن با موفقیت کپی شد!",
        copyTextError: "خطایی در هنگام کپی رخ داد! لطفا دوباره تلاش کنید.",
        copyIdSuccess: "شناسه با موفقیت کپی شد!",
        copyUrlSuccess: "آدرس صفحه با موفقیت کپی شد!",
        copyUrlError: "خطایی در هنگام کپی آدرس رخ داد! لطفا دوباره تلاش کنید.",
      },
      errors: {
        showIdLength: "شناسه وارد شده باید 6 کاراکتر باشد.",
        showIdEmpty: "لطفاً شناسه خود را وارد کنید",
        charLimit: "حداکثر کاراکتر مجاز 10 هزار کاراکتر می باشد.",
        needContent: "لطفاً متن یا فایل وارد کنید",
        fileTooBig: "حجم فایل نباید بیشتر از {maxMb} مگابایت باشد.",
        fileType: "نوع فایل مجاز نیست.",
        expiryInvalid: "مقدار تاریخ انقضا نامعتبر می باشد.",
        viewsInvalid: "مقدار حداکثر تعداد بازدید نامعتبر می باشد.",
        passwordRequired: "لطفاً رمز عبور را وارد کنید",
        shareCreate: "خطا در ایجاد لینک",
        shareCreateRetry: "خطا در ایجاد لینک. لطفاً دوباره تلاش کنید.",
        deleteFailed: "خطا در حذف لینک",
        deleteNoPermission:
          "خطا در حذف لینک. امکان حذف لینک برای شما وجود ندارد. توکن یافت نشد!",
      },
    },
  };

  let locale = (function () {
    const s = localStorage.getItem(STORAGE_KEY);
    if (s === "fa" || s === "en") return s;
    return "en";
  })();

  document.documentElement.lang = locale === "fa" ? "fa" : "en";

  function get(obj, path) {
    return path.split(".").reduce((a, k) => (a && a[k] != null ? a[k] : null), obj);
  }

  function format(tpl, params) {
    if (!tpl) return "";
    return tpl.replace(/\{(\w+)\}/g, (_, k) => (params && params[k] != null ? String(params[k]) : `{${k}}`));
  }

  function t(key, params) {
    const v = get(messages[locale], key);
    const p = params ? localizeParams(params) : {};
    return typeof v === "string" ? format(v, p) : key;
  }

  function numberLocale() {
    return locale === "fa" ? "fa-IR" : "en-US";
  }

  function numberFormatOptions(extra) {
    const base = locale === "fa" ? {} : { numberingSystem: "latn" };
    return Object.assign(base, extra || {});
  }

  function formatLocaleInt(value) {
    const n = Number(value);
    if (!Number.isFinite(n)) return String(value ?? "");
    return new Intl.NumberFormat(
      numberLocale(),
      numberFormatOptions({ maximumFractionDigits: 0 }),
    ).format(n);
  }

  function formatLocaleDecimal(value, maxFractionDigits) {
    const n = Number(value);
    if (!Number.isFinite(n)) return "0";
    return new Intl.NumberFormat(
      numberLocale(),
      numberFormatOptions({
        minimumFractionDigits: 0,
        maximumFractionDigits: maxFractionDigits,
      }),
    ).format(n);
  }

  function localizeParams(params) {
    if (!params || typeof params !== "object") return params;
    const out = { ...params };
    ["n", "maxMb", "text", "file", "total"].forEach((key) => {
      if (out[key] != null && out[key] !== "") out[key] = formatLocaleInt(out[key]);
    });
    return out;
  }

  function normalizeDigitsForLocale(str) {
    const s = String(str ?? "");
    if (locale === "fa") return s;
    const persian = "۰۱۲۳۴۵۶۷۸۹";
    const arabic = "٠١٢٣٤٥٦٧٨٩";
    return s.replace(/[۰-۹٠-٩]/g, (ch) => {
      const p = persian.indexOf(ch);
      if (p >= 0) return String(p);
      const a = arabic.indexOf(ch);
      return a >= 0 ? String(a) : ch;
    });
  }

  function formatBytes(bytes) {
    const n = Number(bytes);
    if (!Number.isFinite(n) || n <= 0) {
      return locale === "fa" ? `${formatLocaleInt(0)} بایت` : "0 B";
    }
    const KB = 1024;
    const MB = KB * 1024;
    const GB = MB * 1024;
    if (locale === "fa") {
      if (n < KB) return `${formatLocaleInt(n)} بایت`;
      if (n < MB) return `${formatLocaleDecimal(n / KB, 1)} کیلو`;
      if (n < GB) return `${formatLocaleDecimal(n / MB, 2)} مگ`;
      return `${formatLocaleDecimal(n / GB, 2)} گیگ`;
    }
    if (n < KB) return `${formatLocaleInt(n)} B`;
    if (n < MB) return `${formatLocaleDecimal(n / KB, 1)} KB`;
    if (n < GB) return `${formatLocaleDecimal(n / MB, 2)} MB`;
    return `${formatLocaleDecimal(n / GB, 2)} GB`;
  }

  function applyCharCounters() {
    document.querySelectorAll(".input-wrapper-counter").forEach((box) => {
      const max = Number(box.getAttribute("data-max")) || 10000;
      const maxEl = box.querySelector(".counter-max");
      const usedEl = box.querySelector(".used");
      const textarea = box.closest(".input-wrapper")?.querySelector("textarea");
      if (maxEl) maxEl.textContent = formatLocaleInt(max);
      const len = textarea
        ? textarea.value.length
        : Number(String(usedEl?.textContent || "0").replace(/\D/g, "")) || 0;
      if (usedEl) usedEl.textContent = formatLocaleInt(len);
    });
  }

  function applyPlatformStats() {
    document.querySelectorAll("[data-stat-text]").forEach((el) => {
      const key = el.getAttribute("data-i18n");
      if (!key) return;
      el.textContent = t(key, {
        text: formatLocaleInt(el.getAttribute("data-stat-text")),
        file: formatLocaleInt(el.getAttribute("data-stat-file")),
        total: formatLocaleInt(el.getAttribute("data-stat-total")),
      });
    });
    document.querySelectorAll("[data-stat-up]").forEach((el) => {
      const key = el.getAttribute("data-i18n");
      if (!key) return;
      el.textContent = t(key, {
        up: formatBytes(el.getAttribute("data-stat-up")),
        down: formatBytes(el.getAttribute("data-stat-down")),
      });
    });
  }

  function applyLayoutDirection() {
    document.documentElement.lang = locale === "fa" ? "fa" : "en";
    document.querySelectorAll("header, footer.app-footer").forEach((el) => el.setAttribute("dir", "ltr"));
    document.querySelectorAll(".content").forEach((el) => {
      el.setAttribute("dir", locale === "fa" ? "rtl" : "ltr");
    });
  }

  function applyTranslations() {
    document.querySelectorAll("[data-i18n]").forEach((el) => {
      if (el.hasAttribute("data-stat-text") || el.hasAttribute("data-stat-up")) return;
      const key = el.getAttribute("data-i18n");
      let params = {};
      const raw = el.getAttribute("data-i18n-params");
      if (raw) {
        try {
          params = localizeParams(JSON.parse(raw));
        } catch (_) {}
      }
      const text = t(key, params);
      if (el.hasAttribute("data-i18n-html")) el.innerHTML = text;
      else el.textContent = text;
    });
    document.querySelectorAll("[data-i18n-placeholder]").forEach((el) => {
      el.placeholder = normalizeDigitsForLocale(t(el.getAttribute("data-i18n-placeholder")));
    });
    document.querySelectorAll("[data-i18n-aria]").forEach((el) => {
      el.setAttribute("aria-label", t(el.getAttribute("data-i18n-aria")));
    });
    document.querySelectorAll("select[data-i18n-options]").forEach((sel) => {
      const prefix = sel.getAttribute("data-i18n-options");
      sel.querySelectorAll("option").forEach((opt) => {
        const label = t(`${prefix}.${opt.value}`);
        if (label && !label.includes(".")) opt.textContent = normalizeDigitsForLocale(label);
      });
    });
    const titleEl = document.querySelector("title[data-i18n]");
    if (titleEl) titleEl.textContent = t(titleEl.getAttribute("data-i18n"));
    const langBtn = document.getElementById("langToggle");
    if (langBtn) {
      const next = locale === "en" ? "fa" : "en";
      langBtn.textContent = next === "fa" ? "FA" : "EN";
      langBtn.setAttribute("data-switch-to", next);
    }
    document.querySelectorAll("[data-show-locale]").forEach((el) => {
      el.hidden = el.getAttribute("data-show-locale") !== locale;
    });
    applyPlatformStats();
    applyCharCounters();
  }

  function applyThemeUi() {
    const theme = document.documentElement.getAttribute("data-theme") || "light";
    const btn = document.getElementById("themeToggle");
    if (!btn) return;
    btn.innerHTML = theme === "dark" ? ICON_SUN : ICON_MOON;
    btn.setAttribute("aria-label", theme === "dark" ? t("theme.light") : t("theme.dark"));
  }

  function initTheme() {
    const stored = localStorage.getItem(THEME_KEY);
    const theme = stored === "dark" || stored === "light" ? stored : "light";
    document.documentElement.setAttribute("data-theme", theme);
    applyThemeUi();
  }

  function toggleTheme() {
    const next = document.documentElement.getAttribute("data-theme") === "dark" ? "light" : "dark";
    localStorage.setItem(THEME_KEY, next);
    document.documentElement.setAttribute("data-theme", next);
    applyThemeUi();
  }

  function setLocale(next) {
    if (next !== "fa" && next !== "en") return;
    locale = next;
    localStorage.setItem(STORAGE_KEY, locale);
    applyLayoutDirection();
    applyTranslations();
    applyThemeUi();
    document.dispatchEvent(new CustomEvent("tshare:locale", { detail: { locale } }));
  }

  function showCopyTooltip(anchor, messageKey) {
    if (!anchor) return;
    const host = anchor.closest(".tshare-copy-anchor") || anchor.parentElement;
    if (!host) return;
    host.classList.add("tshare-copy-anchor");
    let tip = host.querySelector(".tshare-copy-tooltip");
    if (!tip) {
      tip = document.createElement("span");
      tip.className = "tshare-copy-tooltip";
      tip.setAttribute("role", "status");
      tip.setAttribute("aria-live", "polite");
      host.appendChild(tip);
    }
    tip.textContent = t(messageKey || "show.idCopied");
    tip.classList.add("is-visible");
    clearTimeout(tip._hideTimer);
    tip._hideTimer = setTimeout(() => {
      tip.classList.remove("is-visible");
    }, 2200);
  }

  function applyAll() {
    applyLayoutDirection();
    applyTranslations();
    applyThemeUi();
  }

  document.addEventListener("DOMContentLoaded", () => {
    initTheme();
    applyAll();
    const langBtn = document.getElementById("langToggle");
    if (langBtn) {
      langBtn.addEventListener("click", () => {
        const next = langBtn.getAttribute("data-switch-to");
        if (next) setLocale(next);
      });
    }
    const themeBtn = document.getElementById("themeToggle");
    if (themeBtn) themeBtn.addEventListener("click", toggleTheme);

    document.querySelectorAll(".input-wrapper textarea").forEach((ta) => {
      const usedEl = ta.closest(".input-wrapper")?.querySelector(".used");
      if (!usedEl) return;
      ta.addEventListener("input", () => {
        requestAnimationFrame(() => {
          usedEl.textContent = formatLocaleInt(ta.value.length);
        });
      });
    });
  });

  global.TShareI18n = {
    t,
    setLocale,
    getLocale: () => locale,
    applyAll,
    formatLocaleInt,
    formatBytes,
    showCopyTooltip,
  };
})(typeof globalThis !== "undefined" ? globalThis : window);

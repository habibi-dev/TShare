(() => {
  const root = typeof globalThis !== "undefined" ? globalThis : window;
  const i18n = (key, params) =>
    root.TShareI18n && typeof root.TShareI18n.t === "function"
      ? root.TShareI18n.t(key, params)
      : key;
  const notify = {
    error: (m) =>
      window.TShareModal ? TShareModal.error(m) : window.alert(m),
    success: (m) =>
      window.TShareModal ? TShareModal.success(m) : window.alert(m),
  };
  var e = {
      203() {
        function e(e) {
          const t = e.value.trim();
          if (!t) return void (e.style.direction = "rtl");
          const n = t.charAt(0);
          if (
            /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF]/.test(
              n,
            )
          )
            return void (e.style.direction = "rtl");
          const o = (t.match(/[a-zA-Z]/g) || []).length,
            r = (
              t.match(
                /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFFa-zA-Z]/g,
              ) || []
            ).length;
          if (0 === r) return void (e.style.direction = "ltr");
          const d = (o / r) * 100;
          e.style.direction = d > 70 ? "ltr" : "rtl";
        }
        document.addEventListener("DOMContentLoaded", () => {
          const t = document.getElementById("note");
          t &&
            (t.addEventListener("input", () => {
              e(t);
            }),
            e(t));
        });
      },
      240() {
        document.addEventListener("DOMContentLoaded", () => {
          const e = document.querySelector(".copy"),
            t = document.getElementById("note");
          e &&
            e.addEventListener("click", () => {
              try {
                navigator.clipboard
                  .writeText(t.value)
                  .then(() => {
                    (console.log("متن کپی شد!"),
                      notify.success(i18n("toast.copyTextSuccess")));
                  })
                  .catch((e) => {
                    (console.error("Error occurred while copying:", e),
                      notify.error(i18n("toast.copyTextError")));
                  });
              } catch (e) {
                (console.error("Error occurred while copying:", e),
                  notify.error(i18n("toast.copyTextError")));
              }
            });
        });
      },
      399() {
        document.addEventListener("DOMContentLoaded", () => {
          const e = document.querySelector(".id-show-for-copy");
          e &&
            e.addEventListener("click", () => {
              (e.select(),
                e.setSelectionRange(0, 99999),
                navigator.clipboard
                  .writeText(e.value)
                  .then(() => {
                    root.TShareI18n && root.TShareI18n.showCopyTooltip
                      ? root.TShareI18n.showCopyTooltip(e, "show.idCopied")
                      : notify.success(i18n("toast.copyIdSuccess"));
                  })
                  .catch(() => notify.error(i18n("toast.copyTextError"))),
                e.setSelectionRange(0, 0));
            });
        });
      },
      428() {
        document.addEventListener("DOMContentLoaded", () => {
          const e = document.getElementById("note");
          if (!e) return null;
          ((e.style.height = "auto"),
            (e.style.height = e.scrollHeight + "px"),
            e.addEventListener("input", () => {
              ((e.style.height = "auto"),
                (e.style.height = `${e.scrollHeight}px`));
            }));
        });
      },
      886() {
        document.addEventListener("DOMContentLoaded", () => {
          const e = document.querySelector(".share");
          e &&
            e.addEventListener("click", () => {
              const e = window.location.href;
              try {
                navigator
                  .share({
                    title: "اشتراک امن و موقت متن",
                    text: "\nاشتراک امن و موقت متن",
                    url: window.location.href,
                  })
                  .then();
              } catch (t) {
                (console.error("Error occurred while sharing:", t),
                  navigator.clipboard &&
                    navigator.clipboard
                      .writeText(e)
                      .then(() => {
                        (console.log("آدرس صفحه کپی شد!"),
                          notify.success(i18n("toast.copyUrlSuccess")));
                      })
                      .catch((e) => {
                        (console.error("Error occurred while copying URL:", e),
                          notify.error(i18n("toast.copyUrlError")));
                      }));
              }
            });
        });
      },
      895() {
        document.addEventListener("DOMContentLoaded", () => {
          const e = document.getElementById("showForm");
          e &&
            e.addEventListener("submit", async (ev) => {
              ev.preventDefault();
              const id = document.getElementById("show-id").value.trim();
              id
                ? 6 === id.length
                  ? (window.location.href = `/c/${id}`)
                  : notify.error(i18n("errors.showIdLength"))
                : notify.error(i18n("errors.showIdEmpty"));
            });
        });
      },
      936() {
        document.addEventListener("DOMContentLoaded", () => {
          const e = document.getElementById("shareForm"),
            t = document.getElementById("showForm"),
            n = document.querySelector(".home"),
            o = document.getElementById("createLink"),
            r = document.querySelectorAll(".back-btn"),
            d = document.querySelector(".show-btn"),
            a = document.getElementById("restrictIp"),
            i = document.getElementById("ipWrapper"),
            l = document.getElementById("ip"),
            s = document.getElementById("requirePassword"),
            c = document.getElementById("passwordWrapper"),
            u = document.getElementById("password");
          (s &&
            s.addEventListener("change", (e) => {
              e.target.checked
                ? ((c.style.display = "flex"), (u.required = !0))
                : ((c.style.display = "none"),
                  (u.required = !1),
                  (u.value = ""));
            }),
            a &&
              a.addEventListener("change", (e) => {
                e.target.checked
                  ? ((i.style.display = "flex"), (l.required = !0))
                  : ((i.style.display = "none"),
                    (l.required = !1),
                    (l.value = ""));
              }),
            o &&
              o.addEventListener("click", () => {
                ((n.style.display = "none"), (e.style.display = "flex"));
              }),
            r &&
              r.forEach((o) => {
                o.addEventListener("click", () => {
                  ((n.style.display = "flex"),
                    e && (e.style.display = "none"),
                    t && (t.style.display = "none"));
                });
              }),
            d &&
              d.addEventListener("click", () => {
                ((n.style.display = "none"), (t.style.display = "flex"));
              }));
        });
      },
      991() {
        function e(e, t) {
          const n = e.value.length;
          ((t.innerHTML = n.toString()),
            n > 9999 &&
              ((e.value = e.value.substring(0, 1e4)),
              notify.error(i18n("errors.charLimit"))));
        }
        document.addEventListener("DOMContentLoaded", () => {
          const t = document.querySelector(".used"),
            n = document.getElementById("note");
          n &&
            (n.addEventListener("input", () => {
              e(n, t);
            }),
            e(n, t));
        });
      },
    },
    t = {};
  function n(o) {
    var r = t[o];
    if (void 0 !== r) return r.exports;
    var d = (t[o] = { exports: {} });
    return (e[o](d, d.exports, n), d.exports);
  }
  (() => {
    "use strict";
    (document.addEventListener("DOMContentLoaded", () => {
      const e = document.getElementById("file"),
        t = document.getElementById("fileSelectedName");
      e &&
        e.addEventListener("change", () => {
          t &&
            (e.files && e.files.length
              ? (t.textContent = e.files[0].name)
              : (t.textContent = ""));
        });
    }),
      n(936),
      n(991),
      n(203),
      n(399));
    class e {
      constructor(e) {
        this.name = e;
      }
      get(e = null) {
        return this.checkJsonValue(localStorage.getItem(this.name)) ?? e;
      }
      set(e) {
        ("string" != typeof e && (e = JSON.stringify(e)),
          localStorage.setItem(this.name, e));
      }
      remove() {
        localStorage.removeItem(this.name);
      }
      checkJsonValue(e) {
        if (null !== e && "string" == typeof e)
          try {
            return JSON.parse(e);
          } catch (t) {
            return e;
          }
        return e;
      }
    }
    const t = ["5", "10", "30", "60", "180", "360", "720", "1440"],
      o = ["v1", "v5", "v10", "unlimited"];
    (document.addEventListener("DOMContentLoaded", () => {
      const n = document.getElementById("shareForm");
      n &&
        n.addEventListener("submit", async (r) => {
          r.preventDefault();
          const d = document.getElementById("note"),
            a = document.getElementById("file"),
            i = n.querySelector(".submit-btn"),
            l = document.getElementById("requirePassword"),
            s = document.getElementById("password"),
            c = document.getElementById("ip"),
            u = document.getElementById("expiry").value,
            m = document.getElementById("maxViews").value,
            y = document.getElementById("oneTimeUse").checked,
            p = document.getElementById("restrictIp").checked,
            g = l.checked,
            h = l.checked ? s.value : null,
            E = p ? c.value : null,
            v = d ? d.value.trim() : "",
            f = a && a.files && a.files.length > 0;
          if (!v && !f) return void notify.error(i18n("errors.needContent"));
          if (f) {
            const e = (function (e) {
              if (!e || !e.files || !e.files.length) return { ok: !0 };
              const t = e.files[0],
                n = parseFloat(e.dataset.maxMb || "10"),
                o = 1024 * n * 1024,
                r = (d = e.dataset.allowedExtensions || "")
                  ? d
                      .split(",")
                      .map((e) => e.trim().replace(/^\./, "").toLowerCase())
                      .filter(Boolean)
                  : [];
              var d;
              const a = (function (e) {
                const t = e.split(/[/\\]/).pop() || "",
                  n = t.lastIndexOf(".");
                return n <= 0 ? "" : t.slice(n + 1).toLowerCase();
              })(t.name);
              return a && r.includes(a)
                ? t.size > o
                  ? {
                      ok: !1,
                      message: i18n("errors.fileTooBig", { maxMb: n }),
                    }
                  : { ok: !0 }
                : { ok: !1, message: i18n("errors.fileType") };
            })(a);
            if (!e.ok) return void notify.error(e.message);
          }
          if (!t.includes(u))
            return void notify.error(i18n("errors.expiryInvalid"));
          if (!o.includes(m))
            return void notify.error(i18n("errors.viewsInvalid"));
          if (l.checked && !s.value)
            return (notify.error(i18n("errors.passwordRequired")), void s.focus());
          const w = new FormData();
          (v && w.append("note", v),
            w.append("expiry", u),
            w.append("max_views", m),
            y && w.append("one_time_use", "true"),
            p && w.append("restrict_ip", "true"),
            g && w.append("require_password", "true"),
            h && w.append("password", h),
            E && w.append("ip", E),
            f && w.append("file", a.files[0]),
            (i.disabled = !0));
          const S = (p) => {
            i.textContent =
              f && p >= 0 ? `در حال ایجاد... ${p}%` : "در حال ایجاد...";
          };
          S(-1);
          try {
            let n;
            if (f) {
              n = await new Promise((T, k) => {
                const x = new XMLHttpRequest();
                x.open("POST", "/api/share");
                x.upload.onprogress = (e) => {
                  e.lengthComputable &&
                    S(Math.round((e.loaded / e.total) * 100));
                };
                x.onload = () => {
                  if (x.status >= 200 && x.status < 300)
                    try {
                      T(JSON.parse(x.responseText));
                    } catch {
                      k(new Error(i18n("errors.shareCreate")));
                    }
                  else
                    try {
                      const e = JSON.parse(x.responseText);
                      k(
                        new Error(
                          e?.error?.message ||
                            e?.message ||
                            i18n("errors.shareCreate"),
                        ),
                      );
                    } catch {
                      k(new Error(i18n("errors.shareCreate")));
                    }
                };
                x.onerror = () => k(new Error(i18n("errors.shareCreate")));
                x.send(w);
              });
            } else {
              const t = await fetch("/api/share", { method: "POST", body: w });
              if (!t.ok) {
                const e = await t.json().catch(() => null);
                throw new Error(
                  e?.error?.message || e?.message || i18n("errors.shareCreate"),
                );
              }
              n = await t.json();
            }
            const o = new e("links").get({}),
              r = n.data.url;
            ((o[r] = {
              id: n.data.id,
              url: r,
              expiry: n.data.expiry,
              token: n.data.token,
            }),
              new e("links").set(o),
              r && (window.location.href = r));
          } catch (e) {
            (console.error("Error:", e),
              notify.error(e.message || i18n("errors.shareCreateRetry")));
          } finally {
            ((i.disabled = !1), (i.textContent = "ایجاد لینک اشتراک‌گذاری"));
          }
        });
    }),
      n(895),
      n(428),
      n(240),
      n(886),
      document.addEventListener("DOMContentLoaded", () => {
        const t = document.querySelector(".history-list");
        if (!t) return null;
        const n = new e("links").get({});
        let o = !1;
        (Object.keys(n).forEach((e) => {
          const { url: r, expiry: d } = n[e];
          if (new Date(d).getTime() < new Date().getTime())
            return (delete n[e], (o = !0), null);
          const a = `<li><a href="${r}">${r}</a></li>`;
          t.insertAdjacentHTML("beforeend", a);
        }),
          o && new e("links").set(JSON.stringify(n)));
      }),
      document.addEventListener("DOMContentLoaded", () => {
        const t = document.querySelector(".delete");
        return t
          ? (function () {
              const t = window.location.href;
              return new e("links").get({}).hasOwnProperty(t);
            })()
            ? void t.addEventListener("click", async () => {
                try {
                  const n = window.location.href,
                    o = new e("links").get({});
                  if (((t.textContent = "در حال حذف..."), !o.hasOwnProperty(n)))
                    throw new Error(i18n("errors.deleteNoPermission"));
                  if (
                    !confirm(
                      "آیا از حذف این اشتراک گذاری اطمینان دارید؟ این عمل غیرقابل بازگشت است!",
                    )
                  )
                    return void (t.textContent = "حذف این اشتراک گذاری");
                  const { token: r, id: d } = o[n];
                  if (
                    !(
                      await fetch(`/api/share/${d}`, {
                        method: "DELETE",
                        headers: { "Content-Type": "application/json" },
                        body: JSON.stringify({ token: r }),
                      })
                    ).ok
                  )
                    throw new Error(i18n("errors.deleteFailed"));
                  (delete o[n],
                    new e("links").set(o),
                    (window.location.href = "/"));
                } catch (e) {
                  (console.error("Error:", e), notify.error(e.message));
                } finally {
                  ((t.disabled = !1), (t.textContent = "حذف این اشتراک گذاری"));
                }
              })
            : t.remove()
          : null;
      }));
  })();
})();

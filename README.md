<p align="center">
  <img src="./src/assets/icon.png" alt="TShare Logo" width="80" />
</p>

<h1 align="center">TShare – Secure & Temporary Text Sharing</h1>

<p align="center">
  Fast. Private. Ephemeral.
</p>

---

## 🚀 Overview

**TShare** is a lightweight, privacy-first temporary text sharing service built with **Rust** and **Axum**.

No accounts.  
No tracking.  
No persistence beyond what you define.

Paste your text, generate a short link, control its lifetime — done.

---

## ✨ Features

- 🔗 6-character short link generation
- ⏳ Auto-expiration (5 minutes to 24 hours)
- 👁️ View count limitation
- 🔥 One-time (burn-after-read) mode
- 🔐 Optional password protection
- 🌍 IP-based access restriction
- 🗑️ Secure deletion via dedicated token
- 🧠 Local browser history of created links

---

## 🏗 Architecture

- **Backend:** Rust + Axum  
- **Temporary Storage:** Redis (TTL-based expiration)  
- **Metadata & Settings:** SQLite + SeaORM  
- **Template Engine:** Askama  

---

## 🎯 Design Principles

- Minimal surface area
- Stateless where possible
- Explicit expiration control
- High performance with predictable behavior

---

<p align="center">
  Built for speed, control, and privacy.
</p>

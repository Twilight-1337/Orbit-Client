# Orbit Client

<p align="center">
  <b>A lightweight, ultra-fast, and security-focused Minecraft launcher.</b><br>
  Built with <b>Tauri (Rust + HTML/JS/CSS)</b>.
</p>

---

## About Orbit Client

**Orbit Client** is an open-source Minecraft launcher designed to deliver maximum performance (high FPS), low RAM usage, and absolute security. 

It combines the slick, high-performance feel of **Lunar Client** (without the bloat or ads) with the powerful instance and profile management of **Prism Launcher**. 

>**Status:** This project is currently in early active development. I am currently learning Rust and laying down the architecture!

---

## Key Features

- **Auto-Optimization Out-of-the-Box:** 
  Creates instances pre-configured with top-tier optimization mods (*Sodium, Lithium, FerriteCore, ImmediatelyFast, Entity Culling*) and expert-tuned **JVM flags** for maximum FPS and zero micro-stutters.

- **Advanced Security & Anti-Malware:**
  - Strict reliance on **Modrinth API** (No CurseForge risks).
  - Local `.jar` drag-and-drop import with **SHA-512 hash verification** against public databases to protect users from account stealers (e.g., Microsoft account stealers) and malicious payload injections.

- **Smart Account System:**
  - Standard Microsoft Authentication.
  - **1 Microsoft License = Unlimited Offline Profiles** for local testing, alt accounts, or offline play.

- **Ultra-Low Memory Footprint:**
  - Consumes only **~30–50 MB of RAM** thanks to **Tauri & native Rust backend** (unlike Electron apps like Discord or Lunar Client Launcher that swallow 500+ MB).

-  **Linux-First & Windows Ready:**
  - Distributed primarily as a standalone **AppImage** on Linux (ensuring direct GPU driver access without Flatpak sandbox issues).
  - Native `.exe` Windows builds compiled automatically via GitHub Actions.

-  **Clean & Ad-Free UI:**
  - Zero advertisements or paid promotions.
  - Grouping system for instances (*Vanilla, Modded, PvP*).
  - Built-in Playtime tracking per instance.

---

## Tech Stack

- **Backend:** Rust ([Tauri Framework](https://tauri.app/))
- **Frontend:** HTML5, CSS3, JavaScript
- **API Integration:** Modrinth REST API
- **License:** [MIT License](LICENSE)

---

## Contributing & Support

Since this is an open-source project, feedback, bug reports, and pull requests will be welcome as development progresses!

If you like the concept of Orbit Client, feel free to **Star** this repository to follow along!
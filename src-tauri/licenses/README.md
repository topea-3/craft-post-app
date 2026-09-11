# Bundle license files

Installer / app resources for Tauri `bundle.licenseFile` and `bundle.resources`.

| File | Notes |
|------|--------|
| `LICENSE` | Copy of repo-root `/LICENSE` (MIT). Keep byte-identical when syncing. |
| `noto-serif-jp-OFL.txt` | Copy of `/third_party/noto-serif-jp/OFL.txt`. Keep byte-identical when syncing. |
| `THIRD_PARTY_NOTICES.md` | **Distribution-oriented** index (links point at files in this folder). |
| `noto-serif-jp-README.md` | **Distribution-oriented** notes (links point at `./noto-serif-jp-OFL.txt`). |

Do not copy `third_party/.../README.md` or root `THIRD_PARTY_NOTICES.md` verbatim into this folder — repo-relative links break inside the installer.

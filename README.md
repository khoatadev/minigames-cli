# 🎮 MINIGAMES-CLI

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2024_Edition-orange?logo=rust&style=flat-square" alt="Rust Edition 2024" />
  <img src="https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-blue?style=flat-square" alt="Cross-Platform" />
  <img src="https://img.shields.io/badge/CI-Passing-brightgreen?style=flat-square" alt="CI Status" />
  <img src="https://img.shields.io/badge/License-MIT-green?style=flat-square" alt="License MIT" />
</p>

```text
  __  __ ___ _  _ ___   ___   _   __  __ ___ ___ 
 |  \/  |_ _| \| |_ _| / __| /_\ |  \/  | __/ __|
 | |\/| || || .` || | | (_ |/ _ \| |\/| | _|\__ \
 |_|  |_|___|_|\_|___| \___/_/ \_\_|  |_|___|___/
```

> **Bộ sưu tập mini game phong cách Modern Retro Arcade chạy trực tiếp trong terminal của bạn.**  
> Viết bằng **Rust thuần** với hiệu năng cực đại, khởi động tức thì, không giật lag và không phụ thuộc thư viện TUI cồng kềnh.

---

## 🌟 Tính năng nổi bật

- ⚡ **Khởi động tức thì (Zero Latency):** Viết hoàn toàn bằng Rust trên nền tảng `crossterm`, tối ưu dựng hình từng frame mượt mà.
- 🎨 **Giao diện Modern Retro Arcade:** Thiết kế căn giữa hoàn hảo, đóng khung bo góc (`╭─╮`), thanh HUD Arcade Dashboard và thẻ điểm tổng kết (Scorecard) vinh danh.
- 🧩 **3 Game kinh điển đã hoàn thiện:**
  - 💣 **Dò mìn (Minesweeper):** Thuật toán rải mìn an toàn nhát đào đầu (3x3), cơ chế mở nhanh (chord) chuẩn Minesweeper quốc tế, 3 cấp độ khó.
  - ⚡ **Phản xạ (Speed Reaction):** Luyện gõ phím siêu tốc với pool ký tự đa dạng (chữ thường, chữ HOA, ký tự đặc biệt), hiệu ứng đếm ngược và hệ thống tính điểm kèm Combo Streak.
  - ⭕ **Cờ Caro (Gomoku 15x15):** Luật 5 quân liên tiếp, hỗ trợ cả 2 chế độ: Đấu với Bot AI thông minh (PvE) và Hai người chơi trên cùng máy (PvP).
- 🧱 **Kiến trúc tách bạch (Clean Architecture):** Tách biệt 100% logic trò chơi (`board.rs`, `reaction.rs`, `caro.rs`) khỏi phần hiển thị I/O, độ bao phủ kiểm thử đơn vị (Unit Tests) hoàn hảo.
- 🌐 **Đa nền tảng:** Chạy mượt mà trên Linux, macOS và Windows (hỗ trợ tốt nhất trên Windows Terminal).

---

## 🖥️ Trải nghiệm giao diện (UI Showcase)

### 1. Menu chính (Arcade Hub)
```text
       ╭──────────────────────────────────────────────────╮
       │                ★ MINI GAMES ★                    │
       ├──────────────────────────────────────────────────┤
       │                                                  │
       │   ▶ Dò mìn                                       │
       │     Phản xạ                                      │
       │     Caro                                         │
       │     Rắn săn mồi                 (sắp ra mắt)     │
       │                                                  │
       ├──────────────────────────────────────────────────┤
       │       ↑↓ chọn   Enter chơi   Q thoát             │
       ╰──────────────────────────────────────────────────╯
```

### 2. Cờ Caro (Gomoku 15x15)
```text
       ★ CỜ CARO - Đấu với Máy ★   Lượt: [X]   Nước: 12   Thời gian: 00:45
       ┌──────────────────────────────┐
       │ ·  ·  ·  ·  ·  ·  ·  ·  ·  · │
       │ ·  ·  ·  ·  ·  ·  ·  ·  ·  · │
       │ ·  ·  ✕  ◯  ✕  ·  ·  ·  ·  · │
       │ ·  ·  ·  ✕  ◯  ◯  ·  ·  ·  · │
       │ ·  ·  ·  ·  ✕  ·  ·  ·  ·  · │
       │ ·  ·  ·  ·  ·  ·  ·  ·  ·  · │
       └──────────────────────────────┘
         ↑↓←→ di chuyển   Enter/Space/X đánh   R ván mới   Esc menu
```

### 3. Dò mìn (Minesweeper)
```text
           ★ DÒ MÌN - Dễ ★     ⚑ Mìn: 10     Thời gian: 00:24
           ┌──────────────────┐
           │ ■  ■  ■  ■  ■  ■ │
           │ ■  1  1  1  ■  ■ │
           │ ■  1     1  ■  ■ │
           │ ■  1  1  2  ⚑  ■ │
           │ ■  ■  ■  ■  ■  ■ │
           └──────────────────┘
    X đào   Z cắm/gỡ cờ   R chơi lại   Esc về menu
```

### 4. Thẻ điểm Phản xạ (Reaction Scorecard)
```text
       ╭──────────────────────────────────────────────────╮
       │                   ★ HẾT GIỜ! ★                   │
       ├──────────────────────────────────────────────────┤
       │                                                  │
       │   Xếp hạng phong độ:  ★ RANK S ★                 │
       │   Điểm: 820                                      │
       │   Đúng: 28   Sai: 1   Chuỗi dài nhất: 19         │
       │   Phản xạ trung bình: 0.28s   Nhanh nhất: 0.16s  │
       │                                                  │
       ├──────────────────────────────────────────────────┤
       │            R chơi lại   Esc về menu              │
       ╰──────────────────────────────────────────────────╯
```

---

## 🚀 Hướng dẫn cài đặt

Chương trình hoạt động mượt mà trên **Linux, macOS và Windows**. Yêu cầu terminal hỗ trợ UTF-8 (khuyến nghị **Windows Terminal** trên Windows).

### Bước 1: Chuẩn bị môi trường Rust

Cài đặt trình biên dịch C và Rust:

| Hệ điều hành | Cài đặt công cụ nền | Cài đặt Rust (rustup) |
| :--- | :--- | :--- |
| **Ubuntu / Debian** | `sudo apt install -y build-essential curl` | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Fedora** | `sudo dnf install -y gcc curl` | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Arch Linux** | `sudo pacman -S --needed base-devel curl` | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **macOS** | `xcode-select --install` | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Windows** | Tải và chạy `rustup-init.exe` từ [rustup.rs](https://rustup.rs) (chọn MSVC toolchain) | *(Được cài tự động qua `rustup-init`)* |

### Bước 2: Biên dịch và cài đặt game

Clone repository và cài đặt trực tiếp vào thư mục bin của Cargo:

```bash
git clone https://github.com/khoatadev/minigames-cli
cd minigames-cli
cargo install --path .
```

Sau khi cài đặt xong, bạn có thể gõ `game-cli` ở bất kỳ đâu trong terminal!

> [!TIP]
> **Chạy thử ngay không cần cài đặt:**  
> Bạn chỉ cần gõ `cargo run --release` ngay trong thư mục dự án.

---

## 🕹️ Hướng dẫn điều khiển & Gameplay

### ⭕ 1. Cờ Caro (Gomoku 15x15)

Trò chơi cờ Caro cổ điển trên bàn cờ chuẩn `15x15`. Bên nào tạo được đường thẳng gồm **5 quân liên tiếp** (ngang, dọc hoặc đường chéo) trước sẽ giành chiến thắng.

| Phím bấm | Chức năng |
| :---: | :--- |
| `↑` `↓` `←` `→` hoặc `W` `A` `S` `D` | Di chuyển con trỏ trên bàn cờ |
| `Enter` / `Space` / `X` | Đặt quân cờ tại vị trí con trỏ |
| `R` | Bắt đầu ván mới (giữ nguyên chế độ) |
| `Esc` | Quay về menu chọn chế độ / menu chính |

- **2 Chế độ chơi:**
  - **Đấu với Máy (PvE):** Người chơi cầm quân `✕` (đi trước) đấu với thuật toán Bot AI cầm quân `◯`. Bot sử dụng thuật toán Heuristic đánh giá thế cờ theo thời gian thực, biết chủ động chặn các thế cờ 3, 4 nguy hiểm và tận dụng cơ hội kết liễu trận đấu.
  - **Hai người chơi (PvP):** Hai người luân phiên hạ cờ `✕` và `◯` trên cùng bàn phím.
- **Hiệu ứng trực quan:** Tự động highlight nước cờ vừa đánh (màu xám) và làm sáng toàn bộ đường thắng 5 quân (màu xanh lá) khi ván đấu kết thúc.

---

### 💣 2. Dò mìn (Minesweeper)

| Phím bấm | Chức năng |
| :---: | :--- |
| `↑` `↓` `←` `→` | Di chuyển con trỏ chọn ô |
| `X` | **Đào ô**. Nhấn trên ô số đã mở sẽ kích hoạt tính năng **Đào nhanh (Chord)** |
| `Z` | Cắm hoặc gỡ cờ đánh dấu mìn (`⚑`) |
| `R` | Chơi lại ván mới |
| `Esc` | Quay về Menu chính |

#### Quy tắc đặc biệt:
- **Khởi đầu luôn an toàn:** Nhát đào đầu tiên không bao giờ trúng mìn và tự động mở sạch một vùng an toàn `3x3` quanh vị trí đào.
- **Đào nhanh (Chord):** Đứng trên ô số đã mở, nếu xung quanh đã cắm đủ số cờ tương ứng với số hiển thị, bấm `X` sẽ tự động mở tất cả các ô lân cận còn lại. *(Cảnh báo: Nếu bạn cắm cờ sai vị trí, mìn sẽ nổ ngay lập tức!)*
- **3 Cấp độ thử thách:**
  - **Dễ:** Bàn cờ `9x9` với `10` quả mìn.
  - **Vừa:** Bàn cờ `16x16` với `40` quả mìn.
  - **Khó:** Bàn cờ `30x16` với `99` quả mìn.

---

### ⚡ 3. Phản xạ (Speed Typing Reaction)

Một ký tự ngẫu nhiên sẽ xuất hiện trên màn hình kèm màu sắc ngẫu nhiên. Nhiệm vụ của bạn là gõ chính xác phím đó nhanh nhất có thể.

- **Pool ký tự:** Bao gồm chữ thường `a–z`, chữ HOA `A–Z` (yêu cầu gõ kèm `Shift`) và ký tự đặc biệt `! ? @ # $ % & * +`.
- **Hệ thống tính điểm:**
  - Gõ đúng: Nhận từ **10 đến 100 điểm** tỉ lệ nghịch với thời gian phản xạ (càng nhanh điểm càng cao).
  - **Combo Streak:** Mỗi lần gõ đúng liên tiếp cộng thêm **+5 điểm** (tối đa cộng dồn +50 điểm/lần).
  - Gõ sai: Bị trừ **−20 điểm** và lập tức đưa chuỗi combo về 0.
- **Thời lượng:** Mỗi ván kéo dài **60 giây**, mở đầu bằng đồng hồ đếm ngược 3 giây sẵn sàng.
- **Bảng xếp hạng (Rank):** Kết thúc ván, hệ thống sẽ đánh giá phong độ từ **Rank S** (≥800đ), **Rank A**, **Rank B**, **Rank C** đến **Rank D**.

---

## 🛠️ Cấu trúc mã nguồn & Phát triển

Dự án áp dụng mô hình thiết kế phân tầng sạch sẽ:

```
src/
├── main.rs                  # Điểm khởi động, kiểm tra TTY và thiết lập terminal guard
├── term.rs                  # Quản lý Raw mode, Alternate Screen, tính toán căn giữa
├── menu.rs                  # Menu chính, chọn độ khó, chọn chế độ, Arcade Banner
├── board.rs                 # [Pure Logic] Toàn bộ luật chơi & thuật toán Dò mìn (no I/O)
├── reaction.rs              # [Pure Logic] Tính điểm, thời gian phản xạ & combo (no I/O)
├── caro.rs                  # [Pure Logic] Bàn cờ 15x15, phân định thắng thua, Bot AI (no I/O)
└── games/
    ├── mod.rs               # Module quản lý danh sách trò chơi
    ├── minesweeper.rs       # Renderer UI và vòng lặp tương tác game Dò mìn
    ├── reaction.rs          # Renderer UI, đếm ngược & thẻ điểm game Phản xạ
    └── caro.rs              # Renderer UI và vòng lặp tương tác game Cờ Caro
```

### Chạy kiểm thử & Linter

```bash
# Chạy toàn bộ 58 unit tests
cargo test

# Kiểm tra code style và quy chuẩn Rust
cargo clippy --all-targets
```

---

## 🗺️ Lộ trình phát triển (Roadmap)

- [x] Thiết kế giao diện **Modern Retro Arcade** toàn diện.
- [x] Hoàn thiện game **Dò mìn (Minesweeper)** với cơ chế Chord.
- [x] Hoàn thiện game **Phản xạ (Speed Typing)** với Scorecard & Rank.
- [x] 🎮 **Cờ Caro (Gomoku 15x15):** Chế độ 2 người chơi (PvP) và đấu với Bot AI (PvE).
- [ ] 🐍 **Rắn săn mồi (Snake Classic):** Lối chơi cổ điển với bảng điểm High Score.
- [ ] 🏆 **Bảng xếp hạng cục bộ (Local Leaderboard):** Lưu lại kỷ lục người chơi vào file config.

---

## 📄 Bản quyền (License)

Dự án được phát hành theo giấy phép [MIT License](LICENSE). Tự do sử dụng, chỉnh sửa và đóng góp!

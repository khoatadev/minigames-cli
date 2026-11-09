# minigames-cli

Bộ sưu tập mini game chạy ngay trong terminal (`game-cli`), viết bằng Rust. Menu chính điều hướng bằng bàn phím, mỗi game khởi động tức thì.

Hiện có **Dò mìn (Minesweeper)** và **Phản xạ**; Caro và Rắn săn mồi đang được phát triển.

## Cài đặt

Chơi được trên **Linux, macOS và Windows** — code thuần Rust, không gọi API riêng của hệ điều hành nào, và CI build + test trên cả 3 nền tảng. Yêu cầu duy nhất là terminal hỗ trợ UTF-8: macOS dùng Terminal/iTerm2 đều được; trên Windows nên dùng **Windows Terminal** (conhost cổ điển có thể vẽ kém các ký tự `■ ⚑ ✸`).

### Bước 1 — cài Rust và trình biên dịch C

| Hệ điều hành | Cách cài |
|--------------|----------|
| Ubuntu / Debian | `sudo apt install -y build-essential curl` |
| Fedora | `sudo dnf install -y gcc curl` |
| Arch | `sudo pacman -S --needed base-devel curl` |
| macOS | `xcode-select --install` (Command Line Tools) |
| Windows | tải `rustup-init.exe` từ [rustup.rs](https://rustup.rs) — chọn toolchain MSVC, installer sẽ hướng dẫn cài Visual Studio Build Tools |

Sau đó cài Rust trên Linux/macOS:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

(Windows: chạy `rustup-init.exe` là đủ.)

### Bước 2 — cài game

```bash
git clone https://github.com/khoatadev/minigames-cli
cd minigames-cli
cargo install --path .
```

Lệnh `game-cli` sẽ nằm trong `~/.cargo/bin` (Linux/macOS) hoặc `%USERPROFILE%\.cargo\bin` (Windows) — rustup tự thêm vào PATH, mở terminal mới nếu chưa thấy.

Chạy thử không cần cài: `cargo run --release`.

## Quickstart

```bash
game-cli
```

Menu chính hiện ra — chọn game bằng `↑`/`↓` và nhấn `Enter`.

## Dò mìn — điều khiển

| Phím | Hành động |
|------|-----------|
| `←` `→` `↑` `↓` | Di chuyển ô chọn |
| `X` | Đào ô — trên ô số đã mở: đào nhanh các ô kề |
| `Z` | Cắm / gỡ cờ |
| `R` | Chơi lại |
| `Esc` | Về menu |

Ba độ khó: Dễ 9x9 (10 mìn), Vừa 16x16 (40 mìn), Khó 30x16 (99 mìn). Nhát đào đầu luôn mở được một vùng: mìn chỉ được rải sau nhát đào đầu và không bao giờ nằm trong vùng 3x3 quanh nó (chuẩn Minesweeper cổ điển).

Bàn cờ: ô chưa mở hiển thị `■` xám dịu, cờ `⚑` vàng, con trỏ là ô nền trắng; thua thì ô nổ thành khối đỏ và các mìn còn lại hiện `✸` đỏ. Khung xám mờ bao quanh bàn cờ.

**Đào nhanh (chord):** đứng trên một ô số đã mở, nếu số cờ xung quanh đúng bằng số đó, nhấn `X` sẽ mở tất cả ô kề chưa mở (bỏ qua ô có cờ). Theo luật cổ điển: nếu cờ cắm sai và một ô kề không cờ là mìn — nổ.

## Phản xạ — luật chơi

Một ký tự xuất hiện ở vị trí bất kỳ trên màn hình — trong pool có **chữ thường a–z, chữ HOA A–Z và ký tự đặc biệt** `! ? @ # $ % & * +`, mỗi lần mang một màu ngẫu nhiên (7 màu). Gõ đúng **chính xác** ký tự đó (phân biệt hoa/thường, cần Shift) càng nhanh càng được nhiều điểm:

- Đúng: **10–100 điểm** theo tốc độ phản xạ (nhanh nhất = 100)
- Combo: mỗi lần đúng liên tiếp cộng thêm **+5 điểm** (tối đa +50)
- Gõ sai: **−20 điểm** và mất combo
- Ván dài **60 giây**, mở đầu bằng đếm ngược **3-2-1** (`Esc` hủy); `Esc` dừng sớm; `R` chơi lại, `Esc` về menu

Quy ước chung: trong game, `Esc` là phím thoát về menu.

## Phát triển

```bash
cargo test          # chạy unit test (logic bàn cờ + render)
cargo clippy        # lint
```

Cấu trúc:

```
src/
├── main.rs              # điểm vào, dựng terminal và gọi menu
├── board.rs             # logic dò mìn (thuần, không I/O)
├── menu.rs              # menu chính + chọn độ khó
├── term.rs              # thiết lập terminal, poll phím
└── games/
    └── minesweeper.rs   # màn hình game, điều khiển, đồng hồ
```

Thêm game mới: tạo module trong `src/games/`, viết hàm `run(&Difficulty) -> io::Result<()>`, rồi đăng ký vào `GAMES` trong `src/menu.rs`.

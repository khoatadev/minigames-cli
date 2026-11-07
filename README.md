# Terminal Mini Games

Bộ sưu tập mini game chạy ngay trong terminal, viết bằng Rust. Menu chính điều hướng bằng bàn phím, mỗi game khởi động tức thì.

Hiện có **Dò mìn (Minesweeper)**; Caro và Rắn săn mồi đang được phát triển.

## Quickstart

Yêu cầu Rust 1.85+ ([rustup](https://rustup.rs)).

```bash
cargo run --release
```

Menu chính hiện ra — chọn game bằng `↑`/`↓` và nhấn `Enter`.

## Dò mìn — điều khiển

| Phím | Hành động |
|------|-----------|
| `←` `→` `↑` `↓` | Di chuyển ô chọn |
| `X` | Đào ô |
| `Z` | Cắm / gỡ cờ |
| `R` | Chơi lại |
| `Q` / `Esc` | Về menu |

Ba độ khó: Dễ 9x9 (10 mìn), Vừa 16x16 (40 mìn), Khó 30x16 (99 mìn). Lần đào đầu tiên luôn an toàn — mìn chỉ được rải sau nhát đào đầu.

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

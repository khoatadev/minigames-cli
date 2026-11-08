# minigames-cli

Bộ sưu tập mini game chạy ngay trong terminal (`game-cli`), viết bằng Rust. Menu chính điều hướng bằng bàn phím, mỗi game khởi động tức thì.

Hiện có **Dò mìn (Minesweeper)** và **Phản xạ**; Caro và Rắn săn mồi đang được phát triển.

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
| `X` | Đào ô — trên ô số đã mở: đào nhanh các ô kề |
| `Z` | Cắm / gỡ cờ |
| `R` | Chơi lại |
| `Esc` | Về menu |

Ba độ khó: Dễ 9x9 (10 mìn), Vừa 16x16 (40 mìn), Khó 30x16 (99 mìn). Nhát đào đầu luôn mở được một vùng: mìn chỉ được rải sau nhát đào đầu và không bao giờ nằm trong vùng 3x3 quanh nó (chuẩn Minesweeper cổ điển).

Bàn cờ: ô chưa mở hiển thị `■` xám dịu, cờ `⚑` vàng, con trỏ là ô nền trắng; thua thì ô nổ thành khối đỏ và các mìn còn lại hiện `✸` đỏ. Khung xám mờ bao quanh bàn cờ.

**Đào nhanh (chord):** đứng trên một ô số đã mở, nếu số cờ xung quanh đúng bằng số đó, nhấn `X` sẽ mở tất cả ô kề chưa mở (bỏ qua ô có cờ). Theo luật cổ điển: nếu cờ cắm sai và một ô kề không cờ là mìn — nổ.

## Phản xạ — luật chơi

Một chữ cái (a–z) hiện ở vị trí bất kỳ trên màn hình, mỗi lần xuất hiện mang một màu ngẫu nhiên (7 màu); gõ đúng chữ đó càng nhanh càng được nhiều điểm:

- Đúng: **10–100 điểm** theo tốc độ phản xạ (nhanh nhất = 100)
- Combo: mỗi lần đúng liên tiếp cộng thêm **+5 điểm** (tối đa +50)
- Gõ sai: **−20 điểm** và mất combo
- Ván dài **60 giây**; `Esc` dừng sớm; `R` chơi lại, `Esc` về menu

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

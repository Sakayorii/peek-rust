**Tiếng Việt** | [English](README.en.md)

# peek-rust

Bản port Rust của [peek-vanilla](https://github.com/Sakayorii/peek-vanilla), mà peek-vanilla tự nó là bản port 1:1 đã được kiểm chứng của [Peek](https://github.com/doan-labs/peek) do Doan Labs làm (MIT).
Một cái tên đưa vào, một khuôn mặt đi ra: avatar xác định, không mạng, không cơ sở dữ liệu.

Thuộc family peek: `peek` (bản gốc, React), `peek-vanilla` (JS), `peek-kotlin` (JVM), `peek-rust` (Rust).

```
src/
├── js.rs       - Ngữ nghĩa JavaScript, chính xác (in số, tidy, JSON)
├── identity.rs - tidy / FNV-1a / mulberry32 / identify
├── tables.rs   - mặt, màu, bộ phận, biểu cảm (sinh ra từ tables của bản port Kotlin)
├── scene.rs    - cây node mà draw() xây dựng
├── draw.rs     - một hàm hình học thuần túy (port của draw.js)
├── svg.rs      - settle / serialize / to_svg (port của svg.js)
└── lib.rs
```

## Cách dùng

```rust
use sakayori_peek::{to_svg, PeekOptions};

let svg = to_svg("Sakayori", &PeekOptions::default());
// hoặc có tùy chọn:
let svg = to_svg("Sakayori", &PeekOptions {
    expression: "happy".to_string(),
    size: 128.0,
    eyewear: Some("glasses".to_string()),
    headwear: Some("sprout".to_string()),
    neckwear: Some("tie".to_string()),
    ..Default::default()
});
```

## Kiểm chứng

Bản port được kiểm chứng giống peek-vanilla đến từng byte trên corpus golden 500 trường hợp: tên được chọn lọc (unicode, emoji, khoảng trắng, NFC, ký tự điều khiển, escape XML), cả 11 biểu cảm, mọi ghi đè theo trục, mọi slot phụ kiện và các tổ hợp phụ kiện, kích thước, frame, riso, tiêu đề, id tường minh, hướng nhìn, cộng thêm 300 tên fuzz có seed.

```sh
node gen-corpus-rs.mjs   # sinh lại tests/corpus_data.rs từ peek-vanilla
cargo test               # 500 trường hợp golden + kiểm tra tính xác định
```

Một điểm khác biệt có chủ ý so với JS: một `&str` của Rust không thể chứa surrogate đơn lẻ, nên trường hợp corpus đó dùng tên đã được thay bằng U+FFFD ở cả hai phía, chính là thứ TextEncoder đưa vào hàm băm ở phía JS.

## Bản này port từ upstream phiên bản nào?

Bản này port từ upstream @doan-labs/peek 1.1.0: hệ thống phụ kiện (eyewear, headwear, neckwear) trên nền engine 1.0. Các khuôn mặt không có phụ kiện vẫn render giống hệt trước đây đến từng byte. `PeekOptions` mang ba slot dưới dạng các field `eyewear`, `headwear` và `neckwear`.

## Giấy phép

MIT, xem [LICENSE](LICENSE). Peek gốc do Doan Labs làm.

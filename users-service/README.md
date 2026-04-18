# users-service

บริการ Rust `axum` ที่กำหนดขอบเขตตามแนว Onion Architecture

## ข้อบังคับด้านสถาปัตยกรรม (บังคับใช้)

ผู้ร่วมพัฒนาทุกคนต้องปฏิบัติตามกติกานี้ หากโค้ดใดละเมิด ให้รีแฟกเตอร์ก่อน merge

### 1) โครงสร้างชั้น (Layer Structure)

ให้ใช้โครงสร้างเชิงตรรกะดังนี้:

- `presentation`: เรื่อง HTTP เท่านั้น (routes, handlers, request/response mapping, middleware)
- `application`: use cases, orchestration, application services, app-level errors
- `domain`: entities, value objects, กฎธุรกิจ, repository/service ports (traits), domain errors
- `infrastructure`: ฐานข้อมูล, external clients, messaging และ implementation จริงของ ports

### 2) ทิศทาง Dependency

Dependency ต้องชี้เข้าในเท่านั้น:

- `presentation -> application -> domain`
- `infrastructure -> application/domain` (ทำหน้าที่ implement traits ที่ประกาศในชั้นด้านใน)

สิ่งที่ห้ามทำ:

- `domain` import `axum`, `tokio`, crates ด้าน DB หรือรายละเอียด transport/framework
- `application` import types ของ HTTP framework
- `presentation` อ่าน/เขียน DB โดยตรง

### 3) ขอบเขตความรับผิดชอบ

- `presentation` ต้องบาง: parse input, เรียก use case และ map output เป็น HTTP response
- กฎธุรกิจต้องอยู่ใน `domain` และ `application` ไม่ใช่ใน handlers
- งาน persistence และ external I/O ต้องอยู่ใน `infrastructure`

### 4) สัญญาข้อมูล (Contracts) และ Mapping

- ห้ามใช้ struct ตัวเดียวกันทุกชั้น
- ต้องแยก type ตามขอบเขต:
  - HTTP DTOs (`presentation`)
  - use case input/output DTOs (`application`)
  - domain entities/value objects (`domain`)
  - DB entities (`infrastructure`)
- การ map ข้ามชั้นต้อง explicit (`From`/`TryFrom` หรือ mapper function เฉพาะ)

### 5) การจัดการ Error

- แต่ละชั้นต้องมี error type ของตัวเอง
- แปลง error ข้ามชั้นตามลำดับ:
  - `DomainError -> AppError -> HttpError`
- การ map เป็น HTTP status code ทำเฉพาะใน `presentation`

### 6) State และ DI (Axum)

- ประกอบ dependency ใน composition root (`main.rs`/`app.rs`)
- inject ผ่าน shared app state (เช่น `State<Arc<AppState>>`)
- handlers ต้องพึ่ง abstraction (application services) ไม่ใช่ infra type โดยตรง

### 7) นโยบายการทดสอบ

- `domain`: unit tests สำหรับกฎธุรกิจ
- `application`: unit tests โดยใช้ mocked/fake ports
- `presentation`: integration tests สำหรับ routes/handlers และ HTTP contract
- `infrastructure`: integration tests กับ adapter จริงเมื่อเหมาะสม

### 8) กฎคุณภาพโค้ด

- แยกโมดูลให้เล็กและมีหน้าที่เดียว
- ตั้งชื่อให้ชัดเจน และใช้ guard clauses/early return
- ห้ามใช้ `unwrap()`/`expect()` ในเส้นทาง request (ยกเว้นช่วง startup ที่มีเหตุผลชัดเจน)
- หลีกเลี่ยง circular module dependencies

## โครงสร้างโฟลเดอร์ที่แนะนำ

```txt
src/
  main.rs
  app.rs
  presentation/
  application/
  domain/
  infrastructure/
```

## PR Checklist

ก่อนเปิด PR หรือ merge ต้องตรวจให้ครบทุกข้อ:

- [ ] ไม่มีการละเมิดทิศทาง dependency ระหว่าง layers
- [ ] ไม่มี business logic อยู่ใน handlers
- [ ] ไม่มี framework/DB types รั่วเข้า `domain`
- [ ] มีการ map errors ข้ามชั้นอย่างชัดเจน
- [ ] มี mapping ข้ามขอบเขตข้อมูลแบบ explicit
- [ ] เพิ่ม/ปรับปรุง tests ในชั้นที่ได้รับผลกระทบ

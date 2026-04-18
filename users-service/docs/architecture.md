# สถาปัตยกรรม Users Service

## ภาพรวม

`users-service` คือ REST API สำหรับจัดการข้อมูลผู้ใช้ พัฒนาด้วย `axum` และจัดโครงสร้างตาม Onion Architecture เพื่อแยกกฎธุรกิจออกจาก HTTP และฐานข้อมูลให้ชัดเจน

อินเทอร์เฟซปัจจุบันคือ endpoints ภายใต้ `/users` สำหรับ `list/get/create/update/delete` โดยใช้แนวทาง soft delete

## เป้าหมาย

- แยก business logic ออกจาก framework และ persistence details
- ทำให้การเปลี่ยน repository implementation ไม่กระทบ use case หลัก
- ทำให้ service layer ทดสอบได้ด้วย fake/mocked repository
- คง `main.rs` ให้เป็น composition root ที่ชัดเจน

## โครงสร้างแบบเลเยอร์ (Onion)

### 1) Domain layer (กฎหลักของระบบ)

พาธ: `src/domain/users`

- `entity.rs`
  - นิยาม `User` (entity สำหรับการตอบกลับเชิงธุรกิจ)
  - นิยาม `UserNameInput` (input model สำหรับ create/update)
- `repository.rs`
  - นิยาม `UserRepository` trait เป็นสัญญาการเข้าถึงข้อมูลผู้ใช้
- `error.rs`
  - นิยาม `DomainError` (`NotFound`, `RepositoryFailure`)

เลเยอร์นี้ไม่ผูกกับ `axum`, `toasty`, หรือรายละเอียด transport/database

### 2) Application layer (use cases)

พาธ: `src/application/users`

- `users_service.rs`
  - `UserService` orchestration use cases ทั้งหมดของผู้ใช้
  - เมธอดหลัก: `list_users`, `get_user_by_id`, `create_user`, `update_user`, `delete_user`
  - เรียกผ่าน `Arc<dyn UserRepository + Send + Sync>`
- `dto.rs`
  - นิยาม HTTP-facing DTO (`CreateUserDto`, `UpdateUserDto`)
  - map DTO ไป `UserNameInput` อย่าง explicit
- `error.rs`
  - map `DomainError` -> `AppError`
  - implement `IntoResponse` เพื่อแปลงเป็น HTTP status/message

Application layer พึ่งพา domain ได้ แต่ไม่แตะ DB framework โดยตรง

### 3) Presentation layer (input adapters)

พาธ: `src/presentation/users`

- `routes.rs`
  - ผูกเส้นทาง `/users` และ `/users/{id}` เข้ากับ handlers
- `handlers.rs`
  - รับ `Path`, `Json`, `State<Arc<AppState>>`
  - เรียก `UserService`
  - คืน `Result<Json<_>, AppError>`

หน้าที่หลักของ presentation คือ map HTTP request/response เท่านั้น

### 4) Infrastructure layer (framework/system adapters)

พาธ: `src/infrastructure/users`

- `toasty_repository.rs`
  - นิยาม `UserRecord` (Toasty model สำหรับ persistence)
  - implement `UserRepository` ผ่าน `ToastyUserRepository`
  - map DB rows -> domain `User`
  - จัดการ soft delete (`active=false`, `deleted_at=now`)
  - คุม DB access ผ่าน `Arc<Mutex<toasty::Db>>`

Infrastructure คือขอบเขตที่อนุญาตให้พึ่งพา DB/framework types

### 5) Composition root

พาธ: `src/main.rs`, `src/app.rs`

- โหลด env (`dotenvy::dotenv`)
- resolve `AppConfig` จาก environment
- สร้าง Toasty DB connection
- ประกอบ `ToastyUserRepository` -> `UserService` -> `AppState`
- สร้าง `Router`, bind listener, และ `axum::serve`

ไม่ควรวาง business rules ในจุดนี้

## Runtime Flow

1. โปรเซสเริ่มใน `main`
2. โหลด config และประกอบ app dependencies
3. เริ่ม HTTP server ที่ `bind_addr`
4. client เรียก endpoint ใต้ `/users`
5. handler ใน presentation รับคำขอและเรียก `UserService`
6. `UserService` เรียก `UserRepository` trait
7. `ToastyUserRepository` จัดการ persistence และ map ผลลัพธ์เป็น domain entity
8. response ถูกส่งกลับเป็น JSON หรือ HTTP error ตาม `AppError`

## API Surface ปัจจุบัน

Base path: `/users`

- `GET /users` ดึงรายการผู้ใช้ที่ยัง active และไม่ถูก soft delete
- `GET /users/{id}` ดึงผู้ใช้รายเดียว
- `POST /users` สร้างผู้ใช้ใหม่
- `PATCH /users/{id}` แก้ไขข้อมูลผู้ใช้
- `DELETE /users/{id}` soft delete ผู้ใช้

## การตั้งค่า (Configuration)

พาธ: `src/config/mod.rs`

- อ่านค่าจำเป็นจาก environment ผ่าน `AppConfig::from_env()`
- `main.rs` รองรับ `.env` ผ่าน `dotenvy` ก่อนอ่าน config

## ทิศทาง Dependency

กฎการพึ่งพาต้องชี้เข้าแกนกลางเสมอ:

- `presentation -> application -> domain`
- `infrastructure -> domain` (implements ports/traits)
- `domain` ห้ามพึ่งพาเลเยอร์ด้านนอก

กฎนี้ช่วยให้เปลี่ยน framework หรือ storage adapter ได้โดยกระทบ core behavior น้อยที่สุด

## แนวทางการต่อยอด

- เพิ่ม endpoint ใหม่:
  1. ขยาย contract/behavior ใน `domain` (ถ้ามีกฎใหม่)
  2. เพิ่มเมธอด orchestration ใน `UserService`
  3. เพิ่ม handler/route ใน `presentation`
  4. เพิ่ม implementation ใน repository adapter

- เพิ่ม data source ใหม่:
  - คง `UserRepository` trait เดิม
  - implement adapter ใหม่ใน `infrastructure`
  - inject ที่ composition root

- เพิ่ม tests:
  - unit tests ที่ `domain` สำหรับ business rules
  - unit tests ที่ `application` โดย mock repository
  - integration tests ที่ `presentation` สำหรับ HTTP contracts

## ข้อจำกัดปัจจุบัน

- ยังไม่มี validation เชิงลึกของ input DTO (เช่นความยาว/format ของ username/password)
- response ของ `User` ยังรวม `password` อยู่ (ควรแยก response model สำหรับ public API)
- error model ยังเป็นข้อความสั้นแบบ plain text
- ยังไม่มี structured logging, tracing, metrics ที่ชัดเจน

## ข้อเสนอแนะสำหรับการพัฒนาต่อ

- แยก `User` domain entity ออกจาก API response DTO เพื่อปิดข้อมูลอ่อนไหว
- เพิ่ม validation layer สำหรับ `CreateUserDto`/`UpdateUserDto`
- เพิ่ม observability: structured logs + tracing + metrics
- เพิ่ม migration strategy และ transaction policy สำหรับ use case ที่ซับซ้อนขึ้น

## แต่ละโฟลเดอร์มีไว้ทำอะไร

- `src/domain`
  - กฎธุรกิจหลัก, entities, repository traits, domain errors
  - ต้องไม่มีรายละเอียดของ HTTP/DB framework

- `src/application`
  - use cases และการประสานงานธุรกิจระดับระบบ
  - map ข้อมูลเข้า/ออกระหว่าง presentation กับ domain

- `src/presentation`
  - รับ request จาก HTTP แล้วแปลงเป็นการเรียก application
  - map application output/error กลับเป็น HTTP response

- `src/infrastructure`
  - adapter เชื่อมต่อ DB และระบบภายนอก
  - implementation จริงของ domain/application ports

- `src/config`
  - จัดการ configuration จาก environment

- `src/main.rs` และ `src/app.rs`
  - composition root และ server bootstrap
  - ไม่ควรบรรจุ business logic

## ถ้าอยากเพิ่ม endpoint ใหม่ ต้องทำที่ไหน

ตัวอย่างเพิ่ม `GET /users/active`:

1. อัปเดต domain
   - เพิ่ม contract ใน `UserRepository` เช่น `list_active_users`

2. อัปเดต application
   - เพิ่มเมธอดใน `UserService` เพื่อ orchestrate use case

3. อัปเดต infrastructure
   - implement query ใน `ToastyUserRepository`

4. อัปเดต presentation
   - เพิ่ม route และ handler ใหม่ใน `presentation/users`

## ถ้าอยากเพิ่ม logic ธุรกิจ ควรวางไว้ตรงไหน

- กฎธุรกิจล้วนๆ (ไม่แตะ I/O)
  - วางใน `domain`

- logic ที่เป็น orchestration ของหลายขั้นตอน
  - วางใน `application` (`UserService`)

- logic ที่ต้องคุยกับ DB/framework ภายนอก
  - วางใน `infrastructure`

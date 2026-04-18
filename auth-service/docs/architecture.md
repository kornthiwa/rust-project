# สถาปัตยกรรม Auth Service

## ภาพรวม

`auth-service` คือ REST API สำหรับงานยืนยันตัวตน (authentication) โดยเน้น use cases หลัก 2 ส่วน:

- สมัครบัญชี (`register`)
- เข้าสู่ระบบ (`login`)

บริการพัฒนาด้วย `axum` และจัดโครงสร้างตาม Onion Architecture เพื่อแยก business rules ออกจาก HTTP และ persistence ให้ชัดเจน

## เป้าหมาย

- แยกกฎธุรกิจออกจาก framework และฐานข้อมูล
- ให้การเปลี่ยน storage adapter กระทบ use case น้อยที่สุด
- รองรับการทดสอบ service ผ่าน mock/fake repository
- คง `main.rs` และ `app.rs` ให้เป็น composition root ที่อ่านง่าย
- ยกระดับความปลอดภัยด้วย password hashing และ JWT token

## โครงสร้างแบบเลเยอร์ (Onion)

### 1) Domain layer (กฎธุรกิจหลัก)

พาธ: `src/domain/auth`

- `entity.rs`
  - `CreateAccountInput`, `LoginInput` สำหรับข้อมูลนำเข้า
  - `AccountInfo` สำหรับ representation ของ account ในชั้นธุรกิจ
  - `CreateAccountResult`, `LoginResult` สำหรับ output ของ use case
- `repository.rs`
  - นิยาม `AuthRepository` trait เป็นสัญญาในการเข้าถึงข้อมูลบัญชี
  - เมธอดหลัก: `create_account`, `get_account_by_username`
- `error.rs`
  - นิยาม `DomainError` เช่น `NotFound`, `Conflict`, `InvalidCredentials`, `InactiveAccount`, `RepositoryFailure`

เลเยอร์นี้ต้องไม่พึ่งพา `axum`, `toasty`, หรือ HTTP/database details

### 2) Application layer (use cases)

พาธ: `src/application/auth`

- `auth_service.rs`
  - orchestration ของ use cases ทั้งหมด
  - `create_account`:
    - รับ input จาก DTO
    - hash password ด้วย `argon2`
    - ส่งต่อให้ repository บันทึกข้อมูล
  - `login`:
    - โหลด account จาก repository
    - ตรวจสถานะ account (active / inactive)
    - verify password hash ด้วย `argon2`
    - สร้าง JWT access token
  - พึ่งพา `Arc<dyn AuthRepository + Send + Sync>`
- `dto.rs`
  - `CreateAccountDto`, `LoginDto`
  - map DTO -> domain input แบบ explicit
- `error.rs`
  - map `DomainError` -> `AppError`
  - implement `IntoResponse` เพื่อ map เป็น HTTP status/message

Application layer อนุญาตให้พึ่งพา domain และ libraries สำหรับ orchestration ได้ แต่ไม่เข้าถึง DB โดยตรง

### 3) Presentation layer (input adapters)

พาธ: `src/presentation/auth`

- `routes.rs`
  - ผูกเส้นทาง `/auth/register`, `/auth/login`
- `handlers.rs`
  - รับ `Json` + `State<Arc<AppState>>`
  - เรียก `AuthService`
  - คืน `Result<Json<_>, AppError>`

หน้าที่หลักของ presentation คือ map HTTP request/response เท่านั้น

### 4) Infrastructure layer (framework/system adapters)

พาธ: `src/infrastructure/auth`

- `toasty_repository.rs`
  - นิยาม `Account` model สำหรับ table `account`
  - implement `AuthRepository` ผ่าน `ToastyAuthRepository`
  - map DB model -> domain `AccountInfo`
  - ตรวจ soft-delete (`deleted_at`) ในการ query username
  - ป้องกัน username ซ้ำในขั้น create account
  - คุม DB access ผ่าน `Arc<Mutex<toasty::Db>>`

Infrastructure คือจุดที่อนุญาตให้พึ่งพา DB/framework โดยตรง

### 5) Composition root

พาธ: `src/main.rs`, `src/app.rs`, `src/config/mod.rs`

- `main.rs`
  - โหลด environment (`dotenvy::dotenv`)
  - อ่าน config จาก env
  - สร้าง listener และเริ่ม `axum::serve`
- `app.rs`
  - เชื่อม Toasty DB
  - ประกอบ dependency graph:
    - `ToastyAuthRepository` -> `AuthService` -> `AppState`
  - mount routes `/auth/*`
- `config/mod.rs`
  - อ่าน `DATABASE_URL`, `PORT`, `JWT_SECRET`, `JWT_EXP_MINUTES`

ไม่ควรวาง business logic ใน composition root

## Runtime Flow

1. โปรเซสเริ่มที่ `main`
2. โหลด env และสร้าง `AppConfig`
3. `build_app` สร้าง DB connection และประกอบ dependencies
4. HTTP request เข้า `/auth/register` หรือ `/auth/login`
5. handler map request -> service call
6. `AuthService` ประมวลผลกฎธุรกิจ
7. repository adapter ทำงานกับ table `account`
8. response คืนเป็น JSON หรือ HTTP error ตาม `AppError`

## API Surface ปัจจุบัน

Base path: `/auth`

- `POST /auth/register`
  - รับ username + password
  - hash password ก่อนบันทึก
  - กัน username ซ้ำ (ตอบ `409`)
- `POST /auth/login`
  - รับ username + password
  - verify password hash
  - ออก JWT access token เมื่อสำเร็จ

## Security Notes

- ไม่เก็บ password เป็น plain text (เก็บเป็น Argon2 hash)
- ไม่คืน password hash ใน API response
- JWT เซ็นด้วย `JWT_SECRET` จาก environment
- token expiry ควบคุมผ่าน `JWT_EXP_MINUTES` (default: 60 นาที เมื่อไม่กำหนด)
- ควรหมุน `JWT_SECRET` ตามนโยบายความปลอดภัยของระบบ

## การตั้งค่า (Configuration)

ไฟล์: `src/config/mod.rs`

- `DATABASE_URL` (required)
- `PORT` (optional, default `3000`)
- `JWT_SECRET` (required)
- `JWT_EXP_MINUTES` (optional, default `60`)

## ทิศทาง Dependency

กฎการพึ่งพา:

- `presentation -> application -> domain`
- `infrastructure -> domain` (implements ports)
- `domain` ต้องไม่พึ่งพาเลเยอร์นอก

## แนวทางการต่อยอด

- เพิ่ม refresh token flow (`/auth/refresh`)
- รองรับ token revocation / denylist
- ย้าย duplicate-check username ให้พึ่งพา DB unique constraint + error translation เป็นหลัก
- เพิ่ม validation ของ input (length/charset/password policy)
- เพิ่ม unit/integration tests สำหรับ auth flows
- เพิ่ม structured logging, tracing, metrics

## แต่ละโฟลเดอร์มีไว้ทำอะไร

- `src/domain`
  - กฎธุรกิจหลัก, entities, repository traits, domain errors
- `src/application`
  - use cases, orchestration, mapping input/output
- `src/presentation`
  - รับ HTTP request และคืน HTTP response
- `src/infrastructure`
  - adapter เชื่อม DB และ implementation ของ repository
- `src/config`
  - อ่านและจัดการ configuration จาก environment
- `src/main.rs`, `src/app.rs`
  - composition root และการ bootstrap server

## ถ้าอยากเพิ่ม endpoint ใหม่ ต้องทำที่ไหน

ตัวอย่างเพิ่ม `POST /auth/refresh`:

1. อัปเดต domain
   - เพิ่ม contract ที่เกี่ยวกับ token lifecycle
2. อัปเดต application
   - เพิ่มเมธอด orchestration ใน `AuthService`
3. อัปเดต infrastructure
   - เพิ่ม token store/adapter ตามที่ออกแบบ
4. อัปเดต presentation
   - เพิ่ม route + handler ใหม่ใน `presentation/auth`

## ถ้าอยากเพิ่ม logic ธุรกิจ ควรวางไว้ตรงไหน

- กฎธุรกิจล้วนๆ (ไม่แตะ I/O)
  - วางใน `domain`
- orchestration หลายขั้นตอน
  - วางใน `application`
- logic ที่แตะ DB/framework/external systems
  - วางใน `infrastructure`

# สถาปัตยกรรม Discord Service

## ภาพรวม

`discord-service` คือบอท Discord ที่ออกแบบด้วย Onion Architecture
อินเทอร์เฟซปัจจุบันของบอทใช้ Discord Slash Commands (`/ping`, `/help`) และแยก business logic ออกจากรายละเอียดของ Discord SDK

## เป้าหมาย

- ทำให้กฎทางธุรกิจไม่ผูกกับเฟรมเวิร์ก
- แยกการใช้งาน Discord SDK ไปไว้เลเยอร์ด้านนอก
- ทำให้ logic ของคำสั่งทดสอบได้โดยไม่ต้องรัน Discord จริง
- ทำให้ entrypoint เรียบง่ายและชัดเจน

## โครงสร้างแบบเลเยอร์ (Onion)

### 1) Domain layer (กฎหลักของระบบ)

พาธ: `src/domain`

- `command.rs`
  - นิยาม `BotCommand` enum (`Ping`, `Help`, `Unknown`)
  - แปลงชื่อ slash command เป็น domain command ผ่าน `from_slash_name`

เลเยอร์นี้มีแต่ logic บริสุทธิ์ และไม่ขึ้นกับ Serenity หรือ runtime infrastructure

### 2) Application layer (use cases)

พาธ: `src/application`

- `use_cases/handle_message.rs`
  - `HandleSlashCommandUseCase` ทำหน้าที่ประสานการทำงานของคำสั่ง
  - อินพุต: ชื่อ slash command (`&str`)
  - เอาต์พุต: ข้อความตอบกลับ (`&'static str`)

Application layer พึ่งพา domain ได้ แต่ไม่พึ่งพา Discord SDK

### 3) Presentation layer (input adapters)

พาธ: `src/presentation/discord`

- `handler.rs`
  - implement Serenity `EventHandler`
  - ลงทะเบียน global slash commands ตอน `ready`
  - จัดการ `interaction_create` และ map อินพุตไปยัง use case
  - ส่ง interaction response กลับผ่าน Serenity

- `bot.rs`
  - สร้างและเริ่มต้น Discord client
  - wire `AppConfig` + use case + event handler เข้าด้วยกัน
  - ใช้ `GatewayIntents::GUILDS` สำหรับพฤติกรรมที่ขับเคลื่อนด้วย slash command

Presentation layer มีหน้าที่แปลง event จาก transport/framework ให้กลายเป็นการเรียกใช้งาน application

### 4) Infrastructure layer (framework/system adapters)

พาธ: `src/infrastructure`

- เก็บ technical adapters และการเชื่อมต่อระบบภายนอก
- ใน implementation ปัจจุบัน การตอบกลับ Discord สำหรับ interaction flow ทำใน presentation
- เลเยอร์นี้ถูกเตรียมไว้สำหรับ outbound adapters เพิ่มเติม (database, cache, APIs, queues, logging providers ฯลฯ)

### 5) Composition root

พาธ: `src/main.rs`

- โหลด configuration
- สร้าง use case instance
- สร้าง bot instance
- เริ่ม runtime ของบอท

ไม่ควรวาง business rules ไว้ใน `main.rs`

## Runtime Flow

1. โปรเซสเริ่มต้นใน `main`
2. โหลด config จาก environment (`DISCORD_TOKEN`)
3. เริ่ม Discord client พร้อม event handler
4. เมื่อ `ready` จะลงทะเบียน slash commands
5. ผู้ใช้เรียก `/ping` หรือ `/help` ใน Discord
6. `interaction_create` รับคำสั่งเข้ามา
7. Handler เรียก `HandleSlashCommandUseCase`
8. Use case ตีความ domain command และคืนข้อความตอบกลับ
9. Handler ส่ง Discord interaction response กลับไป

## การตั้งค่า (Configuration)

พาธ: `src/config.rs`

- ตัวแปรที่จำเป็น: `DISCORD_TOKEN`
- แหล่งที่มา: process environment (และรองรับ `.env` หากมีการโหลดก่อน resolve config)

## ทิศทาง Dependency

กฎการพึ่งพาต้องชี้เข้าแกนกลางเสมอ:

- `presentation` -> `application` -> `domain`
- `infrastructure` -> `application` หรือ `domain` (เมื่อจำเป็น)
- `domain` ห้ามพึ่งพาเลเยอร์ด้านนอก

กฎนี้ช่วยให้ core behavior คงที่ ขณะที่การเชื่อมต่อ Discord/เฟรมเวิร์กสามารถเปลี่ยนได้อย่างปลอดภัย

## แนวทางการต่อยอด

- เพิ่ม slash command ใหม่:
  1. ขยาย `BotCommand` ใน domain
  2. map ชื่อคำสั่งใน `from_slash_name`
  3. ขยาย response logic ใน `HandleSlashCommandUseCase`
  4. ลงทะเบียน metadata ของคำสั่งใน `presentation/discord/handler.rs`

- เพิ่ม persistence/API integrations:
  - นิยาม abstraction ใน application (ports/interfaces)
  - implement adapter ใน infrastructure
  - inject เข้า use case ที่ composition root

- เพิ่ม tests:
  - unit test การ parse ใน domain
  - unit test พฤติกรรม use case ตามชื่อคำสั่ง
  - integration test ของ presentation แยกตามความจำเป็น

## ข้อจำกัดปัจจุบัน

- Slash commands ถูกลงทะเบียนแบบ global (ใช้เวลาสักพักกว่าจะกระจายครบ)
- ยังไม่มี command options
- ยังไม่มี structured logging/metrics pipeline
- ยังไม่มี persistence layer

## ข้อเสนอแนะสำหรับการพัฒนาต่อ

- ใช้การลงทะเบียนแบบ guild-scoped ระหว่างพัฒนา
- เพิ่ม command options และ validation สำหรับ interaction ที่ซับซ้อนขึ้น
- เพิ่ม observability (structured logs, tracing, metrics)
- เพิ่ม application ports + infrastructure adapters สำหรับฟีเจอร์ที่ใช้ข้อมูล

## แต่ละโฟลเดอร์มีไว้ทำอะไร

- `src/domain`
  - เก็บกฎธุรกิจหลักและ logic บริสุทธิ์
  - ไม่ผูกกับ Discord SDK, database, หรือ framework ภายนอก
  - ตัวอย่าง: `BotCommand`, ฟังก์ชันคำนวณที่ไม่ต้องเรียก service ภายนอก

- `src/application`
  - เก็บ use cases ที่กำหนด flow การทำงานของระบบ
  - รับอินพุตจาก presentation แล้วประสาน domain logic
  - ควรเป็นจุดรวม logic ระดับงาน (application behavior)

- `src/presentation`
  - เก็บ input adapters เช่น Discord interactions
  - แปลงข้อมูลที่รับเข้าเป็นรูปแบบที่ use case ใช้ได้
  - ส่งผลลัพธ์กลับไปยังผู้ใช้ผ่านช่องทางที่เรียกเข้ามา

- `src/infrastructure`
  - เก็บ adapters สำหรับเชื่อมต่อระบบภายนอก เช่น DB, cache, APIs, queue
  - เป็นจุดที่ framework/SDK ภายนอกเข้ามาอยู่
  - เหมาะกับงาน I/O และ implementation ของ ports

- `src/config.rs`
  - เก็บการโหลด environment/config ของระบบ

- `src/main.rs`
  - เป็น composition root สำหรับประกอบทุกเลเยอร์เข้าด้วยกัน
  - ไม่ควรมี business logic

## ถ้าอยากเพิ่มคำสั่งใหม่ ต้องทำที่ไหน

ตัวอย่างเพิ่มคำสั่ง `/sum`:

1. อัปเดต domain
   - เพิ่ม variant ใหม่ใน `BotCommand`
   - เพิ่ม mapping ชื่อคำสั่งใน `from_slash_name`

2. อัปเดต presentation
   - ลงทะเบียนคำสั่งใหม่ใน `presentation/discord/handler.rs`
   - ถ้ามี options ให้ดึงค่าจาก interaction ที่ชั้นนี้ แล้วส่งต่อเข้า use case

3. อัปเดต application
   - เพิ่ม logic ของคำสั่งใน use case (หรือแยก use case ใหม่ตามขอบเขตงาน)
   - คืนข้อความ/ผลลัพธ์กลับให้ presentation ส่งต่อ

## ถ้าอยากเพิ่มฟังก์ชันคำนวณ ควรวางไว้ตรงไหน

- คำนวณล้วนๆ (pure function, ไม่พึ่งภายนอก)
  - วางใน `domain` (เช่นไฟล์ `src/domain/calculation.rs`)

- คำนวณที่เป็นส่วนหนึ่งของ flow คำสั่ง
  - orchestrate ใน `application/use_cases`
  - เรียก domain function จาก use case

- คำนวณที่ต้องดึงข้อมูลจากภายนอกก่อน
  - นิยาม port ใน `application`
  - implement adapter ใน `infrastructure`
  - inject เข้า use case ที่ `main.rs`

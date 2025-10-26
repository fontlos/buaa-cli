# BUAA Cli: Powerful tool to Make BUAA Great Again

## [v0.2.0]

- Feat
  - Add `buaa boya rule <ID>` to query sign rule
  - Add `buaa boya check <ID>` to check-in/out
  - Add `buaa class query <Date>` to query schedule of some day before class begin
  - Now `buaa class auto` can checkin before class begin
  - Now `buaa class checkin <ID>` can checkin before class begin
  - Add `buaa class checkin <Date>`
- Remove
  - `Campus::Other` in Boya
- Chore
  - Update `buaa-api`

## [v0.1.4] - 2025-9-25

- Feat
  - Now we needn't `login` except `sso` and `wifi`
- Fix
  - `class` login (cause by server update)
  - More accurate course matching for `class auto` (cause by server update)
  - `spoc::Schedule` parse failed (cause by server update)
  - `tes` command

## [v0.1.3] - 2025-6-13

- Feat
  - Now we sub-api will auto call `sso.login`

## [v0.1.2] - 2025-1-15

- Feat
  - Add teacher evaluation system command
    - `buaa evaluaton list`: list and fill special form
    - `buaa evaluation fill`: fill all form one by one
- Refactor
  - Print table

## [v0.1.1] - 2024-12-8

- Feat
  - Add `buaa boya status` to query statistics information
  - Auto relogin for `buaa boya query` and `buaa boya status`
- Fix
  - Can't query selected course when course is full

## [v0.1.0] - 2024-11-28

- Feat
  - WiFi Login
  - Smart Classroom Checkin
  - Boya Course automated Select

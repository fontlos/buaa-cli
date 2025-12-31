# BUAA Cli: Powerful tool to Make BUAA Great Again

## [v0.3.4] - 2025-12-31

- Fix
  - `tes` command now work. But there are some **WARNINGS**
    > **Warning!**: Due to the poor design of the evaluation system server,
    > using this may cause the evaluation button on the web page to become unclickable.
    > But don't worry, the evaluation data has been submitted correctly.
    > If you want to view the evaluation results on the web page,
    > you can remove the 'disabled' attribute of the button in the browser console,
    > and you'll be able to click it.
    > Or you might wait a little longer, and it may return to normal.
- Chore
  - Update `buaa_api`
  - Cargo fmt

## [v0.3.3] - 2025-12-30

- Fix
  - `class` command is broken due to server bad update
    - Server NGINX was misconfigured, causing all `/app/` paths on port `8346` to be mounted under `/app/app/`. And change to port `8347` to bypass
- Chore
  - Update `buaa_api`

## [v0.3.2] - 2025-12-29

- Feat
  - Add `CanCheck` field in `buaa boya query` print table
  - `buaa boya check` can waiting for check-in/out
  - Add `--page` for `buaa boya query` for pagination
- Remove
  - `buaa boya rule`
- Refactor
  - Move waiting logic from `buaa boya query` to `buaa boya select`
  - Use RNG from `buaa_api`
  - Split `buaa boya selected` and `buaa boya status`
- Chore
  - Update `buaa_api`

## [v0.3.1] - 2025-12-19

- Fix:
  - `boya::SignInfo` parse failed
- Chore
  - Update `buaa-api`

## [v0.3.0] - 2025-12-17

- Refactor(BREAKING)
  - Update `buaa-api`, use new cookie store. Need relogin
- Fix
  - `boya::SignRule` parse failed (cause by server update)

## [v0.2.1] - 2025-11-21

- Fix
  - Add `disable-tls` flag to disable TLS if necessary. Sometimes server SSL cred maybe invalid

## [v0.2.0] - 2025-10-26

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

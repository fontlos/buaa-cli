# BUAA Cli: Powerful tool to Make BUAA Great Again

Are you still struggling with the hassle of classroom check-in?

Are you distressed that Boya course will never be able to grab?

Are you still putting up with the slow, useless of the "智慧北航"?

Now throw it in the trash and try this tool

<p align="center">
  <img src="./assets/image.png"></img>
</p>


> This project is a derivative of [buaa-api](https://github.com/fontlos/buaa-api)

> 注: 本项目只用于学习分享, 请于下载后 24 小时内删除, 使用产生的一切问题由使用者自行承担, 如有侵权我将删除此储存库和软件
>
> Tips: This project is only for learning and sharing, please delete within 24 hours after downloading,
> all problems caused by the use are borne by the user, if there is any infringement I will delete this repository and software

# Features

- ⚡ Fast
  - 🦀 Written in Rust. High-performance data parsing
  - 🎯 Communicates directly with the server via a reverse interface. Reduce the time it takes to transfer static files to open web pages
- ✨ Lightweight: An executable file that is only 5MB in size
- ❤️ Easy
  - 📦Out-of-the-box. No complicated setup required
  - 🎉 Friendly command line output
- 😎 Powerful
  - ⏰ Support for automated operations

# Install

1. Download from [release](https://github.com/fontlos/buaa-cli/release)
2. Install from source:
   ```sh
   git clone https://github.com/fontlos/buaa-cli
   cd buaa-cli
   cargo build --release
   ```

# Usage

> It is recommended to add the file path to the environment variable

## SSO Login

Whatever you do, it's the first thing you need to do

For the first time to login:

```sh
buaa login -u <Username> -p <Password>
```

You can also use the previous command to change your username and password

## Boya Course

```sh
# Query the available courses
buaa boya query

# Query all courses
buaa boya query --all

# Query courses with pagination, usually need `-all`
buaa boya query --page 2

# Select a course directly by ID
buaa boya select <ID>

# Drop a course directly by ID
buaa boya drop  <ID>

# If command above print rule, this means you can use this to check-in/out by self
buaa boya check <ID>

# Query selected courses
buaa boya selected

# Query statistics information
buaa boya status
```

## Smart Classroom

```sh
# Automatically check in for today's lessons
buaa class auto

# Query one day's schedule, format: YYYYMMDD
buaa class query <Date>

# Query the courses for this term
# eg. '202420251' means the first term of 2024-2025
buaa class query <Term ID>

# Query schedules of one course. ID len is 5
buaa class query <Course ID>

# Check in directly by schedule ID
# It is even possible to check in for the past schedule
buaa class checkin <Schedule ID>

# Checkin for some day. format: YYYYMMDD
buaa class checkin <Date>
```

## Teacher Evaluation System

**Warning!**: Due to the poor design of the evaluation system server,
using this may cause the evaluation button on the web page to become unclickable.
But don't worry, the evaluation data has been submitted correctly.
If you want to view the evaluation results on the web page,
you can remove the 'disabled' attribute of the button in the browser console,
and you'll be able to click it.
Or you might wait a little longer, and it may return to normal.

```sh
# List the forms that need to be filled out and use the index to fill out the specified form
buaa tes list

# Automatically fill all forms
buaa tes auto
```

## WiFi

You can use the following simple command to (dis)connect to **BUAA-WiFi**

```sh
buaa wifi login
buaa wifi logout
```

On the Windows platform, you can use a `.bat` file
and add it to the `C:\Users\<Username>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup` folder,
so that you can automatically connect to WiFi at boot

## Other

Sometimes Server's SSL certificate may be invalid, you can use `--disable-tls`

```sh
buaa --disable-tls [COMMAND]
```

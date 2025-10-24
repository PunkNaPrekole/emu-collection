# 🦀 Rust Emulators Collection

![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue)
![Status](https://img.shields.io/badge/Status-Active-brightgreen)

**Изучаю Rust через создание эмуляторов!** 🚀

Этот репозиторий - мое путешествие в изучение системного программирования и языка Rust через создание эмуляторов различных процессоров и виртуальных машин.

## 🎯 Цели проекта

- **Изучить Rust** на практике через интересные проекты
- **Понять как работают процессоры** на низком уровне
- **Разобраться в компьютерной архитектуре**
- **Создать что-то настоящее** и работающее

## 🕹️ Эмуляторы в разработке

### ✅ CHIP-8 - **ЗАВЕРШЕН!**
- **Статус**: Полностью рабочий 🎉
- **Архитектура**: Виртуальная машина 1970-х
- **Особенности**: 35 инструкций, 64×32 дисплей, 16 цветов
- **Запускает**: Pong, Tetris, Space Invaders и другие классические игры

### 🔄 В планах
- **Intel 8080** - процессор для Space Invaders
- **Zilog Z80** - сердце Game Boy и ZX Spectrum
- **MOS 6502** - процессор NES и Commodore 64

## 🚀 Быстрый старт

```sh
# Клонируем репозиторий
git clone https://github.com/PunkNaPrekole/emu-collection
cd emu-collection

# Запускаем CHIP-8 с игрой
cargo run -p chip8 -- chip8/roms/games/pong.ch8
cargo run -p chip8 -- chip8/roms/games/tetris.ch8

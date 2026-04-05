# oxipe

A fast, customizable terminal-based typing speed test and practice application built with Rust.

## Features

- **Real-time Feedback**: Instantly see correct (green) and incorrect (red/underlined) characters as you type.
- **Custom Themes**: Switch between 6 beautiful themes including Dracula, Monokai, Nord, Gruvbox, and Rosé Pine.
- **Live Statistics**: Track your WPM (Words Per Minute), elapsed time, and accuracy in real-time.
- **Multi-line Support**: Practice with long passages or code snippets by loading text from files.
- **Minimalist UI**: Clean interface with centered text and no distractions.

## Installation

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed on your system.

```bash
cargo install oxipe
```

## Usage

Run with the default pangram sentence:

```bash
oxipe
```

Run with a custom text file:

```bash
oxipe path/to/textfile.txt
```

## Keybindings

| Key       | Action                          |
|-----------|---------------------------------|
| `a-z`     | Type characters                 |
| `Space`   | Type space                      |
| `Enter`   | Newline (for multi-line text)   |
| `Backspace` | Delete last character         |
| `Delete`  | Reset current session           |
| `Tab`     | Cycle through themes            |
| `Esc`     | Quit application                |

## Themes

Press `Tab` to cycle through the available themes:

1. **Default**: Classic terminal colors.
2. **Dracula**: Dark purple and vibrant green/pink.
3. **Monokai**: High contrast with orange accents.
4. **Nord**: Cool, arctic-inspired blue tones.
5. **Gruvbox**: Retro-groove warm colors.
6. **Rosé Pine**: Soothing pine and rose hues.

## License

LGPL-2.1

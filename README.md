# iaKeyLite

A simple CLI application for Windows that uses the Google Gemini API to process text and automatically paste it back into the active application.

## Features

-   **Text Improvement**: Press `F2` to select text, send it to Gemini, and paste the improved version back.
-   **General Query**: Press `F8` to send a general query to Gemini.
-   **Code Generation**: Press `F9` for Python code generation and `F10` for HTML code generation.
-   **Clipboard Integration**: Automatically copies selected text, sends it to the API, and pastes the result.
-   **Tray Icon**: Runs silently in the background with a tray icon.

## Installation

### Prerequisites

-   **Rust**: Ensure Rust is installed ([rustup.rs](https://rustup.rs/)).
-   **Windows**: This application is built for Windows.

### Building and Running

1.  Clone the repository:
    ```bash
    git clone <repository-url>
    cd iaKeyLite
    ```

2.  **Configuration**: Create a `.env` file in the root directory with your API key:
    ```env
    GEMINI_API_KEY=your_gemini_api_key_here
    ```

3.  **Build & Run (Development)**:
    ```bash
    cargo run
    ```

4.  **Build & Run (Optimized)**:
    ```bash
    cargo run --release
    ```

## Usage

1.  Run the application.
2.  Select text in any application.
3.  Press `F2` to improve the text.
4.  Press `F8` for a general query.
5.  Press `F9` to generate Python code.
6.  Press `F10` to generate HTML code.

The application will automatically paste the result back into the active application after processing.
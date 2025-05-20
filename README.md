# Tauri + React + Typescript

This template should help get you started developing with Tauri, React and Typescript in Vite.

## Getting Started

First, clone the repository and install the dependencies:

```bash
git clone <repository-url> # Or your existing project
cd <project-directory>
pnpm install
```

### Configure Gemini API Key

This application uses the Google Gemini API for semantic search and other AI-powered features. To use these features, you need to provide your own Gemini API key.

1.  **Obtain a Gemini API Key:** If you don't have one, you can get a Gemini API key from [Google AI Studio](https://aistudio.google.com/apikey).
2.  **Set up the Environment Variable:**
    *   Create a file named `.env` in the root directory of the project.
    *   Add the following line to the `.env` file, replacing `"YOUR_API_KEY_HERE"` with your actual API key:
        ```
        GEMINI_API_KEY="YOUR_API_KEY_HERE"
        ```
    *   The `.env` file is included in `.gitignore` and will not be committed to the repository.

## Development

To run the application in development mode (with hot-reloading for the frontend):

```bash
pnpm tauri dev
```

This command will:
1. Start the Vite development server for the React frontend.
2. Build and run the Tauri application, which will load the frontend from the Vite server.

## Building for Production

To build the application for production:

```bash
pnpm tauri build
```

This will:
1. Build the React frontend using Vite.
2. Build the Tauri application, bundling the frontend into a native executable.
3. The executables will be located in `src-tauri/target/release/bundle/`.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

import { GoogleGenerativeAI } from "@google/genai";
import { invoke } from "@tauri-apps/api/core";

let genAI: GoogleGenerativeAI | null = null;

invoke<string>("get_gemini_api_key")
  .then((apiKey) => {
    if (apiKey) {
      genAI = new GoogleGenerativeAI(apiKey);
      console.log("Gemini AI initialized successfully");
    } else {
      console.error("Failed to get Gemini API key: Key is empty");
    }
  })
  .catch((error) => {
    console.error("Failed to get Gemini API key:", error);
  });

// Expose a function to be called from Rust
declare global {
  interface Window {
    sendToGemini: (message: string) => Promise<string>;
  }
}

window.sendToGemini = async (message: string): Promise<string> => {
  if (!genAI) {
    return Promise.reject("Gemini AI not initialized");
  }
  try {
    const model = genAI.getGenerativeModel({ model: "gemini-pro" }); // Or your desired model
    const result = await model.generateContent(message);
    const response = await result.response;
    return response.text();
  } catch (error) {
    console.error("Error calling Gemini API:", error);
    return Promise.reject(error.toString());
  }
};

export default genAI;

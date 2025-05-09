PRD & TDD Review: Rust Semantic Search in Tauri
This review assesses the provided Product Requirements Document (PRD) and Technical Design Document (TDD) for the Rust-based semantic search engine within a Tauri desktop application.
Overall Assessment:
The PRD and TDD present a solid foundation for building the described application. The goals are clear, the core features address the user needs, and the technical approach is generally sound and well-suited for a local-first, resource-constrained desktop environment. The choice of Rust, Tauri, and specific crates like LanceDB aligns well with the objectives.
1. Tauri Embeddability & Single Application Goal:
Core Rust Logic: Tauri is explicitly designed to bundle a Rust backend with a web frontend, so embedding the core application logic, file watcher, indexer, and search engine written in Rust is standard practice and highly feasible.
Crate Dependencies:
notify, pdf-extract, tokio, rayon, serde: These are pure Rust or standard ecosystem crates and will bundle without issues.
LanceDB: Designed as an embedded database in Rust. It compiles down and bundles directly into the application. Excellent choice for embeddability.
fastembed: This crate relies on the ONNX Runtime for model inference. While fastembed aims to simplify its usage, ONNX Runtime itself has native library dependencies (.dll, .so, .dylib).
Challenge: Ensuring these native ONNX libraries are correctly packaged within the Tauri application for all target platforms (Windows, macOS, Linux) can sometimes be complex. It often requires specific configurations in your build.rs script or Tauri build settings to locate and bundle these libraries correctly.
Feasibility: It is possible and commonly done, but it's the primary area requiring careful attention during the build and packaging process to ensure a true single-executable deployment works reliably across different user machines. You'll need to test cross-platform builds thoroughly.
Conclusion: Yes, the goal of shipping a single, self-contained Tauri application is achievable, but pay close attention to bundling the ONNX Runtime native dependencies required by fastembed.
2. Crate Selection:
notify: Standard, reliable choice for cross-platform file system monitoring. Good.
pdf-extract: Simple crate for text extraction. Suitable for the task. Alternatives like lopdf exist if more control over the PDF structure is needed, but pdf-extract is fine for just getting text content.
fastembed: Good choice. It simplifies loading and running various sentence-transformer models via ONNX Runtime and handles tokenization internally. Manages the complexity of interacting with ONNX.
LanceDB: Excellent choice for an embedded vector database. It's written in Rust, designed for performance on local machines, supports disk persistence, and integrates well. Its Apache Arrow foundation is also a plus. It supports ANN similarity search, filtering, and persistence.
tokio: The standard async runtime in Rust. Appropriate.
rayon: Good for CPU-bound parallelism where applicable (e.g., potentially parallelizing parts of the indexing other than the sequential embedding step, or during search).
serde / serde_json: Standard for serialization. Good.
Conclusion: The selected crates are appropriate and well-regarded within the Rust ecosystem.
3. Embedding Model (all-MiniLM-L6-v2 & Alternatives):
all-MiniLM-L6-v2: This is a very strong baseline model. It's widely used, provides a good balance between performance (speed, size) and embedding quality, and runs well on CPUs. Quantizing it (as planned) is crucial for meeting the low-spec hardware constraints (RAM/disk usage).
"Embeddable LLM": You asked about embeddable LLMs. It's important to distinguish between embedding models (like MiniLM) and generative LLMs (like Llama, Mistral, Phi).
Embedding Models: Designed specifically to convert text into numerical vectors (embeddings) for tasks like semantic search, clustering, etc. They are relatively small and fast. all-MiniLM-L6-v2 is an embedding model.
Generative LLMs: Designed to generate human-like text. They are significantly larger, slower, and more resource-intensive. Using a generative LLM just to create embeddings would be extremely inefficient and likely violate your performance constraints.
Recommendation: Stick with dedicated embedding models (sentence transformers) for this task. Your current approach is correct.
Alternative Embedding Models (via fastembed): It's worth benchmarking alternatives to potentially find better performance or quality for your specific data/queries:
bge-small-en-v1.5: Often cited as having top performance among smaller models. (BAAI General Embedding). fastembed supports BGE models.
gte-small: Another strong contender in the small model space. (General Text Embeddings). fastembed supports GTE models.
Quantization: Ensure you are using an appropriate quantization level (e.g., INT8) provided by fastembed/ONNX Runtime. Test the accuracy impact vs. performance gain.
Conclusion: all-MiniLM-L6-v2 (quantized) is a solid starting point. Benchmarking against bge-small-en-v1.5 (quantized) is highly recommended as it might offer better relevance. Stick to sentence transformers, not generative LLMs.
4. Architecture, Feasibility & Other Notes:
Architecture: The Watcher -> Indexer -> Search Engine -> Tauri Layer flow is logical.
Hybrid Search: The TDD mentions "hybrid ranking" combining vector and text search.
Clarification Needed: How will the traditional text-based search (e.g., keyword matching, BM25) be implemented? LanceDB has some full-text search (FTS) capabilities, but they might be limited compared to dedicated libraries like tantivy. Evaluate if LanceDB's FTS is sufficient or if integrating tantivy alongside LanceDB is necessary. Define a clear strategy for combining scores. This needs more detail in the TDD.
Sequential Embedding: Stated as a risk mitigation for thread safety. This is a safe approach, especially given potential complexities with PDF libraries or ONNX Runtime concurrency. However, it will be a bottleneck for initial indexing or large updates. Consider if batch embedding (processing multiple PDFs' text sequentially but sending a batch to the embedding model) is feasible and safe within fastembed/ONNX, as this might offer some speedup without complex multi-threading of the model itself.
Error Handling & Robustness: Ensure robust error handling around file watching (permissions, locked files), PDF parsing (corrupted files), and indexing. The deduplication check is good.
Resource Management: Monitor RAM and CPU usage closely during indexing and search, especially on target low-spec machines. LanceDB is designed to be memory-efficient, but embedding generation can still be CPU/RAM intensive.
Recommendations:
Verify ONNX Runtime Bundling: Prioritize creating a minimal Tauri app that uses fastembed to load and run the chosen model. Confirm you can successfully build and package this for Windows, macOS, and Linux, ensuring the ONNX native libraries are included correctly.
Benchmark Embedding Models: Test all-MiniLM-L6-v2 vs. bge-small-en-v1.5 (both quantized) on a representative set of your PDFs and queries to determine the best fit for relevance and performance.
Detail Hybrid Search: Elaborate on the implementation plan for hybrid search in the TDD, evaluating LanceDB's FTS capabilities versus potentially integrating tantivy and defining how results will be combined.
Refine Performance Strategy: While sequential embedding is safe, keep potential batching or other parallelization strategies (outside the model inference itself) in mind if initial indexing proves too slow.
Overall, this is a well-thought-out project plan using a suitable tech stack. Addressing the ONNX bundling and refining the hybrid search strategy will be key next steps.

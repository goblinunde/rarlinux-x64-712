import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import "./App.css";

interface ArchiveEntry {
  name: string;
  size: number;
  packed_size: number;
  ratio: string;
  modified: string;
  is_dir: boolean;
}

function App() {
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [archiveContents, setArchiveContents] = useState<ArchiveEntry[]>([]);
  const [currentArchive, setCurrentArchive] = useState<string | null>(null);
  const [password, setPassword] = useState("");
  const [compressionLevel, setCompressionLevel] = useState(3);
  const [status, setStatus] = useState<string>("");
  const [loading, setLoading] = useState(false);

  // 选择要压缩的文件
  // 选择要压缩的文件
  const handleSelectFiles = async () => {
    console.log("🔘 Button clicked: handleSelectFiles");
    setStatus("⏳ 打开文件选择对话框...");

    try {
      console.log("📂 Calling open dialog...");
      const selected = await open({
        multiple: true,
        title: "选择要压缩的文件",
      });

      console.log("✅ Dialog returned:", selected);

      if (selected) {
        const files = Array.isArray(selected) ? selected : [selected];
        setSelectedFiles(files);
        setStatus(`已选择 ${files.length} 个文件`);
        console.log("📁 Selected files:", files);
      } else {
        console.log("⚠️ No files selected (user cancelled)");
        setStatus("未选择文件");
      }
    } catch (error) {
      console.error("❌ Error in handleSelectFiles:", error);
      setStatus(`❌ 打开文件选择器失败: ${error}`);
    }
  };

  // 创建 RAR 归档
  const handleCreateArchive = async () => {
    if (selectedFiles.length === 0) {
      setStatus("❌ 请先选择文件");
      return;
    }

    const archivePath = await save({
      defaultPath: "archive.rar",
      filters: [{ name: "RAR Archive", extensions: ["rar"] }],
    });

    if (!archivePath) return;

    setLoading(true);
    setStatus("⏳ 正在创建归档...");

    try {
      const result = await invoke<string>("create_archive", {
        archivePath: archivePath,
        files: selectedFiles,
        password: password || null,
        compressionLevel,
        splitSize: null,
      });

      setStatus(`✅ ${result}`);
      setSelectedFiles([]);
    } catch (error) {
      setStatus(`❌ 创建失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  // 打开 RAR 归档
  const handleOpenArchive = async () => {
    const selected = await open({
      multiple: false,
      title: "打开归档文件",
      filters: [
        { name: "Archive Files", extensions: ["rar", "zip"] },
      ],
    });

    if (!selected || Array.isArray(selected)) return;

    setLoading(true);
    setStatus("⏳ 正在读取归档...");

    try {
      const contents = await invoke<ArchiveEntry[]>("list_archive_contents", {
        archivePath: selected,
      });

      setArchiveContents(contents);
      setCurrentArchive(selected);
      setStatus(`✅ 已加载 ${contents.length} 个项目`);
    } catch (error) {
      setStatus(`❌ 读取失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  // 解压归档
  const handleExtractArchive = async () => {
    if (!currentArchive) {
      setStatus("❌ 请先打开归档文件");
      return;
    }

    const destPath = await open({
      directory: true,
      title: "选择解压目标文件夹",
    });

    if (!destPath || Array.isArray(destPath)) return;

    setLoading(true);
    setStatus("⏳ 正在解压...");

    try {
      const result = await invoke<string>("extract_archive", {
        archivePath: currentArchive,
        destPath: destPath,
        password: password || null,
      });

      setStatus(`✅ ${result}`);
    } catch (error) {
      setStatus(`❌ 解压失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  // 测试归档
  const handleTestArchive = async () => {
    if (!currentArchive) {
      setStatus("❌ 请先打开归档文件");
      return;
    }

    setLoading(true);
    setStatus("⏳ 正在测试归档完整性...");

    try {
      const isValid = await invoke<boolean>("test_archive", {
        archivePath: currentArchive,
        password: password || null,
      });

      setStatus(isValid ? "✅ 归档完整,无错误" : "❌ 归档损坏或密码错误");
    } catch (error) {
      setStatus(`❌ 测试失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="app-container">
      {/* Header */}
      <header className="app-header">
        <h1>📦 RARLinux GUI</h1>
        <p className="subtitle">WinRAR Compatible Archive Manager</p>
      </header>

      {/* Main Content */}
      <main className="app-main">
        {/* Toolbar */}
        <section className="toolbar">
          <div className="toolbar-group">
            <button onClick={handleSelectFiles} disabled={loading} className="btn btn-primary">
              📁 选择文件
            </button>
            <button onClick={handleCreateArchive} disabled={loading || selectedFiles.length === 0} className="btn btn-success">
              ➕ 创建归档
            </button>
          </div>

          <div className="toolbar-group">
            <button onClick={handleOpenArchive} disabled={loading} className="btn btn-primary">
              📂 打开归档
            </button>
            <button onClick={handleExtractArchive} disabled={loading || !currentArchive} className="btn btn-warning">
              📤 解压
            </button>
            <button onClick={handleTestArchive} disabled={loading || !currentArchive} className="btn btn-info">
              🔍 测试
            </button>
          </div>
        </section>

        {/* Settings Panel */}
        <section className="settings-panel">
          <div className="setting-item">
            <label>
              🔐 密码 (可选):
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="留空表示无密码"
                disabled={loading}
              />
            </label>
          </div>

          <div className="setting-item">
            <label>
              📊 压缩级别: {compressionLevel}
              <input
                type="range"
                min="0"
                max="5"
                value={compressionLevel}
                onChange={(e) => setCompressionLevel(Number(e.target.value))}
                disabled={loading}
              />
              <span className="range-labels">
                <span>存储</span>
                <span>最佳</span>
              </span>
            </label>
          </div>
        </section>

        {/* File List */}
        <section className="file-panel">
          <h2>{selectedFiles.length > 0 ? "待压缩文件" : "归档内容"}</h2>
          <div className="file-list">
            {selectedFiles.length > 0 ? (
              selectedFiles.map((file, idx) => (
                <div key={idx} className="file-item">
                  <span className="file-icon">📄</span>
                  <span className="file-name">{file.split(/[/\\]/).pop()}</span>
                </div>
              ))
            ) : archiveContents.length > 0 ? (
              archiveContents.map((entry, idx) => (
                <div key={idx} className="file-item">
                  <span className="file-icon">{entry.is_dir ? "📁" : "📄"}</span>
                  <span className="file-name">{entry.name}</span>
                </div>
              ))
            ) : (
              <div className="empty-state">
                <p>📦 选择文件以创建归档,或打开现有归档</p>
              </div>
            )}
          </div>
        </section>
      </main>

      {/* Status Bar */}
      <footer className="status-bar">
        {loading ? (
          <div className="loader-container">
            <div className="loader"></div>
            <span>{status}</span>
          </div>
        ) : (
          <span>{status || "准备就绪"}</span>
        )}
      </footer>
    </div>
  );
}

export default App;


import { useEffect, useState } from "react";

interface Connection {
  nombre: string;
  url: string;
  token: string;
}

export default function App() {
  const [connections, setConnections] = useState<Connection[]>([]);
  const [nodeVersion, setNodeVersion] = useState<string>("");
  const [nodeOk, setNodeOk] = useState<boolean | null>(null);

  useEffect(() => {
    fetch("http://localhost:47821/connections")
      .then(r => r.json())
      .then(d => setConnections(d.connections || []))
      .catch(() => {});

    fetch("http://localhost:47821/node-check")
      .then(r => r.json())
      .then(d => {
        setNodeOk(d.ok);
        setNodeVersion(d.version || "");
      })
      .catch(() => setNodeOk(false));
  }, []);

  return (
    <div style={{
      fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, sans-serif",
      background: "#ffffff",
      height: "100vh",
      display: "flex",
      flexDirection: "column",
      color: "#0f172a",
      userSelect: "none" as const,
    }}>
      {/* Header */}
      <div style={{
        background: "#1e4d8c",
        padding: "18px 20px",
        display: "flex",
        alignItems: "center",
        gap: "12px",
      }}>
        <div style={{
          width: "34px", height: "34px",
          background: "rgba(255,255,255,0.15)",
          borderRadius: "8px",
          display: "flex", alignItems: "center", justifyContent: "center",
          fontWeight: "800", color: "#fff", fontSize: "17px",
          letterSpacing: "-0.02em",
        }}>L</div>
        <div>
          <div style={{ color: "#fff", fontWeight: "600", fontSize: "14px" }}>Lexiius Connector</div>
          <div style={{ color: "rgba(255,255,255,0.65)", fontSize: "11px" }}>v1.0.0 · Puerto 47821</div>
        </div>
        <div style={{ marginLeft: "auto", display: "flex", alignItems: "center", gap: "6px" }}>
          <div style={{ width: "7px", height: "7px", background: "#4ade80", borderRadius: "50%" }} />
          <span style={{ color: "rgba(255,255,255,0.8)", fontSize: "11px" }}>Activo</span>
        </div>
      </div>

      {/* Content */}
      <div style={{ flex: 1, padding: "16px 20px", overflowY: "auto" }}>

        {/* Node.js warning */}
        {nodeOk === false && (
          <div style={{
            padding: "12px 14px",
            background: "#fef2f2",
            border: "1px solid #fecaca",
            borderRadius: "8px",
            marginBottom: "14px",
            fontSize: "12px",
            color: "#dc2626",
            lineHeight: "1.5",
          }}>
            <strong>Node.js no encontrado.</strong> Es necesario para conectar Claude Desktop.
            {" "}<a href="https://nodejs.org" target="_blank" rel="noreferrer" style={{ color: "#dc2626" }}>
              Descargar Node.js →
            </a>
          </div>
        )}

        {/* Conexiones */}
        <div style={{ marginBottom: "16px" }}>
          <div style={{
            fontSize: "10px", fontWeight: "600", color: "#94a3b8",
            textTransform: "uppercase" as const, letterSpacing: "0.06em",
            marginBottom: "8px",
          }}>
            Conexiones con Claude Desktop
          </div>
          {connections.length === 0 ? (
            <div style={{
              padding: "14px",
              background: "#f8fafc",
              borderRadius: "8px",
              border: "1px solid #e2e8f0",
              fontSize: "12px", color: "#64748b", textAlign: "center" as const,
              lineHeight: "1.6",
            }}>
              Sin conexiones activas.
              <br />
              <span style={{ fontSize: "11px" }}>
                Lexiius → Vault → Conectar IA → Claude
              </span>
            </div>
          ) : (
            connections.map(c => (
              <div key={c.nombre} style={{
                padding: "10px 12px",
                background: "#f0fdf4",
                border: "1px solid #bbf7d0",
                borderRadius: "8px",
                marginBottom: "6px",
                display: "flex", alignItems: "center", gap: "10px",
              }}>
                <div style={{ width: "7px", height: "7px", background: "#16a34a", borderRadius: "50%", flexShrink: 0 }} />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontSize: "13px", fontWeight: "600" }}>{c.nombre}</div>
                  <div style={{ fontSize: "10px", color: "#64748b", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" as const }}>
                    {c.token ? c.token.substring(0, 22) + "..." : ""}
                  </div>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Sistema */}
        <div>
          <div style={{
            fontSize: "10px", fontWeight: "600", color: "#94a3b8",
            textTransform: "uppercase" as const, letterSpacing: "0.06em",
            marginBottom: "8px",
          }}>Sistema</div>
          <div style={{ background: "#f8fafc", border: "1px solid #e2e8f0", borderRadius: "8px", overflow: "hidden" }}>
            {[
              { label: "Servidor local", value: "localhost:47821 ✓" },
              {
                label: "Node.js",
                value: nodeOk === null ? "Verificando..." : nodeOk ? `${nodeVersion} ✓` : "No instalado ✗",
                color: nodeOk === false ? "#dc2626" : "#0f172a",
              },
              { label: "Compatible con", value: "Windows 10/11 64-bit" },
            ].map((item, i, arr) => (
              <div key={i} style={{
                display: "flex", justifyContent: "space-between", alignItems: "center",
                padding: "9px 12px",
                borderBottom: i < arr.length - 1 ? "1px solid #e2e8f0" : "none",
              }}>
                <span style={{ fontSize: "12px", color: "#64748b" }}>{item.label}</span>
                <span style={{ fontSize: "12px", fontWeight: "500", color: (item as any).color || "#0f172a" }}>
                  {item.value}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Footer */}
      <div style={{
        padding: "12px 20px",
        borderTop: "1px solid #e2e8f0",
        display: "flex", gap: "8px",
      }}>
        <a
          href="https://app.lexiius.com"
          target="_blank"
          rel="noreferrer"
          style={{
            flex: 1, textAlign: "center" as const,
            padding: "8px",
            background: "#1e4d8c", color: "#fff",
            borderRadius: "6px", textDecoration: "none",
            fontSize: "12px", fontWeight: "500",
          }}
        >
          Abrir Lexiius
        </a>
        <a
          href="https://lexiius.com/connector"
          target="_blank"
          rel="noreferrer"
          style={{
            padding: "8px 14px",
            border: "1px solid #e2e8f0",
            borderRadius: "6px", textDecoration: "none",
            fontSize: "12px", color: "#64748b",
          }}
        >
          Ayuda
        </a>
      </div>
    </div>
  );
}

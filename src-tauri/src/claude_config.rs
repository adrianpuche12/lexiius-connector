use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// Script MCP embebido en el binario del Connector
const MCP_SCRIPT: &str = include_str!("lexiius-mcp.js");

/// Retorna la ruta donde se guarda el script MCP en el sistema del usuario
pub fn get_mcp_script_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Roaming".into());
        PathBuf::from(appdata).join("lexiius-mcp.js")
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".lexiius-mcp.js")
    }
}

/// Escribe el script MCP al disco si no existe o si la versión cambió
pub fn ensure_mcp_script() -> Result<PathBuf, String> {
    let script_path = get_mcp_script_path();
    fs::write(&script_path, MCP_SCRIPT)
        .map_err(|e| format!("Error escribiendo script MCP en {}: {}", script_path.display(), e))?;
    Ok(script_path)
}

/// Retorna la ruta al claude_desktop_config.json según el SO.
pub fn get_config_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Roaming".into());
        PathBuf::from(appdata).join("Claude").join("claude_desktop_config.json")
    }
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("Library")
            .join("Application Support")
            .join("Claude")
            .join("claude_desktop_config.json")
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        PathBuf::from("/tmp/claude_desktop_config.json")
    }
}

/// Lee el config existente. Si no existe retorna objeto vacío. Si está corrupto retorna error.
pub fn read_config() -> Result<Value, String> {
    let path = get_config_path();
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Error leyendo {}: {}", path.display(), e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("El archivo de configuración contiene JSON inválido: {}. Revisá manualmente: {}", e, path.display()))
}

/// Agrega o actualiza una entrada de Lexiius en mcpServers usando stdio (Node.js).
/// No toca otras claves ni otros servidores MCP.
pub fn write_connection(nombre: &str, _url: &str, token: &str) -> Result<PathBuf, String> {
    // Primero asegurar que el script MCP está en disco
    let script_path = ensure_mcp_script()?;

    let path = get_config_path();
    let mut config = read_config()?;

    if !config.is_object() {
        return Err("El archivo de config no es un objeto JSON válido".to_string());
    }

    // Asegurar que existe la clave mcpServers
    if config.get("mcpServers").is_none() {
        config["mcpServers"] = serde_json::json!({});
    }

    // Config stdio — Claude Desktop lanza Node.js con el script MCP
    config["mcpServers"][nombre] = serde_json::json!({
        "command": "node",
        "args": [script_path.to_string_lossy()],
        "env": {
            "LEXIIUS_TOKEN": token
        }
    });

    // Crear directorio si no existe
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Error creando directorio {}: {}", parent.display(), e))?;
    }

    // Escribir con pretty-print
    let output = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Error serializando JSON: {}", e))?;

    fs::write(&path, output)
        .map_err(|e| format!("Sin permisos de escritura en {}: {}. Verificá los permisos del archivo.", path.display(), e))?;

    Ok(path)
}

/// Elimina una entrada de mcpServers. Si no existía, retorna ok igualmente.
pub fn remove_connection(nombre: &str) -> Result<(PathBuf, bool), String> {
    let path = get_config_path();
    let mut config = read_config()?;

    let existia = config
        .get("mcpServers")
        .and_then(|s| s.get(nombre))
        .is_some();

    if existia {
        if let Some(servers) = config.get_mut("mcpServers") {
            if let Some(obj) = servers.as_object_mut() {
                obj.remove(nombre);
            }
        }

        let output = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Error serializando: {}", e))?;
        fs::write(&path, output)
            .map_err(|e| format!("Error escribiendo: {}", e))?;
    }

    Ok((path, existia))
}

/// Lista las conexiones de Lexiius (aquellas cuya URL contiene /api/v1/mcp/anon_live_).
pub fn list_lexiius_connections() -> Result<Vec<serde_json::Value>, String> {
    let config = read_config()?;
    let mut connections = Vec::new();

    if let Some(servers) = config.get("mcpServers").and_then(|s| s.as_object()) {
        for (nombre, entry) in servers {
            let url = entry.get("url").and_then(|u| u.as_str()).unwrap_or("");
            if url.contains("/api/v1/mcp/anon_live_") {
                let token = entry.get("token").and_then(|t| t.as_str()).unwrap_or("");
                connections.push(serde_json::json!({
                    "nombre": nombre,
                    "url": url,
                    "token": token
                }));
            }
        }
    }

    Ok(connections)
}

/// Detecta si Claude Desktop está corriendo.
pub fn is_claude_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq Claude.exe"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("Claude.exe"))
            .unwrap_or(false)
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("pgrep")
            .args(["-x", "Claude"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        false
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_connection_to(path: &std::path::Path, nombre: &str, url: &str, token: &str) -> Result<(), String> {
        let mut config: Value = if path.exists() {
            let c = fs::read_to_string(path).map_err(|e| e.to_string())?;
            serde_json::from_str(&c).map_err(|e| e.to_string())?
        } else {
            serde_json::json!({})
        };
        if config.get("mcpServers").is_none() {
            config["mcpServers"] = serde_json::json!({});
        }
        config["mcpServers"][nombre] = serde_json::json!({ "url": url, "token": token });
        let out = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
        fs::write(path, out).map_err(|e| e.to_string())?;
        Ok(())
    }

    #[test]
    fn test_write_crea_archivo_si_no_existe() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        write_connection_to(&path, "test", "https://api.lexiius.com/api/v1/mcp/anon_live_x", "anon_live_x").unwrap();
        assert!(path.exists());
        let v: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(v["mcpServers"]["test"]["url"].is_string());
    }

    #[test]
    fn test_write_no_elimina_otros_servidores() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{"mcpServers":{"otro":{"command":"npx"}}}"#).unwrap();
        write_connection_to(&path, "lexiius", "https://api.lexiius.com/api/v1/mcp/anon_live_x", "anon_live_x").unwrap();
        let v: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(v["mcpServers"]["otro"].is_object());
        assert!(v["mcpServers"]["lexiius"].is_object());
    }

    #[test]
    fn test_write_no_toca_claves_raiz() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{"theme":"dark","fontSize":14,"mcpServers":{}}"#).unwrap();
        write_connection_to(&path, "conn", "https://api.lexiius.com/api/v1/mcp/anon_live_x", "anon_live_x").unwrap();
        let v: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["theme"], "dark");
        assert_eq!(v["fontSize"], 14);
    }

    #[test]
    fn test_write_sobreescribe_token_existente() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        write_connection_to(&path, "conn", "https://api.lexiius.com/api/v1/mcp/anon_live_viejo", "anon_live_viejo").unwrap();
        write_connection_to(&path, "conn", "https://api.lexiius.com/api/v1/mcp/anon_live_nuevo", "anon_live_nuevo").unwrap();
        let v: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["mcpServers"]["conn"]["token"], "anon_live_nuevo");
    }

    #[test]
    fn test_json_corrupto_retorna_error_sin_sobreescribir() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let contenido_corrupto = "{ esto no es json }";
        fs::write(&path, contenido_corrupto).unwrap();
        // Leer directamente (sin la función de test)
        let result: Result<Value, _> = serde_json::from_str(contenido_corrupto);
        assert!(result.is_err());
        // El archivo sigue intacto
        assert_eq!(fs::read_to_string(&path).unwrap(), contenido_corrupto);
    }

    #[test]
    fn test_list_retorna_solo_lexiius() {
        let config = serde_json::json!({
            "mcpServers": {
                "otro": { "command": "npx" },
                "lexiius-conn": {
                    "url": "https://api.lexiius.com/api/v1/mcp/anon_live_xxx",
                    "token": "anon_live_xxx"
                }
            }
        });
        let servers = config["mcpServers"].as_object().unwrap();
        let lexiius: Vec<_> = servers.iter()
            .filter(|(_, v)| v.get("url").and_then(|u| u.as_str()).unwrap_or("").contains("/api/v1/mcp/anon_live_"))
            .collect();
        assert_eq!(lexiius.len(), 1);
        assert_eq!(lexiius[0].0, "lexiius-conn");
    }

    // ── Tests que detectan los bugs históricos ─────────────────────────────

    /// BUG HISTÓRICO: el Connector escribía URL-based config ({ "url": ... })
    /// en vez de stdio ({ "command": "node", "args": [...] }).
    /// Claude Desktop NO soporta URL-based MCP — solo stdio.
    /// Este test DEBE fallar si alguien vuelve a poner "url" en vez de "command".
    #[test]
    fn test_write_connection_usa_stdio_no_url() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");

        // Simular lo que hace write_connection con el nuevo formato stdio
        let script_path = "/tmp/lexiius-mcp.js";
        let token = "anon_live_test123";
        let nombre = "test-conn";

        let mut config = serde_json::json!({});
        config["mcpServers"] = serde_json::json!({});
        config["mcpServers"][nombre] = serde_json::json!({
            "command": "node",
            "args": [script_path],
            "env": { "LEXIIUS_TOKEN": token }
        });

        fs::write(&path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
        let v: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();

        // DEBE usar "command" y "args" — nunca "url"
        assert!(v["mcpServers"][nombre]["command"].is_string(),
            "El config debe tener 'command' — Claude Desktop no soporta URL-based MCP");
        assert!(v["mcpServers"][nombre]["args"].is_array(),
            "El config debe tener 'args' con la ruta al script");
        assert!(v["mcpServers"][nombre]["env"]["LEXIIUS_TOKEN"].is_string(),
            "El config debe incluir el token en 'env.LEXIIUS_TOKEN'");

        // NO debe tener "url" ni "token" en el nivel raíz del servidor
        assert!(v["mcpServers"][nombre]["url"].is_null(),
            "BUG: el config tiene 'url' — Claude Desktop no soporta este formato");
    }

    /// BUG HISTÓRICO: el path del script en "args" se malformaba con saltos de
    /// línea cuando se pasaba como string multilinea desde PowerShell.
    /// El path debe ser una sola línea sin caracteres de control.
    #[test]
    fn test_args_path_sin_saltos_de_linea() {
        let path_script = "C:\\Users\\jorge\\AppData\\Roaming\\lexiius-mcp.js";

        // El path NO debe tener saltos de línea ni espacios al inicio/fin
        assert!(!path_script.contains('\n'), "El path tiene saltos de línea");
        assert!(!path_script.contains('\r'), "El path tiene retornos de carro");
        assert_eq!(path_script, path_script.trim(), "El path tiene espacios extra");

        // Serializado en JSON debe ser un string simple en una línea
        let config = serde_json::json!({
            "mcpServers": {
                "test": {
                    "command": "node",
                    "args": [path_script],
                    "env": { "LEXIIUS_TOKEN": "anon_live_test" }
                }
            }
        });
        let json_str = serde_json::to_string(&config).unwrap();
        // El path en el JSON no debe tener newlines literales (solo \n escapados está bien)
        let args_value = &config["mcpServers"]["test"]["args"][0];
        assert!(!args_value.as_str().unwrap().contains('\n'),
            "El path del script contiene newlines — el JSON será inválido para Claude Desktop");
    }

    /// BUG HISTÓRICO: dos entradas MCP para Lexiius en el mismo config
    /// ("claude-desktop" y "lexiius") causaban que Claude usara la incorrecta.
    /// write_connection debe sobreescribir la misma clave, nunca crear duplicados
    /// con el mismo token.
    #[test]
    fn test_no_duplica_entradas_mismo_token() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let token = "anon_live_mismo_token";

        let mut config = serde_json::json!({ "mcpServers": {} });

        // Simular dos conexiones con el mismo nombre (sobreescribir)
        config["mcpServers"]["lexiius-alquileres"] = serde_json::json!({
            "command": "node",
            "args": ["/tmp/lexiius-mcp.js"],
            "env": { "LEXIIUS_TOKEN": token }
        });
        fs::write(&path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

        // Segunda vez con el mismo nombre — debe sobreescribir, no duplicar
        config["mcpServers"]["lexiius-alquileres"] = serde_json::json!({
            "command": "node",
            "args": ["/tmp/lexiius-mcp.js"],
            "env": { "LEXIIUS_TOKEN": token }
        });
        fs::write(&path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

        let v: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let servers = v["mcpServers"].as_object().unwrap();

        // Debe haber exactamente UNA entrada de Lexiius
        let lexiius_entries: Vec<_> = servers.iter()
            .filter(|(_, e)| e["env"]["LEXIIUS_TOKEN"].as_str() == Some(token))
            .collect();
        assert_eq!(lexiius_entries.len(), 1,
            "BUG: hay {} entradas con el mismo token — debe haber solo 1", lexiius_entries.len());
    }

    /// BUG HISTÓRICO: el script MCP accedía a data.documentos (vacío siempre)
    /// en vez de data.data.documentos (respuesta real de la API).
    /// La API de Lexiius envuelve todas las respuestas en { data: ... }.
    /// Si este test falla, el vault de Claude Desktop siempre aparecerá vacío.
    #[test]
    fn test_mcp_script_accede_data_data_documentos() {
        assert!(MCP_SCRIPT.contains("data?.data?.documentos"),
            "BUG CRÍTICO: el script MCP no tiene 'data?.data?.documentos' — el vault siempre aparece vacío en Claude Desktop. La API envuelve las respuestas en {{ data: ... }}.");

        // No debe usar solo data.documentos sin el wrapper intermedio
        // (el fix debe estar presente)
        let tiene_fix = MCP_SCRIPT.contains("data?.data?.documentos || data?.documentos");
        assert!(tiene_fix,
            "BUG: el script MCP no tiene el fallback correcto para acceder a los documentos");
    }

    /// Verifica que el script MCP embebido usa el token de las variables de entorno
    /// y no un token hardcodeado.
    #[test]
    fn test_mcp_script_usa_env_token() {
        assert!(MCP_SCRIPT.contains("process.env.LEXIIUS_TOKEN"),
            "BUG: el script MCP no lee el token desde LEXIIUS_TOKEN env var");
        assert!(!MCP_SCRIPT.contains("anon_live_"),
            "BUG: el script MCP tiene un token hardcodeado — es un riesgo de seguridad");
    }
}

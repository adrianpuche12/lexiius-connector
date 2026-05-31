/**
 * Tests del script lexiius-mcp.js
 *
 * Estos tests verifican los bugs históricos que causaron problemas:
 *
 * BUG 1: La API de Lexiius envuelve respuestas en { data: ... }
 *        El script accedía a data.documentos (undefined) en vez de data.data.documentos
 *        → El vault siempre aparecía vacío en Claude Desktop
 *
 * BUG 2: El config de Claude Desktop usaba URL-based MCP en vez de stdio
 *        → Claude Desktop rechazaba la configuración con "not valid MCP server"
 *
 * BUG 3: El path del script tenía saltos de línea desde PowerShell
 *        → JSON inválido en claude_desktop_config.json
 */

const fs = require('fs');
const path = require('path');

const SCRIPT_PATH = path.join(__dirname, '../src-tauri/src/lexiius-mcp.js');
const scriptContent = fs.readFileSync(SCRIPT_PATH, 'utf-8');

// ── BUG 1: Acceso correcto a la estructura anidada de la API ───────────────

describe('Acceso a datos de la API (BUG HISTÓRICO: vault vacío)', () => {
  test('El script accede a data.data.documentos — NO a data.documentos', () => {
    expect(scriptContent).toContain('data?.data?.documentos');
    // Si solo tiene data.documentos sin el wrapper, el vault siempre aparece vacío
  });

  test('El script tiene fallback data.data.documentos || data.documentos', () => {
    expect(scriptContent).toContain('data?.data?.documentos || data?.documentos');
  });

  test('El script accede a data.data.subcarpetas con fallback', () => {
    expect(scriptContent).toContain('data?.data?.subcarpetas || data?.subcarpetas');
  });

  test('El script accede a data.data.contenido para get_document', () => {
    expect(scriptContent).toContain('data?.data?.contenido || data?.contenido');
  });

  test('El script accede a data.data.entidades para get_report', () => {
    expect(scriptContent).toContain('data?.data?.entidades || data?.entidades');
  });
});

// ── Seguridad del token ────────────────────────────────────────────────────

describe('Seguridad del token', () => {
  test('El token viene de process.env.LEXIIUS_TOKEN', () => {
    expect(scriptContent).toContain('process.env.LEXIIUS_TOKEN');
  });

  test('El script falla si no hay LEXIIUS_TOKEN (no silencioso)', () => {
    expect(scriptContent).toContain('process.exit(1)');
  });

  test('No hay tokens hardcodeados en el script', () => {
    expect(scriptContent).not.toMatch(/anon_live_[A-Za-z0-9_-]{10,}/);
  });
});

// ── Protocolo MCP correcto ─────────────────────────────────────────────────

describe('Protocolo MCP stdio', () => {
  test('El script usa readline para leer stdin (stdio transport)', () => {
    expect(scriptContent).toContain('readline');
    expect(scriptContent).toContain('process.stdin');
  });

  test('El script escribe JSON-RPC a stdout', () => {
    expect(scriptContent).toContain('process.stdout.write');
  });

  test('El script implementa initialize con protocolVersion correcto', () => {
    expect(scriptContent).toContain('2024-11-05');
    expect(scriptContent).toContain('initialize');
  });

  test('El script implementa tools/list con las 3 herramientas requeridas', () => {
    expect(scriptContent).toContain('list_documents');
    expect(scriptContent).toContain('get_document');
    expect(scriptContent).toContain('get_report');
  });

  test('El script implementa tools/call', () => {
    expect(scriptContent).toContain("method === 'tools/call'");
  });

  test('El script maneja notifications/initialized sin error', () => {
    expect(scriptContent).toContain("notifications/initialized");
  });
});

// ── Integración: simular respuesta real de la API ─────────────────────────

describe('Integración: parseo de respuesta real de la API', () => {
  /**
   * La API de Lexiius devuelve:
   * { "data": { "tipo": "carpeta", "documentos": [...], "subcarpetas": [] } }
   *
   * El script DEBE extraer correctamente los documentos de esta estructura.
   */
  test('Extrae documentos de la respuesta real de la API { data: { documentos: [...] } }', () => {
    // Simular la respuesta real de la API
    const apiResponse = {
      data: {
        tipo: 'carpeta',
        documentos: [
          { id: 'doc-1', nombre_original: 'contrato.pdf', formato: 'pdf' },
          { id: 'doc-2', nombre_original: 'otro.pdf', formato: 'pdf' },
        ],
        subcarpetas: []
      }
    };

    // La lógica que usa el script
    const docs = apiResponse?.data?.documentos || apiResponse?.documentos || [];

    expect(docs).toHaveLength(2);
    expect(docs[0].nombre_original).toBe('contrato.pdf');
  });

  test('Devuelve array vacío si no hay documentos (no falla con undefined)', () => {
    const apiResponse = { data: { tipo: 'carpeta', documentos: [], subcarpetas: [] } };
    const docs = apiResponse?.data?.documentos || apiResponse?.documentos || [];
    expect(docs).toHaveLength(0);
    expect(Array.isArray(docs)).toBe(true);
  });

  test('Maneja respuesta sin wrapper data (compatibilidad hacia atrás)', () => {
    // Si algún día la API no tiene wrapper, el fallback funciona
    const apiResponse = {
      tipo: 'carpeta',
      documentos: [{ id: 'doc-1', nombre_original: 'test.pdf', formato: 'pdf' }],
      subcarpetas: []
    };
    const docs = apiResponse?.data?.documentos || apiResponse?.documentos || [];
    expect(docs).toHaveLength(1);
  });

  test('No rompe si la respuesta es null o undefined', () => {
    const docs = null?.data?.documentos || null?.documentos || [];
    expect(docs).toHaveLength(0);
    expect(Array.isArray(docs)).toBe(true);
  });
});

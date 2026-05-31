#!/usr/bin/env node
// Lexiius MCP Server — stdio transport para Claude Desktop
// Protocolo: JSON-RPC sobre stdin/stdout

const readline = require('readline');
const https = require('https');
const http = require('http');

const TOKEN = process.env.LEXIIUS_TOKEN;
const API_BASE = process.env.LEXIIUS_API || 'https://anonimizador-backend-prod-production.up.railway.app';

if (!TOKEN) {
  process.stderr.write('ERROR: LEXIIUS_TOKEN no configurado\n');
  process.exit(1);
}

function write(obj) {
  process.stdout.write(JSON.stringify(obj) + '\n');
}

function ok(id, result) {
  write({ jsonrpc: '2.0', id, result });
}

function err(id, code, message) {
  write({ jsonrpc: '2.0', id, error: { code, message } });
}

function get(path) {
  return new Promise((resolve, reject) => {
    const url = new URL(API_BASE + path);
    const mod = url.protocol === 'https:' ? https : http;
    const req = mod.get(url.toString(), (res) => {
      let data = '';
      res.on('data', c => data += c);
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch (e) { reject(new Error('Respuesta inválida del servidor')); }
      });
    });
    req.on('error', reject);
    req.setTimeout(15000, () => { req.destroy(); reject(new Error('Timeout')); });
  });
}

const TOOLS = [
  {
    name: 'list_documents',
    description: 'Lista los documentos anonimizados disponibles en el vault de Lexiius. Usá esto primero para obtener los IDs de documentos.',
    inputSchema: { type: 'object', properties: {}, required: [] },
  },
  {
    name: 'get_document',
    description: 'Lee el contenido completo de un documento anonimizado. Los datos sensibles (nombres, DNI, CUIT, montos, etc.) fueron reemplazados por variables como {{NOMBRE_1}}, {{DNI_1}}.',
    inputSchema: {
      type: 'object',
      properties: {
        document_id: { type: 'string', description: 'ID del documento obtenido con list_documents' },
      },
      required: ['document_id'],
    },
  },
  {
    name: 'get_report',
    description: 'Obtiene el reporte de qué datos sensibles fueron detectados y anonimizados en un documento.',
    inputSchema: {
      type: 'object',
      properties: {
        document_id: { type: 'string', description: 'ID del documento' },
      },
      required: ['document_id'],
    },
  },
];

const rl = readline.createInterface({ input: process.stdin, terminal: false });

rl.on('line', async (line) => {
  if (!line.trim()) return;
  let msg;
  try { msg = JSON.parse(line); } catch { return; }

  const { method, params, id } = msg;

  if (method === 'initialize') {
    ok(id, {
      protocolVersion: '2024-11-05',
      capabilities: { tools: {} },
      serverInfo: { name: 'lexiius', version: '1.0.0' },
    });
    return;
  }

  if (method === 'notifications/initialized') return;

  if (method === 'tools/list') {
    ok(id, { tools: TOOLS });
    return;
  }

  if (method === 'tools/call') {
    const name = params?.name;
    const args = params?.arguments || {};

    try {
      if (name === 'list_documents') {
        const data = await get(`/api/v1/mcp/${TOKEN}/list`);
        const docs = data?.data?.documentos || data?.documentos || [];
        const subs = data?.data?.subcarpetas || data?.subcarpetas || [];
        const text = [
          docs.length === 0
            ? 'No hay documentos anonimizados disponibles.'
            : `Documentos disponibles (${docs.length}):\n${docs.map(d =>
                `• ID: ${d.id}\n  Nombre: ${d.nombre_original}\n  Formato: ${d.formato?.toUpperCase()}`
              ).join('\n\n')}`,
          subs.length > 0
            ? `\nSubcarpetas:\n${subs.map(s => `• ${s.nombre} (ID: ${s.id})`).join('\n')}`
            : '',
        ].join('');
        ok(id, { content: [{ type: 'text', text }] });

      } else if (name === 'get_document') {
        if (!args.document_id) { err(id, -32602, 'document_id requerido'); return; }
        const data = await get(`/api/v1/mcp/${TOKEN}/document/${args.document_id}`);
        if (data?.error) { err(id, -32000, data.error.message || 'Error al obtener documento'); return; }
        const content = data?.data?.contenido || data?.contenido || 'Sin contenido disponible';
        const nombre = data?.data?.nombre || data?.nombre || 'Documento';
        ok(id, { content: [{ type: 'text', text: `Documento: ${nombre}\n\n${content}` }] });

      } else if (name === 'get_report') {
        if (!args.document_id) { err(id, -32602, 'document_id requerido'); return; }
        const data = await get(`/api/v1/mcp/${TOKEN}/document/${args.document_id}/report`);
        const entidades = data?.data?.entidades || data?.entidades || [];
        const text = entidades.length === 0
          ? 'Sin entidades registradas para este documento.'
          : `Reporte de anonimización:\n${entidades.map(e =>
              `• ${e.variable} → tipo: ${e.tipo}, ${e.ocurrencias} ocurrencia(s)`
            ).join('\n')}`;
        ok(id, { content: [{ type: 'text', text }] });

      } else {
        err(id, -32601, `Herramienta desconocida: ${name}`);
      }
    } catch (e) {
      err(id, -32000, `Error: ${e.message}`);
    }
    return;
  }

  err(id, -32601, `Método no soportado: ${method}`);
});
